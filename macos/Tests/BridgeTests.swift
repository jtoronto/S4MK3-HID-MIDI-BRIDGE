import AppKit
import Combine
import Darwin
import Foundation
import XCTest
@testable import S4Bridge

final class BridgeTests: XCTestCase {
    @MainActor
    func testStatusSymbolUsesGreenOnlyWhileRunning() throws {
        let running = try XCTUnwrap(bridgeStatusImage(for: .running))
        XCTAssertFalse(running.isTemplate)
        let bitmap = NSBitmapImageRep(cgImage: try XCTUnwrap(
            running.cgImage(forProposedRect: nil, context: nil, hints: nil)
        ))
        var greenPixels = 0
        for y in 0..<bitmap.pixelsHigh {
            for x in 0..<bitmap.pixelsWide {
                if let color = bitmap.colorAt(x: x, y: y)?.usingColorSpace(.deviceRGB),
                   color.alphaComponent > 0.5,
                   color.greenComponent > color.redComponent + 0.1,
                   color.greenComponent > color.blueComponent + 0.1 {
                    greenPixels += 1
                }
            }
        }
        XCTAssertGreaterThan(greenPixels, 0)
        for state: BridgeState in [.stopped, .starting, .stopping, .checking, .failed] {
            XCTAssertTrue(try XCTUnwrap(bridgeStatusImage(for: state)).isTemplate)
        }
    }

    private func defaultData() throws -> Data {
        let root = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        return try Data(contentsOf: root.appendingPathComponent("examples/led-config.json"))
    }

    func testLEDPreferencesRoundTripAndPartialOverride() throws {
        let defaults = try defaultData()
        let base = try LedPreferences.decode(defaults, defaults: defaults)
        XCTAssertEqual(base, try LedPreferences.decode(base.encoded(), defaults: defaults))
        let override = Data(#"{"loop_color":"blue","chase_period_ms":1000}"#.utf8)
        let changed = try LedPreferences.decode(override, defaults: defaults)
        XCTAssertEqual(changed.loopColor, "blue")
        XCTAssertEqual(changed.chasePeriodMs, 1000)
        XCTAssertEqual(changed.deckColors, base.deckColors)
        let keys = try XCTUnwrap(JSONSerialization.jsonObject(with: base.encoded()) as? [String: Any])
        XCTAssertNotNil(keys["chase_period_ms"])
        XCTAssertNil(keys["chasePeriodMs"])
    }

    func testInvalidLEDPreferencesCannotReachArrayEditorsOrBridge() throws {
        let defaults = try defaultData()
        for input in [
            #"{"deck_colors":["red"]}"#, #"{"meter_gamma":0}"#,
            #"{"chase_period_ms":0}"#, #"{"unknown":true}"#,
            #"{"loop_color":"not-a-color"}"#, #"{"tempo_center_tolerance":0}"#
        ] {
            XCTAssertThrowsError(try LedPreferences.decode(Data(input.utf8), defaults: defaults))
        }
    }

    func testLaunchArgumentsMatchProfilesAndDuration() throws {
        let path = URL(fileURLWithPath: "/tmp/LED config with spaces.json")
        let jogPath = URL(fileURLWithPath: "/tmp/Jog config with spaces.json")
        let tensionPath = URL(fileURLWithPath: "/tmp/Tension config with spaces.json")
        var settings = RunPreferences()
        XCTAssertEqual(settings.arguments(ledURL: path, jogURL: jogPath, tensionURL: tensionPath), [
            "--midi", "full", "--screen-control-stdin", "--screens", "--led-config", path.path,
            "--jog-config", jogPath.path, "--tension-config", tensionPath.path, "--until-stopped"
        ])
        settings.profile = .minimal
        settings.limitDuration = true
        settings.durationSeconds = 60
        settings.rawReports = true
        XCTAssertEqual(settings.arguments(ledURL: path, jogURL: jogPath, tensionURL: tensionPath), ["--midi", "minimal", "--seconds", "60", "--raw"])
        settings.profile = .probe
        XCTAssertEqual(settings.arguments(ledURL: path, jogURL: jogPath, tensionURL: tensionPath), ["--seconds", "60", "--raw"])
        settings.durationSeconds = 0
        XCTAssertThrowsError(try settings.validate())
    }

    func testOutputBufferBoundsBacklogAndSchedulesOneDelivery() {
        let output = OutputBuffer()
        XCTAssertTrue(output.append("first"))
        XCTAssertFalse(output.append(String(repeating: "x", count: 70000)))
        XCTAssertEqual(output.drain().utf8.count, 60000)
        XCTAssertEqual(output.drain(), "")
        XCTAssertTrue(output.append("next"))
        XCTAssertEqual(output.drain(), "next\n")
    }

    @MainActor
    func testBridgeLogEmitsAppendAndPrefixTrimUpdates() {
        let model = BridgeLog()
        var updates: [BridgeLog.Update] = []
        let subscription = model.updates.sink { updates.append($0) }

        model.append(String(repeating: "a", count: 40000))
        model.append(String(repeating: "b", count: 30000))

        let expected = String(repeating: "a", count: 29998) + "\n"
            + String(repeating: "b", count: 30000) + "\n"
        XCTAssertEqual(model.text, expected)
        XCTAssertEqual(updates.count, 2)
        XCTAssertEqual(updates[0].removedUTF16Count, 0)
        XCTAssertEqual(updates[0].appendedText, String(repeating: "a", count: 40000) + "\n")
        XCTAssertEqual(updates[1].removedUTF16Count, 10002)
        XCTAssertEqual(updates[1].appendedText, String(repeating: "b", count: 30000) + "\n")
        withExtendedLifetime(subscription) {}

        // Native text storage uses UTF-16 ranges, including Unicode track names
        // and one incoming batch larger than the entire retained log.
        for (prefix, incoming, expected) in [
            (
                String(repeating: "\u{1F3A7}", count: 40000),
                String(repeating: "b", count: 30000),
                String(repeating: "\u{1F3A7}", count: 29998) + "\n"
                    + String(repeating: "b", count: 30000) + "\n"
            ),
            (
                String(repeating: "a", count: 30000),
                String(repeating: "\u{E9}", count: 70000),
                String(repeating: "\u{E9}", count: 59999) + "\n"
            )
        ] {
            let model = BridgeLog()
            let displayed = NSMutableString(string: "")
            let subscription = model.updates.sink { update in
                displayed.deleteCharacters(in: NSRange(location: 0, length: update.removedUTF16Count))
                displayed.append(update.appendedText)
            }
            model.append(prefix)
            model.append(incoming)
            XCTAssertEqual(model.text, expected)
            XCTAssertTrue((displayed as String) == expected, "Incremental UTF-16 log differs from the expected retained text")
            withExtendedLifetime(subscription) {}
        }
    }

    @MainActor
    func testDiagnosticTrafficDoesNotInvalidateSettings() throws {
        let fixture = try makeFixture()
        let model = try testController(resourceURL: fixture, supportURL: fixture.appendingPathComponent("support"))
        var settingsUpdates = 0
        let subscription = model.objectWillChange.sink { settingsUpdates += 1 }
        var logUpdates = 0
        var platterUpdates = 0
        let logSubscription = model.logOutput.$text.dropFirst().sink { _ in logUpdates += 1 }
        let platterSubscription = model.livePlatter.$state.dropFirst().sink { _ in platterUpdates += 1 }
        let status = #"S4_PLATTER_STATUS {"wheels":[{"deck":1,"rpm":33,"bend_counts_per_second":0,"mode":"off"},{"deck":2,"rpm":0,"bend_counts_per_second":0,"mode":"off"}]}"#

        for _ in 0..<100 {
            model.appendLog("ordinary output")
            model.acceptPlatterStatus(status)
        }

        XCTAssertTrue(model.log.contains("ordinary output"))
        XCTAssertEqual(model.platterDiagnostics.latest?.left.rpm, 33)
        model.acceptPlatterStatus("S4_PLATTER_STATUS invalid")
        XCTAssertEqual(logUpdates, 100)
        XCTAssertEqual(platterUpdates, 100)
        XCTAssertEqual(settingsUpdates, 0)
        model.jog.ttNudgeCurveExponent = 2
        XCTAssertEqual(settingsUpdates, 1)
        withExtendedLifetime((subscription, logSubscription, platterSubscription)) {}
    }

    @MainActor
    func testMotorDiagnosticCaptureRequiresDebugLogLevel() throws {
        // Given the default info-level bridge configuration.
        let fixture = try makeFixture()
        let support = fixture.appendingPathComponent("support")
        let model = try testController(resourceURL: fixture, supportURL: support)

        // When a motor diagnostic capture is requested.
        model.startMotorDiagnosticCapture()

        // Then the app refuses an empty position capture until Debug is selected.
        XCTAssertFalse(model.isCapturingMotorDiagnostic)
        XCTAssertNil(model.motorDiagnosticURL)
        XCTAssertFalse(model.canStartMotorDiagnosticCapture)
        XCTAssertFalse(FileManager.default.fileExists(
            atPath: model.motorDiagnosticDirectoryURL.path
        ))
    }

    @MainActor
    func testMotorDiagnosticCaptureFiltersEventsAndOutlivesTheDisplayBuffer() throws {
        // Given Debug output and an explicit motor diagnostic capture.
        let fixture = try makeFixture()
        let support = fixture.appendingPathComponent("support")
        let model = try testController(resourceURL: fixture, supportURL: support)
        model.run.logLevel = "debug"
        model.startMotorDiagnosticCapture()
        let url = try XCTUnwrap(model.motorDiagnosticURL)

        // When many position events and unrelated Debug noise are received.
        let records = (0..<900).map { position in
            """
            2026-10-29T13:33:37.968918Z DEBUG MOTOR_JOG_POSITION side="right" deck=2 position=\(position % 128)
            2026-10-29T13:33:37.970920Z DEBUG TENSION_WRITE written=11
            """
        }.joined(separator: "\n")
        model.appendLog(records)
        model.appendLog(
            "2026-10-29T13:33:37.999000Z DEBUG MOTOR_JOG_BEND side=\"right\" deck=2 steps=1\n" +
            "2026-10-29T13:33:38.000500Z DEBUG MOTOR_REGULATOR data={\"host_us\":123456,\"side\":1,\"mode\":\"assisting\",\"counter\":702,\"sensor_timer\":12345600,\"motor_rpm\":0.0,\"requested_force\":5280,\"regulator\":null}\n" +
            "2026-10-29T13:33:38.000000Z  INFO MOTOR_PLAYBACK_GATE side=\"right\" playback_gate_open=true\n" +
            "2026-10-29T13:33:38.001000Z  INFO MOTOR_TT_STATE host_us=100000 touch=[false, false]"
        )
        model.stopMotorDiagnosticCapture()
        let capture = String(decoding: try Data(contentsOf: url), as: UTF8.self)

        // Then the complete filtered trace survives past the bounded UI buffer.
        XCTAssertFalse(model.isCapturingMotorDiagnostic)
        XCTAssertLessThanOrEqual(model.log.utf8.count, 60000)
        XCTAssertGreaterThan(capture.utf8.count, 60000)
        XCTAssertTrue(capture.contains("position=0"))
        XCTAssertTrue(capture.contains("position=3"))
        XCTAssertTrue(capture.contains("MOTOR_JOG_BEND"))
        XCTAssertTrue(capture.contains(
            "2026-10-29T13:33:38.000500Z DEBUG MOTOR_REGULATOR data={\"host_us\":123456,\"side\":1,\"mode\":\"assisting\",\"counter\":702,\"sensor_timer\":12345600,\"motor_rpm\":0.0,\"requested_force\":5280,\"regulator\":null}"
        ))
        XCTAssertTrue(capture.contains("MOTOR_PLAYBACK_GATE"))
        XCTAssertTrue(capture.contains("MOTOR_TT_STATE"))
        XCTAssertFalse(capture.contains("MOTOR_SCRATCH_MODE"))
        XCTAssertFalse(capture.contains("TENSION_WRITE"))
        XCTAssertTrue(url.path.hasPrefix(model.motorDiagnosticDirectoryURL.path))
    }

    @MainActor
    func testMotorCaptureUsesEachLaunchedReceiptAcrossRestartAndBoundedLogs() async throws {
        let fixture = try makeFixture()
        let fifo = fixture.appendingPathComponent("control.fifo")
        XCTAssertEqual(mkfifo(fifo.path, 0o600), 0)
        let descriptor = Darwin.open(fifo.path, O_RDWR)
        guard descriptor >= 0 else { throw AppError.invalid("Could not open test FIFO.") }
        defer { XCTAssertEqual(Darwin.close(descriptor), 0) }
        let executable = fixture.appendingPathComponent("s4mk3-hid-midi-bridge")
        let script = """
        #!/bin/sh
        trap 'exit 0' INT
        jog=""
        while [ $# -gt 0 ]; do
          case "$1" in
            --jog-config) jog="$2"; shift 2;;
            *) shift;;
          esac
        done
        [ -n "$jog" ] || exit 2
        jog_json=$(tr -d '[:space:]' < "$jog")
        printf 'S4_MOTOR_CONFIG {"jog":%s}\\n' "$jog_json"
        case "$jog_json" in
          *'"platter_target_rpm":33'*) ready=READY_INITIAL;;
          *'"platter_target_rpm":45'*) ready=READY_REPLACEMENT;;
          *) exit 3;;
        esac
        exec 3< '\(fifo.path)'
        printf '%s\\n' "$ready"
        IFS= read -r line <&3
        """
        try Data(script.utf8).write(to: executable)
        try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: executable.path)
        let model = try testController(resourceURL: fixture, supportURL: fixture.appendingPathComponent("support"))
        model.run.logLevel = "debug"
        let firstReady = expectation(description: "first launched receipt reached the app")
        let replacementReady = expectation(description: "replacement launched with saved jog settings")
        let stopped = expectation(description: "replacement child acknowledged stop")
        var finalStop = false
        var didSeeInitial = false
        var didSeeReplacement = false
        let output = model.logOutput.$text.sink { text in
            if !didSeeInitial && text.contains("READY_INITIAL") {
                didSeeInitial = true
                firstReady.fulfill()
            }
            if !didSeeReplacement && text.contains("READY_REPLACEMENT") {
                didSeeReplacement = true
                replacementReady.fulfill()
            }
        }
        let state = model.$state.sink { current in
            if finalStop && current == .stopped {
                finalStop = false
                stopped.fulfill()
            }
        }

        model.start()
        await fulfillment(of: [firstReady], timeout: 10)
        model.appendLog(String(repeating: "bounded display noise\n", count: 4000))
        model.jog.ttPitchBendGain = 10
        try model.save()
        model.startMotorDiagnosticCapture()
        let captureURL = try XCTUnwrap(model.motorDiagnosticURL)
        model.jog.platterTargetRpm = 45
        model.apply()
        await fulfillment(of: [replacementReady], timeout: 10)
        XCTAssertTrue(model.log.contains("\"platter_target_rpm\":45"))
        finalStop = true
        model.stop()
        await fulfillment(of: [stopped], timeout: 10)
        model.stopMotorDiagnosticCapture()
        withExtendedLifetime((output, state)) {}

        let capture = String(decoding: try Data(contentsOf: captureURL), as: UTF8.self)
        let receipts = capture.components(separatedBy: "\n").filter {
            $0.hasPrefix("# S4_MOTOR_CONFIG launchID=") && !$0.hasSuffix("pending")
        }
        guard receipts.count == 2 else {
            XCTFail("Expected two launched configuration receipts, received \(receipts.count).")
            return
        }
        let settings = try receipts.map { line -> [String: Any] in
            let start = try XCTUnwrap(line.firstIndex(of: "{"))
            let payload = try XCTUnwrap(JSONSerialization.jsonObject(
                with: Data(line[start...].utf8)
            ) as? [String: Any])
            return try XCTUnwrap(payload["jog"] as? [String: Any])
        }
        XCTAssertEqual(settings[0]["tt_pitch_bend_gain"] as? Double, 5)
        XCTAssertEqual(settings[1]["tt_pitch_bend_gain"] as? Double, 10)
        XCTAssertEqual(settings[0]["platter_target_rpm"] as? Int, 33)
        XCTAssertEqual(settings[1]["platter_target_rpm"] as? Int, 45)
        XCTAssertNotEqual(
            receipts[0].components(separatedBy: " ")[2],
            receipts[1].components(separatedBy: " ")[2]
        )
        XCTAssertLessThanOrEqual(model.log.utf8.count, 60000)
        XCTAssertTrue(FileManager.default.fileExists(atPath: model.jogURL.path))
        let saved = try JogPreferences.decode(Data(contentsOf: model.jogURL), defaults: jogDefaultsData())
        XCTAssertEqual(saved.platterTargetRpm, 45)
    }

    @MainActor
    func testInvalidSavedPreferencesArePreservedUntilExplicitReset() throws {
        let fixture = try makeFixture()
        let support = fixture.appendingPathComponent("support")
        try FileManager.default.createDirectory(at: support, withIntermediateDirectories: true)
        let saved = support.appendingPathComponent("led-config.json")
        let invalid = Data(#"{"deck_colors":[]}"#.utf8)
        try invalid.write(to: saved)
        let model = try testController(resourceURL: fixture, supportURL: support)
        XCTAssertTrue(model.requiresReset)
        XCTAssertThrowsError(try model.save())
        XCTAssertEqual(try Data(contentsOf: saved), invalid)
        model.resetPreferences()
        try model.save()
        XCTAssertFalse(model.requiresReset)
        let loaded = try LedPreferences.decode(Data(contentsOf: saved), defaults: defaultData())
        XCTAssertEqual(loaded, model.leds)
    }

    @MainActor
    func testDiagnosticProcessDrainsRealPipeBeforeReturningStopped() async throws {
        let fixture = try makeFixture()
        let executable = fixture.appendingPathComponent("s4mk3-hid-midi-bridge")
        try Data("#!/bin/sh\nprintf 'CHECK:%s\\n' \"$*\"\n".utf8).write(to: executable)
        try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: executable.path)
        let model = try testController(resourceURL: fixture, supportURL: fixture.appendingPathComponent("support"))
        let ended = expectation(description: "enumeration subprocess exited and output was drained")
        var checking = false
        let subscription = model.$state.sink { state in
            if state == .checking { checking = true }
            if checking && state == .stopped { ended.fulfill() }
        }
        model.checkController()
        await fulfillment(of: [ended], timeout: 10)
        withExtendedLifetime(subscription) {}
        XCTAssertFalse(model.isBusy)
        XCTAssertEqual(model.state, .stopped)
        XCTAssertTrue(model.log.contains("CHECK:--list"))
        XCTAssertFalse(model.log.contains("--jog-config"))
        XCTAssertFalse(model.log.contains("--tension-config"))
        XCTAssertFalse(model.log.contains("--screens"))
        XCTAssertFalse(FileManager.default.fileExists(atPath: model.jogURL.path))
        XCTAssertFalse(FileManager.default.fileExists(atPath: model.tensionURL.path))
        XCTAssertTrue(model.log.contains("EXIT: 0"))
    }

    @MainActor
    func testStopSendsSIGINTAndWaitsForChildAcknowledgement() async throws {
        let fixture = try makeFixture()
        let argumentsURL = fixture.appendingPathComponent("received-arguments.txt")
        let fifo = fixture.appendingPathComponent("control.fifo")
        XCTAssertEqual(mkfifo(fifo.path, 0o600), 0)
        let descriptor = Darwin.open(fifo.path, O_RDWR)
        guard descriptor >= 0 else { throw AppError.invalid("Could not open test FIFO.") }
        defer { XCTAssertEqual(Darwin.close(descriptor), 0) }
        let executable = fixture.appendingPathComponent("s4mk3-hid-midi-bridge")
        let script = """
        #!/bin/sh
        trap 'printf "STOP_ACK\\n"; exit 0' INT
        printf '%s\\n' "$@" > '\(argumentsURL.path)'
        exec 3< '\(fifo.path)'
        printf 'HELPER_READY\\n'
        IFS= read -r line <&3
        """
        try Data(script.utf8).write(to: executable)
        try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: executable.path)
        let model = try testController(resourceURL: fixture, supportURL: fixture.appendingPathComponent("support"))
        let ready = expectation(description: "child installed its signal handler and opened the FIFO")
        let exited = expectation(description: "child acknowledged SIGINT and exited")
        var didSignalReady = false
        var sawStopping = false
        let output = model.logOutput.$text.sink { text in
            if !didSignalReady && text.contains("HELPER_READY") {
                didSignalReady = true
                ready.fulfill()
            }
        }
        let state = model.$state.sink { current in
            if current == .stopping { sawStopping = true }
            if sawStopping && current == .stopped { exited.fulfill() }
        }
        model.start()
        await fulfillment(of: [ready], timeout: 10)
        XCTAssertEqual(try String(contentsOf: argumentsURL, encoding: .utf8)
            .split(separator: "\n").map(String.init), [
                "--midi", "full", "--screen-control-stdin", "--screens", "--led-config", model.ledURL.path,
                "--jog-config", model.jogURL.path, "--tension-config", model.tensionURL.path, "--until-stopped"
            ])
        model.stop()
        await fulfillment(of: [exited], timeout: 10)
        withExtendedLifetime((output, state)) {}
        XCTAssertFalse(model.isBusy)
        XCTAssertTrue(model.log.contains("STOP_ACK"))
        XCTAssertTrue(model.log.contains("EXIT: 0"))
    }

    func testJogPreferencesRoundTripAndSchema() throws {
        let defaults = try jogDefaultsData()
        let base = try JogPreferences.decode(defaults, defaults: defaults)
        XCTAssertEqual(base.scratchSpeed, 2.7)
        XCTAssertEqual(base.scratchReaction, 150)
        XCTAssertEqual(base.pitchBendSpeed, 2.7)
        XCTAssertEqual(base.pitchBendReaction, 17)
        XCTAssertTrue(base.backspinEnabled)
        XCTAssertEqual(base.backspinAcceleration, 150000.0)
        XCTAssertEqual(base.backspinSpeedIncrease, 5.2)
        XCTAssertTrue(base.backspinAssistEnabled)
        XCTAssertEqual(base.backspinAssistStrength, 100)
        XCTAssertTrue(base.vinylEnabledAtStartup)
        XCTAssertFalse(base.ttVinylEnabledAtStartup)
        XCTAssertEqual(base.ttPitchBendFloorPercent, 1)
        XCTAssertEqual(base.platterTargetRpm, 33)
        XCTAssertEqual(base.platterMaxForce, 20000)
        XCTAssertEqual(base.platterBaseForce, 7000)
        XCTAssertEqual(base.platterForcePerRpm, 110)
        XCTAssertEqual(base.platterKp, 4000)
        XCTAssertEqual(base.platterKi, 3000)
        XCTAssertEqual(base.platterForceRisePerSecond, 200000)
        XCTAssertEqual(base.platterStartupForceRisePerSecond, 0)
        XCTAssertEqual(base.platterStartupMs, 200)
        XCTAssertEqual(base.platterOverspeedLimitRpm, 0)
        XCTAssertEqual(base, JogPreferences())
        XCTAssertEqual(base, try JogPreferences.decode(base.encoded(), defaults: defaults))
        let fourFieldSaved = Data(#"{"scratch_speed":3.5,"scratch_reaction":140,"pitch_bend_speed":2.2,"pitch_bend_reaction":42}"#.utf8)
        let mergedFourField = try JogPreferences.decode(fourFieldSaved, defaults: defaults)
        XCTAssertEqual(mergedFourField.scratchSpeed, 3.5)
        XCTAssertEqual(mergedFourField.scratchReaction, 140)
        XCTAssertEqual(mergedFourField.pitchBendSpeed, 2.2)
        XCTAssertEqual(mergedFourField.pitchBendReaction, 42)
        XCTAssertTrue(mergedFourField.backspinEnabled)
        XCTAssertEqual(mergedFourField.backspinAcceleration, 150000.0)
        XCTAssertEqual(mergedFourField.backspinSpeedIncrease, 5.2)
        XCTAssertTrue(mergedFourField.backspinAssistEnabled)
        XCTAssertTrue(mergedFourField.vinylEnabledAtStartup)
        XCTAssertEqual(mergedFourField.platterTargetRpm, 33)
        XCTAssertEqual(mergedFourField.platterMaxForce, 20000)
        let sevenFieldSaved = Data(#"{"scratch_speed":2.7,"scratch_reaction":150,"pitch_bend_speed":2.7,"pitch_bend_reaction":17,"backspin_enabled":true,"backspin_acceleration":150000.0,"backspin_speed_increase":5.2}"#.utf8)
        let mergedSevenField = try JogPreferences.decode(sevenFieldSaved, defaults: defaults)
        XCTAssertEqual(mergedSevenField.scratchSpeed, 2.7)
        XCTAssertEqual(mergedSevenField.scratchReaction, 150)
        XCTAssertEqual(mergedSevenField.pitchBendSpeed, 2.7)
        XCTAssertEqual(mergedSevenField.pitchBendReaction, 17)
        XCTAssertTrue(mergedSevenField.backspinEnabled)
        XCTAssertEqual(mergedSevenField.backspinAcceleration, 150000.0)
        XCTAssertEqual(mergedSevenField.backspinSpeedIncrease, 5.2)
        XCTAssertTrue(mergedSevenField.backspinAssistEnabled)
        XCTAssertEqual(mergedSevenField.backspinAssistStrength, 100)
        XCTAssertTrue(mergedSevenField.vinylEnabledAtStartup)
        XCTAssertEqual(mergedSevenField.platterTargetRpm, 33)
        let nineFieldLegacy = Data(#"{"scratch_speed":2.7,"scratch_reaction":150,"pitch_bend_speed":2.7,"pitch_bend_reaction":17,"backspin_enabled":true,"backspin_assist_enabled":true,"backspin_assist_strength":80,"backspin_acceleration":150000.0,"backspin_speed_increase":5.6}"#.utf8)
        let mergedNineField = try JogPreferences.decode(nineFieldLegacy, defaults: nineFieldLegacy)
        XCTAssertEqual(mergedNineField.backspinAssistStrength, 80)
        XCTAssertTrue(mergedNineField.vinylEnabledAtStartup)
        XCTAssertEqual(mergedNineField.platterTargetRpm, 33)
        XCTAssertEqual(mergedNineField.platterMaxForce, 20000)
        XCTAssertEqual(mergedNineField.platterBaseForce, 7000)
        XCTAssertEqual(mergedNineField.platterForcePerRpm, 110)
        XCTAssertEqual(mergedNineField.platterKp, 4000)
        XCTAssertEqual(mergedNineField.platterKi, 3000)
        XCTAssertEqual(mergedNineField.platterForceRisePerSecond, 200000)
        XCTAssertEqual(mergedNineField.platterStartupForceRisePerSecond, 0)
        XCTAssertEqual(mergedNineField.platterStartupMs, 200)
        XCTAssertEqual(mergedNineField.platterOverspeedLimitRpm, 0)
        let savedOff = Data(#"{"backspin_speed_increase":0.0,"backspin_assist_enabled":false,"backspin_assist_strength":37}"#.utf8)
        let keptOff = try JogPreferences.decode(savedOff, defaults: defaults)
        XCTAssertEqual(keptOff.backspinSpeedIncrease, 0.0)
        XCTAssertFalse(keptOff.backspinAssistEnabled)
        XCTAssertEqual(keptOff.backspinAssistStrength, 37)
        XCTAssertEqual(keptOff, try JogPreferences.decode(keptOff.encoded(), defaults: defaults))
        let override = Data(#"{"scratch_speed":3.5,"pitch_bend_reaction":42,"backspin_enabled":false,"backspin_acceleration":200000.0,"backspin_speed_increase":5.2,"backspin_assist_enabled":true,"backspin_assist_strength":37,"vinyl_enabled_at_startup":false,"platter_target_rpm":45,"platter_max_force":18000,"platter_base_force":1500,"platter_force_per_rpm":120,"platter_kp":4500,"platter_ki":2500,"platter_force_rise_per_second":250000,"platter_startup_force_rise_per_second":50000,"platter_startup_ms":300,"platter_overspeed_limit_rpm":90}"#.utf8)
        let changed = try JogPreferences.decode(override, defaults: defaults)
        XCTAssertEqual(changed.scratchSpeed, 3.5)
        XCTAssertEqual(changed.pitchBendReaction, 42)
        XCTAssertFalse(changed.backspinEnabled)
        XCTAssertEqual(changed.backspinAcceleration, 200000.0)
        XCTAssertEqual(changed.backspinSpeedIncrease, 5.2)
        XCTAssertTrue(changed.backspinAssistEnabled)
        XCTAssertEqual(changed.backspinAssistStrength, 37)
        XCTAssertFalse(changed.vinylEnabledAtStartup)
        XCTAssertEqual(changed.platterTargetRpm, 45)
        XCTAssertEqual(changed.platterMaxForce, 18000)
        XCTAssertEqual(changed.platterBaseForce, 1500)
        XCTAssertEqual(changed.platterForcePerRpm, 120)
        XCTAssertEqual(changed.platterKp, 4500)
        XCTAssertEqual(changed.platterKi, 2500)
        XCTAssertEqual(changed.platterForceRisePerSecond, 250000)
        XCTAssertEqual(changed.platterStartupForceRisePerSecond, 50000)
        XCTAssertEqual(changed.platterStartupMs, 300)
        XCTAssertEqual(changed.platterOverspeedLimitRpm, 90)
        XCTAssertEqual(changed.scratchReaction, base.scratchReaction)
        XCTAssertEqual(changed.pitchBendSpeed, base.pitchBendSpeed)
        XCTAssertEqual(changed, try JogPreferences.decode(changed.encoded(), defaults: defaults))
        let keys = try XCTUnwrap(JSONSerialization.jsonObject(with: base.encoded()) as? [String: Any])
        XCTAssertNotNil(keys["scratch_speed"])
        XCTAssertNotNil(keys["pitch_bend_reaction"])
        XCTAssertNotNil(keys["backspin_enabled"])
        XCTAssertNotNil(keys["backspin_acceleration"])
        XCTAssertNotNil(keys["backspin_speed_increase"])
        XCTAssertNotNil(keys["backspin_assist_enabled"])
        XCTAssertEqual(keys["backspin_assist_strength"] as? Int, 100)
        XCTAssertEqual(keys["vinyl_enabled_at_startup"] as? Bool, true)
        XCTAssertEqual(keys["platter_target_rpm"] as? Int, 33)
        XCTAssertEqual(keys["platter_max_force"] as? Int, 20000)
        XCTAssertEqual(keys["platter_base_force"] as? Int, 7000)
        XCTAssertEqual(keys["platter_force_per_rpm"] as? Int, 110)
        XCTAssertEqual(keys["platter_kp"] as? Int, 4000)
        XCTAssertEqual(keys["platter_ki"] as? Int, 3000)
        XCTAssertEqual(keys["platter_force_rise_per_second"] as? Int, 200000)
        XCTAssertEqual(keys["platter_startup_force_rise_per_second"] as? Int, 0)
        XCTAssertEqual(keys["platter_startup_ms"] as? Int, 200)
        XCTAssertEqual(keys["platter_overspeed_limit_rpm"] as? Int, 0)
        XCTAssertNil(keys["scratchSpeed"])
        XCTAssertNil(keys["backspinEnabled"])
        XCTAssertNil(keys["backspinAssistEnabled"])
        XCTAssertNil(keys["backspinAssistStrength"])
        XCTAssertNil(keys["vinylEnabledAtStartup"])
        XCTAssertNil(keys["platterTargetRpm"])
        XCTAssertNil(keys["platterMaxForce"])
        XCTAssertNil(keys["platterBaseForce"])
        XCTAssertNil(keys["platterForcePerRpm"])
        XCTAssertNil(keys["platterKp"])
        XCTAssertNil(keys["platterKi"])
        XCTAssertNil(keys["platterForceRisePerSecond"])
        XCTAssertNil(keys["platterStartupForceRisePerSecond"])
        XCTAssertNil(keys["platterStartupMs"])
        XCTAssertNil(keys["platterOverspeedLimitRpm"])
        for input in [
            #"{"scratch_speed":0}"#, #"{"scratch_speed":-2.5}"#,
            #"{"pitch_bend_speed":0}"#, #"{"scratch_reaction":-1}"#,
            #"{"scratch_reaction":151}"#, #"{"pitch_bend_reaction":151}"#,
            #"{"scratch_reaction":1.5}"#, #"{"unknown":true}"#,
            #"{"backspin_enabled":1}"#, #"{"backspin_enabled":"true"}"#,
            #"{"backspin_acceleration":0}"#, #"{"backspin_acceleration":-1000}"#,
            #"{"backspin_speed_increase":-0.1}"#,
            #"{"backspin_assist_enabled":1}"#, #"{"backspin_assist_enabled":"true"}"#,
            #"{"backspin_assist_enabled":null}"#,
            #"{"backspin_assist_strength":-1}"#, #"{"backspin_assist_strength":101}"#,
            #"{"backspin_assist_strength":1.5}"#, #"{"backspin_assist_strength":1.0}"#,
            #"{"backspin_assist_strength":1e2}"#, #"{"backspin_assist_strength":true}"#,
            #"{"backspin_assist_strength":false}"#, #"{"backspin_assist_strength":"50"}"#,
            #"{"backspin_assist_strength":null}"#,
            #"{"vinyl_enabled_at_startup":1}"#, #"{"vinyl_enabled_at_startup":"true"}"#,
            #"{"vinyl_enabled_at_startup":null}"#,
            #"{"platter_target_rpm":0}"#, #"{"platter_target_rpm":46}"#,
            #"{"platter_target_rpm":33.0}"#, #"{"platter_target_rpm":true}"#,
            #"{"platter_max_force":0}"#, #"{"platter_max_force":25001}"#,
            #"{"platter_max_force":20000.0}"#, #"{"platter_max_force":false}"#,
            #"{"platter_base_force":-1}"#, #"{"platter_base_force":25001}"#,
            #"{"platter_base_force":1650.0}"#,
            #"{"platter_force_per_rpm":-1}"#, #"{"platter_force_per_rpm":25001}"#,
            #"{"platter_force_per_rpm":110.5}"#,
            #"{"platter_kp":-1}"#, #"{"platter_kp":25001}"#,
            #"{"platter_kp":4000.0}"#,
            #"{"platter_ki":-1}"#, #"{"platter_ki":25001}"#,
            #"{"platter_ki":3000.0}"#,
            #"{"platter_force_rise_per_second":-1}"#, #"{"platter_force_rise_per_second":1000001}"#,
            #"{"platter_force_rise_per_second":1e5}"#,
            #"{"platter_startup_force_rise_per_second":-1}"#, #"{"platter_startup_force_rise_per_second":1000001}"#,
            #"{"platter_startup_force_rise_per_second":0.0}"#,
            #"{"platter_startup_ms":-1}"#, #"{"platter_startup_ms":2001}"#,
            #"{"platter_startup_ms":200.0}"#,
            #"{"platter_overspeed_limit_rpm":-1}"#, #"{"platter_overspeed_limit_rpm":251}"#,
            #"{"platter_overspeed_limit_rpm":71.0}"#
        ] {
            XCTAssertThrowsError(try JogPreferences.decode(Data(input.utf8), defaults: defaults), input)
        }
        var overflow = base
        overflow.scratchSpeed = .greatestFiniteMagnitude
        overflow.backspinSpeedIncrease = .greatestFiniteMagnitude
        XCTAssertThrowsError(try overflow.validate())
        var nonFiniteAccel = base
        nonFiniteAccel.backspinAcceleration = .infinity
        XCTAssertThrowsError(try nonFiniteAccel.validate())
        var nonFiniteIncrease = base
        nonFiniteIncrease.backspinSpeedIncrease = .nan
        XCTAssertThrowsError(try nonFiniteIncrease.validate())
    }

    @MainActor
    func testRestoreMotorizedDefaultsKeepsOtherEditorsAndSavedPreferences() throws {
        let fixture = try makeFixture()
        let support = fixture.appendingPathComponent("support")
        try FileManager.default.createDirectory(at: support, withIntermediateDirectories: true)
        let model = try testController(resourceURL: fixture, supportURL: support)
        let defaults = model.jog
        var expected = defaults
        expected.scratchSpeed = 4.5
        expected.scratchReaction = 80
        expected.scratchDirection = .reversed
        expected.pitchBendSpeed = 3.1
        expected.pitchBendReaction = 40
        expected.backspinEnabled = false
        expected.backspinAcceleration = 180000
        expected.backspinSpeedIncrease = 1.5
        expected.backspinAssistEnabled = false
        expected.backspinAssistStrength = 37
        model.jog = expected
        model.jog.vinylEnabledAtStartup = false
        model.jog.ttVinylEnabledAtStartup = true
        model.jog.ttPitchBendFloorPercent = 73
        model.jog.ttPitchBendGain = 2
        model.jog.ttBrakeGain = 4
        model.jog.ttNudgeGain = 6
        model.jog.ttBrakeCurveExponent = 0.5
        model.jog.ttNudgeCurveExponent = 2.5
        model.jog.platterTargetRpm = 45
        model.jog.platterMaxForce = 18000
        model.jog.platterBaseForce = 3500
        model.jog.platterForcePerRpm = 120
        model.jog.platterKp = 4500
        model.jog.platterKi = 2500
        model.jog.platterForceRisePerSecond = 250000
        model.jog.platterStartupForceRisePerSecond = 50000
        model.jog.platterStartupMs = 300
        model.jog.platterOverspeedLimitRpm = 90
        model.tension.tension = 37
        model.run.durationSeconds = 45
        let tension = model.tension
        let run = model.run
        let leds = model.leds
        let saved = try model.jog.encoded()
        try saved.write(to: model.jogURL)

        model.restoreMotorizedDefaults()

        XCTAssertEqual(model.jog, expected)
        XCTAssertEqual(model.tension, tension)
        XCTAssertEqual(model.run, run)
        XCTAssertEqual(model.leds, leds)
        XCTAssertEqual(try Data(contentsOf: model.jogURL), saved)
        XCTAssertEqual(model.state, .stopped)
        XCTAssertEqual(defaults.effectiveTTBrakeGain, 10)
        XCTAssertEqual(defaults.effectiveTTNudgeGain, 3)
    }

    func testRetiredAbsolutePlatterSettingLoadsButIsNotSaved() throws {
        let defaults = try jogDefaultsData()
        let legacy = Data(#"{"scratch_speed":3.5,"scratch_direction":"reversed","platter_position_sensitivity":11.5}"#.utf8)
        let merged = try JogPreferences.decode(legacy, defaults: defaults)
        XCTAssertEqual(merged.scratchSpeed, 3.5)
        XCTAssertEqual(merged.scratchDirection, .reversed)
        XCTAssertEqual(try JogPreferences.decode(merged.encoded(), defaults: defaults), merged)
        let keys = try XCTUnwrap(JSONSerialization.jsonObject(with: merged.encoded()) as? [String: Any])
        XCTAssertNil(keys["platter_position_sensitivity"])
        XCTAssertEqual(keys["scratch_direction"] as? String, "reversed")

        for input in [
            #"{"scratch_direction":true}"#,
            #"{"scratch_direction":"sideways"}"#,
            #"{"unrecognized_setting":14}"#
        ] {
            XCTAssertThrowsError(try JogPreferences.decode(Data(input.utf8), defaults: defaults), input)
        }
    }

    func testTurntablePreferencesMergeLegacyAndRoundTripIndependently() throws {
        let defaults = try jogDefaultsData()
        let legacy = Data(#"{"vinyl_enabled_at_startup":false}"#.utf8)
        let merged = try JogPreferences.decode(legacy, defaults: defaults)
        XCTAssertFalse(merged.vinylEnabledAtStartup)
        XCTAssertFalse(merged.ttVinylEnabledAtStartup)
        XCTAssertEqual(merged.ttPitchBendFloorPercent, 1)

        var changed = merged
        changed.ttVinylEnabledAtStartup = true
        changed.ttPitchBendFloorPercent = 73
        let reloaded = try JogPreferences.decode(changed.encoded(), defaults: defaults)
        XCTAssertFalse(reloaded.vinylEnabledAtStartup)
        XCTAssertTrue(reloaded.ttVinylEnabledAtStartup)
        XCTAssertEqual(reloaded.ttPitchBendFloorPercent, 73)
        let keys = try XCTUnwrap(JSONSerialization.jsonObject(with: reloaded.encoded()) as? [String: Any])
        XCTAssertEqual(keys["tt_vinyl_enabled_at_startup"] as? Bool, true)
        XCTAssertEqual(keys["tt_pitch_bend_floor_percent"] as? Int, 73)

        for input in [
            #"{"tt_pitch_bend_floor_percent":0}"#,
            #"{"tt_pitch_bend_floor_percent":96}"#,
            #"{"tt_pitch_bend_floor_percent":50.0}"#,
            #"{"tt_pitch_bend_floor_percent":true}"#
        ] {
            XCTAssertThrowsError(try JogPreferences.decode(Data(input.utf8), defaults: defaults), input)
        }
    }

    func testTurntableGainDefaultsAndRoundTripsWithoutChangingFloor() throws {
        let defaults = try jogDefaultsData()
        let legacy = Data(#"{"tt_pitch_bend_floor_percent":1}"#.utf8)
        let base = try JogPreferences.decode(legacy, defaults: defaults)
        XCTAssertEqual(base.ttPitchBendGain, 5.0)
        for gain in [0.1, 1.0, 4.0, 10.0] {
            var changed = base
            changed.ttPitchBendGain = gain
            let reloaded = try JogPreferences.decode(changed.encoded(), defaults: defaults)
            XCTAssertEqual(reloaded.ttPitchBendGain, gain)
            XCTAssertEqual(reloaded.ttPitchBendFloorPercent, 1)
            let encoded = try XCTUnwrap(JSONSerialization.jsonObject(with: reloaded.encoded()) as? [String: Any])
            XCTAssertEqual(encoded["tt_pitch_bend_gain"] as? Double, gain)
        }
        for gain in [0.0, 0.09, 10.1, .nan, .infinity] {
            var invalid = base
            invalid.ttPitchBendGain = gain
            XCTAssertThrowsError(try invalid.validate())
        }
        for input in [#"{"tt_pitch_bend_gain":true}"#, #"{"tt_pitch_bend_gain":"2"}"#] {
            XCTAssertThrowsError(try JogPreferences.decode(Data(input.utf8), defaults: defaults))
        }
    }

    func testTurntableBrakingAndNudgeCurvePreferencesInheritLegacyAndValidateIndependently() throws {
        let defaults = try jogDefaultsData()
        let legacy = Data(#"{"tt_pitch_bend_gain":5.0}"#.utf8)
        let inherited = try JogPreferences.decode(legacy, defaults: defaults)
        XCTAssertEqual(inherited.ttPitchBendGain, 5.0)
        XCTAssertNil(inherited.ttBrakeGain)
        XCTAssertNil(inherited.ttNudgeGain)
        XCTAssertEqual(inherited.effectiveTTBrakeGain, 10.0)
        XCTAssertEqual(inherited.effectiveTTNudgeGain, 5.0)
        XCTAssertEqual(inherited.ttBrakeCurveExponent, 1.0)
        XCTAssertEqual(inherited.ttNudgeCurveExponent, 1.6)

        let inheritedKeys = try XCTUnwrap(JSONSerialization.jsonObject(with: inherited.encoded()) as? [String: Any])
        XCTAssertNil(inheritedKeys["tt_brake_gain"])
        XCTAssertNil(inheritedKeys["tt_nudge_gain"])
        XCTAssertEqual(inheritedKeys["tt_brake_curve_exponent"] as? Double, 1.0)
        XCTAssertEqual(inheritedKeys["tt_nudge_curve_exponent"] as? Double, 1.6)

        let nullOptional = Data(#"{"tt_pitch_bend_gain":4.0,"tt_brake_gain":null,"tt_nudge_gain":null}"#.utf8)
        let decodedNull = try JogPreferences.decode(nullOptional, defaults: defaults)
        XCTAssertNil(decodedNull.ttBrakeGain)
        XCTAssertNil(decodedNull.ttNudgeGain)
        XCTAssertEqual(decodedNull.effectiveTTBrakeGain, 8.0)
        XCTAssertEqual(decodedNull.effectiveTTNudgeGain, 4.0)

        var explicit = inherited
        explicit.ttBrakeGain = 10.0
        explicit.ttNudgeGain = 2.5
        explicit.ttBrakeCurveExponent = 0.75
        explicit.ttNudgeCurveExponent = 1.8
        let reloaded = try JogPreferences.decode(explicit.encoded(), defaults: defaults)
        XCTAssertEqual(reloaded, explicit)
        XCTAssertEqual(reloaded.effectiveTTBrakeGain, 10.0)
        XCTAssertEqual(reloaded.effectiveTTNudgeGain, 2.5)
        let explicitKeys = try XCTUnwrap(JSONSerialization.jsonObject(with: reloaded.encoded()) as? [String: Any])
        XCTAssertEqual(explicitKeys["tt_brake_gain"] as? Double, 10.0)
        XCTAssertEqual(explicitKeys["tt_nudge_gain"] as? Double, 2.5)
        XCTAssertEqual(explicitKeys["tt_brake_curve_exponent"] as? Double, 0.75)
        XCTAssertEqual(explicitKeys["tt_nudge_curve_exponent"] as? Double, 1.8)

        for brakeGain in [0.1, 20.0] {
            var valid = inherited
            valid.ttBrakeGain = brakeGain
            XCTAssertEqual(try JogPreferences.decode(valid.encoded(), defaults: defaults).ttBrakeGain, brakeGain)
        }
        for nudgeGain in [0.1, 10.0] {
            var valid = inherited
            valid.ttNudgeGain = nudgeGain
            XCTAssertEqual(try JogPreferences.decode(valid.encoded(), defaults: defaults).ttNudgeGain, nudgeGain)
        }
        for exponent in [0.25, 1.0, 4.0] {
            var valid = inherited
            valid.ttBrakeCurveExponent = exponent
            valid.ttNudgeCurveExponent = exponent
            let roundTripped = try JogPreferences.decode(valid.encoded(), defaults: defaults)
            XCTAssertEqual(roundTripped.ttBrakeCurveExponent, exponent)
            XCTAssertEqual(roundTripped.ttNudgeCurveExponent, exponent)
        }

        for invalidBrake in [0.0, 0.09, 20.1, .nan, .infinity] {
            var invalid = inherited
            invalid.ttBrakeGain = invalidBrake
            XCTAssertThrowsError(try invalid.validate())
        }
        for invalidNudge in [0.0, 0.09, 10.1, .nan, .infinity] {
            var invalid = inherited
            invalid.ttNudgeGain = invalidNudge
            XCTAssertThrowsError(try invalid.validate())
        }
        for invalidExponent in [0.0, 0.24, 4.01, .nan, .infinity] {
            var invalidBrakeExp = inherited
            invalidBrakeExp.ttBrakeCurveExponent = invalidExponent
            XCTAssertThrowsError(try invalidBrakeExp.validate())
            var invalidNudgeExp = inherited
            invalidNudgeExp.ttNudgeCurveExponent = invalidExponent
            XCTAssertThrowsError(try invalidNudgeExp.validate())
        }
        for input in [
            #"{"tt_brake_gain":true}"#,
            #"{"tt_brake_gain":"10"}"#,
            #"{"tt_nudge_gain":false}"#,
            #"{"tt_nudge_gain":"5"}"#,
            #"{"tt_brake_curve_exponent":true}"#,
            #"{"tt_brake_curve_exponent":null}"#,
            #"{"tt_nudge_curve_exponent":"1.0"}"#,
            #"{"tt_nudge_curve_exponent":null}"#
        ] {
            XCTAssertThrowsError(try JogPreferences.decode(Data(input.utf8), defaults: defaults), input)
        }
    }

    func testTTResponseCurveMatchesFormulaQuietBandSmoothOnsetExponentsAndLowTargetRpm() {
        let asymmetricParams = TTResponseParameters(
            targetRpm: 33,
            brakeGain: 10.0,
            brakeExponent: 0.5,
            nudgeGain: 5.0,
            nudgeExponent: 2.0
        )
        for measured in [33.0, 33.0 - 2.25, 33.0 + 2.25, 33.0 - (2.25 + 1e-9), 33.0 + (2.25 + 1e-9)] {
            XCTAssertEqual(
                TTResponseCurve.signedCc4Rate(parameters: asymmetricParams, measuredRpm: measured),
                0.0,
                accuracy: 1e-12
            )
        }

        let legacy5xParams = TTResponseParameters(
            targetRpm: 33,
            brakeGain: 10.0,
            brakeExponent: 1.0,
            nudgeGain: 5.0,
            nudgeExponent: 1.0
        )
        let midOnsetBraking = TTResponseCurve.signedCc4Rate(
            parameters: legacy5xParams,
            measuredRpm: 33.0 - 2.75
        )
        XCTAssertEqual(midOnsetBraking, -2.75 * 0.5 * 10.0 * 48.0, accuracy: 1e-9)

        let fullStopLegacy5x = TTResponseCurve.signedCc4Rate(
            parameters: legacy5xParams,
            measuredRpm: 0.0
        )
        XCTAssertEqual(fullStopLegacy5x, -33.0 * 10.0 * 48.0, accuracy: 1e-9)

        let halfTargetError = 16.5
        let brakeShape1 = TTResponseCurve.signedCc4Rate(
            parameters: legacy5xParams,
            measuredRpm: 33.0 - halfTargetError
        )
        let brakeSoft = TTResponseCurve.signedCc4Rate(
            parameters: TTResponseParameters(targetRpm: 33, brakeGain: 10.0, brakeExponent: 2.0, nudgeGain: 5.0, nudgeExponent: 1.0),
            measuredRpm: 33.0 - halfTargetError
        )
        let brakeStrong = TTResponseCurve.signedCc4Rate(
            parameters: TTResponseParameters(targetRpm: 33, brakeGain: 10.0, brakeExponent: 0.5, nudgeGain: 5.0, nudgeExponent: 1.0),
            measuredRpm: 33.0 - halfTargetError
        )
        XCTAssertEqual(brakeSoft, brakeShape1 * 0.5, accuracy: 1e-9)
        XCTAssertEqual(brakeStrong, brakeShape1 * pow(0.5, -0.5), accuracy: 1e-9)
        XCTAssertLessThan(abs(brakeSoft), abs(brakeShape1))
        XCTAssertGreaterThan(abs(brakeStrong), abs(brakeShape1))

        let nudgeAboveTarget = TTResponseCurve.signedCc4Rate(
            parameters: TTResponseParameters(targetRpm: 33, brakeGain: 10.0, brakeExponent: 0.5, nudgeGain: 3.0, nudgeExponent: 3.5),
            measuredRpm: 33.0 + 40.0
        )
        XCTAssertEqual(nudgeAboveTarget, 40.0 * 1.0 * 3.0 * 1.0 * 48.0, accuracy: 1e-9)

        let lowTargetBrakingSamples = TTResponseCurve.samples(
            direction: .braking,
            parameters: TTResponseParameters(targetRpm: 1, brakeGain: 10.0, brakeExponent: 0.5, nudgeGain: 5.0, nudgeExponent: 1.0),
            steps: 16
        )
        XCTAssertTrue(lowTargetBrakingSamples.allSatisfy {
            $0.signedCc4Rate == 0.0 && $0.magnitudeCc4Rate == 0.0 && $0.signedCc4Rate.isFinite
        })
        let lowTargetHighNudge = TTResponseCurve.signedCc4Rate(
            parameters: TTResponseParameters(targetRpm: 1, brakeGain: 10.0, brakeExponent: 0.5, nudgeGain: 2.0, nudgeExponent: 2.5),
            measuredRpm: 4.5
        )
        XCTAssertEqual(lowTargetHighNudge, 3.5 * 1.0 * 2.0 * 1.0 * 48.0, accuracy: 1e-9)
    }

    func testMotorizedPlatterBoundsAndIntegerJSONTypes() throws {
        let defaults = try jogDefaultsData()
        let fields: [(String, WritableKeyPath<JogPreferences, Int>, ClosedRange<Int>)] = [
            ("tt_pitch_bend_floor_percent", \.ttPitchBendFloorPercent, 1...95),
            ("platter_target_rpm", \.platterTargetRpm, 1...45),
            ("platter_max_force", \.platterMaxForce, 1...25000),
            ("platter_base_force", \.platterBaseForce, 0...25000),
            ("platter_force_per_rpm", \.platterForcePerRpm, 0...25000),
            ("platter_kp", \.platterKp, 0...25000),
            ("platter_ki", \.platterKi, 0...25000),
            ("platter_force_rise_per_second", \.platterForceRisePerSecond, 0...1000000),
            ("platter_startup_force_rise_per_second", \.platterStartupForceRisePerSecond, 0...1000000),
            ("platter_startup_ms", \.platterStartupMs, 0...2000),
            ("platter_overspeed_limit_rpm", \.platterOverspeedLimitRpm, 0...250)
        ]
        for (key, path, range) in fields {
            for bound in [range.lowerBound, range.upperBound] {
                var value = JogPreferences()
                value[keyPath: path] = bound
                XCTAssertEqual(try JogPreferences.decode(value.encoded(), defaults: defaults), value, key)
                let partial = Data("{\"\(key)\":\(bound)}".utf8)
                XCTAssertEqual(try JogPreferences.decode(partial, defaults: defaults)[keyPath: path], bound, key)
            }
            for invalid in [range.lowerBound - 1, range.upperBound + 1] {
                var value = JogPreferences()
                value[keyPath: path] = invalid
                XCTAssertThrowsError(try value.encoded(), key)
            }
            for token in ["true", "false", "null", "\"33\"", "[]", "{}", "1.0", "1e0", "1.5"] {
                let invalid = Data("{\"\(key)\":\(token)}".utf8)
                XCTAssertThrowsError(try JogPreferences.decode(invalid, defaults: defaults), "\(key): \(token)")
            }
        }
    }

    @MainActor
    func testInvalidMotorizedPlatterPreferencesPreserveSavedFiles() throws {
        let fixture = try makeFixture()
        let support = fixture.appendingPathComponent("support")
        let model = try testController(resourceURL: fixture, supportURL: support)
        model.leds.loopColor = "blue"
        model.tension.tension = 42
        model.run.logLevel = "debug"
        model.jog.scratchSpeed = 3.5
        try model.save()
        let urls = [model.jogURL, model.ledURL, model.tensionURL, model.runURL]
        let saved = try urls.map { try Data(contentsOf: $0) }
        model.jog.platterTargetRpm = 46
        model.apply()
        XCTAssertFalse(model.errorMessage.isEmpty)
        XCTAssertEqual(model.state, .stopped)
        XCTAssertFalse(model.hasChildProcess)
        XCTAssertEqual(try urls.map { try Data(contentsOf: $0) }, saved)

        let invalid = Data(#"{"platter_max_force":25001}"#.utf8)
        try invalid.write(to: model.jogURL)
        let broken = try testController(resourceURL: fixture, supportURL: support)
        XCTAssertTrue(broken.requiresReset)
        XCTAssertEqual(broken.leds, model.leds)
        XCTAssertEqual(broken.tension, model.tension)
        XCTAssertEqual(broken.run, model.run)
        XCTAssertThrowsError(try broken.save())
        XCTAssertEqual(try Data(contentsOf: broken.jogURL), invalid)
        broken.resetPreferences()
        XCTAssertEqual(broken.jog, JogPreferences())
        XCTAssertEqual(try Data(contentsOf: broken.jogURL), invalid)
        try broken.save()
        let restored = try testController(resourceURL: fixture, supportURL: support)
        XCTAssertFalse(restored.requiresReset)
        XCTAssertEqual(restored.jog, JogPreferences())
    }

    @MainActor
    func testJogSavedPreferencesPersistenceAbsenceAndInvalid() throws {
        let absent = try makeFixture()
        let absentModel = try testController(resourceURL: absent, supportURL: absent.appendingPathComponent("support"))
        XCTAssertFalse(absentModel.requiresReset)
        XCTAssertEqual(absentModel.jog, try JogPreferences.decode(jogDefaultsData(), defaults: jogDefaultsData()))

        let fixture = try makeFixture()
        let support = fixture.appendingPathComponent("support")
        try FileManager.default.createDirectory(at: support, withIntermediateDirectories: true)
        try Data(#"{"scratch_speed":2.7,"scratch_reaction":150,"pitch_bend_speed":2.7,"pitch_bend_reaction":17,"backspin_enabled":true,"backspin_acceleration":150000.0,"backspin_speed_increase":5.2}"#.utf8)
            .write(to: support.appendingPathComponent("jog-config.json"))
        let persisted = try testController(resourceURL: fixture, supportURL: support)
        XCTAssertFalse(persisted.requiresReset)
        XCTAssertEqual(persisted.jog.scratchSpeed, 2.7)
        XCTAssertEqual(persisted.jog.pitchBendReaction, 17)
        XCTAssertEqual(persisted.jog.scratchReaction, 150)
        XCTAssertTrue(persisted.jog.backspinEnabled)
        XCTAssertEqual(persisted.jog.backspinAcceleration, 150000.0)
        XCTAssertEqual(persisted.jog.backspinSpeedIncrease, 5.2)
        XCTAssertTrue(persisted.jog.backspinAssistEnabled)
        XCTAssertEqual(persisted.jog.backspinAssistStrength, 100)
        XCTAssertTrue(persisted.jog.vinylEnabledAtStartup)
        XCTAssertEqual(persisted.jog.platterTargetRpm, 33)
        XCTAssertEqual(persisted.jog.platterMaxForce, 20000)
        XCTAssertEqual(persisted.jog.platterBaseForce, 7000)
        XCTAssertEqual(persisted.jog.platterForcePerRpm, 110)
        XCTAssertEqual(persisted.jog.platterKp, 4000)
        XCTAssertEqual(persisted.jog.platterKi, 3000)
        XCTAssertEqual(persisted.jog.platterForceRisePerSecond, 200000)
        XCTAssertEqual(persisted.jog.platterStartupForceRisePerSecond, 0)
        XCTAssertEqual(persisted.jog.platterStartupMs, 200)
        XCTAssertEqual(persisted.jog.platterOverspeedLimitRpm, 0)
        persisted.jog.backspinAssistEnabled = true
        persisted.jog.backspinAssistStrength = 37
        persisted.jog.backspinEnabled = false
        persisted.jog.vinylEnabledAtStartup = false
        persisted.jog.platterTargetRpm = 45
        persisted.jog.platterMaxForce = 16000
        persisted.jog.platterBaseForce = 1500
        persisted.jog.platterForcePerRpm = 100
        persisted.jog.platterKp = 3500
        persisted.jog.platterKi = 2000
        persisted.jog.platterForceRisePerSecond = 150000
        persisted.jog.platterStartupForceRisePerSecond = 25000
        persisted.jog.platterStartupMs = 150
        persisted.jog.platterOverspeedLimitRpm = 80
        XCTAssertTrue(persisted.jog.backspinAssistEnabled)
        try persisted.save()
        let reloaded = try testController(resourceURL: fixture, supportURL: support)
        XCTAssertEqual(reloaded.jog, persisted.jog)
        XCTAssertFalse(reloaded.jog.backspinEnabled)
        XCTAssertEqual(reloaded.jog.backspinSpeedIncrease, 5.2)
        XCTAssertTrue(reloaded.jog.backspinAssistEnabled)
        XCTAssertEqual(reloaded.jog.backspinAssistStrength, 37)
        XCTAssertFalse(reloaded.jog.vinylEnabledAtStartup)
        XCTAssertEqual(reloaded.jog.platterTargetRpm, 45)
        XCTAssertEqual(reloaded.jog.platterMaxForce, 16000)
        XCTAssertEqual(reloaded.jog.platterBaseForce, 1500)
        XCTAssertEqual(reloaded.jog.platterForcePerRpm, 100)
        XCTAssertEqual(reloaded.jog.platterKp, 3500)
        XCTAssertEqual(reloaded.jog.platterKi, 2000)
        XCTAssertEqual(reloaded.jog.platterForceRisePerSecond, 150000)
        XCTAssertEqual(reloaded.jog.platterStartupForceRisePerSecond, 25000)
        XCTAssertEqual(reloaded.jog.platterStartupMs, 150)
        XCTAssertEqual(reloaded.jog.platterOverspeedLimitRpm, 80)

        let invalid = Data(#"{"scratch_reaction":151}"#.utf8)
        try invalid.write(to: support.appendingPathComponent("jog-config.json"))
        let broken = try testController(resourceURL: fixture, supportURL: support)
        XCTAssertTrue(broken.requiresReset)
        XCTAssertThrowsError(try broken.save())
        XCTAssertEqual(try Data(contentsOf: support.appendingPathComponent("jog-config.json")), invalid)
        broken.resetPreferences()
        try broken.save()
        XCTAssertFalse(broken.requiresReset)
        let restored = try JogPreferences.decode(
            Data(contentsOf: support.appendingPathComponent("jog-config.json")),
            defaults: jogDefaultsData()
        )
        XCTAssertEqual(restored, broken.jog)
    }

    func testTensionPreferencesDefaultsPartialOverridesAndRoundTrip() throws {
        let defaults = try tensionDefaultsData()
        let base = try TensionPreferences.decode(defaults, defaults: defaults)
        XCTAssertEqual(base.tension, -77)
        XCTAssertEqual(TensionPreferences(), base)
        XCTAssertEqual(try TensionPreferences.decode(Data("{}".utf8), defaults: defaults), base)
        for tension in [-100, -37, 0, 42, 100] {
            let value = TensionPreferences(tension: tension)
            XCTAssertEqual(try TensionPreferences.decode(value.encoded(), defaults: defaults), value)
            XCTAssertEqual(try TensionPreferences.decode(Data("{}".utf8), defaults: value.encoded()), value)
            let object = try XCTUnwrap(JSONSerialization.jsonObject(with: value.encoded()) as? [String: Int])
            XCTAssertEqual(object, ["tension": tension])
        }
    }

    func testTensionPreferencesRejectInvalidJSONTypesRangesAndFields() throws {
        let defaults = try tensionDefaultsData()
        for input in [
            "", "{", "[]", "null", "0", #""tension""#,
            #"{"tension":-101}"#, #"{"tension":101}"#,
            #"{"tension":1.5}"#, #"{"tension":1.0}"#, #"{"tension":0.0}"#,
            #"{"tension":1e2}"#, #"{"tension":true}"#, #"{"tension":false}"#,
            #"{"tension":"50"}"#, #"{"tension":null}"#, #"{"tension":[]}"#,
            #"{"tension":{}}"#, #"{"tension":9223372036854775808}"#,
            #"{"tension":0,"unknown":1}"#
        ] {
            XCTAssertThrowsError(try TensionPreferences.decode(Data(input.utf8), defaults: defaults), input)
        }
        for tension in [-101, 101] {
            XCTAssertThrowsError(try TensionPreferences(tension: tension).encoded())
        }
    }

    @MainActor
    func testTensionPersistenceDefaultsAndInvalidEditorPreserveOtherPreferences() throws {
        let fixture = try makeFixture()
        let support = fixture.appendingPathComponent("support")
        let model = try testController(resourceURL: fixture, supportURL: support)
        XCTAssertFalse(model.requiresReset)
        XCTAssertEqual(model.tension.tension, -77)
        XCTAssertFalse(FileManager.default.fileExists(atPath: model.tensionURL.path))
        model.leds.loopColor = "blue"
        model.jog.scratchSpeed = 3.5
        model.run.logLevel = "debug"
        try model.save()
        let ledData = try Data(contentsOf: model.ledURL)
        let jogData = try Data(contentsOf: model.jogURL)
        let runData = try Data(contentsOf: model.runURL)
        model.tension.tension = -70
        try model.save()
        let savedTension = try Data(contentsOf: model.tensionURL)
        let reloaded = try testController(resourceURL: fixture, supportURL: support)
        XCTAssertEqual(reloaded.tension, model.tension)
        XCTAssertEqual(reloaded.leds, model.leds)
        XCTAssertEqual(reloaded.jog, model.jog)
        XCTAssertEqual(reloaded.run, model.run)

        model.tension.tension = 101
        model.apply()
        XCTAssertFalse(model.errorMessage.isEmpty)
        XCTAssertEqual(model.tension.tension, 101)
        XCTAssertEqual(try Data(contentsOf: model.tensionURL), savedTension)
        XCTAssertEqual(try Data(contentsOf: model.ledURL), ledData)
        XCTAssertEqual(try Data(contentsOf: model.jogURL), jogData)
        XCTAssertEqual(try Data(contentsOf: model.runURL), runData)
    }

    @MainActor
    func testTensionInvalidSavedValuesRequireExplicitReset() throws {
        let fixture = try makeFixture()
        let support = fixture.appendingPathComponent("support")
        let initial = try testController(resourceURL: fixture, supportURL: support)
        initial.leds.loopColor = "blue"
        initial.jog.scratchSpeed = 3.5
        try initial.save()
        let invalid = Data(#"{"tension":true}"#.utf8)
        try invalid.write(to: initial.tensionURL)
        let broken = try testController(resourceURL: fixture, supportURL: support)
        XCTAssertTrue(broken.requiresReset)
        XCTAssertFalse(broken.errorMessage.isEmpty)
        XCTAssertEqual(broken.leds, initial.leds)
        XCTAssertEqual(broken.jog, initial.jog)
        XCTAssertThrowsError(try broken.save())
        XCTAssertEqual(try Data(contentsOf: broken.tensionURL), invalid)
        broken.resetPreferences()
        XCTAssertFalse(broken.requiresReset)
        XCTAssertEqual(broken.tension.tension, -77)
        XCTAssertEqual(try Data(contentsOf: broken.tensionURL), invalid)
        try broken.save()
        let reloaded = try testController(resourceURL: fixture, supportURL: support)
        XCTAssertFalse(reloaded.requiresReset)
        XCTAssertEqual(reloaded.tension.tension, -77)
    }

    @MainActor
    func testTensionSaveAndRestartForwardsCurrentConfiguration() async throws {
        let fixture = try makeFixture()
        let received = fixture.appendingPathComponent("received-tension.json")
        let fifo = fixture.appendingPathComponent("control.fifo")
        XCTAssertEqual(mkfifo(fifo.path, 0o600), 0)
        let descriptor = Darwin.open(fifo.path, O_RDWR)
        guard descriptor >= 0 else { throw AppError.invalid("Could not open test FIFO.") }
        defer { XCTAssertEqual(Darwin.close(descriptor), 0) }
        let executable = fixture.appendingPathComponent("s4mk3-hid-midi-bridge")
        let script = """
        #!/bin/sh
        trap 'exit 0' INT
        cfg=""; screens=0
        while [ $# -gt 0 ]; do
          case "$1" in
            --tension-config) cfg="$2"; shift 2;;
            --screens) screens=1; shift;;
            *) shift;;
          esac
        done
        [ -n "$cfg" ] || exit 2
        [ "$screens" -eq 1 ] || exit 4
        cp "$cfg" '\(received.path)' || exit 3
        exec 3< '\(fifo.path)'
        printf 'TENSION_READY\\n'
        IFS= read -r line <&3
        """
        try Data(script.utf8).write(to: executable)
        try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: executable.path)
        let model = try testController(resourceURL: fixture, supportURL: fixture.appendingPathComponent("support"))
        let ready = expectation(description: "helper read first saved tension and installed signal handler")
        let restarted = expectation(description: "restarted helper read updated saved tension")
        let stopped = expectation(description: "restarted helper exited after final stop")
        var readyCount = 0
        var finalStop = false
        let output = model.logOutput.$text.sink { text in
            let count = text.components(separatedBy: "TENSION_READY").count - 1
            if count > readyCount {
                readyCount = count
                if count == 1 { ready.fulfill() }
                if count == 2 { restarted.fulfill() }
            }
        }
        let state = model.$state.sink { state in
            if finalStop && state == .stopped { stopped.fulfill() }
        }
        model.tension.tension = -35
        model.start()
        await fulfillment(of: [ready], timeout: 10)
        XCTAssertEqual(try TensionPreferences.decode(Data(contentsOf: received), defaults: tensionDefaultsData()).tension, -35)
        model.tension.tension = 60
        model.apply()
        await fulfillment(of: [restarted], timeout: 10)
        XCTAssertEqual(try TensionPreferences.decode(Data(contentsOf: received), defaults: tensionDefaultsData()).tension, 60)
        XCTAssertTrue(model.commandPreview.contains("--tension-config"))
        XCTAssertTrue(model.commandPreview.contains("--screens"))
        finalStop = true
        model.stop()
        await fulfillment(of: [stopped], timeout: 10)
        withExtendedLifetime((output, state)) {}
        XCTAssertFalse(model.isBusy)
    }

    @MainActor
    func testMotorizedPlatterSaveAndRestartForwardsJogConfigurationAndPreservesUnrelatedPreferences() async throws {
        let fixture = try makeFixture()
        let receivedJog = fixture.appendingPathComponent("received-jog.json")
        let fifo = fixture.appendingPathComponent("control.fifo")
        XCTAssertEqual(mkfifo(fifo.path, 0o600), 0)
        let descriptor = Darwin.open(fifo.path, O_RDWR)
        guard descriptor >= 0 else { throw AppError.invalid("Could not open test FIFO.") }
        defer { XCTAssertEqual(Darwin.close(descriptor), 0) }
        let executable = fixture.appendingPathComponent("s4mk3-hid-midi-bridge")
        let script = """
        #!/bin/sh
        trap 'exit 0' INT
        jog=""
        while [ $# -gt 0 ]; do
          case "$1" in
            --jog-config) jog="$2"; shift 2;;
            *) shift;;
          esac
        done
        [ -n "$jog" ] || exit 2
        cp "$jog" '\(receivedJog.path)' || exit 3
        exec 3< '\(fifo.path)'
        printf 'JOG_READY\\n'
        IFS= read -r line <&3
        """
        try Data(script.utf8).write(to: executable)
        try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: executable.path)
        let model = try testController(resourceURL: fixture, supportURL: fixture.appendingPathComponent("support"))
        model.leds.loopColor = "cyan"
        model.tension.tension = -20
        model.run.logLevel = "debug"
        model.jog.vinylEnabledAtStartup = false
        model.jog.platterTargetRpm = 45
        model.jog.platterMaxForce = 18000
        model.jog.platterBaseForce = 1500
        model.jog.platterForcePerRpm = 120
        model.jog.platterKp = 4500
        model.jog.platterKi = 2500
        model.jog.platterForceRisePerSecond = 250000
        model.jog.platterStartupForceRisePerSecond = 50000
        model.jog.platterStartupMs = 150
        model.jog.platterOverspeedLimitRpm = 85

        let ready = expectation(description: "helper read initial motorized platter jog config")
        let restarted = expectation(description: "restarted helper read updated motorized platter jog config")
        let stopped = expectation(description: "restarted helper exited after final stop")
        var readyCount = 0
        var finalStop = false
        let output = model.logOutput.$text.sink { text in
            let count = text.components(separatedBy: "JOG_READY").count - 1
            if count > readyCount {
                readyCount = count
                if count == 1 { ready.fulfill() }
                if count == 2 { restarted.fulfill() }
            }
        }
        let state = model.$state.sink { state in
            if finalStop && state == .stopped { stopped.fulfill() }
        }
        model.start()
        await fulfillment(of: [ready], timeout: 10)
        let initialDecoded = try JogPreferences.decode(Data(contentsOf: receivedJog), defaults: jogDefaultsData())
        XCTAssertEqual(initialDecoded, model.jog)
        XCTAssertFalse(initialDecoded.vinylEnabledAtStartup)
        XCTAssertEqual(initialDecoded.platterTargetRpm, 45)
        XCTAssertEqual(initialDecoded.platterMaxForce, 18000)

        let savedLedData = try Data(contentsOf: model.ledURL)
        let savedTensionData = try Data(contentsOf: model.tensionURL)
        let savedRunData = try Data(contentsOf: model.runURL)

        model.jog.vinylEnabledAtStartup = true
        model.jog.platterTargetRpm = 33
        model.jog.platterStartupForceRisePerSecond = 0
        model.jog.platterOverspeedLimitRpm = 0
        model.apply()
        await fulfillment(of: [restarted], timeout: 10)
        let updatedDecoded = try JogPreferences.decode(Data(contentsOf: receivedJog), defaults: jogDefaultsData())
        XCTAssertEqual(updatedDecoded, model.jog)
        XCTAssertTrue(updatedDecoded.vinylEnabledAtStartup)
        XCTAssertEqual(updatedDecoded.platterTargetRpm, 33)
        XCTAssertEqual(updatedDecoded.platterStartupForceRisePerSecond, 0)
        XCTAssertEqual(updatedDecoded.platterOverspeedLimitRpm, 0)
        XCTAssertEqual(try Data(contentsOf: model.ledURL), savedLedData)
        XCTAssertEqual(try Data(contentsOf: model.tensionURL), savedTensionData)
        XCTAssertEqual(try Data(contentsOf: model.runURL), savedRunData)

        finalStop = true
        model.stop()
        await fulfillment(of: [stopped], timeout: 10)
        withExtendedLifetime((output, state)) {}
        XCTAssertFalse(model.isBusy)
    }

    @MainActor
    func testExportMappingForwardsCurrentJogAndInstallsOutput() async throws {
        let fixture = try makeFixture()
        let argsURL = fixture.appendingPathComponent("received-args.txt")
        let configURL = fixture.appendingPathComponent("received-config.json")
        let executable = fixture.appendingPathComponent("s4mk3-hid-midi-bridge")
        let script = """
        #!/bin/sh
        printf '%s' "$*" > '\(argsURL.path)'
        out=""; cfg=""
        while [ $# -gt 0 ]; do
          case "$1" in
            --generate-mapping) out="$2"; shift 2;;
            --jog-config) cfg="$2"; shift 2;;
            *) shift 1;;
          esac
        done
        if [ -z "$out" ] || [ -z "$cfg" ]; then
          echo "missing arguments" >&2
          exit 2
        fi
        cat "$cfg" > '\(configURL.path)'
        printf 'MAPPING-BYTES' > "$out"
        """
        try Data(script.utf8).write(to: executable)
        try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: executable.path)
        let model = try testController(resourceURL: fixture, supportURL: fixture.appendingPathComponent("support"))
        model.jog.scratchSpeed = 3.5
        model.jog.pitchBendReaction = 42
        model.tension.tension = 85
        let destination = fixture.appendingPathComponent("S4 MK3 Bridge.djayMidiMapping")
        try Data("OLD-MAPPING".utf8).write(to: destination)
        try await model.exportMapping(to: destination)
        XCTAssertFalse(model.isExporting)
        XCTAssertFalse(model.isBusy)
        XCTAssertEqual(model.state, .stopped)
        XCTAssertEqual(try Data(contentsOf: destination), Data("MAPPING-BYTES".utf8))
        let args = try String(contentsOf: argsURL, encoding: .utf8)
        XCTAssertTrue(args.contains("--generate-mapping"))
        XCTAssertTrue(args.contains("--jog-config"))
        XCTAssertFalse(args.contains("--tension-config"))
        XCTAssertFalse(args.contains("--screens"))
        XCTAssertFalse(FileManager.default.fileExists(atPath: model.tensionURL.path))
        let forwarded = try JogPreferences.decode(Data(contentsOf: configURL), defaults: jogDefaultsData())
        XCTAssertEqual(forwarded, model.jog)
        XCTAssertTrue(model.log.contains("EXPORT_EXIT: 0"))
    }

    @MainActor
    func testExportMappingReportsGeneratorFailureWithoutTouchingDestination() async throws {
        let fixture = try makeFixture()
        let executable = fixture.appendingPathComponent("s4mk3-hid-midi-bridge")
        try Data("#!/bin/sh\necho 'generator boom' >&2\nexit 3\n".utf8).write(to: executable)
        try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: executable.path)
        let model = try testController(resourceURL: fixture, supportURL: fixture.appendingPathComponent("support"))
        let destination = fixture.appendingPathComponent("S4 MK3 Bridge.djayMidiMapping")
        try Data("OLD-MAPPING".utf8).write(to: destination)
        do {
            try await model.exportMapping(to: destination)
            XCTFail("expected exportMapping to throw")
        } catch {}
        XCTAssertFalse(model.isExporting)
        XCTAssertFalse(model.isBusy)
        XCTAssertEqual(model.state, .stopped)
        XCTAssertEqual(try Data(contentsOf: destination), Data("OLD-MAPPING".utf8))
        XCTAssertTrue(model.log.contains("EXPORT_EXIT: 3"))
    }

    @MainActor
    func testExportMappingPreservesDirectoryDestinationOnInstallationFailure() async throws {
        // Given an exporter and a destination directory containing user data.
        let fixture = try makeFixture()
        let executable = fixture.appendingPathComponent("s4mk3-hid-midi-bridge")
        try Data("#!/bin/sh\nprintf 'MAPPING-BYTES' > \"$2\"\n".utf8).write(to: executable)
        try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: executable.path)
        let model = try testController(resourceURL: fixture, supportURL: fixture.appendingPathComponent("support"))
        let destination = fixture.appendingPathComponent("existing-directory")
        try FileManager.default.createDirectory(at: destination, withIntermediateDirectories: true)
        let sentinel = destination.appendingPathComponent("keep.txt")
        let contents = Data("KEEP".utf8)
        try contents.write(to: sentinel)
        // When a file installation is attempted at that directory.
        var failed = false
        do { try await model.exportMapping(to: destination) }
        catch { failed = true }
        // Then installation fails without removing the destination or its contents.
        XCTAssertTrue(failed)
        XCTAssertEqual(try Data(contentsOf: sentinel), contents)
        XCTAssertFalse(model.isExporting)
    }

    @MainActor
    private func testController(resourceURL: URL, supportURL: URL) throws -> BridgeController {
        try BridgeController(resourceURL: resourceURL, supportURL: supportURL, agent: TestNIHardwareAgent())
    }

    private func jogDefaultsData() throws -> Data {
        let root = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        return try Data(contentsOf: root.appendingPathComponent("examples/jog-config.json"))
    }

    private func tensionDefaultsData() throws -> Data {
        let root = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        return try Data(contentsOf: root.appendingPathComponent("examples/tension-config.json"))
    }

    private func makeFixture() throws -> URL {
        let directory = FileManager.default.temporaryDirectory
            .appendingPathComponent("S4 Bridge Tests \(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        try defaultData().write(to: directory.appendingPathComponent("default-led-config.json"))
        try jogDefaultsData().write(to: directory.appendingPathComponent("default-jog-config.json"))
        try tensionDefaultsData().write(to: directory.appendingPathComponent("default-tension-config.json"))
        addTeardownBlock { try FileManager.default.removeItem(at: directory) }
        return directory
    }
}
