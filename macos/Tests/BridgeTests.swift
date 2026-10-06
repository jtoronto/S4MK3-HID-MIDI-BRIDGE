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
        let executable = fixture.appendingPathComponent("s4-connectivity-probe")
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
        let executable = fixture.appendingPathComponent("s4-connectivity-probe")
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

    private func makeFixture() throws -> URL {
        let directory = FileManager.default.temporaryDirectory
            .appendingPathComponent("S4 Bridge Tests \(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        try defaultData().write(to: directory.appendingPathComponent("default-led-config.json"))
        addTeardownBlock { try FileManager.default.removeItem(at: directory) }
        return directory
    }
}
