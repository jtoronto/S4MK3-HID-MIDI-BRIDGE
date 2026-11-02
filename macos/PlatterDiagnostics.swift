import CoreFoundation
import Foundation
import SwiftUI

enum PlatterRoutingMode: String, Codable, CaseIterable, Equatable, Sendable {
    case off
    case waiting
    case bend
    case scratch
    case pausedScratch = "paused_scratch"
    case blocked
    case stale
    case seek
    case grid

    var label: String {
        switch self {
        case .off: return "Off"
        case .waiting: return "Waiting"
        case .bend: return "Bend"
        case .scratch: return "Scratch"
        case .pausedScratch: return "Paused scratch"
        case .blocked: return "Blocked"
        case .stale: return "Stale"
        case .seek: return "Seek"
        case .grid: return "Grid"
        }
    }
}

struct PlatterWheelStatus: Equatable, Sendable {
    let deck: Int
    let rpm: Double?
    let bendCountsPerSecond: Double
    let mode: PlatterRoutingMode

    var deckLabel: String {
        switch deck {
        case 1: return "Deck A (1)"
        case 2: return "Deck B (2)"
        case 3: return "Deck C (3)"
        case 4: return "Deck D (4)"
        default: return "Deck \(deck)"
        }
    }
}

struct PlatterStatusSnapshot: Equatable, Sendable {
    static let prefix = "S4_PLATTER_STATUS "
    static let staleInterval: TimeInterval = 0.5

    let left: PlatterWheelStatus
    let right: PlatterWheelStatus

    var wheels: [PlatterWheelStatus] { [left, right] }

    static func parseLine(_ line: String) -> PlatterStatusSnapshot? {
        guard line.hasPrefix(prefix) else { return nil }
        let payload = String(line.dropFirst(prefix.count))
        return try? decode(Data(payload.utf8))
    }

    static func decode(_ data: Data) throws -> PlatterStatusSnapshot {
        guard let root = try JSONSerialization.jsonObject(with: data) as? [String: Any],
              let rawWheels = root["wheels"] as? [Any],
              rawWheels.count == 2,
              let leftObject = rawWheels[0] as? [String: Any],
              let rightObject = rawWheels[1] as? [String: Any] else {
            throw AppError.invalid("Platter status requires a JSON object with exactly two wheel records.")
        }
        let left = try decodeWheel(leftObject, validDecks: [1, 3], sideName: "left")
        let right = try decodeWheel(rightObject, validDecks: [2, 4], sideName: "right")
        return PlatterStatusSnapshot(left: left, right: right)
    }

    private static func decodeWheel(_ object: [String: Any], validDecks: Set<Int>, sideName: String) throws -> PlatterWheelStatus {
        guard let deck = exactInteger(object["deck"]), validDecks.contains(deck) else {
            throw AppError.invalid("Invalid \(sideName) platter deck.")
        }
        guard let modeString = object["mode"] as? String,
              let mode = PlatterRoutingMode(rawValue: modeString) else {
            throw AppError.invalid("Invalid \(sideName) platter routing mode.")
        }
        guard let bendCountsPerSecond = finiteDouble(object["bend_counts_per_second"]) else {
            throw AppError.invalid("Invalid \(sideName) platter bend_counts_per_second.")
        }
        let rpm: Double?
        if let rawRpm = object["rpm"], !(rawRpm is NSNull) {
            guard let parsedRpm = finiteDouble(rawRpm) else {
                throw AppError.invalid("Invalid \(sideName) platter rpm.")
            }
            rpm = parsedRpm
        } else {
            rpm = nil
        }
        return PlatterWheelStatus(
            deck: deck,
            rpm: rpm,
            bendCountsPerSecond: bendCountsPerSecond,
            mode: mode
        )
    }

    private static func exactInteger(_ value: Any?) -> Int? {
        guard let number = value as? NSNumber,
              CFGetTypeID(number) != CFBooleanGetTypeID(),
              !["f", "d"].contains(String(cString: number.objCType)) else {
            return nil
        }
        return Int(exactly: number.int64Value)
    }

    private static func finiteDouble(_ value: Any?) -> Double? {
        guard let number = value as? NSNumber,
              CFGetTypeID(number) != CFBooleanGetTypeID() else {
            return nil
        }
        let doubleValue = number.doubleValue
        guard doubleValue.isFinite else { return nil }
        return doubleValue
    }
}

struct PlatterWheelPresentation: Equatable {
    let sideTitle: String
    let deckText: String
    let modeText: String
    let rpm: Double?
    let bendOutput: Double?
    let isStaleOrStopped: Bool

    var rpmText: String {
        rpm.map { PlatterDiagnosticsState.formatSigned($0, unit: "RPM") } ?? "—"
    }

    var bendOutputText: String {
        bendOutput.map { PlatterDiagnosticsState.formatSigned($0, unit: "counts/s") } ?? "—"
    }
}

struct PlatterDiagnosticsPresentation: Equatable {
    let left: PlatterWheelPresentation
    let right: PlatterWheelPresentation
    let isStale: Bool
}

struct PlatterDiagnosticsState: Equatable {
    private(set) var latest: PlatterStatusSnapshot?
    private(set) var receivedAt: Date?

    mutating func accept(_ snapshot: PlatterStatusSnapshot, at now: Date = Date()) {
        latest = snapshot
        receivedAt = now
    }

    @discardableResult
    mutating func acceptLine(_ line: String, at now: Date = Date()) -> Bool {
        guard let snapshot = PlatterStatusSnapshot.parseLine(line) else { return false }
        accept(snapshot, at: now)
        return true
    }

    mutating func reset() {
        latest = nil
        receivedAt = nil
    }

    func isSilenceStale(at now: Date, threshold: TimeInterval = PlatterStatusSnapshot.staleInterval) -> Bool {
        guard let receivedAt else { return false }
        return now.timeIntervalSince(receivedAt) > threshold
    }

    func presentation(bridgeRunning: Bool, at now: Date = Date(), staleThreshold: TimeInterval = PlatterStatusSnapshot.staleInterval) -> PlatterDiagnosticsPresentation {
        guard bridgeRunning else {
            return PlatterDiagnosticsPresentation(
                left: Self.stoppedWheel(sideTitle: "Left platter", defaultDecks: "Decks A / C"),
                right: Self.stoppedWheel(sideTitle: "Right platter", defaultDecks: "Decks B / D"),
                isStale: false
            )
        }
        guard let latest else {
            return PlatterDiagnosticsPresentation(
                left: Self.waitingWheel(sideTitle: "Left platter", defaultDecks: "Decks A / C"),
                right: Self.waitingWheel(sideTitle: "Right platter", defaultDecks: "Decks B / D"),
                isStale: false
            )
        }
        let silenceStale = isSilenceStale(at: now, threshold: staleThreshold)
        return PlatterDiagnosticsPresentation(
            left: Self.wheelPresentation(sideTitle: "Left platter", wheel: latest.left, silenceStale: silenceStale),
            right: Self.wheelPresentation(sideTitle: "Right platter", wheel: latest.right, silenceStale: silenceStale),
            isStale: silenceStale || latest.left.mode == .stale || latest.right.mode == .stale
        )
    }

    private static func stoppedWheel(sideTitle: String, defaultDecks: String) -> PlatterWheelPresentation {
        PlatterWheelPresentation(
            sideTitle: sideTitle,
            deckText: defaultDecks,
            modeText: "Stopped",
            rpm: nil,
            bendOutput: nil,
            isStaleOrStopped: true
        )
    }

    private static func waitingWheel(sideTitle: String, defaultDecks: String) -> PlatterWheelPresentation {
        PlatterWheelPresentation(
            sideTitle: sideTitle,
            deckText: defaultDecks,
            modeText: "Waiting",
            rpm: nil,
            bendOutput: nil,
            isStaleOrStopped: true
        )
    }

    private static func wheelPresentation(sideTitle: String, wheel: PlatterWheelStatus, silenceStale: Bool) -> PlatterWheelPresentation {
        if silenceStale {
            return PlatterWheelPresentation(
                sideTitle: sideTitle,
                deckText: wheel.deckLabel,
                modeText: PlatterRoutingMode.stale.label,
                rpm: nil,
                bendOutput: nil,
                isStaleOrStopped: true
            )
        }
        let isWheelStale = wheel.mode == .stale || wheel.rpm == nil
        return PlatterWheelPresentation(
            sideTitle: sideTitle,
            deckText: wheel.deckLabel,
            modeText: wheel.mode.label,
            rpm: isWheelStale ? nil : wheel.rpm,
            bendOutput: isWheelStale ? nil : wheel.bendCountsPerSecond,
            isStaleOrStopped: isWheelStale
        )
    }

    static func formatSigned(_ value: Double, unit: String) -> String {
        let normalized = abs(value) < 0.05 ? 0.0 : value
        let number: String
        if normalized == 0.0 {
            number = "0.0"
        } else {
            number = String(format: "%+.1f", normalized)
        }
        return "\(number) \(unit)"
    }
}

struct PlatterDiagnosticsGroupView: View {
    let diagnostics: PlatterDiagnosticsState
    let bridgeRunning: Bool
    var now: Date?

    var body: some View {
        GroupBox("Live platter diagnostics") {
            VStack(alignment: .leading, spacing: 8) {
                if let fixedNow = now {
                    content(for: diagnostics.presentation(bridgeRunning: bridgeRunning, at: fixedNow))
                } else if bridgeRunning && diagnostics.latest != nil {
                    TimelineView(.periodic(from: .now, by: 0.25)) { context in
                        content(for: diagnostics.presentation(bridgeRunning: bridgeRunning, at: context.date))
                    }
                } else {
                    content(for: diagnostics.presentation(bridgeRunning: bridgeRunning, at: Date()))
                }
                Text("Bend output shows outgoing MIDI CC 4 rate (counts/s), not measured audible Djay pitch or playback speed.")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            .padding(8)
            .frame(maxWidth: .infinity, alignment: .leading)
        }
    }

    @ViewBuilder
    private func content(for presentation: PlatterDiagnosticsPresentation) -> some View {
        HStack(alignment: .top, spacing: 12) {
            wheelColumn(presentation.left)
            Divider()
            wheelColumn(presentation.right)
        }
        .fixedSize(horizontal: false, vertical: true)
    }

    private func wheelColumn(_ wheel: PlatterWheelPresentation) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(wheel.sideTitle).font(.subheadline.weight(.semibold))
                Spacer()
                Text(wheel.deckText)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            metricRow(title: "Mode", value: wheel.modeText, muted: wheel.isStaleOrStopped)
            metricRow(title: "Measured speed", value: wheel.rpmText, muted: wheel.isStaleOrStopped)
            metricRow(title: "Bend output", value: wheel.bendOutputText, muted: wheel.isStaleOrStopped)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    private func metricRow(title: String, value: String, muted: Bool) -> some View {
        HStack {
            Text(title)
                .font(.caption)
                .foregroundColor(.secondary)
            Spacer()
            Text(value)
                .font(.system(.caption, design: .monospaced))
                .foregroundColor(muted ? .secondary : .primary)
        }
    }
}
