import AppKit
import Combine
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

    func arguments(ledURL: URL) -> [String] {
        var arguments: [String] = []
        if profile != .probe { arguments += ["--midi", profile.rawValue] }
        if profile == .full { arguments += ["--led-config", ledURL.path] }
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
    @Published private(set) var state: BridgeState = .stopped
    @Published private(set) var log = ""
    @Published private(set) var message = ""
    @Published private(set) var errorMessage = ""
    @Published private(set) var requiresReset = false

    let resourceURL: URL
    let supportURL: URL
    let defaults: Data
    private var process: Process?
    private var isDiagnostic = false
    private var stopRequested = false
    private var restartRequested = false
    private var quitRequested = false

    var isBusy: Bool { process != nil }
    var canStop: Bool { isBusy && state != .stopping }
    var ledURL: URL { supportURL.appendingPathComponent("led-config.json") }
    var runURL: URL { supportURL.appendingPathComponent("run-settings.json") }
    var executableURL: URL { resourceURL.appendingPathComponent("s4-connectivity-probe") }
    var commandPreview: String {
        ([executableURL.path] + run.arguments(ledURL: ledURL))
            .map { "'" + $0.replacingOccurrences(of: "'", with: "'\\''") + "'" }
            .joined(separator: " ")
    }

    init(resourceURL: URL, supportURL: URL) throws {
        self.resourceURL = resourceURL
        self.supportURL = supportURL
        defaults = try Data(contentsOf: resourceURL.appendingPathComponent("default-led-config.json"))
        leds = try LedPreferences.decode(defaults, defaults: defaults)
        do {
            if FileManager.default.fileExists(atPath: runURL.path) {
                run = try JSONDecoder().decode(RunPreferences.self, from: Data(contentsOf: runURL))
                try run.validate()
            }
            if FileManager.default.fileExists(atPath: ledURL.path) {
                leds = try LedPreferences.decode(Data(contentsOf: ledURL), defaults: defaults)
            }
        } catch {
            requiresReset = true
            errorMessage = "Saved preferences could not be loaded: \(error.localizedDescription). Import valid LED settings or reset preferences before saving."
        }
    }

    func appendLog(_ text: String) {
        log = String((log + text + "\n").suffix(60000))
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
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        let runData = try encoder.encode(run)
        try FileManager.default.createDirectory(at: supportURL, withIntermediateDirectories: true)
        try ledData.write(to: ledURL, options: .atomic)
        try runData.write(to: runURL, options: .atomic)
        errorMessage = ""
    }

    func apply() {
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
            try launch(arguments: run.arguments(ledURL: ledURL), diagnostic: false)
        } catch {
            state = .failed
            report(error)
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
        child.executableURL = executableURL
        child.arguments = arguments
        var environment = ProcessInfo.processInfo.environment
        environment["RUST_LOG"] = run.logLevel
        child.environment = environment
        child.standardInput = FileHandle.nullDevice
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
        let deliver: @Sendable () -> Void = { [weak self] in
            DispatchQueue.main.async { self?.flush(output) }
        }
        let reportReadError: @Sendable (Error) -> Void = { [weak self] error in
            DispatchQueue.main.async { self?.report(error) }
        }
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
                self?.completed(completed)
            }
        }
        appendLog("LAUNCH: \(arguments.joined(separator: " "))")
        do {
            try child.run()
            try pipe.fileHandleForWriting.close()
            if !diagnostic { state = .running }
        } catch {
            child.terminationHandler = nil
            if !child.isRunning {
                process = nil
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
        if kill(child.processIdentifier, SIGINT) != 0 {
            restartRequested = false
            stopRequested = false
            report(AppError.invalid("Could not send the bridge its stop signal: \(String(cString: strerror(errno)))."))
            state = .failed
            if quitRequested {
                quitRequested = false
                NSApp.reply(toApplicationShouldTerminate: false)
            }
        }
    }

    private func completed(_ child: Process) {
        guard process === child else { return }
        let normalStop = child.terminationStatus == 0 ||
            (stopRequested && child.terminationReason == .uncaughtSignal && child.terminationStatus == SIGINT)
        let restart = restartRequested && normalStop
        let wasDiagnostic = isDiagnostic
        process = nil
        restartRequested = false
        state = normalStop ? .stopped : .failed
        appendLog("EXIT: \(child.terminationStatus)")
        if !normalStop {
            errorMessage = "Bridge exited with status \(child.terminationStatus). See Diagnostics for the error."
        } else if wasDiagnostic {
            message = "Controller check complete. See Diagnostics for USB results."
        } else {
            message = "Bridge stopped."
        }
        if quitRequested {
            NSApp.reply(toApplicationShouldTerminate: true)
        } else if restart {
            start()
        }
    }

    func prepareToQuit() -> NSApplication.TerminateReply {
        guard isBusy else { return .terminateNow }
        quitRequested = true
        if state != .stopping { stop() }
        return quitRequested ? .terminateLater : .terminateCancel
    }

    func resetPreferences() {
        do {
            leds = try LedPreferences.decode(defaults, defaults: defaults)
            run = RunPreferences()
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
            if requiresReset { run = RunPreferences() }
            requiresReset = false
            errorMessage = ""
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

    func installMapping() {
        let panel = NSSavePanel()
        panel.title = "Install Djay mapping"
        panel.prompt = "Install"
        panel.nameFieldStringValue = "S4 MK3 Bridge.djayMidiMapping"
        panel.canCreateDirectories = true
        panel.directoryURL = FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent("Music/djay/MIDI Mappings", isDirectory: true)
        guard panel.runModal() == .OK, let url = panel.url else { return }
        do {
            let source = resourceURL.appendingPathComponent("S4 MK3 Bridge.djayMidiMapping")
            try Data(contentsOf: source).write(to: url, options: .atomic)
            message = "Mapping installed. Select S4 MK3 Bridge for S4 MK3 MIDI Full in Djay."
            if !NSWorkspace.shared.open(url) {
                throw AppError.invalid("Mapping saved, but macOS could not open it. Open \(url.path) in Djay.")
            }
        } catch { report(error) }
    }
}

struct NumberRow: View {
    let title: String
    @Binding var value: Int
    let range: ClosedRange<Int>
    var unit = ""

    var body: some View {
        HStack {
            Text(title)
            Spacer()
            TextField(title, value: $value, format: .number.grouping(.never))
                .frame(width: 80).multilineTextAlignment(.trailing)
            if !unit.isEmpty { Text(unit).foregroundColor(.secondary) }
            Stepper(title, value: $value, in: range).labelsHidden().fixedSize()
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

struct BridgeSettingsView: View {
    @ObservedObject var controller: BridgeController

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
            TabView {
                bridgeTab.tabItem { Label("Bridge", systemImage: "slider.horizontal.3") }
                ledTab.tabItem { Label("LEDs", systemImage: "lightbulb") }
                paletteTab.tabItem { Label("Palette", systemImage: "paintpalette") }
                diagnosticsTab.tabItem { Label("Diagnostics", systemImage: "terminal") }
            }.padding(.horizontal, 16)
            HStack {
                Text(controller.message).font(.caption).foregroundColor(.secondary)
                    .lineLimit(2).frame(maxWidth: .infinity, alignment: .leading)
                Button("Reset preferences…") { confirmReset() }
                Button(controller.isBusy ? "Save and restart" : "Save preferences") { controller.apply() }
                    .disabled(controller.state == .stopping || controller.state == .checking)
                    .keyboardShortcut("s", modifiers: .command)
            }.padding(.horizontal, 20).padding(.bottom, 16)
        }
        .frame(minWidth: 640, minHeight: 540)
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
                    }
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
                        Button("Install Djay mapping…") { controller.installMapping() }
                        Button("Check controller") { controller.checkController() }.disabled(controller.isBusy)
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
            HStack {
                Picker("Log detail", selection: $controller.run.logLevel) {
                    Text("Warnings").tag("warn"); Text("Normal").tag("info"); Text("Debug").tag("debug")
                }
                Toggle("Raw HID reports", isOn: $controller.run.rawReports)
            }
            Text("Debug/raw reports can generate substantial output. Logs are bounded to the most recent 60,000 characters.")
                .font(.caption).foregroundColor(.secondary)
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
final class AppDelegate: NSObject, NSApplicationDelegate {
    private var controller: BridgeController?
    private var statusItem: NSStatusItem?
    private var settingsWindow: NSWindow?
    private var observation: AnyCancellable?

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
            item.button?.image = NSImage(systemSymbolName: "waveform", accessibilityDescription: "S4 MK3 Bridge")
            item.button?.image?.isTemplate = true
            item.button?.title = " S4"
            observation = model.$state.sink { [weak self] state in self?.updateMenu(state) }
            showSettings()
        } catch {
            let alert = NSAlert(error: error)
            alert.runModal()
            NSApp.terminate(nil)
        }
    }

    private func updateMenu(_ state: BridgeState) {
        let menu = NSMenu()
        menu.autoenablesItems = false
        let status = NSMenuItem(title: "Bridge: \(state.rawValue)", action: nil, keyEquivalent: "")
        status.isEnabled = false
        menu.addItem(status)
        menu.addItem(.separator())
        addItem("Start bridge", action: #selector(start), to: menu, enabled: controller?.isBusy == false)
        addItem("Stop bridge", action: #selector(stop), to: menu, enabled: controller?.isBusy == true && state != .stopping)
        menu.addItem(.separator())
        addItem("Settings…", action: #selector(showSettings), to: menu)
        addItem("Install Djay mapping…", action: #selector(installMapping), to: menu)
        addItem("Check controller", action: #selector(checkController), to: menu, enabled: controller?.isBusy == false)
        menu.addItem(.separator())
        addItem("Quit S4 MK3 Bridge", action: #selector(quit), to: menu)
        statusItem?.menu = menu
        statusItem?.button?.setAccessibilityLabel("S4 MK3 Bridge, \(state.rawValue)")
    }

    private func addItem(_ title: String, action: Selector, to menu: NSMenu, enabled: Bool = true) {
        let item = NSMenuItem(title: title, action: action, keyEquivalent: "")
        item.target = self
        item.isEnabled = enabled
        menu.addItem(item)
    }

    @objc private func start() { controller?.start() }
    @objc private func stop() { controller?.stop() }
    @objc private func checkController() { controller?.checkController() }
    @objc private func quit() { NSApp.terminate(nil) }
    @objc private func installMapping() {
        NSApp.activate(ignoringOtherApps: true)
        controller?.installMapping()
    }

    @objc private func showSettings() {
        guard let controller else { return }
        if settingsWindow == nil {
            let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 760, height: 680),
                                  styleMask: [.titled, .closable, .miniaturizable, .resizable],
                                  backing: .buffered, defer: false)
            window.title = "S4 MK3 Bridge"
            window.contentMinSize = NSSize(width: 640, height: 540)
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
