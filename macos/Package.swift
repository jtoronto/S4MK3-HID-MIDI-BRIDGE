// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "S4Bridge",
    platforms: [.macOS(.v12)],
    products: [.executable(name: "S4Bridge", targets: ["S4Bridge"])],
    targets: [
        .executableTarget(
            name: "S4Bridge",
            path: ".",
            exclude: ["build-app.sh", "Info.plist", "README.md", "DESIGN.md", "Tests"],
            sources: [
                "S4BridgeApp.swift", "PlatterDiagnostics.swift", "ScreenPreferences.swift",
                "ScreenSupport.swift", "ScreenSupport+Control.swift", "ScreenSettingsView.swift",
                "BridgeController+Screens.swift", "ScreenPrompts.swift", "NIHardwareAgent.swift",
                "NIHardwareAgentNative.swift", "BridgeStatusMenu.swift", "TTResponseCurveView.swift"
            ]
        ),
        .testTarget(name: "S4BridgeTests", dependencies: ["S4Bridge"], path: "Tests")
    ]
)
