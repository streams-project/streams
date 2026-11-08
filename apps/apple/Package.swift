// swift-tools-version: 5.10
import PackageDescription

let package = Package(
    name: "AppleFamily",
    platforms: [
        .macOS(.v14),
        .tvOS(.v17),
        .iOS(.v17),
    ],
    products: [
        .library(name: "CoreBridge", targets: ["CoreBridge"]),
        .library(name: "Contract", targets: ["Contract"]),
        .library(name: "Ports", targets: ["Ports"]),
        .library(name: "CoreHost", targets: ["CoreHost"]),
        .library(name: "Adapters", targets: ["Adapters"]),
        .library(name: "SurfacesShared", targets: ["SurfacesShared"]),
        .library(name: "SurfacesTV", targets: ["SurfacesTV"]),
        .library(name: "SurfacesPhone", targets: ["SurfacesPhone"]),
    ],
    targets: [
        .target(
            name: "CoreBridge",
            path: "Sources/CoreBridge",
            publicHeadersPath: "include",
            // Each platform links its own copy of the core (build-core.sh writes target/tvos and target/ios).
            linkerSettings: [
                .unsafeFlags(["-L\(Context.packageDirectory)/target/tvos", "-lbridge_c"], .when(platforms: [.tvOS])),
                .unsafeFlags(["-L\(Context.packageDirectory)/target/ios", "-lbridge_c"], .when(platforms: [.iOS])),
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
            name: "SurfacesShared",
            dependencies: ["Contract", "Ports"],
            path: "Sources/SurfacesShared",
            exclude: ["README.md"],
            resources: [.process("Resources")]
        ),
        .target(
            name: "SurfacesPhone",
            dependencies: ["Contract", "Ports", "SurfacesShared"],
            path: "Sources/SurfacesPhone",
            exclude: ["README.md"]
        ),
        .target(
            name: "SurfacesTV",
            dependencies: ["Contract", "Ports", "SurfacesShared"],
            path: "Sources/SurfacesTV",
            exclude: ["README.md"],
            resources: [.process("Resources")]
        ),
    ]
)
