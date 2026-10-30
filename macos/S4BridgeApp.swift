import AppKit
import Combine
import CoreFoundation
import Darwin
import SwiftUI
import UniformTypeIdentifiers

enum AppError: LocalizedError {
    case invalid(String)

    var errorDescription: String? {
        switch self {
        case .invalid(let message): return message
        }
    }
}

enum BridgeProfile: String, Codable, CaseIterable {
    case full, minimal, probe

    var title: String {
        switch self {
        case .full: return "Full controller + LED feedback"
        case .minimal: return "Minimal MIDI (Play, Cue, faders)"
        case .probe: return "HID probe (no MIDI)"
        }
    }
}

struct RunPreferences: Codable, Equatable {
    var profile: BridgeProfile = .full
    var limitDuration = false
    var durationSeconds = 3600
    var rawReports = false
    var logLevel = "info"

    func validate() throws {
        guard (1...604800).contains(durationSeconds) else {
            throw AppError.invalid("Run duration must be between 1 second and 7 days.")
        }
        guard ["warn", "info", "debug"].contains(logLevel) else {
            throw AppError.invalid("Choose a supported log level.")
        }
    }

    func arguments(ledURL: URL, jogURL: URL, tensionURL: URL, screens: Bool = true) -> [String] {
        var arguments: [String] = []
        if profile != .probe { arguments += ["--midi", profile.rawValue] }
        if profile == .full {
            arguments += ["--screen-control-stdin"]
            if screens { arguments += ["--screens"] }
            arguments += ["--led-config", ledURL.path, "--jog-config", jogURL.path, "--tension-config", tensionURL.path]
        }
        arguments += limitDuration ? ["--seconds", String(durationSeconds)] : ["--until-stopped"]
        if rawReports { arguments.append("--raw") }
        return arguments
    }
}

struct LedPreferences: Codable, Equatable {
    static let colors = [
        "red", "carrot", "orange", "honey", "yellow", "lime", "green", "aqua",
        "cyan", "sky", "blue", "purple", "fuchsia", "magenta", "azalea", "salmon", "white"
    ]

    var deckColors: [String]
    var stemColors: [String]
    var djayPadColors: [String]
    var djayWhite: String
    var hotcueColors: String
    var fixedHotcueColors: [String]
    var activeBrightness: Int
    var inactiveBrightness: Int
    var inactiveDisplay: String
    var paletteActiveIntensity: Int
    var paletteInactiveIntensity: Int
    var stemMuteBright: Bool
    var muteColor: String
    var quantizeMixed: String
    var tempoCenterTolerance: Double?
    var loopEnabled: Bool
    var loopColor: String
    var chasePeriodMs: Int
    var meterBrightness: Int
    var meterGamma: Double
    var meterStaleMs: Int

    static func decode(_ data: Data, defaults: Data) throws -> LedPreferences {
        guard let base = try JSONSerialization.jsonObject(with: defaults) as? [String: Any],
              let overrides = try JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            throw AppError.invalid("LED preferences must be a JSON object.")
        }
        let unknown = Set(overrides.keys).subtracting(base.keys)
        guard unknown.isEmpty else {
            throw AppError.invalid("Unknown LED settings: \(unknown.sorted().joined(separator: ", ")).")
        }
        let merged = base.merging(overrides) { _, override in override }
        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let value = try decoder.decode(LedPreferences.self, from: JSONSerialization.data(withJSONObject: merged))
        try value.validate()
        return value
    }

    func encoded() throws -> Data {
        try validate()
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        return try encoder.encode(self)
    }

    func validate() throws {
        for (values, count, label) in [
            (deckColors, 4, "Deck colors"), (stemColors, 4, "Stem colors"),
            (djayPadColors, 8, "Djay pad colors"), (fixedHotcueColors, 8, "Fixed hotcue colors")
        ] {
            guard values.count == count, values.allSatisfy(Self.colors.contains) else {
                throw AppError.invalid("\(label) requires \(count) supported colors.")
            }
        }
        guard [djayWhite, muteColor, loopColor].allSatisfy(Self.colors.contains) else {
            throw AppError.invalid("Choose supported white, mute, and loop colors.")
        }
        guard ["djay", "fixed_slots"].contains(hotcueColors),
              ["dim", "dark"].contains(inactiveDisplay),
              ["dim", "dark"].contains(quantizeMixed) else {
            throw AppError.invalid("Unknown LED presentation mode.")
        }
        guard (1...127).contains(activeBrightness), (0...127).contains(inactiveBrightness),
              (0...3).contains(paletteActiveIntensity), (0...3).contains(paletteInactiveIntensity) else {
            throw AppError.invalid("Lamp brightness is 1–127 active / 0–127 inactive; palette intensity is 0–3.")
        }
        if let tolerance = tempoCenterTolerance {
            guard tolerance.isFinite, tolerance > 0, tolerance <= 0.1 else {
                throw AppError.invalid("Tempo-center tolerance must be greater than 0 and at most 0.1.")
            }
        }
        guard (250...10000).contains(chasePeriodMs), [0, 125, 127].contains(meterBrightness),
              meterGamma.isFinite, (0.1...5).contains(meterGamma),
              (100...5000).contains(meterStaleMs) else {
            throw AppError.invalid("Chase: 250–10000 ms. Meter brightness: 0/125/127, curve: 0.1–5, freshness: 100–5000 ms.")
        }
    }
}

enum ScratchDirection: String, Codable, CaseIterable, Identifiable {
    case normal
    case reversed

    var id: String { rawValue }
}

struct JogPreferences: Codable, Equatable {
    var scratchSpeed = 2.7
    var scratchReaction = 150
    var scratchDirection: ScratchDirection = .normal
    var pitchBendSpeed = 2.7
    var pitchBendReaction = 17
    var backspinEnabled = true
    var backspinAcceleration = 150000.0
    var backspinSpeedIncrease = 5.6
    var backspinAssistEnabled = true
    var backspinAssistStrength = 100
    var vinylEnabledAtStartup = true
    var ttVinylEnabledAtStartup = false
    var ttPitchBendFloorPercent = 50
    var platterTargetRpm = 33
    var platterPositionSensitivity = 14.0
    var platterMaxForce = 20000
    var platterBaseForce = 1650
    var platterForcePerRpm = 110
    var platterKp = 4000
    var platterKi = 3000
    var platterForceRisePerSecond = 200000
    var platterStartupForceRisePerSecond = 0
    var platterStartupMs = 200
    var platterOverspeedLimitRpm = 0

    private static let integerFields: [(key: String, label: String, range: ClosedRange<Int>)] = [
        ("scratch_reaction", "Scratch reaction", 0...150),
        ("pitch_bend_reaction", "Pitch bend reaction", 0...150),
        ("backspin_assist_strength", "Backspin assist strength", 0...100),
        ("tt_pitch_bend_floor_percent", "TT pitch-bend floor", 1...95),
        ("platter_target_rpm", "Platter target RPM", 1...45),
        ("platter_max_force", "Platter maximum force", 1...25000),
        ("platter_base_force", "Platter base force", 0...25000),
        ("platter_force_per_rpm", "Platter force per RPM", 0...25000),
        ("platter_kp", "Platter proportional gain (Kp)", 0...25000),
        ("platter_ki", "Platter integral gain (Ki)", 0...25000),
        ("platter_force_rise_per_second", "Platter force rise rate", 0...1000000),
        ("platter_startup_force_rise_per_second", "Platter startup force rise rate", 0...1000000),
        ("platter_startup_ms", "Platter startup duration", 0...2000),
        ("platter_overspeed_limit_rpm", "Platter overspeed limit", 0...250)
    ]

    static func decode(_ data: Data, defaults: Data) throws -> JogPreferences {
        guard let defaultValues = try JSONSerialization.jsonObject(with: JogPreferences().encoded()) as? [String: Any],
              let base = try JSONSerialization.jsonObject(with: defaults) as? [String: Any],
              let overrides = try JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            throw AppError.invalid("Jog preferences must be a JSON object.")
        }
        let allowed = Set(base.keys).union(defaultValues.keys)
        for object in [base, overrides] {
            let unknown = Set(object.keys).subtracting(allowed)
            guard unknown.isEmpty else {
                throw AppError.invalid("Unknown jog settings: \(unknown.sorted().joined(separator: ", ")).")
            }
            for (key, label, range) in integerFields {
                if let raw = object[key] {
                    guard let number = raw as? NSNumber,
                          CFGetTypeID(number) != CFBooleanGetTypeID(),
                          !["f", "d"].contains(String(cString: number.objCType)) else {
                        throw AppError.invalid("\(label) must be an integer from \(range.lowerBound) to \(range.upperBound).")
                    }
                }
            }
        }
        var merged = defaultValues.merging(base) { _, baseValue in baseValue }
        merged.merge(overrides) { _, override in override }
        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let value = try decoder.decode(JogPreferences.self, from: JSONSerialization.data(withJSONObject: merged))
        try value.validate()
        return value
    }

    func encoded() throws -> Data {
        try validate()
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        return try encoder.encode(self)
    }

    func validate() throws {
        for speed in [scratchSpeed, pitchBendSpeed] {
            guard speed.isFinite, speed > 0 else {
                throw AppError.invalid("Jog speeds must be finite and greater than 0.")
            }
        }
        guard platterPositionSensitivity.isFinite, (0.1...100).contains(platterPositionSensitivity) else {
            throw AppError.invalid("Motorized platter sensitivity must be between 0.1 and 100.")
        }
        guard (0...150).contains(scratchReaction), (0...150).contains(pitchBendReaction) else {
            throw AppError.invalid("Jog reactions must be integers from 0 to 150.")
        }
        guard backspinAcceleration.isFinite, backspinAcceleration > 0 else {
            throw AppError.invalid("Backspin acceleration must be finite and greater than 0.")
        }
        guard backspinSpeedIncrease.isFinite, backspinSpeedIncrease >= 0,
              (scratchSpeed + backspinSpeedIncrease).isFinite else {
            throw AppError.invalid("Backspin speed increase must be finite and at least 0.")
        }
        guard (0...100).contains(backspinAssistStrength) else {
            throw AppError.invalid("Backspin assist strength must be an integer from 0 to 100.")
        }
        guard (1...95).contains(ttPitchBendFloorPercent) else {
            throw AppError.invalid("TT pitch-bend floor must be an integer from 1 to 95.")
        }
        guard (1...45).contains(platterTargetRpm) else {
            throw AppError.invalid("Platter target RPM must be an integer from 1 to 45.")
        }
        guard (1...25000).contains(platterMaxForce) else {
            throw AppError.invalid("Platter maximum force must be an integer from 1 to 25000.")
        }
        for (value, label) in [
            (platterBaseForce, "Platter base force"),
            (platterForcePerRpm, "Platter force per RPM"),
            (platterKp, "Platter proportional gain (Kp)"),
            (platterKi, "Platter integral gain (Ki)")
        ] {
            guard (0...25000).contains(value) else {
                throw AppError.invalid("\(label) must be an integer from 0 to 25000.")
            }
        }
        for (value, label) in [
            (platterForceRisePerSecond, "Platter force rise rate"),
            (platterStartupForceRisePerSecond, "Platter startup force rise rate")
        ] {
            guard (0...1000000).contains(value) else {
                throw AppError.invalid("\(label) must be an integer from 0 to 1000000.")
            }
        }
        guard (0...2000).contains(platterStartupMs) else {
            throw AppError.invalid("Platter startup duration must be an integer from 0 to 2000.")
        }
        guard (0...250).contains(platterOverspeedLimitRpm) else {
            throw AppError.invalid("Platter overspeed limit must be an integer from 0 to 250.")
        }
    }
}

struct TensionPreferences: Codable, Equatable {
    var tension = 0

    static func decode(_ data: Data, defaults: Data) throws -> TensionPreferences {
        guard let base = try JSONSerialization.jsonObject(with: defaults) as? [String: Any],
              let overrides = try JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            throw AppError.invalid("Hardware tension preferences must be a JSON object.")
        }
        for object in [base, overrides] {
            let unknown = Set(object.keys).subtracting(["tension"])
            guard unknown.isEmpty else {
                throw AppError.invalid("Unknown hardware tension settings: \(unknown.sorted().joined(separator: ", ")).")
            }
            if let raw = object["tension"] {
                // JSONDecoder accepts integral floating-point tokens such as 1.0.
                // Reject them before re-encoding to match the runtime's integer schema.
                guard let number = raw as? NSNumber,
                      CFGetTypeID(number) != CFBooleanGetTypeID(),
                      !["f", "d"].contains(String(cString: number.objCType)) else {
                    throw AppError.invalid("Hardware tension must be an integer from -100 to 100.")
                }
            }
        }
        let merged = base.merging(overrides) { _, override in override }
        let value = try JSONDecoder().decode(TensionPreferences.self, from: JSONSerialization.data(withJSONObject: merged))
        try value.validate()
        return value
    }

    func encoded() throws -> Data {
        try validate()
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        return try encoder.encode(self)
    }

    func validate() throws {
        guard (-100...100).contains(tension) else {
            throw AppError.invalid("Hardware tension must be an integer from -100 to 100.")
        }
    }
}

enum BridgeState: String {
    case stopped = "Stopped"
    case starting = "Starting"
    case running = "Running"
    case stopping = "Stopping"
    case checking = "Checking controller"
    case failed = "Failed"
}

final class OutputBuffer: @unchecked Sendable {
    private let lock = NSLock()
    private var bytes = Data()
    private var scheduled = false

    func append(_ line: String) -> Bool {
        lock.lock()
        defer { lock.unlock() }
        bytes.append(contentsOf: line.utf8)
        bytes.append(10)
        if bytes.count > 60000 { bytes.removeFirst(bytes.count - 60000) }
        let shouldSchedule = !scheduled
        scheduled = true
        return shouldSchedule
    }

    func drain() -> String {
        lock.lock()
        defer { lock.unlock() }
        let text = String(decoding: bytes, as: UTF8.self)
        bytes.removeAll(keepingCapacity: true)
        scheduled = false
        return text
    }
}

@MainActor
final class BridgeController: ObservableObject {
    @Published var run = RunPreferences()
    @Published var leds: LedPreferences
    @Published var jog: JogPreferences
    @Published var tension: TensionPreferences
    @Published private(set) var state: BridgeState = .stopped
    @Published private(set) var log = ""
    @Published private(set) var motorDiagnosticURL: URL?
    @Published private(set) var isCapturingMotorDiagnostic = false
    @Published private(set) var message = ""
    @Published private(set) var errorMessage = ""
    @Published private(set) var requiresReset = false
    @Published private(set) var isExporting = false

    let resourceURL: URL
    let supportURL: URL
    let defaults: Data
    let jogDefaults: Data
    let tensionDefaults: Data
    let screens: ScreenSupport
    private var screenObservation: AnyCancellable?
    private var screenInput: FileHandle?
    private var launchID: UUID?
    private var invalidSavedTension = false
    private var process: Process?
    private var motorDiagnosticFileHandle: FileHandle?
    private var activeLogLevel: String?
    private var activeProfile: BridgeProfile?
    private var isDiagnostic = false
    private var stopRequested = false
    private var restartRequested = false
    private var quitRequested = false
    var replyToQuit: (Bool) -> Void = { NSApp.reply(toApplicationShouldTerminate: $0) }
    var screenConsent: () -> ScreenConsent = ScreenPrompts.confirmStop
    var confirmDriverRestore: () async -> Bool = { ScreenPrompts.confirmStopBridge() }

    var isBusy: Bool { process != nil || screens.busy }
    var hasChildProcess: Bool { process != nil }
    var canStop: Bool { process != nil && state != .stopping && !screens.busy }
    var canStartMotorDiagnosticCapture: Bool {
        if process == nil { return run.profile == .full && run.logLevel == "debug" }
        return !isDiagnostic && activeProfile == .full && activeLogLevel == "debug"
    }
    var ledURL: URL { supportURL.appendingPathComponent("led-config.json") }
    var jogURL: URL { supportURL.appendingPathComponent("jog-config.json") }
    var tensionURL: URL { supportURL.appendingPathComponent("tension-config.json") }
    var runURL: URL { supportURL.appendingPathComponent("run-settings.json") }
    var motorDiagnosticDirectoryURL: URL {
        supportURL.appendingPathComponent("diagnostics", isDirectory: true)
    }
    var executableURL: URL { resourceURL.appendingPathComponent("s4mk3-hid-midi-bridge") }
    var commandPreview: String {
        ([executableURL.path] + run.arguments(ledURL: ledURL, jogURL: jogURL, tensionURL: tensionURL, screens: screens.preferences.enabled))
            .map { "'" + $0.replacingOccurrences(of: "'", with: "'\\''") + "'" }
            .joined(separator: " ")
    }

    private static let motorDiagnosticEvents = [
        "MOTOR_JOG_BEND",
        "MOTOR_JOG_POSITION",
        "MOTOR_SCRATCH_MODE",
        "MOTOR_SCRATCH_HANDOFF",
        "MOTOR_TT_SELECTION",
        "MOTOR_PLAYBACK_GATE"
    ]

    init(resourceURL: URL, supportURL: URL, agent: NIHardwareAgentControlling? = nil) throws {
        self.resourceURL = resourceURL
        self.supportURL = supportURL
        screens = ScreenSupport(supportURL: supportURL, agent: agent ?? NIHardwareAgent())
        defaults = try Data(contentsOf: resourceURL.appendingPathComponent("default-led-config.json"))
        leds = try LedPreferences.decode(defaults, defaults: defaults)
        jogDefaults = try Data(contentsOf: resourceURL.appendingPathComponent("default-jog-config.json"))
        jog = try JogPreferences.decode(jogDefaults, defaults: jogDefaults)
        tensionDefaults = try Data(contentsOf: resourceURL.appendingPathComponent("default-tension-config.json"))
        tension = try TensionPreferences.decode(tensionDefaults, defaults: tensionDefaults)
        do {
            if FileManager.default.fileExists(atPath: runURL.path) {
                run = try JSONDecoder().decode(RunPreferences.self, from: Data(contentsOf: runURL))
                try run.validate()
            }
            if FileManager.default.fileExists(atPath: ledURL.path) {
                leds = try LedPreferences.decode(Data(contentsOf: ledURL), defaults: defaults)
            }
            if FileManager.default.fileExists(atPath: jogURL.path) {
                jog = try JogPreferences.decode(Data(contentsOf: jogURL), defaults: jogDefaults)
            }
        } catch {
            requiresReset = true
            errorMessage = "Saved preferences could not be loaded: \(error.localizedDescription). Import valid settings or reset preferences before saving."
        }
        do {
            if FileManager.default.fileExists(atPath: tensionURL.path) {
                tension = try TensionPreferences.decode(Data(contentsOf: tensionURL), defaults: tensionDefaults)
            }
        } catch {
            invalidSavedTension = true
            requiresReset = true
            errorMessage = "Saved hardware tension could not be loaded: \(error.localizedDescription). Reset preferences before saving."
        }
        if let error = screens.loadError {
            requiresReset = true
            errorMessage = "Saved screen preferences could not be loaded: \(error.localizedDescription). Reset preferences before saving."
        }
        screenObservation = screens.objectWillChange.sink { [weak self] in
            self?.objectWillChange.send()
        }
    }

    func appendLog(_ text: String) {
        log = String((log + text + "\n").suffix(60000))
        guard let handle = motorDiagnosticFileHandle else { return }
        let capturedLines = text.split(whereSeparator: \.isNewline).filter { line in
            Self.motorDiagnosticEvents.contains { line.contains($0) }
        }
        guard !capturedLines.isEmpty else { return }
        do {
            let data = Data((capturedLines.joined(separator: "\n") + "\n").utf8)
            try handle.write(contentsOf: data)
        } catch {
            motorDiagnosticFileHandle = nil
            isCapturingMotorDiagnostic = false
            try? handle.close()
            errorMessage = "Motor diagnostic file write failed: \(error.localizedDescription)"
        }
    }

    func startMotorDiagnosticCapture() {
        guard !isCapturingMotorDiagnostic else { return }
        guard canStartMotorDiagnosticCapture else {
            errorMessage = "Set Log detail to Debug and start or restart the bridge before capturing motor positions."
            return
        }
        let startedAt = ISO8601DateFormatter().string(from: Date())
        let safeTimestamp = startedAt.replacingOccurrences(of: ":", with: "-")
        let filename = "motor-diagnostic-\(safeTimestamp)-\(UUID().uuidString.prefix(8)).log"
        let url = motorDiagnosticDirectoryURL.appendingPathComponent(filename)
        let fileManager = FileManager.default
        var openedHandle: FileHandle?
        do {
            try fileManager.createDirectory(
                at: motorDiagnosticDirectoryURL,
                withIntermediateDirectories: true
            )
            guard fileManager.createFile(atPath: url.path, contents: nil) else {
                throw AppError.invalid("Could not create the motor diagnostic file.")
            }
            let handle = try FileHandle(forWritingTo: url)
            openedHandle = handle
            let header = """
            # S4 MK3 Bridge motor diagnostic
            # Started: \(startedAt)
            # Captured events: \(Self.motorDiagnosticEvents.joined(separator: ", "))

            """
            try handle.write(contentsOf: Data(header.utf8))
            motorDiagnosticFileHandle = handle
            motorDiagnosticURL = url
            isCapturingMotorDiagnostic = true
            errorMessage = ""
            message = "Motor diagnostic capture started."
        } catch {
            try? openedHandle?.close()
            errorMessage = "Could not start motor diagnostic capture: \(error.localizedDescription)"
        }
    }

    func stopMotorDiagnosticCapture() {
        guard let handle = motorDiagnosticFileHandle else { return }
        motorDiagnosticFileHandle = nil
        isCapturingMotorDiagnostic = false
        do {
            let stoppedAt = ISO8601DateFormatter().string(from: Date())
            try handle.write(contentsOf: Data("# Stopped: \(stoppedAt)\n".utf8))
            try handle.synchronize()
            try handle.close()
            if let motorDiagnosticURL {
                message = "Motor diagnostic saved to \(motorDiagnosticURL.path)"
            }
            errorMessage = ""
        } catch {
            try? handle.close()
            errorMessage = "Could not finish motor diagnostic capture: \(error.localizedDescription)"
        }
    }

    func revealMotorDiagnostic() {
        guard let motorDiagnosticURL else { return }
        NSWorkspace.shared.activateFileViewerSelecting([motorDiagnosticURL])
    }

    func report(_ error: Error) {
        errorMessage = error.localizedDescription
        appendLog("APP_ERROR: \(error.localizedDescription)")
    }

    func save() throws {
        guard !requiresReset else {
            throw AppError.invalid("Reset the invalid saved preferences before saving.")
        }
        try run.validate()
        let ledData = try leds.encoded()
        let jogData = try jog.encoded()
        let tensionData = try tension.encoded()
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        let runData = try encoder.encode(run)
        try screens.save()
        try FileManager.default.createDirectory(at: supportURL, withIntermediateDirectories: true)
        try ledData.write(to: ledURL, options: .atomic)
        try jogData.write(to: jogURL, options: .atomic)
        try tensionData.write(to: tensionURL, options: .atomic)
        try runData.write(to: runURL, options: .atomic)
        errorMessage = ""
    }

    func apply() {
        guard !screens.busy else { return }
        do {
            try save()
            message = "Preferences saved."
            if isBusy && !isDiagnostic { stop(restart: true) }
        } catch { report(error) }
    }

    func start() {
        guard !isBusy else { return }
        do {
            try save()
        } catch {
            state = .failed
            report(error)
            return
        }
        state = .starting
        screens.beginOperation()
        Task { @MainActor in
            defer { finishScreenOperation() }
            do {
                let enabled = try await screens.prepare(full: run.profile == .full, consent: screenConsent)
                guard let enabled, !quitRequested else {
                    try await screens.restoreOwnedDriver()
                    state = .stopped
                    return
                }
                screens.bridgeStarted(screens: enabled, full: run.profile == .full)
                try launch(arguments: run.arguments(
                    ledURL: ledURL, jogURL: jogURL, tensionURL: tensionURL, screens: enabled
                ), diagnostic: false)
            } catch {
                await screens.bridgeExited(restarting: false)
                state = .failed
                report(error)
            }
        }
    }

    func checkController() {
        guard !isBusy else { return }
        do { try launch(arguments: ["--list"], diagnostic: true) }
        catch {
            state = .failed
            report(error)
        }
    }

    private func launch(arguments: [String], diagnostic: Bool) throws {
        let child = Process()
        let pipe = Pipe()
        let token = UUID()
        launchID = token
        child.executableURL = executableURL
        child.arguments = arguments
        var environment = ProcessInfo.processInfo.environment
        environment["RUST_LOG"] = run.logLevel
        child.environment = environment
        activeLogLevel = run.logLevel
        activeProfile = run.profile
        let input = try screenControlInput(for: child, diagnostic: diagnostic)
        child.standardOutput = pipe.fileHandleForWriting
        child.standardError = pipe.fileHandleForWriting
        isDiagnostic = diagnostic
        stopRequested = false
        restartRequested = false
        errorMessage = ""
        message = ""
        process = child
        state = diagnostic ? .checking : .starting
        let output = OutputBuffer()
        let reader = readOutput(pipe: pipe, output: output, token: token)
        child.terminationHandler = { [weak self] completed in
            Task { @MainActor in
                await reader.value
                self?.flush(output)
                await self?.completed(completed)
            }
        }
        appendLog("LAUNCH: \(arguments.joined(separator: " "))")
        do {
            try child.run()
            try input?.fileHandleForReading.close()
            try pipe.fileHandleForWriting.close()
            if !diagnostic { state = .running }
        } catch {
            if child.isRunning {
                report(error)
                stop()
                return
            }
            child.terminationHandler = nil
            if !child.isRunning {
                process = nil
                activeLogLevel = nil
                activeProfile = nil
                closeScreenInput()
                try input?.fileHandleForReading.close()
                try pipe.fileHandleForWriting.close()
            }
            throw error
        }
    }

    private func flush(_ output: OutputBuffer) {
        let text = output.drain()
        if !text.isEmpty { appendLog(text) }
    }

    func stop(restart: Bool = false) {
        guard let child = process, state != .stopping else { return }
        restartRequested = restart
        stopRequested = true
        state = .stopping
        guard child.isRunning else { return }
        if kill(child.processIdentifier, SIGINT) != 0 {
            restartRequested = false
            stopRequested = false
            report(AppError.invalid("Could not send the bridge its stop signal: \(String(cString: strerror(errno)))."))
            state = .failed
            if quitRequested {
                quitRequested = false
                replyToQuit(false)
            }
        }
    }

    private func screenControlInput(for child: Process, diagnostic: Bool) throws -> Pipe? {
        let input = !diagnostic && run.profile == .full ? Pipe() : nil
        if let input {
            guard fcntl(input.fileHandleForWriting.fileDescriptor, F_SETNOSIGPIPE, 1) != -1 else {
                throw AppError.invalid("Could not configure the screen control pipe.")
            }
            child.standardInput = input.fileHandleForReading
            screenInput = input.fileHandleForWriting
            screens.sendCommand = { [weak self] generation, command in
                guard let writer = self?.screenInput else {
                    throw AppError.invalid("The bridge's screen control connection is closed.")
                }
                try writer.write(contentsOf: Data("S4_SCREEN_CONTROL \(generation) \(command.rawValue)\n".utf8))
            }
        } else {
            child.standardInput = FileHandle.nullDevice
        }
        return input
    }

    private func readOutput(pipe: Pipe, output: OutputBuffer, token: UUID) -> Task<Void, Never> {
        let deliver: @Sendable () -> Void = { [weak self] in
            DispatchQueue.main.async { self?.flush(output) }
        }
        let reportReadError: @Sendable (Error) -> Void = { [weak self] error in
            DispatchQueue.main.async { self?.report(error) }
        }
        let screenStatus: @Sendable (String) -> Void = { [weak self] line in
            DispatchQueue.main.async {
                guard let self, self.launchID == token else { return }
                self.screens.acceptStatus(line)
            }
        }
        return Task.detached {
            do {
                for try await line in pipe.fileHandleForReading.bytes.lines {
                    if line.hasPrefix("S4_SCREEN_STATUS ") { screenStatus(line) }
                    if output.append(line) { deliver() }
                }
                try pipe.fileHandleForReading.close()
            } catch { reportReadError(error) }
        }
    }


    private func closeScreenInput() {
        if let input = screenInput {
            do { try input.close() }
            catch { report(error) }
        }
        screenInput = nil
        launchID = nil
        screens.sendCommand = nil
    }

    private func completed(_ child: Process) async {
        guard process === child else { return }
        let normalStop = child.terminationStatus == 0 ||
            (stopRequested && child.terminationReason == .uncaughtSignal && child.terminationStatus == SIGINT)
        let restart = restartRequested && normalStop
        let wasDiagnostic = isDiagnostic
        process = nil
        activeLogLevel = nil
        activeProfile = nil
        closeScreenInput()
        restartRequested = false
        screens.beginOperation()
        if !wasDiagnostic || screens.restoreAfterExit {
            await screens.bridgeExited(restarting: restart && !quitRequested)
        }
        screens.endOperation()
        state = normalStop ? .stopped : .failed
        appendLog("EXIT: \(child.terminationStatus)")
        if !normalStop {
            errorMessage = "Bridge exited with status \(child.terminationStatus). See Diagnostics for the error."
        } else if wasDiagnostic {
            message = "Controller check complete. See Diagnostics for USB results."
        } else {
            message = "Bridge stopped."
        }
        if quitRequested && !screens.busy {
            stopMotorDiagnosticCapture()
            replyToQuit(true)
        } else if restart {
            start()
        }
    }

    func prepareToQuit() -> NSApplication.TerminateReply {
        guard isBusy || (screens.preferences.restorePending && screens.preferences.restoreNIOnStop) else {
            stopMotorDiagnosticCapture()
            return .terminateNow
        }
        quitRequested = true
        guard !screens.busy else { return .terminateLater }
        if process == nil && !screens.busy {
            screens.beginOperation()
            Task { @MainActor in
                await screens.bridgeExited(restarting: false)
                screens.endOperation()
                stopMotorDiagnosticCapture()
                replyToQuit(true)
            }
        } else if process != nil && state != .stopping { stop() }
        return quitRequested ? .terminateLater : .terminateCancel
    }

    func finishScreenOperation() {
        screens.endOperation()
        guard quitRequested && !screens.busy else { return }
        if process != nil {
            if state != .stopping { stop() }
        } else {
            screens.beginOperation()
            Task { @MainActor in
                await screens.bridgeExited(restarting: false)
                screens.endOperation()
                stopMotorDiagnosticCapture()
                replyToQuit(true)
            }
        }
    }

    func resetPreferences() {
        do {
            leds = try LedPreferences.decode(defaults, defaults: defaults)
            jog = try JogPreferences.decode(jogDefaults, defaults: jogDefaults)
            tension = try TensionPreferences.decode(tensionDefaults, defaults: tensionDefaults)
            run = RunPreferences()
            screens.reset()
            invalidSavedTension = false
            requiresReset = false
            errorMessage = ""
            message = "Defaults restored in the editor. Save to apply."
        } catch { report(error) }
    }

    func importLEDs() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [.json]
        panel.canChooseDirectories = false
        panel.allowsMultipleSelection = false
        guard panel.runModal() == .OK, let url = panel.url else { return }
        do {
            let imported = try LedPreferences.decode(Data(contentsOf: url), defaults: defaults)
            leds = imported
            if !invalidSavedTension {
                if requiresReset { run = RunPreferences() }
                requiresReset = false
                errorMessage = ""
            }
            message = "LED settings imported into the editor. Save to apply."
        } catch { report(error) }
    }

    func exportLEDs() {
        let panel = NSSavePanel()
        panel.allowedContentTypes = [.json]
        panel.nameFieldStringValue = "s4-led-config.json"
        guard panel.runModal() == .OK, let url = panel.url else { return }
        do {
            try leds.encoded().write(to: url, options: .atomic)
            message = "LED settings exported."
        } catch { report(error) }
    }

    func showPreferences() {
        do {
            try FileManager.default.createDirectory(at: supportURL, withIntermediateDirectories: true)
            guard NSWorkspace.shared.open(supportURL) else {
                throw AppError.invalid("Could not open the preferences folder in Finder.")
            }
        } catch { report(error) }
    }

    /// Generates a Djay mapping from the current jog settings and installs it
    /// at `destination`. The Rust generator refuses to overwrite, so output
    /// always goes to an owned fresh temporary path first; `destination` may
    /// already exist after the save-panel overwrite confirmation. Runs
    /// asynchronously without blocking the main actor, never touches the
    /// source bridge `process`, and cleans up its temporary directory.
    func exportMapping(to destination: URL) async throws {
        guard !isExporting else {
            throw AppError.invalid("Mapping export already in progress.")
        }
        isExporting = true
        defer { isExporting = false }
        let config = try jog.encoded()
        let temporary = FileManager.default.temporaryDirectory
            .appendingPathComponent("S4BridgeMapping-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: temporary, withIntermediateDirectories: true)
        defer {
            do { try FileManager.default.removeItem(at: temporary) }
            catch { report(error) }
        }
        let configURL = temporary.appendingPathComponent("jog-config.json")
        let outputURL = temporary.appendingPathComponent("S4 MK3 Bridge.djayMidiMapping")
        try config.write(to: configURL, options: .atomic)
        let status = try await runExportProcess(arguments: [
            "--generate-mapping", outputURL.path, "--jog-config", configURL.path
        ])
        guard status == 0 else {
            throw AppError.invalid("Mapping export failed with status \(status). See Diagnostics for the error.")
        }
        guard FileManager.default.fileExists(atPath: outputURL.path) else {
            throw AppError.invalid("Mapping export produced no file. See Diagnostics for the error.")
        }
        try FileManager.default.createDirectory(at: destination.deletingLastPathComponent(), withIntermediateDirectories: true)
        try Data(contentsOf: outputURL).write(to: destination, options: .atomic)
    }

    private func runExportProcess(arguments: [String]) async throws -> Int32 {
        let child = Process()
        let pipe = Pipe()
        child.executableURL = executableURL
        child.arguments = arguments
        child.standardInput = FileHandle.nullDevice
        child.standardOutput = pipe.fileHandleForWriting
        child.standardError = pipe.fileHandleForWriting
        appendLog("EXPORT: \(arguments.joined(separator: " "))")
        let output = OutputBuffer()
        let deliver: @Sendable () -> Void = { [weak self] in
            DispatchQueue.main.async { self?.flush(output) }
        }
        let reportReadError: @Sendable (Error) -> Void = { [weak self] error in
            DispatchQueue.main.async { self?.report(error) }
        }
        return try await withCheckedThrowingContinuation { continuation in
            let reader = Task.detached {
                do {
                    for try await line in pipe.fileHandleForReading.bytes.lines {
                        if output.append(line) { deliver() }
                    }
                    try pipe.fileHandleForReading.close()
                } catch { reportReadError(error) }
            }
            child.terminationHandler = { [weak self] completed in
                Task { @MainActor in
                    await reader.value
                    self?.flush(output)
                    self?.appendLog("EXPORT_EXIT: \(completed.terminationStatus)")
                    continuation.resume(returning: completed.terminationStatus)
                }
            }
            do {
                try child.run()
                try pipe.fileHandleForWriting.close()
            } catch {
                child.terminationHandler = nil
                let launchError = error
                do { try pipe.fileHandleForWriting.close() }
                catch { report(error) }
                Task { @MainActor in
                    await reader.value
                    self.flush(output)
                    continuation.resume(throwing: launchError)
                }
            }
        }
    }

    func installMapping() {
        let panel = NSSavePanel()
        panel.title = "Install Djay mapping"
        panel.prompt = "Install"
        panel.nameFieldStringValue = "S4 MK3 Bridge.djayMidiMapping"
        panel.canCreateDirectories = true
        panel.directoryURL = FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent("Music/djay/MIDI Mappings", isDirectory: true)
        guard panel.runModal() == .OK, let url = panel.url else { return }
        Task { @MainActor in
            do {
                try await exportMapping(to: url)
                message = "Mapping installed. Select S4 MK3 Bridge for S4 MK3 MIDI Full in Djay."
                if !NSWorkspace.shared.open(url) {
                    throw AppError.invalid("Mapping saved, but macOS could not open it. Open \(url.path) in Djay.")
                }
            } catch { report(error) }
        }
    }
}

struct NumberRow: View {
    let title: String
    @Binding var value: Int
    let range: ClosedRange<Int>
    var step = 1
    var unit = ""

    var body: some View {
        HStack {
            Text(title)
            Spacer()
            TextField(title, value: $value, format: .number.grouping(.never))
                .frame(width: 80).multilineTextAlignment(.trailing)
            if !unit.isEmpty { Text(unit).foregroundColor(.secondary) }
            Stepper(title, value: $value, in: range, step: step).labelsHidden().fixedSize()
        }
    }
}

struct DecimalRow: View {
    let title: String
    @Binding var value: Double
    let range: ClosedRange<Double>
    var unit = ""

    var body: some View {
        HStack {
            Text(title)
            Spacer()
            TextField(title, value: $value, format: .number.precision(.fractionLength(1)).grouping(.never))
                .frame(width: 80).multilineTextAlignment(.trailing)
            if !unit.isEmpty { Text(unit).foregroundColor(.secondary) }
            Stepper(title, value: $value, in: range, step: 0.1).labelsHidden().fixedSize()
        }
    }
}

struct PalettePicker: View {
    let title: String
    @Binding var value: String

    var body: some View {
        Picker(title, selection: $value) {
            ForEach(LedPreferences.colors, id: \.self) { color in
                Text(color.capitalized).tag(color)
            }
        }
    }
}

struct ColorArrayEditor: View {
    let title: String
    let labels: [String]
    @Binding var values: [String]

    var body: some View {
        GroupBox(title) {
            VStack(spacing: 8) {
                ForEach(labels.indices, id: \.self) { index in
                    PalettePicker(title: labels[index], value: $values[index])
                }
            }.padding(8)
        }
    }
}

enum SettingsTab: Hashable {
    case bridge, screens, jog, leds, palette, diagnostics
}

struct BridgeSettingsView: View {
    @ObservedObject var controller: BridgeController
    @State var selectedTab: SettingsTab = .bridge

    var body: some View {
        VStack(spacing: 12) {
            HStack(spacing: 12) {
                Image(systemName: "waveform").font(.title2)
                VStack(alignment: .leading, spacing: 4) {
                    Text("S4 MK3 Bridge").font(.title3.weight(.semibold))
                    Text(controller.state.rawValue).foregroundColor(.secondary)
                }
                Spacer()
                Button("Start bridge") { controller.start() }
                    .disabled(controller.isBusy || controller.requiresReset)
                Button("Stop") { controller.stop() }.disabled(!controller.canStop)
            }.padding(.horizontal, 20).padding(.top, 16)
            if !controller.errorMessage.isEmpty {
                Text(controller.errorMessage).foregroundColor(.red)
                    .lineLimit(3).help(controller.errorMessage)
                    .frame(maxWidth: .infinity, alignment: .leading).padding(.horizontal, 20)
            }
            TabView(selection: $selectedTab) {
                bridgeTab.tabItem { Label("Bridge", systemImage: "slider.horizontal.3") }.tag(SettingsTab.bridge)
                ScreenSettingsView(controller: controller)
                    .tabItem { Label("Screens", systemImage: "display.2") }.tag(SettingsTab.screens)
                jogTab.tabItem { Label("Jog", systemImage: "rotate.right") }.tag(SettingsTab.jog)
                ledTab.tabItem { Label("LEDs", systemImage: "lightbulb") }.tag(SettingsTab.leds)
                paletteTab.tabItem { Label("Palette", systemImage: "paintpalette") }.tag(SettingsTab.palette)
                diagnosticsTab.tabItem { Label("Diagnostics", systemImage: "terminal") }.tag(SettingsTab.diagnostics)
            }.padding(.horizontal, 16)
            HStack {
                Text(controller.message).font(.caption).foregroundColor(.secondary)
                    .lineLimit(2).frame(maxWidth: .infinity, alignment: .leading)
                Button("Reset preferences…") { confirmReset() }
                Button(controller.isBusy ? "Save and restart" : "Save preferences") { controller.apply() }
                    .disabled(controller.state == .stopping || controller.state == .checking || controller.screens.busy)
                    .keyboardShortcut("s", modifiers: .command)
            }.padding(.horizontal, 20).padding(.bottom, 16)
        }
        .frame(minWidth: 760, minHeight: 540)
    }

    private func section<Content: View>(_ title: String, @ViewBuilder content: () -> Content) -> some View {
        GroupBox(title) { VStack(alignment: .leading, spacing: 12, content: content).padding(12) }
    }

    private var bridgeTab: some View {
        ScrollView {
            VStack(spacing: 20) {
                section("Bridge operation") {
                    Picker("Profile", selection: $controller.run.profile) {
                        ForEach(BridgeProfile.allCases, id: \.self) { Text($0.title).tag($0) }
                    }.disabled(controller.screens.busy || controller.isCapturingMotorDiagnostic)
                    Toggle("Stop after a fixed duration", isOn: $controller.run.limitDuration)
                    if controller.run.limitDuration {
                        NumberRow(title: "Run duration", value: $controller.run.durationSeconds, range: 1...604800, unit: "seconds")
                    }
                    Text("Without a time limit, the bridge runs until you stop it or quit this app.")
                        .font(.caption).foregroundColor(.secondary)
                }
                section("Djay mapping") {
                    Text("Install once, then select S4 MK3 Bridge under MIDI > Configure S4 MK3 MIDI Full in Djay.")
                    HStack {
                        Button("Install Djay mapping…") { controller.installMapping() }.disabled(controller.isExporting)
                        Button("Check controller") { controller.checkController() }.disabled(controller.isBusy)
                    }
                    if controller.isExporting {
                        Text("Generating mapping from the current jog settings…").font(.caption).foregroundColor(.secondary)
                    }
                    Text("Start the bridge before opening Djay. If Djay shows only a blank S4 MK3 configuration, restart Djay while the bridge stays running. Do not run another CLI bridge alongside this app.")
                        .font(.caption).foregroundColor(.secondary)
                }
                section("Files") {
                    Text(controller.supportURL.path).font(.system(.caption, design: .monospaced)).textSelection(.enabled)
                    Button("Show preferences in Finder") { controller.showPreferences() }
                    Text("The app bundles its own bridge. Rust and Cargo are not required on the DJ laptop.")
                        .font(.caption).foregroundColor(.secondary)
                }
            }.padding(20).frame(maxWidth: .infinity)
        }
    }

    private var jogTab: some View {
        ScrollView {
            VStack(spacing: 20) {
                section("Hardware") {
                    HStack {
                        Text("Jog Wheel Tension")
                        Spacer()
                        Text(controller.tension.tension, format: .number.grouping(.never))
                    }
                    Slider(value: Binding(
                        get: { Double(controller.tension.tension) },
                        set: { controller.tension.tension = Int($0) }
                    ), in: -100...100, step: 1) {
                        Text("Jog Wheel Tension")
                    }
                    .labelsHidden()
                    HStack {
                        Text("Assist").frame(maxWidth: .infinity, alignment: .leading)
                        Text("Neutral (0)").frame(maxWidth: .infinity)
                        Text("Brake").frame(maxWidth: .infinity, alignment: .trailing)
                    }.font(.caption).foregroundColor(.secondary)
                    Text("One setting for both wheels; each responds to its own movement and touch. Channel 1's fader remains a normal mixer control.")
                        .font(.caption).foregroundColor(.secondary)
                    Text("Full-controller profile only. Save preferences to apply on the next start, or Save and restart while running. No new Djay mapping is needed.")
                        .font(.caption).foregroundColor(.secondary)
                }
                section("Backspin") {
                    Toggle("Detect backspins and continue scratch after release", isOn: $controller.jog.backspinEnabled)
                    HStack {
                        Text("Acceleration threshold")
                        Spacer()
                        TextField("Acceleration threshold", value: $controller.jog.backspinAcceleration, format: .number.grouping(.never))
                            .frame(width: 90).multilineTextAlignment(.trailing)
                        Text("counts/s^2").foregroundColor(.secondary)
                        Stepper("Acceleration threshold", value: $controller.jog.backspinAcceleration, in: 1000...Double.greatestFiniteMagnitude, step: 10000)
                            .labelsHidden().fixedSize()
                    }
                    .disabled(!controller.jog.backspinEnabled)
                    HStack {
                        Text("Scratch speed increase")
                        Slider(value: $controller.jog.backspinSpeedIncrease, in: 0...10, step: 0.1)
                        Text(controller.jog.backspinSpeedIncrease, format: .number.precision(.fractionLength(1)).grouping(.never))
                            .frame(width: 35, alignment: .trailing)
                        Text("%").foregroundColor(.secondary)
                    }
                    .disabled(!controller.jog.backspinEnabled)
                    Toggle("Apply motor assistance during detected backspins", isOn: $controller.jog.backspinAssistEnabled)
                        .disabled(!controller.jog.backspinEnabled)
                    HStack {
                        Text("Motor assist strength")
                        Slider(value: Binding(
                            get: { Double(controller.jog.backspinAssistStrength) },
                            set: { controller.jog.backspinAssistStrength = Int($0) }
                        ), in: 0...100, step: 1)
                        .accessibilityLabel("Motor assist strength")
                        Text("\(controller.jog.backspinAssistStrength)%")
                            .frame(width: 42, alignment: .trailing)
                    }
                    .disabled(!controller.jog.backspinEnabled || !controller.jog.backspinAssistEnabled)
                    Text("Adjusts motor assistance during detected backspins. Assistance continues after release and fades out within one second. Normal Jog Wheel Tension and scratch speed increase are independent settings.")
                        .font(.caption).foregroundColor(.secondary)
                    Text("Backspin options apply on the next start after saving, or through Save and restart while running; no new Djay mapping export is needed.")
                        .font(.caption).foregroundColor(.secondary)
                }
                section("Testing: Motorized playback") {
                    Toggle("Enable Vinyl at normal-mode startup", isOn: $controller.jog.vinylEnabledAtStartup)
                    Toggle("Enable Vinyl mode at TT startup", isOn: $controller.jog.ttVinylEnabledAtStartup)
                    Text("Normal and TT modes have separate Vinyl defaults. Each mode remembers its last choice during the session. TT motors always start off and must be engaged on the controller.")
                        .font(.caption).foregroundColor(.secondary)
                    NumberRow(title: "TT pitch-bend floor", value: $controller.jog.ttPitchBendFloorPercent, range: 1...95, unit: "%")
                    Text("Testing: the floor also sets a symmetric upper ceiling (50% floor means 150% ceiling). Both limits hand off to Scratch. These are provisional diagnostic bounds, not verified CC 4 limits.")
                        .font(.caption).foregroundColor(.secondary)
                    NumberRow(title: "Target speed", value: $controller.jog.platterTargetRpm, range: 1...45, unit: "RPM")
                    DecimalRow(title: "Absolute platter sensitivity", value: $controller.jog.platterPositionSensitivity, range: 0.1...100, unit: "%")
                    NumberRow(title: "Maximum force", value: $controller.jog.platterMaxForce, range: 1...25000, step: 500, unit: "force units")
                    NumberRow(title: "Base force (0 RPM)", value: $controller.jog.platterBaseForce, range: 0...25000, step: 50, unit: "force units")
                    NumberRow(title: "Force per RPM", value: $controller.jog.platterForcePerRpm, range: 0...25000, step: 10, unit: "force units/RPM")
                    NumberRow(title: "Proportional gain (Kp)", value: $controller.jog.platterKp, range: 0...25000, step: 100, unit: "force units/RPM")
                    NumberRow(title: "Integral gain (Ki)", value: $controller.jog.platterKi, range: 0...25000, step: 100, unit: "force units/(RPM*s)")
                    NumberRow(title: "Force rise rate", value: $controller.jog.platterForceRisePerSecond, range: 0...1000000, step: 10000, unit: "force units/s")
                    NumberRow(title: "Startup duration", value: $controller.jog.platterStartupMs, range: 0...2000, step: 50, unit: "ms")
                    NumberRow(title: "Startup force rise rate", value: $controller.jog.platterStartupForceRisePerSecond, range: 0...1000000, step: 10000, unit: "force units/s")
                    NumberRow(title: "Overspeed limit", value: $controller.jog.platterOverspeedLimitRpm, range: 0...250, unit: "RPM")
                    Text("Zero semantics: 0 ms startup duration skips the startup window; 0 force units/s startup force rise rate applies an immediate step; 0 force units/s force rise rate holds post-startup force from increasing; 0 RPM overspeed limit disables the overspeed guard.")
                        .font(.caption).foregroundColor(.secondary)
                    Text("Motor force and Vinyl startup are runtime preferences; save and restart to apply. Absolute platter sensitivity controls the generated CC 8 mapping: save, install a new Djay mapping, and select it to apply. The starting sensitivity was estimated from a timed 145 BPM test.")
                        .font(.caption).foregroundColor(.secondary)
                    Text("Testing: Turn TT mode off or stop the bridge before quitting Djay. Motorized playback uses the last reported playback state; silence from Djay does not stop it.")
                        .font(.caption).foregroundColor(.secondary)
                }
                section("Scratch (platter touch)") {
                    DecimalRow(title: "Scratch speed", value: $controller.jog.scratchSpeed, range: 0.1...Double.greatestFiniteMagnitude, unit: "%")
                    NumberRow(title: "Scratch reaction", value: $controller.jog.scratchReaction, range: 0...150, unit: "%")
                    Picker("Touch-scratch direction", selection: $controller.jog.scratchDirection) {
                        Text("Normal").tag(ScratchDirection.normal)
                        Text("Reversed").tag(ScratchDirection.reversed)
                    }
                    Text("Applies to manual top-touch Scratch CC only; motorized absolute platter position keeps its own direction.")
                        .font(.caption).foregroundColor(.secondary)
                }
                section("Pitch bend (no touch)") {
                    DecimalRow(title: "Pitch bend speed", value: $controller.jog.pitchBendSpeed, range: 0.1...Double.greatestFiniteMagnitude, unit: "%")
                    NumberRow(title: "Pitch bend reaction", value: $controller.jog.pitchBendReaction, range: 0...150, unit: "%")
                }
                section("Applying jog mapping changes") {
                    Text("Scratch speed, touch direction, absolute platter sensitivity and pitch bend affect the exported mapping only. Save preferences, then install a new mapping and select it in Djay; restarting the bridge is not required. Seek behavior is unchanged.")
                        .font(.caption).foregroundColor(.secondary)
                    HStack {
                        Button("Install Djay mapping…") { controller.installMapping() }.disabled(controller.isExporting)
                        if controller.isExporting {
                            Text("Generating mapping…").font(.caption).foregroundColor(.secondary)
                        }
                        Spacer()
                    }
                }
            }.padding(20).frame(maxWidth: .infinity)
        }
    }

    private var ledTab: some View {
        ScrollView {
            VStack(spacing: 20) {
                if controller.run.profile != .full {
                    Text("LED preferences apply to the full-controller profile. Minimal MIDI and HID probe keep their original light behavior.")
                        .foregroundColor(.secondary)
                }
                section("Brightness and inactive lights") {
                    NumberRow(title: "Active lamp brightness", value: $controller.leds.activeBrightness, range: 1...127)
                    NumberRow(title: "Inactive lamp brightness", value: $controller.leds.inactiveBrightness, range: 0...127)
                    Picker("Inactive display", selection: $controller.leds.inactiveDisplay) {
                        Text("Dim").tag("dim"); Text("Dark").tag("dark")
                    }
                    NumberRow(title: "Active palette intensity", value: $controller.leds.paletteActiveIntensity, range: 0...3)
                    NumberRow(title: "Inactive palette intensity", value: $controller.leds.paletteInactiveIntensity, range: 0...3)
                    Picker("Mixed quantize state", selection: $controller.leds.quantizeMixed) {
                        Text("Dim").tag("dim"); Text("Dark").tag("dark")
                    }
                }
                section("Jog rings") {
                    NumberRow(title: "Chase revolution", value: $controller.leds.chasePeriodMs, range: 250...10000, unit: "ms")
                    Text("Decorative spinning while playing, not track-position or tempo tracking.")
                        .font(.caption).foregroundColor(.secondary)
                    Toggle("Flash the whole ring during loops", isOn: $controller.leds.loopEnabled)
                    PalettePicker(title: "Loop color", value: $controller.leds.loopColor)
                    Text("Loop flash cadence belongs to the controller firmware. End-of-track warnings are unavailable.")
                        .font(.caption).foregroundColor(.secondary)
                }
                section("Channel meters") {
                    Picker("Meter brightness", selection: $controller.leds.meterBrightness) {
                        Text("Off").tag(0); Text("Dim").tag(125); Text("Bright").tag(127)
                    }
                    HStack {
                        Text("Level curve")
                        Slider(value: $controller.leds.meterGamma, in: 0.1...5, step: 0.1)
                        Text(controller.leds.meterGamma, format: .number.precision(.fractionLength(1))).frame(width: 35)
                    }
                    NumberRow(title: "Level freshness", value: $controller.leds.meterStaleMs, range: 100...5000, unit: "ms")
                    Text("Master meters remain hardware-driven.")
                        .font(.caption).foregroundColor(.secondary)
                }
                section("Tempo-center indicator") {
                    Toggle("Enable calibrated physical-center indicator", isOn: Binding(
                        get: { controller.leds.tempoCenterTolerance != nil },
                        set: { controller.leds.tempoCenterTolerance = $0 ? 0.01 : nil }
                    ))
                    if controller.leds.tempoCenterTolerance != nil {
                        HStack {
                            Text("Center tolerance")
                            Slider(value: Binding(
                                get: { controller.leds.tempoCenterTolerance ?? 0.01 },
                                set: { controller.leds.tempoCenterTolerance = $0 }
                            ), in: 0.001...0.1, step: 0.001)
                            Text(controller.leds.tempoCenterTolerance ?? 0, format: .number.precision(.fractionLength(3)))
                                .frame(width: 45)
                        }
                    }
                    Text("Leave disabled unless you have calibrated the fader detent. This is not a software pickup indicator.")
                        .font(.caption).foregroundColor(.secondary)
                }
                HStack {
                    Button("Import LED JSON…") { controller.importLEDs() }
                    Button("Export LED JSON…") { controller.exportLEDs() }
                    Spacer()
                }
            }.padding(20).frame(maxWidth: .infinity)
        }
    }

    private var paletteTab: some View {
        ScrollView {
            VStack(spacing: 20) {
                ColorArrayEditor(title: "Deck colors", labels: ["Deck A", "Deck B", "Deck C", "Deck D"], values: $controller.leds.deckColors)
                section("Hotcue presentation") {
                    Picker("Hotcue colors", selection: $controller.leds.hotcueColors) {
                        Text("Match Djay feedback").tag("djay"); Text("Fixed slot colors").tag("fixed_slots")
                    }
                    Text("Djay token colors can be calibrated here. Unknown cue tokens stay dark; the full palette has not yet been physically calibrated.")
                        .font(.caption).foregroundColor(.secondary)
                    PalettePicker(title: "Djay white token", value: $controller.leds.djayWhite)
                }
                ColorArrayEditor(title: "Djay color-token correspondence", labels: (1...8).map { "Token \($0)" }, values: $controller.leds.djayPadColors)
                ColorArrayEditor(title: "Optional fixed hotcue slots", labels: (1...8).map { "Pad \($0)" }, values: $controller.leds.fixedHotcueColors)
                ColorArrayEditor(title: "Stem colors", labels: (1...4).map { "Stem \($0)" }, values: $controller.leds.stemColors)
                section("Stem mute display") {
                    Toggle("Bright means muted", isOn: $controller.leds.stemMuteBright)
                    PalettePicker(title: "Engaged mute color", value: $controller.leds.muteColor)
                    Text("Mute indicates the mute switch, not effective audibility under another stem's solo state.")
                        .font(.caption).foregroundColor(.secondary)
                }
            }.padding(20).frame(maxWidth: .infinity)
        }
    }

    private var diagnosticsTab: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("App version: \(appVersion)")
                .font(.caption).foregroundColor(.secondary)
                .textSelection(.enabled)
            HStack {
                Picker("Log detail", selection: $controller.run.logLevel) {
                    Text("Warnings").tag("warn"); Text("Normal").tag("info"); Text("Debug").tag("debug")
                }
                .disabled(controller.isCapturingMotorDiagnostic)
                Toggle("Raw HID reports", isOn: $controller.run.rawReports)
            }
            Text("Debug/raw reports can generate substantial output. Logs are bounded to the most recent 60,000 characters.")
                .font(.caption).foregroundColor(.secondary)
            HStack {
                Button(controller.isCapturingMotorDiagnostic ? "Stop motor diagnostic capture" : "Start motor diagnostic capture") {
                    if controller.isCapturingMotorDiagnostic {
                        controller.stopMotorDiagnosticCapture()
                    } else {
                        controller.startMotorDiagnosticCapture()
                    }
                }
                .disabled(!controller.isCapturingMotorDiagnostic && !controller.canStartMotorDiagnosticCapture)
                if controller.motorDiagnosticURL != nil {
                    Button("Show file") { controller.revealMotorDiagnostic() }
                }
            }
            Text("Use the Full profile and select Debug before starting or restarting the bridge. Start capture before the test; timestamped relative jog-bend, motor-position, and playback-mode events are saved under Application Support/S4 MK3 Bridge/diagnostics.")
                .font(.caption).foregroundColor(.secondary)
            if let motorDiagnosticURL = controller.motorDiagnosticURL {
                Text(motorDiagnosticURL.path)
                    .font(.system(.caption, design: .monospaced))
                    .textSelection(.enabled)
                    .fixedSize(horizontal: false, vertical: true)
            }
            Text(controller.commandPreview).font(.system(.caption, design: .monospaced))
                .textSelection(.enabled).fixedSize(horizontal: false, vertical: true)
            ScrollView([.vertical, .horizontal]) {
                Text(controller.log.isEmpty ? "No bridge output yet." : controller.log)
                    .font(.system(.caption, design: .monospaced)).textSelection(.enabled)
                    .frame(maxWidth: .infinity, alignment: .topLeading).padding(12)
            }
            .background(Color(NSColor.textBackgroundColor))
            .overlay(RoundedRectangle(cornerRadius: 4).stroke(Color.secondary.opacity(0.25)))
            Button("Copy log") {
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(controller.log, forType: .string)
            }
        }.padding(20)
    }

    private var appVersion: String {
        let version = Bundle.main.object(forInfoDictionaryKey: "S4BridgeReleaseVersion") as? String ?? "Unknown"
        let build = Bundle.main.object(forInfoDictionaryKey: "CFBundleVersion") as? String ?? "Unknown"
        return "\(version) (build \(build))"
    }

    private func confirmReset() {
        let alert = NSAlert()
        alert.messageText = "Reset bridge preferences?"
        alert.informativeText = "This restores the editor to bundled defaults. Save and apply to replace the saved preferences."
        alert.addButton(withTitle: "Reset")
        alert.addButton(withTitle: "Cancel")
        if alert.runModal() == .alertFirstButtonReturn { controller.resetPreferences() }
    }
}

@MainActor
func bridgeStatusImage(for state: BridgeState, screenAttention: Bool = false) -> NSImage? {
    let image = NSImage(systemSymbolName: "waveform", accessibilityDescription: "S4 MK3 Bridge")
    if state == .running {
        return image?.withSymbolConfiguration(NSImage.SymbolConfiguration(
            paletteColors: [screenAttention ? .systemRed : .systemGreen]
        ))
    }
    image?.isTemplate = true
    return image
}

@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate, NSMenuDelegate {
    private var controller: BridgeController?
    private var statusItem: NSStatusItem?
    private var settingsWindow: NSWindow?
    private var observation: AnyCancellable?
    private var screenObservation: AnyCancellable?

    init(controller: BridgeController? = nil) {
        self.controller = controller
        super.init()
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        do {
            guard let resources = Bundle.main.resourceURL else {
                throw AppError.invalid("The app bundle has no Resources directory.")
            }
            let support = try FileManager.default.url(for: .applicationSupportDirectory, in: .userDomainMask, appropriateFor: nil, create: true)
                .appendingPathComponent("S4 MK3 Bridge", isDirectory: true)
            let model = try BridgeController(resourceURL: resources, supportURL: support)
            controller = model
            let item = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
            statusItem = item
            item.button?.title = " S4"
            observation = model.$state.sink { [weak self] state in self?.updateMenu(state) }
            screenObservation = model.screens.objectWillChange.sink { [weak self, weak model] in
                DispatchQueue.main.async {
                    guard let model else { return }
                    self?.updateMenu(model.state)
                }
            }
            showSettings()
        } catch {
            let alert = NSAlert(error: error)
            alert.runModal()
            NSApp.terminate(nil)
        }
    }

    private func updateMenu(_ state: BridgeState) {
        statusItem?.menu = bridgeStatusMenu(controller: controller, state: state, target: self)
        statusItem?.button?.image = bridgeStatusImage(for: state, screenAttention: controller?.screens.needsAttention == true)
        statusItem?.button?.setAccessibilityLabel("S4 MK3 Bridge, \(state.rawValue), \(controller?.screens.state.rawValue ?? "")")
    }

    @objc func start() { controller?.start() }
    @objc func toggleScreens() {
        guard let controller else { return }
        controller.setScreensEnabled(!controller.screens.preferences.enabled)
    }
    @objc func enableScreens() { controller?.enableScreensByStoppingDriver() }
    @objc func restoreDriver() { controller?.restartNIDriver() }
    func menuWillOpen(_ menu: NSMenu) {
        guard let controller else { return }
        controller.screens.refreshDriverStatus()
        let refreshed = bridgeStatusMenu(controller: controller, state: controller.state, target: self)
        menu.removeAllItems()
        for item in refreshed.items {
            refreshed.removeItem(item)
            menu.addItem(item)
        }
    }
    @objc func stop() { controller?.stop() }
    @objc func checkController() { controller?.checkController() }
    @objc func quit() { NSApp.terminate(nil) }
    @objc func installMapping() {
        NSApp.activate(ignoringOtherApps: true)
        controller?.installMapping()
    }

    @objc func showSettings() {
        guard let controller else { return }
        if settingsWindow == nil {
            let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 760, height: 680),
                                  styleMask: [.titled, .closable, .miniaturizable, .resizable],
                                  backing: .buffered, defer: false)
            window.title = "S4 MK3 Bridge"
            window.contentMinSize = NSSize(width: 760, height: 540)
            window.contentView = NSHostingView(rootView: BridgeSettingsView(controller: controller))
            window.isReleasedWhenClosed = false
            window.center()
            settingsWindow = window
        }
        NSApp.activate(ignoringOtherApps: true)
        settingsWindow?.makeKeyAndOrderFront(nil)
    }

    func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
        controller?.prepareToQuit() ?? .terminateNow
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool { false }
}

#if !APP_TESTS
@main
struct S4BridgeApp {
    @MainActor static func main() {
        let application = NSApplication.shared
        let delegate = AppDelegate()
        application.setActivationPolicy(.accessory)
        application.delegate = delegate
        withExtendedLifetime(delegate) { application.run() }
    }
}
#endif
