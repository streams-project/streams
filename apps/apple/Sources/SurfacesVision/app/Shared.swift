// Every window screen sees the brand's shared pieces (colours, font, icons, numbers, words)
// without importing them one file at a time. The same line the other two surfaces open with
// (`SurfacesTV/app/Shared.swift`, `SurfacesPhone/app/Shared.swift`).
@_exported import SurfacesShared

import SwiftUI

/// In this module a View is SwiftUI's, never the contract's View of the core.
public typealias View = SwiftUI.View
