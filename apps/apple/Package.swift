// swift-tools-version: 5.10
import PackageDescription

let package = Package(
    name: "AppleFamily",
    platforms: [
        .tvOS(.v17),
        .iOS(.v17),
    ],
    products: [
        .library(name: "CoreBridge", targets: ["CoreBridge"]),
        .library(name: "Contract", targets: ["Contract"]),
        .library(name: "Ports", targets: ["Ports"]),
        .library(name: "CoreHost", targets: ["CoreHost"]),
        .library(name: "Adapters", targets: ["Adapters"]),
        .library(name: "SurfacesTV", targets: ["SurfacesTV"]),
    ],
    targets: [
        .target(
            name: "CoreBridge",
            path: "Sources/CoreBridge",
            publicHeadersPath: "include",
            linkerSettings: [
                .unsafeFlags([
                    "-L\(Context.packageDirectory)/target",
                    "-lbridge_c",
                ])
            ]
        ),
        .target(
            name: "Contract",
            path: "Sources/Contract"
        ),
        .target(
            name: "Ports",
            dependencies: ["Contract"],
            path: "Sources/Ports"
        ),
        .target(
            name: "CoreHost",
            dependencies: ["CoreBridge", "Contract", "Ports"],
            path: "Sources/CoreHost"
        ),
        .target(
            name: "Adapters",
            dependencies: ["Contract", "Ports"],
            path: "Sources/Adapters"
        ),
        .target(
            name: "SurfacesTV",
            dependencies: ["Contract", "Ports", "CoreHost"],
            path: "Sources/SurfacesTV"
        ),
    ]
)
