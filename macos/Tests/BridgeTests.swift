import Combine
import Darwin
import Foundation
import XCTest
@testable import S4Bridge

final class BridgeTests: XCTestCase {
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
        var settings = RunPreferences()
        XCTAssertEqual(settings.arguments(ledURL: path), [
            "--midi", "full", "--led-config", path.path, "--until-stopped"
        ])
        settings.profile = .minimal
        settings.limitDuration = true
        settings.durationSeconds = 60
        settings.rawReports = true
        XCTAssertEqual(settings.arguments(ledURL: path), ["--midi", "minimal", "--seconds", "60", "--raw"])
        settings.profile = .probe
        XCTAssertEqual(settings.arguments(ledURL: path), ["--seconds", "60", "--raw"])
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
    func testInvalidSavedPreferencesArePreservedUntilExplicitReset() throws {
        let fixture = try makeFixture()
        let support = fixture.appendingPathComponent("support")
        try FileManager.default.createDirectory(at: support, withIntermediateDirectories: true)
        let saved = support.appendingPathComponent("led-config.json")
        let invalid = Data(#"{"deck_colors":[]}"#.utf8)
        try invalid.write(to: saved)
        let model = try BridgeController(resourceURL: fixture, supportURL: support)
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
        let model = try BridgeController(resourceURL: fixture, supportURL: fixture.appendingPathComponent("support"))
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
        XCTAssertTrue(model.log.contains("EXIT: 0"))
    }

    @MainActor
    func testStopSendsSIGINTAndWaitsForChildAcknowledgement() async throws {
        let fixture = try makeFixture()
        let fifo = fixture.appendingPathComponent("control.fifo")
        XCTAssertEqual(mkfifo(fifo.path, 0o600), 0)
        let descriptor = Darwin.open(fifo.path, O_RDWR)
        guard descriptor >= 0 else { throw AppError.invalid("Could not open test FIFO.") }
        defer { XCTAssertEqual(Darwin.close(descriptor), 0) }
        let executable = fixture.appendingPathComponent("s4mk3-hid-midi-bridge")
        let script = """
        #!/bin/sh
        trap 'printf "STOP_ACK\\n"; exit 0' INT
        exec 3< '\(fifo.path)'
        printf 'HELPER_READY\\n'
        IFS= read -r line <&3
        """
        try Data(script.utf8).write(to: executable)
        try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: executable.path)
        let model = try BridgeController(resourceURL: fixture, supportURL: fixture.appendingPathComponent("support"))
        let ready = expectation(description: "child installed its signal handler and opened the FIFO")
        let exited = expectation(description: "child acknowledged SIGINT and exited")
        var didSignalReady = false
        var sawStopping = false
        let output = model.$log.sink { text in
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
        XCTAssertEqual(base, try JogPreferences.decode(base.encoded(), defaults: defaults))
        let override = Data(#"{"scratch_speed":3.5,"pitch_bend_reaction":42}"#.utf8)
        let changed = try JogPreferences.decode(override, defaults: defaults)
        XCTAssertEqual(changed.scratchSpeed, 3.5)
        XCTAssertEqual(changed.pitchBendReaction, 42)
        XCTAssertEqual(changed.scratchReaction, base.scratchReaction)
        XCTAssertEqual(changed.pitchBendSpeed, base.pitchBendSpeed)
        let keys = try XCTUnwrap(JSONSerialization.jsonObject(with: base.encoded()) as? [String: Any])
        XCTAssertNotNil(keys["scratch_speed"])
        XCTAssertNotNil(keys["pitch_bend_reaction"])
        XCTAssertNil(keys["scratchSpeed"])
        for input in [
            #"{"scratch_speed":0}"#, #"{"scratch_speed":-2.5}"#,
            #"{"pitch_bend_speed":0}"#, #"{"scratch_reaction":-1}"#,
            #"{"scratch_reaction":151}"#, #"{"pitch_bend_reaction":151}"#,
            #"{"scratch_reaction":1.5}"#, #"{"unknown":true}"#
        ] {
            XCTAssertThrowsError(try JogPreferences.decode(Data(input.utf8), defaults: defaults))
        }
    }

    @MainActor
    func testJogSavedPreferencesPersistenceAbsenceAndInvalid() throws {
        let absent = try makeFixture()
        let absentModel = try BridgeController(resourceURL: absent, supportURL: absent.appendingPathComponent("support"))
        XCTAssertFalse(absentModel.requiresReset)
        XCTAssertEqual(absentModel.jog, try JogPreferences.decode(jogDefaultsData(), defaults: jogDefaultsData()))

        let fixture = try makeFixture()
        let support = fixture.appendingPathComponent("support")
        try FileManager.default.createDirectory(at: support, withIntermediateDirectories: true)
        try Data(#"{"scratch_speed":3.5,"pitch_bend_reaction":42}"#.utf8)
            .write(to: support.appendingPathComponent("jog-config.json"))
        let persisted = try BridgeController(resourceURL: fixture, supportURL: support)
        XCTAssertFalse(persisted.requiresReset)
        XCTAssertEqual(persisted.jog.scratchSpeed, 3.5)
        XCTAssertEqual(persisted.jog.pitchBendReaction, 42)
        XCTAssertEqual(persisted.jog.scratchReaction, 150)
        try persisted.save()
        let reloaded = try BridgeController(resourceURL: fixture, supportURL: support)
        XCTAssertEqual(reloaded.jog, persisted.jog)

        let invalid = Data(#"{"scratch_reaction":151}"#.utf8)
        try invalid.write(to: support.appendingPathComponent("jog-config.json"))
        let broken = try BridgeController(resourceURL: fixture, supportURL: support)
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
        let model = try BridgeController(resourceURL: fixture, supportURL: fixture.appendingPathComponent("support"))
        model.jog.scratchSpeed = 3.5
        model.jog.pitchBendReaction = 42
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
        let model = try BridgeController(resourceURL: fixture, supportURL: fixture.appendingPathComponent("support"))
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
        let model = try BridgeController(resourceURL: fixture, supportURL: fixture.appendingPathComponent("support"))
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

    private func jogDefaultsData() throws -> Data {
        let root = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        return try Data(contentsOf: root.appendingPathComponent("examples/jog-config.json"))
    }

    private func makeFixture() throws -> URL {
        let directory = FileManager.default.temporaryDirectory
            .appendingPathComponent("S4 Bridge Tests \(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        try defaultData().write(to: directory.appendingPathComponent("default-led-config.json"))
        try jogDefaultsData().write(to: directory.appendingPathComponent("default-jog-config.json"))
        addTeardownBlock { try FileManager.default.removeItem(at: directory) }
        return directory
    }
}
