import SwiftUI

struct JogSettingsView: View {
    @ObservedObject var controller: BridgeController
    @State var showAdvanced = false

    var body: some View {
        ScrollView {
            VStack(spacing: 20) {
                motorized
                wheelFeel
                backspin
                mapping
            }.padding(20).frame(maxWidth: .infinity)
        }
    }

    private func hint(_ text: String) -> some View {
        Text(text).font(.caption).foregroundColor(.secondary)
            .fixedSize(horizontal: false, vertical: true)
            .frame(maxWidth: .infinity, alignment: .leading)
    }

    private func section<Content: View>(_ title: String, @ViewBuilder content: () -> Content) -> some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12, content: content).padding(12)
        } label: {
            Text(title).font(.headline)
        }
    }

    private var motorized: some View {
        section("Motorized platters") {
            HStack {
                Spacer()
                Button("Restore defaults") { controller.restoreMotorizedDefaults() }
            }
            HStack {
                Text("Base force")
                Spacer()
                Text(controller.jog.platterBaseForce, format: .number)
                    .monospacedDigit()
                Text("force units").foregroundColor(.secondary)
            }
            Slider(value: Binding(
                get: { Double(controller.jog.platterBaseForce) },
                set: { controller.jog.platterBaseForce = Int($0) }
            ), in: 0...25000, step: 50) {
                Text("Base force")
            }.labelsHidden()
            HStack {
                Text("Weak")
                Spacer()
                Text("Strong")
            }.font(.caption).foregroundColor(.secondary)
            hint("Sets the baseline motor drive, with speed-based drive and corrections added on top. Lower values give a gentler push; higher values give a stronger push.")
            Text("Pitch bend response").font(.headline).padding(.top, 8)
            TTResponseCurvesSectionView(jog: $controller.jog)
            Toggle("Advanced settings", isOn: $showAdvanced).toggleStyle(.checkbox)
            if showAdvanced {
                Divider()
                playbackAndTouch
                motorResponse
            }
        }
    }

    private var playbackAndTouch: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Playback and touch").font(.headline)
            NumberRow(title: "Target speed", value: $controller.jog.platterTargetRpm, range: 1...45, unit: "RPM")
            hint("The speed the motor aims to maintain during playback.")
            Toggle("Start normal mode with Vinyl enabled", isOn: $controller.jog.vinylEnabledAtStartup)
            hint("Starts normal mode with touch scratching enabled.")
            Toggle("Start motorized mode with Vinyl enabled", isOn: $controller.jog.ttVinylEnabledAtStartup)
            hint("Starts TT mode with touch scratching enabled.")
            NumberRow(title: "Switch to scratching below", value: $controller.jog.ttPitchBendFloorPercent, range: 1...95, unit: "% of target speed")
            hint("Changes from pitch bending to scratching when the platter slows below this share of its target speed.")
        }.padding(.top, 8)
    }

    private var motorResponse: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Motor response").font(.headline)
            NumberRow(title: "Maximum force", value: $controller.jog.platterMaxForce, range: 1...25000, step: 500, unit: "force units")
            hint("Limits the total motor drive, including base force and speed corrections.")
            NumberRow(title: "Speed-based force", value: $controller.jog.platterForcePerRpm, range: 0...25000, step: 10, unit: "force units/RPM")
            hint("Adds motor drive as the target speed rises.")
            NumberRow(title: "Speed correction", value: $controller.jog.platterKp, range: 0...25000, step: 100)
            hint("How strongly the motor reacts when the platter is above or below its target speed.")
            NumberRow(title: "Sustained speed correction", value: $controller.jog.platterKi, range: 0...25000, step: 100)
            hint("Adds correction when a speed difference persists, helping the platter settle at its target speed.")
            NumberRow(title: "Running force rise", value: $controller.jog.platterForceRisePerSecond, range: 0...1000000, step: 10000, unit: "force units/s")
            hint("How quickly motor strength can build after startup. A value of 0 prevents further increases.")
            NumberRow(title: "Startup duration", value: $controller.jog.platterStartupMs, range: 0...2000, step: 50, unit: "ms")
            hint("How long the starting response is used before switching to the running response. A value of 0 skips it.")
            NumberRow(title: "Startup force rise", value: $controller.jog.platterStartupForceRisePerSecond, range: 0...1000000, step: 10000, unit: "force units/s")
            hint("How quickly starting strength builds. A value of 0 applies the starting force immediately.")
            NumberRow(title: "Overspeed limit", value: $controller.jog.platterOverspeedLimitRpm, range: 0...250, unit: "RPM")
            hint("Stops motor drive if the platter exceeds this speed. A value of 0 turns this limit off.")
        }.padding(.top, 8)
    }

    private var wheelFeel: some View {
        section("Jog wheel feel") {
            HStack {
                Text("Wheel tension")
                Spacer()
                Text(controller.tension.tension, format: .number).monospacedDigit()
            }
            Slider(value: Binding(
                get: { Double(controller.tension.tension) },
                set: { controller.tension.tension = Int($0) }
            ), in: -100...100, step: 1) { Text("Wheel tension") }.labelsHidden()
            HStack {
                Text("Assist").frame(maxWidth: .infinity, alignment: .leading)
                Text("Neutral").frame(maxWidth: .infinity)
                Text("Brake").frame(maxWidth: .infinity, alignment: .trailing)
            }.font(.caption).foregroundColor(.secondary)
            hint("Adjusts the feel of both wheels. Assist helps them spin; Brake adds resistance.")
        }
    }

    private var backspin: some View {
        section("Backspin") {
            Toggle("Continue a detected backspin after release", isOn: $controller.jog.backspinEnabled)
            hint("Lets a quick reverse spin continue briefly after you let go.")
            HStack {
                Text("Detection threshold")
                Spacer()
                TextField("Detection threshold", value: $controller.jog.backspinAcceleration, format: .number.grouping(.never))
                    .frame(width: 90).multilineTextAlignment(.trailing)
                Text("counts/s²").foregroundColor(.secondary)
                Stepper("Detection threshold", value: $controller.jog.backspinAcceleration, in: 1000...Double.greatestFiniteMagnitude, step: 1000)
                    .labelsHidden().fixedSize()
            }.disabled(!controller.jog.backspinEnabled)
            hint("Higher values require a sharper backward flick to trigger a backspin.")
            HStack {
                Text("Extra scratch speed")
                Slider(value: $controller.jog.backspinSpeedIncrease, in: 0...10, step: 0.1)
                    .accessibilityLabel("Extra scratch speed")
                Text(controller.jog.backspinSpeedIncrease, format: .number.precision(.fractionLength(1)))
                    .frame(width: 35, alignment: .trailing)
                Text("%").foregroundColor(.secondary)
            }.disabled(!controller.jog.backspinEnabled)
            hint("Adds scratch speed during a detected backspin.")
            Toggle("Use motor assistance for backspins", isOn: $controller.jog.backspinAssistEnabled)
                .disabled(!controller.jog.backspinEnabled)
            hint("Helps backspins keep moving in normal mode. TT uses its own playback assistance.")
            HStack {
                Text("Motor assist strength")
                Slider(value: Binding(
                    get: { Double(controller.jog.backspinAssistStrength) },
                    set: { controller.jog.backspinAssistStrength = Int($0) }
                ), in: 0...100, step: 1).accessibilityLabel("Motor assist strength")
                Text("\(controller.jog.backspinAssistStrength)%").frame(width: 42, alignment: .trailing)
            }.disabled(!controller.jog.backspinEnabled || !controller.jog.backspinAssistEnabled)
            hint("Lower it for gentler follow-through; raise it for a stronger push.")
        }
    }

    private var mapping: some View {
        section("Scratch and pitch bend mapping") {
            DecimalRow(title: "Scratch speed", value: $controller.jog.scratchSpeed, range: 0.1...Double.greatestFiniteMagnitude, unit: "%")
            hint("How far the track moves when you turn a platter while touching its top.")
            NumberRow(title: "Scratch reaction", value: $controller.jog.scratchReaction, range: 0...150, unit: "%")
            hint("How immediately Djay follows your scratching movement.")
            Picker("Touch-scratch direction", selection: $controller.jog.scratchDirection) {
                Text("Normal").tag(ScratchDirection.normal)
                Text("Reversed").tag(ScratchDirection.reversed)
            }
            hint("Choose the track direction when you turn a touched platter forward.")
            DecimalRow(title: "Pitch bend speed", value: $controller.jog.pitchBendSpeed, range: 0.1...Double.greatestFiniteMagnitude, unit: "%")
            hint("How much an untouched platter movement speeds up or slows down the track.")
            NumberRow(title: "Pitch bend reaction", value: $controller.jog.pitchBendReaction, range: 0...150, unit: "%")
            hint("How quickly temporary speed changes respond to your hand.")
            Button("Install Djay mapping…") { controller.installMapping() }.disabled(controller.isExporting)
            hint("Install and select a new mapping in Djay after changing these controls.")
        }
    }
}
