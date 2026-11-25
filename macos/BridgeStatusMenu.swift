import AppKit

@MainActor
func bridgeStatusMenu(controller: BridgeController?, state: BridgeState, target: AppDelegate) -> NSMenu {
    let menu = NSMenu()
    menu.delegate = target
    menu.autoenablesItems = false
    func add(_ title: String, _ selector: Selector?, enabled: Bool = true, id: String? = nil) {
        let item = NSMenuItem(title: title, action: selector, keyEquivalent: "")
        item.target = target
        item.isEnabled = enabled
        if let id { item.identifier = NSUserInterfaceItemIdentifier(id) }
        menu.addItem(item)
    }
    add("Bridge: \(state.rawValue)", nil, enabled: false)
    if let controller {
        add(controller.run.mode == .ni ? "Screens: NI service" : controller.screens.state.rawValue, nil, enabled: false)
    }
    menu.addItem(.separator())
    add("Start bridge", #selector(AppDelegate.start), enabled: controller?.isBusy == false)
    add("Stop bridge", #selector(AppDelegate.stop), enabled: controller?.canStop == true)
    menu.addItem(.separator())
    if controller?.run.mode != .ni {
    add("Enable S4 screens", #selector(AppDelegate.toggleScreens),
        enabled: controller?.screens.busy == false && state != .stopping, id: "screens.enabled")
    menu.items.last?.state = controller?.screens.preferences.enabled == true ? .on : .off
    if controller?.screens.needsAttention == true {
        add("Enable screens - stop NI driver", #selector(AppDelegate.enableScreens),
            enabled: state == .running && controller?.screens.busy == false, id: "screens.retry")
    }
    add("Restart Native Instruments screen driver", #selector(AppDelegate.restoreDriver),
        enabled: controller?.screens.busy == false && state != .stopping
            && (controller?.screens.preferences.enabled == false || controller?.screens.driverRunning != true),
        id: "screens.restore")
    }
    menu.addItem(.separator())
    add("Settings…", #selector(AppDelegate.showSettings))
    add("Install Djay mapping…", #selector(AppDelegate.installMapping))
    add("Check controller", #selector(AppDelegate.checkController), enabled: controller?.isBusy == false)
    menu.addItem(.separator())
    add("Quit S4 MK3 Bridge", #selector(AppDelegate.quit))
    return menu
}
