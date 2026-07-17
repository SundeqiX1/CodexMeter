// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "CodexQuotaWidget",
    platforms: [
        .macOS(.v14)
    ],
    products: [
        .executable(name: "CodexQuotaWidget", targets: ["CodexQuotaWidget"])
    ],
    targets: [
        .executableTarget(
            name: "CodexQuotaWidget",
            path: "Sources/CodexQuotaWidget"
        ),
        .testTarget(
            name: "CodexQuotaWidgetTests",
            dependencies: ["CodexQuotaWidget"],
            path: "Tests/CodexQuotaWidgetTests"
        )
    ],
    swiftLanguageModes: [.v5]
)
