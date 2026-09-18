// swift-tools-version:5.9
import PackageDescription
let package = Package(
    name: "tauri-plugin-native-glass",
    platforms: [.iOS(.v16)],
    products: [.library(name: "tauri-plugin-native-glass", type: .static, targets: ["NativeGlass"])],
    dependencies: [.package(name: "Tauri", path: "../.tauri/tauri-api")],
    targets: [.target(name: "NativeGlass", dependencies: [.byName(name: "Tauri")], path: "Sources")]
)
