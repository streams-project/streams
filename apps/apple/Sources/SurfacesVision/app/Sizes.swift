import CoreGraphics
import SwiftUI

/// The geometry of a window in space, and where every number in it comes from.
///
/// Read this before copying anything out of a reference file into this module.
///
/// **What does not apply.** `reference/netflix-on-the-tv.md` is measured in reference pixels at one
/// fixed size, 1920 x 1080, and the television surface draws at exactly that size
/// (`SurfacesTV/app/Round.swift`). Its page margin of 72, its 311 x 505 closed tile, its 328 pitch
/// and its y 199-708 focus band are all lengths on a screen whose size is known in advance. A
/// window in space is resized by the viewer and sits at a distance the viewer chooses, so not one
/// of those lengths has a meaning here, and none of them is copied. Two of its facts are about
/// focus rather than length -- that the focused row is pinned to a fixed band, and that the focused
/// tile swaps to a 2.91x wider still -- and those do not apply either, for the reason in
/// ``Reach``: nothing here knows what is focused.
///
/// **What does apply.** `reference/netflix-on-the-web.md` measures the only container in either
/// file that the viewer resizes, and the thing it establishes is a *rule* rather than a number:
/// across nine widths the tiles fill the content box exactly, with the page margin and the number
/// across as the two step functions and everything else derived. That rule is reused below,
/// unchanged, in ``tileWide(_:)``. Its measured step thresholds, its 8 px gutter and its margins
/// are *not* reused: they were read off a browser page at arm's length with a mouse, where the
/// smallest thing that has to be hittable is a few pixels, and the number that sets the steps here
/// is ``Reach/tileFloor``, which comes from how coarsely an eye can be aimed.
///
/// The 16:9 tile shape is reused, because both reference files measure it independently.
///
/// **Everything else below is this surface's own**, chosen here, and said so one by one. None of it
/// has been seen on a device: there is no Vision Pro on this machine and no visionOS simulator
/// either, so every number here is a considered starting point and not a measurement.
///
/// Points, not reference pixels. The television works in a reference unit because it draws to one
/// known raster; the phone works in points (`SurfacesPhone/app/Sizes.swift`) because it does not.
/// A window is the second case, so `Measure.swift`'s `.ref` is not used in this module.

/// The gap between two tiles in a row, in points. **This surface's own.**
///
/// It is not a free choice in one direction: whatever it is, two tiles that each clear
/// ``Reach/target`` already have their centres far enough apart, so the gutter is free to be small.
/// It is kept visible rather than hairline so a row reads as separate things to look at.
public let gutter: CGFloat = 16

/// The margin down each side of the page, in points, by how wide the window is. **This surface's
/// own**, in three steps.
///
/// Stepped rather than a percentage for the reason the web reference gives for its own margin: a
/// percentage grows without limit on a wide window and collapses on a narrow one, and the margin is
/// there to keep content off the edge rather than to be a share of anything.
public func pageEdge(_ width: CGFloat) -> CGFloat {
    width < 900 ? 32 : width < 1400 ? 44 : 56
}

/// The widest a tile can come out, in points, wherever two or more fit across. **Derived, not
/// chosen**, and proved rather than asserted.
///
/// ``across(_:)`` takes the largest count whose tile still clears ``Reach/tileFloor``, so the next
/// count up would fall below it. Writing `C` for the content width and `g` for the ``gutter``, that
/// means `C < floor * (n + 1) + g * n`, and therefore
/// `tile(n) = (C - g(n - 1)) / n < floor * (n + 1) / n + g / n`, which is largest at `n = 2`:
/// `1.5 * floor + g / 2`. Checked by exhaustion over every integer width from 400 to 3000 on
/// 2026-11-14: the tile lands in 213.33...327.50 at every one of them, hitting both ends.
///
/// There is deliberately no ceiling *rule* in ``across(_:)`` to go with this. One was written and
/// removed the same day: a loop that fitted another tile across whenever the tile passed a ceiling
/// could never run, because the arithmetic above shows a tile can only exceed any sensible ceiling
/// when exactly one fits across, which is the case such a loop must not touch. It would have been
/// a line that read as a safeguard and was never reached.
///
/// Worth recording because it is a check and not a source: this band, arrived at from how coarsely
/// an eye can be aimed, lands on top of the 226.7...281.6 the web reference measured for a tile
/// across nine browser widths. The two were derived from unrelated quantities.
public let tileWidest: CGFloat = 1.5 * Reach.tileFloor + gutter / 2

/// How many tiles fit across a window this wide.
///
/// The largest count whose tile still clears ``Reach/tileFloor``. That is the whole rule: the count
/// steps so the tile never becomes too small to look at, which is the same shape as the web
/// reference's step functions with a different quantity deciding where the steps fall.
///
/// A window narrower than one floor-width plus its margins answers 1 and draws a tile below the
/// floor, because the alternative is drawing nothing. No visionOS window is that narrow -- the
/// smallest this surface ever asks for is ``windowWide`` -- and `Tests/VisionTests` states the
/// bound rather than leaving it to be found.
public func across(_ width: CGFloat) -> Int {
    var best = 1
    for count in 1...12 where tileWide(width, count) >= Reach.tileFloor {
        best = count
    }
    return best
}

/// The width of one tile, given how many are across. **The rule reused from the web reference**:
/// the tiles and the gutters between them fill the content box exactly, with nothing left over.
public func tileWide(_ width: CGFloat, _ count: Int) -> CGFloat {
    let count = CGFloat(max(count, 1))
    return (width - 2 * pageEdge(width) - (count - 1) * gutter) / count
}

/// The width of one tile in a window this wide.
public func tileWide(_ width: CGFloat) -> CGFloat {
    tileWide(width, across(width))
}

/// The height of one tile: 16:9, the one tile shape both reference files measure.
public func tileHigh(_ width: CGFloat) -> CGFloat {
    (tileWide(width) * 9 / 16).rounded()
}

/// The box a tile's picture fills.
public func tileBox(_ width: CGFloat) -> CGSize {
    CGSize(width: tileWide(width), height: tileHigh(width))
}

/// The card at the top of a page of rows, as a share of the window's width. **This surface's own.**
///
/// The web reference measures a hero at 0.4587 of its width, constant to four decimals across seven
/// widths, and the television's is 0.431. Neither is borrowed: both are cards that fill the whole
/// of a screen the viewer is sitting in front of, and a window in space that opened with a card
/// taller than the room left for a row would put every tile below the fold on first sight, which is
/// what both of those references measure happening and the one part of them worth not repeating.
/// So this card is shallower on purpose, and it is a ratio for the same reason theirs are: it has
/// to survive being resized.
public let heroRatio: CGFloat = 0.34

/// The gap between the bottom of one row and the title of the next, in points. **This surface's
/// own.**
public let rowGap: CGFloat = 36

/// The gap between a row's title and its tiles, in points. **This surface's own.**
public let titleGap: CGFloat = 12

/// The size this surface asks its window to open at, in points. **This surface's own**: at this
/// width ``across(_:)`` answers five, with a 225.6 pt tile.
public let windowWide: CGFloat = 1280
public let windowHigh: CGFloat = 820

/// How much larger a corner is in a window about a metre away than on a phone in the hand, in
/// points per brand unit. **This surface's own**, between the phone's 1 and the television's 1.5
/// (`SurfacesTV/app/Round.swift`), because the viewing distance is between the two.
private let windowScale: CGFloat = 1.25

/// The brand's corners in a window, by role, in points. Every rounded part takes one of these, so
/// `brand.json` changes them all at once -- the same arrangement as `Round` on the television and
/// `Round.kt` on android, with this surface's own scale.
public enum Round {
    /// Tiles, stills, the pictures in a grid.
    public static let card = (Corner.card * windowScale).rounded()

    /// The hero card, panels, sheets.
    public static let panel = (Corner.panel * windowScale).rounded()

    /// Fields and boxes of text.
    public static let field = (Corner.field * windowScale).rounded()

    /// Badges and small labels.
    public static let chip = (field / 2).rounded(.down)
}

extension SwiftUI.View {
    /// Text lines a set distance apart, as the references measure them. The same helper the
    /// television keeps in `app/Details.swift`; it is four lines of arithmetic about SwiftUI's
    /// line box and not a number, so each surface having one is not a second home for anything.
    func lines(_ size: CGFloat, _ height: CGFloat) -> some SwiftUI.View {
        lineSpacing(max(0, height - size * 1.21))
    }
}
