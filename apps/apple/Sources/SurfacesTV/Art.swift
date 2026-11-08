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
enum Pictures {
    private static var kept: [String: SwiftUI.Image] = [:]
    private static var order: [String] = []
    private static let most = 400

    static func now(_ url: String) -> SwiftUI.Image? { kept[url] }

    static func keep(_ url: String, _ image: SwiftUI.Image) {
        if kept[url] == nil {
            order.append(url)
            if order.count > most { kept.removeValue(forKey: order.removeFirst()) }
        }
        kept[url] = image
    }
}

/// A picture filling its box, or fitting it.
public struct Art: View {
    public let url: String?
    public var fit: Bool

    public init(_ url: String?, fit: Bool = false) {
        self.url = url
        self.fit = fit
    }

    public var body: some View {
        Color.clear
            .overlay {
                if let url, let kept = Pictures.now(url) {
                    kept.resizable().aspectRatio(contentMode: fit ? .fit : .fill)
                } else if let url, let address = URL(string: url) {
                    AsyncImage(url: address) { phase in
                        if case .success(let image) = phase {
                            image.resizable().aspectRatio(contentMode: fit ? .fit : .fill)
                                .onAppear { Pictures.keep(url, image) }
                        }
                    }
                }
            }
            .clipped()
    }
}

/// One of the brand's icons, tinted.
public struct Icon: View {
    public let name: String
    public let size: CGFloat
    public var tint: Color

    public init(_ name: String, _ size: CGFloat, tint: Color = .white) {
        self.name = name
        self.size = size
        self.tint = tint
    }

    public var body: some View {
        SwiftUI.Image(name, bundle: .module)
            .renderingMode(.template)
            .resizable()
            .aspectRatio(contentMode: .fit)
            .frame(width: size.ref, height: size.ref)
            .foregroundStyle(tint)
    }
}

/// The brand's mark, from brand.json at build time.
public struct Mark: View {
    public let size: CGFloat

    public init(_ size: CGFloat) {
        self.size = size
    }

    public var body: some View {
        SwiftUI.Image("mark", bundle: .module)
            .resizable()
            .aspectRatio(contentMode: .fit)
            .frame(width: size.ref, height: size.ref)
    }
}
