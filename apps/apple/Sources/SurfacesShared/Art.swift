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
///
/// A picture is decoration unless it is given words. Every place this is drawn - a tile, a hero, a
/// face, a billboard - already draws the title's name, or sits inside a control that carries it, so
/// the picture repeats what is said beside it. `says` is for the one case that does not: a picture
/// that is the only thing on screen saying what it is. Leaving it out hides the picture from a
/// screen reader, which is the right answer for the rest and is why it is the default.
public struct Art: SwiftUI.View {
    public let url: String?
    public var fit: Bool
    /// Runs once the picture is there, from the network or from memory.
    public var loaded: (() -> Void)?
    /// Words for a picture that carries meaning of its own. From `strings/` or the catalogue.
    public var says: String = ""

    public init(_ url: String?, fit: Bool = false, says: String = "") {
        self.url = url
        self.fit = fit
        self.says = says
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
            .modifier(Says(words: says))
    }
}

/// Words for a picture, or none at all. Written once so the three pictures below answer the
/// question the same way: words a person hears, or gone from the tree entirely. An empty label set
/// on a picture would leave a reader stopping on it with nothing to read - measured on iPhone 17 on
/// 2026-11-12 - which is the one answer that is worse than both.
private struct Says: ViewModifier {
    let words: String

    func body(content: Content) -> some SwiftUI.View {
        if words.isEmpty {
            content.accessibilityHidden(true)
        } else {
            content.accessibilityLabel(words)
        }
    }
}

/// One of the brand's icons, tinted.
///
/// An icon is decoration unless it is given words, because almost every one here is drawn beside
/// the word it stands for or inside a control that carries it. The default is not a convenience:
/// measured through `Tests/Heard` on 2026-11-12, an unmarked `Icon("icon_search", 34)` is read out
/// as "icon_search" - the asset's own file name, in no language - and there are hundreds of them.
///
/// `says` is for an icon that is the whole of a control, where the control itself cannot carry the
/// words. Most icon-only controls should instead label the control (`says(_:)` in `Says.swift`), so
/// the words belong to the thing that is pressed.
public struct Icon: SwiftUI.View {
    public let name: String
    public let size: CGFloat
    public var tint: Color
    /// Words for an icon that carries meaning on its own. From `strings/`, never written here.
    public var says: String = ""

    public init(_ name: String, _ size: CGFloat, tint: Color = .white, says: String = "") {
        self.name = name
        self.size = size
        self.tint = tint
        self.says = says
    }

    public var body: some SwiftUI.View {
        SwiftUI.Image(name, bundle: .module)
            .renderingMode(.template)
            .resizable()
            .aspectRatio(contentMode: .fit)
            .frame(width: size.ref, height: size.ref)
            .foregroundStyle(tint)
            .flipsForRightToLeftLayoutDirection(name == "icon_back" || name == "icon_leave")
            .modifier(Says(words: says))
    }
}

/// The brand's mark, from brand.json at build time.
///
/// Always decoration, with no way to give it words: every place it is drawn has the page's own name
/// beside it, saying the same thing. Unmarked it was read out as "mark" (measured 2026-11-12). A
/// place that needs the brand's name spoken needs the name, from `brand.json`, not this picture.
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
            .accessibilityHidden(true)
    }
}
