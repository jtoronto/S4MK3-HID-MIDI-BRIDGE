import Combine
import Foundation
import XCTest
@testable import S4Bridge

@MainActor
final class NIBridgeTests: XCTestCase {
    func testLegacyPreferencesDefaultToDirectMode() throws {
        let old = Data(#"{"profile":"minimal","limitDuration":false,"durationSeconds":3600,"rawReports":false,"logLevel":"info"}"#.utf8)
        XCTAssertEqual(try JSONDecoder().decode(RunPreferences.self, from: old).mode, .direct)
    }

    func testNIProcessWaitsForReadinessUpdatesTensionAndStops() async throws {
        let model = try makeScreenController(self, agent: TestNIHardwareAgent())
        let executable = model.executableURL
        let script = """
        #!/bin/sh
        printf '%s\\n' 'S4_BRIDGE_STATUS {"version":1,"state":"ready","profile":"full","backend":"core-midi","readiness":{"hid":true,"midi_input":true,"midi_output":true,"controller_initialized":true},"error":null}'
        while IFS= read -r line; do
          case "$line" in
            'ni_jog_tension 255') printf 'tension received\\n';;
            'S4_BRIDGE_CONTROL 1 STOP') exit 0;;
          esac
        done
        """
        try Data(script.utf8).write(to: executable)
        try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: executable.path)
        model.run.mode = .ni
        let ready = expectation(description: "managed ready")
        let updated = expectation(description: "live tension reached child")
        let stopped = expectation(description: "managed child exited")
        let state = model.$state.sink {
            if $0 == .running { ready.fulfill() }
            if $0 == .stopped && model.log.contains("LAUNCH:") { stopped.fulfill() }
        }
        let logs = model.logOutput.updates.sink {
            if $0.appendedText.contains("tension received") { updated.fulfill() }
        }
        model.start()
        await fulfillment(of: [ready], timeout: 10)
        model.setNIJogTension(255)
        await fulfillment(of: [updated], timeout: 10)
        model.stop()
        await fulfillment(of: [stopped], timeout: 10)
        let reloaded = try BridgeController(resourceURL: model.resourceURL, supportURL: model.supportURL, agent: TestNIHardwareAgent())
        XCTAssertEqual(reloaded.run.mode, .ni)
        XCTAssertEqual(reloaded.niJog.value, 255)
        withExtendedLifetime((state, logs)) {}
    }
}
