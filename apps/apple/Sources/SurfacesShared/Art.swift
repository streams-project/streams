import Contract
import SwiftUI

private func find(_ images: [Contract.Image]?, _ role: ImageRole) -> String? {
    images?.first(where: { $0.role == role })?.url
}

/// The picture for a wide place.
public func wide(_ images: [Contract.Image]?) -> String? {
    find(images, .backdrop) ?? find(images, .still) ?? find(images, .poster)
}

/// The picture for a tall place.
public func tall(_ images: [Contract.Image]?) -> String? {
    find(images, .poster) ?? wide(images)
}

/// The title's own logo, when the catalogue has one.
public func logo(_ images: [Contract.Image]?) -> String? {
    find(images, .logo)
}

/// Pictures already drawn once, kept so a row scrolled back draws at once.
@MainActor
/// Pictures already shown, kept so a page that comes back draws them at once.
public enum Pictures {
    private static var kept: [String: SwiftUI.Image] = [:]
    private static var order: [String] = []
    private static let most = 400

    public static func now(_ url: String) -> SwiftUI.Image? { kept[url] }

    public static func keep(_ url: String, _ image: SwiftUI.Image) {
        if kept[url] == nil {
            order.append(url)
            if order.count > most { kept.removeValue(forKey: order.removeFirst()) }
        }
        kept[url] = image
    }
}

/// A picture filling its box, or fitting it.
public struct Art: SwiftUI.View {
    public let url: String?
    public var fit: Bool
    /// Runs once the picture is there, from the network or from memory.
    public var loaded: (() -> Void)?

    public init(_ url: String?, fit: Bool = false) {
        self.url = url
        self.fit = fit
    }

    public init(_ url: String?, loaded: @escaping () -> Void) {
        self.url = url
        self.fit = false
        self.loaded = loaded
    }

    public var body: some SwiftUI.View {
        Color.clear
            .overlay {
                if let url, let kept = Pictures.now(url) {
                    kept.resizable().aspectRatio(contentMode: fit ? .fit : .fill)
                        .onAppear { loaded?() }
                } else if let url, let address = URL(string: url) {
                    AsyncImage(url: address) { phase in
                        switch phase {
                        case .success(let image):
                            image.resizable().aspectRatio(contentMode: fit ? .fit : .fill)
                                .onAppear {
                                    Pictures.keep(url, image)
                                    loaded?()
                                }
                        case .failure:
                            Color.clear.onAppear { loaded?() }
                        default:
                            Color.clear
                        }
                    }
                }
            }
            .clipped()
    }
}

/// One of the brand's icons, tinted.
public struct Icon: SwiftUI.View {
    public let name: String
    public let size: CGFloat
    public var tint: Color

    public init(_ name: String, _ size: CGFloat, tint: Color = .white) {
        self.name = name
        self.size = size
        self.tint = tint
    }

    public var body: some SwiftUI.View {
        SwiftUI.Image(name, bundle: .module)
            .renderingMode(.template)
            .resizable()
            .aspectRatio(contentMode: .fit)
            .frame(width: size.ref, height: size.ref)
            .foregroundStyle(tint)
            .flipsForRightToLeftLayoutDirection(name == "icon_back" || name == "icon_leave")
    }
}

/// The brand's mark, from brand.json at build time.
public struct Mark: SwiftUI.View {
    public let size: CGFloat

    public init(_ size: CGFloat) {
        self.size = size
    }

    public var body: some SwiftUI.View {
        SwiftUI.Image("mark", bundle: .module)
            .resizable()
            .aspectRatio(contentMode: .fit)
            .frame(width: size.ref, height: size.ref)
    }
}
