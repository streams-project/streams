import Contract
import Ports

public func titleAddress(_ key: TitleKey) -> String {
    Address.title + key.rawValue.replacingOccurrences(of: ":", with: "-")
}

public func watchAddress(_ key: TitleKey) -> String {
    Address.watch + key.rawValue.replacingOccurrences(of: ":", with: "/")
}

public func tabAddress(_ screen: String) -> String {
    Address.of(screen)
}

extension Tile {
    public var isSeries: Bool { title.rawValue.hasPrefix("tv:") }
}
