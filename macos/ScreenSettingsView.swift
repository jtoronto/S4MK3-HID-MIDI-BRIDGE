import SwiftUI

struct ScreenSettingsView: View {
    @ObservedObject var controller: BridgeController
    @ObservedObject var screens: ScreenSupport

    init(controller: BridgeController) {
        self.controller = controller
        screens = controller.screens
    }

    var body: some View {
        Group {
            if controller.run.mode == .ni {
                VStack(alignment: .leading, spacing: 12) {
                    Text("Screens are managed by the Native Instruments service in NI Bridge mode.")
                    Text("The direct-mode screen handover settings are not used.")
                        .font(.caption).foregroundColor(.secondary)
                    Spacer()
                }.padding(20).frame(maxWidth: .infinity, alignment: .leading)
            } else {
                directSettings
            }
        }
    }

    private var directSettings: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                GroupBox("S4 screens") {
                    VStack(alignment: .leading, spacing: 12) {
                        Toggle("Enable S4 screens", isOn: Binding(
                            get: { screens.preferences.enabled },
                            set: { controller.setScreensEnabled($0) }
                        ))
                        .disabled(screens.busy || controller.state == .stopping)
                        Label(screens.state.rawValue, systemImage: screens.needsAttention ? "exclamationmark.triangle" : "display.2")
                            .foregroundColor(screens.needsAttention ? .red : .secondary)
                        Text("HOTCUE and SAMPLES show jump, loop, key and tempo range. STEMS and MUTE show the mute/solo legend. Full-controller profile only.")
                            .font(.caption).foregroundColor(.secondary)
                        if screens.needsAttention && controller.state == .running {
                            Button("Enable screens - stop NI driver") {
                                controller.enableScreensByStoppingDriver()
                            }.disabled(screens.busy || !screens.fullProfile)
                        }
                    }.padding(12).frame(maxWidth: .infinity, alignment: .leading)
                }
                GroupBox("Native Instruments driver") {
                    VStack(alignment: .leading, spacing: 12) {
                        Text(driverStatus)
                        Toggle("Automatically stop the NI driver when starting screens", isOn: Binding(
                            get: { screens.preferences.automaticallyStopNI },
                            set: { controller.setScreenAutomation(stop: $0) }
                        ))
                        Toggle("Restore the NI driver when stopping the bridge", isOn: Binding(
                            get: { screens.preferences.restoreNIOnStop },
                            set: { controller.setScreenAutomation(restore: $0) }
                        ))
                        Text("Automatic restore applies only when this app stopped the driver. Screen changes and these options are saved immediately.")
                            .font(.caption).foregroundColor(.secondary)
                        Text("Close Traktor, Maschine and other NI controller apps before stopping their shared driver.")
                            .font(.caption).foregroundColor(.secondary)
                    }.disabled(screens.busy).padding(12)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
                GroupBox("Manual recovery") {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Restart the Native Instruments screen driver before returning to Traktor. This works even if another app stopped it, or this app previously crashed.")
                        if screens.preferences.restorePending {
                            Label("The NI driver may still need restoring.", systemImage: "exclamationmark.triangle")
                                .foregroundColor(.secondary)
                        }
                        HStack(spacing: 8) {
                            Button("Restart Native Instruments screen driver") { controller.restartNIDriver() }
                                .disabled(controller.state == .stopping
                                    || (screens.preferences.enabled && screens.driverRunning == true))
                            Button("Stop NI driver") { controller.stopNIDriver() }
                                .disabled(screens.preferences.enabled && screens.driverRunning == false)
                        }.disabled(screens.busy)
                        Text("If the bridge is running, restarting the NI driver first offers to stop the bridge and release the screens. Recovery remains available with screen support off.")
                            .font(.caption).foregroundColor(.secondary)
                    }.padding(12).frame(maxWidth: .infinity, alignment: .leading)
                }
                if !screens.errorMessage.isEmpty {
                    Text(screens.errorMessage).foregroundColor(.red).textSelection(.enabled)
                }
            }.padding(20).frame(maxWidth: .infinity)
        }
        .onAppear { screens.refreshDriverStatus() }
    }

    private var driverStatus: String {
        if !screens.preferences.enabled { return "Automatic driver checks are off." }
        switch screens.driverRunning {
        case true: return "NI screen driver is running."
        case false: return "NI screen driver is not running."
        case nil: return "Automatic driver checks are off."
        }
    }
}
