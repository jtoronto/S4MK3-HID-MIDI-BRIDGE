import Foundation
import SwiftUI

enum TTResponseDirection: String, CaseIterable, Equatable, Sendable {
    case braking, nudge
    var title: String {
        switch self { case .braking: return "Braking (slow down)"; case .nudge: return "Nudge (speed up)" }
    }
    var gainRange: ClosedRange<Double> {
        switch self { case .braking: return 0.1...20.0; case .nudge: return 0.1...10.0 }
    }
    var sign: Double {
        switch self { case .braking: return -1.0; case .nudge: return 1.0 }
    }
}

struct TTResponseSample: Equatable, Sendable {
    let deviationRpm, deviationPercent, signedCc4Rate, magnitudeCc4Rate: Double
}

struct TTResponseParameters: Equatable, Sendable {
    var targetRpm = 33
    var brakeGain = 2.0, brakeExponent = 1.0, nudgeGain = 1.0, nudgeExponent = 1.0
}

struct TTResponseCurve {
    static let quietBandRpm = 2.25
    static let quietBandEpsilon = 1e-9
    static let smoothOnsetWidthRpm = 1.0
    static let countsPerRpmSecond = 48.0
    static let exponentRange: ClosedRange<Double> = 0.25...4.0

    static func signedCc4Rate(parameters: TTResponseParameters, measuredRpm: Double) -> Double {
        let target = Double(parameters.targetRpm)
        let error = measuredRpm - target
        let absError = abs(error)
        guard absError > quietBandRpm + quietBandEpsilon else { return 0.0 }

        let gain = error < 0 ? parameters.brakeGain : parameters.nudgeGain
        let exponent = error < 0 ? parameters.brakeExponent : parameters.nudgeExponent
        let a = min(max((absError - quietBandRpm) / smoothOnsetWidthRpm, 0.0), 1.0)
        let w = a * a * (3.0 - 2.0 * a)
        let x = min(absError / target, 1.0)
        let factor = pow(x, exponent - 1.0)
        return error * w * gain * factor * countsPerRpmSecond
    }

    static func samples(direction: TTResponseDirection, parameters: TTResponseParameters, steps: Int = 64) -> [TTResponseSample] {
        let clampedTarget = max(parameters.targetRpm, 1)
        let target = Double(clampedTarget)
        let stepCount = max(steps, 2)
        return (0...stepCount).map { index in
            let fraction = Double(index) / Double(stepCount)
            let deviationRpm = fraction * target
            let signedRate = signedCc4Rate(parameters: parameters, measuredRpm: target + direction.sign * deviationRpm)
            return TTResponseSample(deviationRpm: deviationRpm, deviationPercent: fraction * 100.0, signedCc4Rate: signedRate, magnitudeCc4Rate: abs(signedRate))
        }
    }

    static func shapeDescription(for exponent: Double) -> String {
        if exponent == 1.0 { return "1.00 (existing soft-knee)" }
        return String(format: exponent > 1.0 ? "%.2f (softer near center)" : "%.2f (stronger near center)", exponent)
    }
}

struct TTResponseCurvesSectionView: View {
    @Binding var jog: JogPreferences

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack(alignment: .top, spacing: 12) {
                TTSingleCurveEditorView(
                    direction: .braking,
                    targetRpm: jog.platterTargetRpm,
                    gain: Binding(get: { jog.effectiveTTBrakeGain }, set: { jog.ttBrakeGain = $0 }),
                    exponent: $jog.ttBrakeCurveExponent
                )
                TTSingleCurveEditorView(
                    direction: .nudge,
                    targetRpm: jog.platterTargetRpm,
                    gain: Binding(get: { jog.effectiveTTNudgeGain }, set: { jog.ttNudgeGain = $0 }),
                    exponent: $jog.ttNudgeCurveExponent
                )
            }
            Text("Existing preferences keep their response. The earlier shared gain already included a 2x braking boost: 5x therefore appears as 10x braking and 5x nudge here.")
                .font(.caption).foregroundColor(.secondary)
            Text("Shape 1 keeps the current response. Higher values soften movement near target speed; lower values strengthen it. The quiet band and smooth onset stay fixed. These graphs show MIDI commands, not measured audio tempo. Save and restart to apply; no mapping export is needed.")
                .font(.caption).foregroundColor(.secondary)
        }
    }
}

struct TTSingleCurveEditorView: View {
    let direction: TTResponseDirection
    let targetRpm: Int
    @Binding var gain: Double
    @Binding var exponent: Double

    private var canPreview: Bool {
        (1...45).contains(targetRpm) && direction.gainRange.contains(gain)
            && TTResponseCurve.exponentRange.contains(exponent)
    }

    private func numberFormat(for value: Double) -> FloatingPointFormatStyle<Double> {
        if abs(value) >= 100 || (value != 0 && abs(value) < 0.01) {
            return .number.notation(.scientific).precision(.significantDigits(1...3)).grouping(.never)
        }
        return .number.grouping(.never)
    }

    private var samples: [TTResponseSample] {
        TTResponseCurve.samples(
            direction: direction,
            parameters: TTResponseParameters(
                targetRpm: targetRpm,
                brakeGain: gain,
                brakeExponent: exponent,
                nudgeGain: gain,
                nudgeExponent: exponent
            )
        )
    }

    var body: some View {
        GroupBox(direction.title) {
            VStack(alignment: .leading, spacing: 8) {
                HStack(spacing: 8) {
                    Text("Strength")
                    Slider(value: $gain, in: direction.gainRange, step: 0.1)
                        .accessibilityLabel("\(direction.title) strength")
                    TextField("\(direction.title) strength", value: $gain, format: numberFormat(for: gain))
                        .frame(width: 80).multilineTextAlignment(.trailing)
                    Text("x").foregroundColor(.secondary)
                    Stepper("\(direction.title) strength", value: $gain, in: direction.gainRange, step: 0.1)
                        .labelsHidden().fixedSize()
                }
                HStack(spacing: 8) {
                    Text("Shape")
                    Slider(value: $exponent, in: TTResponseCurve.exponentRange, step: 0.05)
                        .accessibilityLabel("\(direction.title) curve shape")
                    TextField("\(direction.title) curve shape", value: $exponent, format: numberFormat(for: exponent))
                        .frame(width: 80).multilineTextAlignment(.trailing)
                    Stepper("\(direction.title) curve shape", value: $exponent, in: TTResponseCurve.exponentRange, step: 0.05)
                        .labelsHidden().fixedSize()
                }
                if canPreview {
                    Text(TTResponseCurve.shapeDescription(for: exponent))
                        .font(.caption).foregroundColor(.secondary)
                    TTResponsePlotView(direction: direction, targetRpm: targetRpm, samples: samples)
                } else {
                    Text("Enter a valid strength and shape, with a target speed from 1 to 45 RPM, to preview the curve.")
                        .font(.caption).foregroundColor(.red)
                        .frame(height: 130)
                }
            }.padding(8).frame(maxWidth: .infinity, alignment: .leading)
        }
    }
}

struct TTResponsePlotView: View {
    let direction: TTResponseDirection
    let targetRpm: Int
    let samples: [TTResponseSample]

    private var maxMagnitude: Double { max(samples.map(\.magnitudeCc4Rate).max() ?? 0.0, 1.0) }
    private var peakSignedRate: Double { samples.last?.signedCc4Rate ?? 0.0 }
    private var quietFraction: Double { min(TTResponseCurve.quietBandRpm / Double(max(targetRpm, 1)), 1.0) }

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            VStack(alignment: .leading, spacing: 2) {
                Text("MIDI CC 4 bend rate (counts/s)")
                Text(String(format: "At %d RPM deviation: %+.0f counts/s", targetRpm, peakSignedRate))
            }.font(.caption).fixedSize(horizontal: false, vertical: true)
            GeometryReader { geometry in
                let size = geometry.size
                ZStack(alignment: .bottomLeading) {
                    RoundedRectangle(cornerRadius: 4).fill(Color(NSColor.controlBackgroundColor))
                    Path { path in
                        for fraction in [0.25, 0.5, 0.75] {
                            let y = size.height * (1.0 - fraction)
                            path.move(to: CGPoint(x: 0, y: y))
                            path.addLine(to: CGPoint(x: size.width, y: y))
                            let x = size.width * fraction
                            path.move(to: CGPoint(x: x, y: 0))
                            path.addLine(to: CGPoint(x: x, y: size.height))
                        }
                    }.stroke(Color.secondary.opacity(0.2), lineWidth: 1)
                    if quietFraction > 0 {
                        Rectangle().fill(Color.secondary.opacity(0.12))
                            .frame(width: size.width * quietFraction, height: size.height)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    Path { path in
                        guard let first = samples.first, let last = samples.last else { return }
                        path.move(to: CGPoint(x: point(for: first, in: size).x, y: size.height))
                        for sample in samples { path.addLine(to: point(for: sample, in: size)) }
                        path.addLine(to: CGPoint(x: point(for: last, in: size).x, y: size.height))
                        path.closeSubpath()
                    }.fill(Color.accentColor.opacity(0.16))
                    Path { path in
                        guard let first = samples.first else { return }
                        path.move(to: point(for: first, in: size))
                        for sample in samples.dropFirst() { path.addLine(to: point(for: sample, in: size)) }
                    }.stroke(Color.accentColor, style: StrokeStyle(lineWidth: 2, lineCap: .round, lineJoin: .round))
                }.overlay(RoundedRectangle(cornerRadius: 4).stroke(Color.secondary.opacity(0.3), lineWidth: 1))
            }
            .frame(height: 96)
            .accessibilityLabel("\(direction.title) response preview, peak \(String(format: "%.0f", peakSignedRate)) counts per second at \(targetRpm) RPM deviation")
            HStack {
                Text("0 RPM (0%)")
                Spacer()
                Text("\(direction == .braking ? "-" : "+")\(targetRpm) RPM (100%)")
            }.font(.caption)
            Text("Shaded: quiet within 2.25 RPM of target")
                .font(.caption).foregroundColor(.secondary)
        }
    }

    private func point(for sample: TTResponseSample, in size: CGSize) -> CGPoint {
        let xFraction = min(max(sample.deviationPercent / 100.0, 0.0), 1.0)
        let yFraction = min(max(sample.magnitudeCc4Rate / maxMagnitude, 0.0), 1.0)
        return CGPoint(x: xFraction * size.width, y: (1.0 - yFraction) * size.height)
    }
}
