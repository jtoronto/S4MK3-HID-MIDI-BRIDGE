import Combine
import Darwin
import Foundation
import XCTest
@testable import S4Bridge

final class PlatterDiagnosticsTests: XCTestCase {
    func testPlatterStatusDecodesAllModesSignedValuesNullRpmAndIgnoresExtraFields() throws {
        for mode in PlatterRoutingMode.allCases {
            let line = """
            S4_PLATTER_STATUS {"extra_root":true,"wheels":[{"deck":1,"rpm":-33.3,"bend_counts_per_second":14.2,"mode":"\(mode.rawValue)","extra_wheel":1},{"deck":4,"rpm":null,"bend_counts_per_second":-0.0,"mode":"stale","extra_wheel":"ok"}]}
            """
            let snapshot = try XCTUnwrap(PlatterStatusSnapshot.parseLine(line), mode.rawValue)
            XCTAssertEqual(snapshot.left.deck, 1)
            XCTAssertEqual(snapshot.left.rpm, -33.3)
            XCTAssertEqual(snapshot.left.bendCountsPerSecond, 14.2)
            XCTAssertEqual(snapshot.left.mode, mode)
            XCTAssertEqual(snapshot.right.deck, 4)
            XCTAssertNil(snapshot.right.rpm)
            XCTAssertEqual(snapshot.right.bendCountsPerSecond, 0.0)
            XCTAssertEqual(snapshot.right.mode, .stale)
        }
    }

    func testPlatterStatusRejectsMalformedPayloadsSidesDecksModesAndTypes() {
        let invalidPayloads = [
            "",
            "{}",
            #"{"wheels":[]}"#,
            #"{"wheels":[{"deck":1,"rpm":33.3,"bend_counts_per_second":0,"mode":"bend"}]}"#,
            #"{"wheels":[{"deck":1,"rpm":33.3,"bend_counts_per_second":0,"mode":"bend"},{"deck":2,"rpm":33.3,"bend_counts_per_second":0,"mode":"bend"},{"deck":3,"rpm":33.3,"bend_counts_per_second":0,"mode":"bend"}]}"#,
            #"{"wheels":[{"deck":2,"rpm":33.3,"bend_counts_per_second":0,"mode":"bend"},{"deck":1,"rpm":33.3,"bend_counts_per_second":0,"mode":"bend"}]}"#,
            #"{"wheels":[{"deck":0,"rpm":33.3,"bend_counts_per_second":0,"mode":"bend"},{"deck":2,"rpm":33.3,"bend_counts_per_second":0,"mode":"bend"}]}"#,
            #"{"wheels":[{"deck":5,"rpm":33.3,"bend_counts_per_second":0,"mode":"bend"},{"deck":2,"rpm":33.3,"bend_counts_per_second":0,"mode":"bend"}]}"#,
            #"{"wheels":[{"deck":1.0,"rpm":33.3,"bend_counts_per_second":0,"mode":"bend"},{"deck":2,"rpm":33.3,"bend_counts_per_second":0,"mode":"bend"}]}"#,
            #"{"wheels":[{"deck":true,"rpm":33.3,"bend_counts_per_second":0,"mode":"bend"},{"deck":2,"rpm":33.3,"bend_counts_per_second":0,"mode":"bend"}]}"#,
            #"{"wheels":[{"deck":1,"rpm":33.3,"bend_counts_per_second":0,"mode":"unknown"},{"deck":2,"rpm":33.3,"bend_counts_per_second":0,"mode":"bend"}]}"#,
            #"{"wheels":[{"deck":1,"rpm":true,"bend_counts_per_second":0,"mode":"bend"},{"deck":2,"rpm":33.3,"bend_counts_per_second":0,"mode":"bend"}]}"#,
            #"{"wheels":[{"deck":1,"rpm":"33.3","bend_counts_per_second":0,"mode":"bend"},{"deck":2,"rpm":33.3,"bend_counts_per_second":0,"mode":"bend"}]}"#,
            #"{"wheels":[{"deck":1,"rpm":33.3,"bend_counts_per_second":false,"mode":"bend"},{"deck":2,"rpm":33.3,"bend_counts_per_second":0,"mode":"bend"}]}"#,
            #"{"wheels":[{"deck":1,"rpm":33.3,"bend_counts_per_second":null,"mode":"bend"},{"deck":2,"rpm":33.3,"bend_counts_per_second":0,"mode":"bend"}]}"#,
            #"{"wheels":[{"deck":1,"rpm":1e500,"bend_counts_per_second":0,"mode":"bend"},{"deck":2,"rpm":33.3,"bend_counts_per_second":0,"mode":"bend"}]}"#,
            #"{"wheels":[{"deck":1,"rpm":33.3,"bend_counts_per_second":-1e500,"mode":"bend"},{"deck":2,"rpm":33.3,"bend_counts_per_second":0,"mode":"bend"}]}"#
        ]
        for payload in invalidPayloads {
            XCTAssertThrowsError(try PlatterStatusSnapshot.decode(Data(payload.utf8)), payload)
            XCTAssertNil(PlatterStatusSnapshot.parseLine("S4_PLATTER_STATUS " + payload), payload)
        }
        XCTAssertNil(PlatterStatusSnapshot.parseLine("INFO something else"))
    }

    func testPlatterDiagnosticsPresentationDistinguishesModesAndExpiresAfterSilence() {
        var state = PlatterDiagnosticsState()
        let t0 = Date(timeIntervalSince1970: 1000.0)

        let stopped = state.presentation(bridgeRunning: false, at: t0)
        XCTAssertNil(stopped.left.rpm)
        XCTAssertNil(stopped.left.bendOutput)
        XCTAssertTrue(stopped.left.isStaleOrStopped)

        let waitingInitial = state.presentation(bridgeRunning: true, at: t0)
        XCTAssertNil(waitingInitial.left.rpm)
        XCTAssertNil(waitingInitial.left.bendOutput)
        XCTAssertNil(waitingInitial.right.rpm)

        let line1 = #"S4_PLATTER_STATUS {"wheels":[{"deck":3,"rpm":33.33,"bend_counts_per_second":-12.4,"mode":"bend"},{"deck":2,"rpm":0.0,"bend_counts_per_second":0.0,"mode":"paused_scratch"}]}"#
        XCTAssertTrue(state.acceptLine(line1, at: t0))

        let fresh = state.presentation(bridgeRunning: true, at: t0.addingTimeInterval(0.25))
        XCTAssertFalse(fresh.isStale)
        XCTAssertEqual(fresh.left.rpm, 33.33)
        XCTAssertEqual(fresh.left.bendOutput, -12.4)
        XCTAssertFalse(fresh.left.isStaleOrStopped)

        XCTAssertEqual(fresh.right.rpm, 0.0)
        XCTAssertEqual(fresh.right.bendOutput, 0.0)
        XCTAssertFalse(fresh.right.isStaleOrStopped)

        let atBoundary = state.presentation(bridgeRunning: true, at: t0.addingTimeInterval(0.5))
        XCTAssertFalse(atBoundary.isStale)
        XCTAssertEqual(atBoundary.left.rpm, 33.33)

        let expired = state.presentation(bridgeRunning: true, at: t0.addingTimeInterval(0.501))
        XCTAssertTrue(expired.isStale)
        XCTAssertNil(expired.left.rpm)
        XCTAssertNil(expired.left.bendOutput)
        XCTAssertTrue(expired.left.isStaleOrStopped)
        XCTAssertNil(expired.right.rpm)
        XCTAssertNil(expired.right.bendOutput)

        state.reset()
        XCTAssertNil(state.latest)
        XCTAssertNil(state.receivedAt)
    }

    @MainActor
    func testLivePlatterPipeUpdatesModelCapturesTelemetryOmitsDisplayLogAndClearsOnStop() async throws {
        let fixture = try makeFixture()
        let fifo = fixture.appendingPathComponent("control.fifo")
        XCTAssertEqual(mkfifo(fifo.path, 0o600), 0)
        let descriptor = Darwin.open(fifo.path, O_RDWR)
        guard descriptor >= 0 else { throw AppError.invalid("Could not open test FIFO.") }
        defer { XCTAssertEqual(Darwin.close(descriptor), 0) }

        let executable = fixture.appendingPathComponent("s4mk3-hid-midi-bridge")
        let script = """
        #!/bin/sh
        trap 'printf "STOPPED_CLEAN\\n"; exit 0' INT
        exec 3< '\(fifo.path)'
        printf 'S4_PLATTER_STATUS {"wheels":[{"deck":1,"rpm":33.3,"bend_counts_per_second":8.5,"mode":"bend"},{"deck":2,"rpm":0.0,"bend_counts_per_second":0.0,"mode":"scratch"}]}\\n'
        printf 'S4_PLATTER_STATUS {"wheels":[]}\\n'
        printf 'ORDINARY_READY\\n'
        IFS= read -r line <&3
        """
        try Data(script.utf8).write(to: executable)
        try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: executable.path)

        let model = try BridgeController(
            resourceURL: fixture,
            supportURL: fixture.appendingPathComponent("support"),
            agent: TestNIHardwareAgent()
        )
        model.run.logLevel = "debug"
        model.startMotorDiagnosticCapture()
        let captureURL = try XCTUnwrap(model.motorDiagnosticURL)

        let platterReceived = expectation(description: "received live platter status snapshot")
        let ordinaryLogged = expectation(description: "received ordinary stdout line")
        let exited = expectation(description: "child stopped and exited")

        var didFulfillPlatter = false
        var didFulfillLog = false
        var sawStopping = false

        let platterSub = model.livePlatter.$state.sink { state in
            if !didFulfillPlatter, state.latest != nil {
                didFulfillPlatter = true
                platterReceived.fulfill()
            }
        }
        let logSub = model.logOutput.$text.sink { text in
            if !didFulfillLog, text.contains("ORDINARY_READY") {
                didFulfillLog = true
                ordinaryLogged.fulfill()
            }
        }
        let stateSub = model.$state.sink { current in
            if current == .stopping { sawStopping = true }
            if sawStopping && current == .stopped { exited.fulfill() }
        }

        model.start()
        await fulfillment(of: [platterReceived, ordinaryLogged], timeout: 10)

        let latest = try XCTUnwrap(model.platterDiagnostics.latest)
        XCTAssertEqual(latest.left.deck, 1)
        XCTAssertEqual(latest.left.rpm, 33.3)
        XCTAssertEqual(latest.left.bendCountsPerSecond, 8.5)
        XCTAssertEqual(latest.left.mode, .bend)
        XCTAssertEqual(latest.right.deck, 2)
        XCTAssertEqual(latest.right.mode, .scratch)
        XCTAssertFalse(model.log.contains("S4_PLATTER_STATUS"))

        model.stop()
        XCTAssertNil(model.platterDiagnostics.latest)
        await fulfillment(of: [exited], timeout: 10)
        withExtendedLifetime((platterSub, logSub, stateSub)) {}

        model.stopMotorDiagnosticCapture()
        let captured = String(decoding: try Data(contentsOf: captureURL), as: UTF8.self)
        XCTAssertTrue(captured.contains(#"S4_PLATTER_STATUS {"wheels":[{"deck":1"#))
        XCTAssertFalse(captured.contains(#"S4_PLATTER_STATUS {"wheels":[]}"#))
        XCTAssertFalse(model.log.contains("S4_PLATTER_STATUS"))
        XCTAssertNil(model.platterDiagnostics.latest)
    }

    private func makeFixture() throws -> URL {
        let root = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        let directory = FileManager.default.temporaryDirectory
            .appendingPathComponent("S4 Platter Tests \(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        try Data(contentsOf: root.appendingPathComponent("examples/led-config.json"))
            .write(to: directory.appendingPathComponent("default-led-config.json"))
        try Data(contentsOf: root.appendingPathComponent("examples/jog-config.json"))
            .write(to: directory.appendingPathComponent("default-jog-config.json"))
        try Data(contentsOf: root.appendingPathComponent("examples/tension-config.json"))
            .write(to: directory.appendingPathComponent("default-tension-config.json"))
        addTeardownBlock { try FileManager.default.removeItem(at: directory) }
        return directory
    }
}
