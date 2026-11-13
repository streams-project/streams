import Contract
import Ports
import SwiftUI
import UIKit

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

/// The box a picture will fill, as the pixels it covers on this screen.
///
/// Points times the screen's scale, because the port's box is in pixels and a 120 point tile on a
/// phone is 360 of them. A side that is not a number, or is not positive - a layout that says
/// `maxWidth: .infinity`, a measure that has not happened yet - is 0, which the port reads as the
/// picture's own size: no box is the one safe answer, because a box guessed too small decodes a
/// picture too blurry to draw and there is no second chance at it.
public func inPixels(_ box: CGSize, _ scale: CGFloat) -> (wide: Int, high: Int) {
    func side(_ length: CGFloat) -> Int {
        guard length.isFinite, length > 0, length < 100_000 else { return 0 }
        return Int((length * max(scale, 1)).rounded())
    }
    return (side(box.width), side(box.height))
}

/// One picture wanted, in the box the drawing will fill.
///
/// Written once because every draw site has to ask the same way: a warm read and the draw that
/// follows it land on one cache entry only while both name the same box (`Adapters/Pictures.swift`
/// files a picture under the rung its box falls on).
public func wanted(_ url: String, _ box: CGSize, _ scale: CGFloat, pixels: Bool = false) -> Want {
    let (wide, high) = inPixels(box, scale)
    return Want(url: url, wide: wide, high: high, pixels: pixels)
}

/// The picture at an address, drawn, with nothing of how it was fetched in here.
///
/// The port answers with whatever its adapter decoded. This family's adapter hands over a `UIImage`
/// (`Adapters/Pictures.swift`), which is a fact about the adapter and not about the port, so a
/// picture that is anything else is read as no picture at all rather than as a crash.
public func drawn(_ picture: Ports.Picture?) -> SwiftUI.Image? {
    guard case .painted(let held) = picture, let image = held as? UIImage else { return nil }
    return SwiftUI.Image(uiImage: image)
}

/// A picture filling its box, or fitting it.
///
/// A picture is decoration unless it is given words. Every place this is drawn - a tile, a hero, a
/// face, a billboard - already draws the title's name, or sits inside a control that carries it, so
/// the picture repeats what is said beside it. `says` is for the one case that does not: a picture
/// that is the only thing on screen saying what it is. Leaving it out hides the picture from a
/// screen reader, which is the right answer for the rest and is why it is the default.
///
/// The picture comes through the ``Ports/Pictures`` port, which is taken by value and has no
/// default: a draw site that has no port to hand does not compile, which is the only way a seam
/// like this stays plugged in. What is left here is what a surface is for - a box, a shape, words
/// for a reader, and a picture drawn once the port answers.
public struct Art: SwiftUI.View {
    public let url: String?
    /// The box this picture will fill, in points, as the layout around it sets it.
    ///
    /// No default. A box that could be left out would be left out - it is the same seam the port
    /// itself refuses to default (`Ports/Pictures`), and the cost of forgetting it is invisible:
    /// nothing on screen looks wrong when a 1000 by 1500 poster is held whole behind a 119 point
    /// tile. A site with no fixed box of its own passes the widest it can be drawn at; a side of 0
    /// says the picture's own size and is read as a mistake by `PicturesTests`.
    public var box: CGSize
    public var fit: Bool
    /// Runs once the picture is there, or once it is certain there will not be one. Both answers
    /// end a wait, and a screen that waits on a picture that is not coming never draws.
    public var loaded: (() -> Void)?
    /// Words for a picture that carries meaning of its own. From `strings/` or the catalogue.
    public var says: String = ""

    private let pictures: Ports.Pictures
    /// The picture this view read, and the address it is of. Both, because a lazy row hands one of
    /// these views to a second address as the viewer scrolls, and a picture kept without its
    /// address would be drawn under the wrong title until the next read landed.
    @State private var shown: SwiftUI.Image?
    @State private var shownFor: String?
    /// How many pixels a point is on the screen this is drawn on, so the box reaches the port in
    /// the unit the port asks for. Read from the environment rather than from the screen, because
    /// a view knows which screen it is on and `UIScreen.main` does not.
    @Environment(\.displayScale) private var scale

    public init(_ pictures: Ports.Pictures, _ url: String?, box: CGSize, fit: Bool = false, says: String = "") {
        self.pictures = pictures
        self.url = url
        self.box = box
        self.fit = fit
        self.says = says
    }

    public init(_ pictures: Ports.Pictures, _ url: String?, box: CGSize, loaded: @escaping () -> Void) {
        self.pictures = pictures
        self.url = url
        self.box = box
        self.fit = false
        self.loaded = loaded
    }

    /// What this view asks the port for: the address and the box it will fill.
    private func want(_ url: String) -> Want {
        wanted(url, box, scale)
    }

    /// A picture the port is already holding for this address, or nothing.
    private func inHand() -> SwiftUI.Image? {
        guard let url, !url.isEmpty else { return nil }
        return drawn(pictures.kept(want(url)))
    }

    public var body: some SwiftUI.View {
        // Asked for in the draw and not in the task below, because a task runs after this view's
        // first frame is on screen: a row scrolled back over pictures the port is still holding
        // would blink once per tile if this waited for it.
        let now = (shownFor == url ? shown : nil) ?? inHand()
        return Color.clear
            .overlay {
                if let now {
                    now.resizable().aspectRatio(contentMode: fit ? .fit : .fill)
                }
            }
            .clipped()
            .modifier(Says(words: says))
            .task(id: url) { await read() }
    }

    private func read() async {
        guard let url, !url.isEmpty else { return }
        if (shownFor == url && shown != nil) || inHand() != nil {
            loaded?()
            return
        }
        let answer = await pictures.picture(want(url))
        if let picture = drawn(answer) {
            shown = picture
            shownFor = url
        }
        loaded?()
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
