import AppKit
import Combine
import SwiftUI
import XCTest
@testable import S4Bridge

@MainActor
final class SettingsVisibilityTests: XCTestCase {
    func testMountedPlatterViewShowsStopWithoutWaitingForAnotherSnapshot() async throws {
        _ = NSApplication.shared
        let model = LivePlatterDiagnostics()
        XCTAssertTrue(model.state.acceptLine(
            #"S4_PLATTER_STATUS {"wheels":[{"deck":1,"rpm":33,"bend_counts_per_second":0,"mode":"bend"},{"deck":2,"rpm":0,"bend_counts_per_second":0,"mode":"off"}]}"#
        ))
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 760, height: 300),
                              styleMask: [.titled], backing: .buffered, defer: false)
        window.isReleasedWhenClosed = false
        defer {
            window.close()
            model.display.setPresentationActive(false, bridgeRunning: false)
        }
        let started = expectation(description: "mounted live display shows the current sample")
        let starting = model.display.$presentation.filter { $0.left.modeText == "Bend" }.prefix(1)
            .sink { _ in started.fulfill() }
        let host = NSHostingView(rootView: LivePlatterDiagnosticsView(model: model.display, bridgeRunning: true))
        window.contentView = host
        host.layoutSubtreeIfNeeded()
        await fulfillment(of: [started], timeout: 3)

        let stopped = expectation(description: "mounted display responds to bridge stop")
        let stopping = model.display.$presentation.filter { $0.left.modeText == "Stopped" }.prefix(1)
            .sink { _ in stopped.fulfill() }
        host.rootView = LivePlatterDiagnosticsView(model: model.display, bridgeRunning: false)
        host.layoutSubtreeIfNeeded()
        await fulfillment(of: [stopped], timeout: 3)
        XCTAssertNil(model.display.presentation.left.rpm)
        XCTAssertEqual(model.state.latest?.left.rpm, 33)
        withExtendedLifetime((starting, stopping)) {}
    }

    func testHiddenSettingsUnmountsDiagnosticsAndReopensWithRetainedOutput() async throws {
        _ = NSApplication.shared
        let model = try makeScreenController(self, agent: TestNIHardwareAgent())
        let visibility = SettingsWindowVisibility()
        let window = VisibilityWindow(contentRect: NSRect(x: 0, y: 0, width: 760, height: 680),
                                      styleMask: [.titled], backing: .buffered, defer: false)
        window.isReleasedWhenClosed = false
        defer { window.close() }
        let host = NSHostingView(rootView: BridgeSettingsView(
            controller: model, visibility: visibility, selectedTab: .diagnostics
        ))
        window.contentView = host
        window.reportedVisible = true
        window.reportedOcclusion = .visible
        visibility.update(for: window)
        host.layoutSubtreeIfNeeded()
        XCTAssertFalse(textViews(in: host).isEmpty)

        let hiddenUpdate = expectation(description: "unmounted diagnostics has no expiration updates")
        hiddenUpdate.isInverted = true
        let hidden = model.livePlatter.display.objectWillChange.sink { _ in hiddenUpdate.fulfill() }
        visibility.suspend()
        host.layoutSubtreeIfNeeded()
        model.acceptPlatterStatus(
            #"S4_PLATTER_STATUS {"wheels":[{"deck":3,"rpm":34,"bend_counts_per_second":9,"mode":"bend"},{"deck":4,"rpm":0,"bend_counts_per_second":0,"mode":"off"}]}"#
        )
        model.appendLog("RECEIVED_WHILE_SETTINGS_HIDDEN")
        await fulfillment(of: [hiddenUpdate], timeout: 0.7)
        hidden.cancel()

        visibility.update(for: window)
        host.layoutSubtreeIfNeeded()
        XCTAssertEqual(model.platterDiagnostics.latest?.left.deck, 3)
        XCTAssertEqual(model.livePlatter.display.presentation.left.modeText, "Stopped")
        XCTAssertTrue(textViews(in: host).contains { $0.string.contains("RECEIVED_WHILE_SETTINGS_HIDDEN") })
        visibility.suspend()
        host.layoutSubtreeIfNeeded()
    }

    func testSettingsOnlyPublishesVisibilityTransitionsForUsableWindows() {
        _ = NSApplication.shared
        let visibility = SettingsWindowVisibility()
        let window = VisibilityWindow()
        window.isReleasedWhenClosed = false
        var transitions: [Bool] = []
        let observation = visibility.$isVisible.dropFirst().sink { transitions.append($0) }
        defer { window.close() }

        for (visible, miniaturized, occluded, expected) in [
            (false, false, false, false),
            (true, false, false, true),
            (true, false, false, true),
            (true, false, true, false),
            (true, true, false, false),
            (true, false, false, true),
            (false, false, false, false)
        ] {
            window.reportedVisible = visible
            window.reportedMiniaturized = miniaturized
            window.reportedOcclusion = occluded ? [] : .visible
            visibility.update(for: window)
            XCTAssertEqual(visibility.isVisible, expected)
        }

        window.reportedVisible = true
        visibility.update(for: window)
        visibility.suspend()
        visibility.suspend()
        XCTAssertEqual(transitions, [true, false, true, false, true, false])
        withExtendedLifetime(observation) {}
    }

    private func textViews(in view: NSView) -> [NSTextView] {
        (view as? NSTextView).map { [$0] } ?? view.subviews.flatMap { textViews(in: $0) }
    }
}

@MainActor
private final class VisibilityWindow: NSWindow {
    var reportedVisible = false
    var reportedMiniaturized = false
    var reportedOcclusion: NSWindow.OcclusionState = []

    override var isVisible: Bool { reportedVisible }
    override var isMiniaturized: Bool { reportedMiniaturized }
    override var occlusionState: NSWindow.OcclusionState { reportedOcclusion }
}
