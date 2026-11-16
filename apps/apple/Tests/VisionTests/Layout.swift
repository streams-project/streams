#if canImport(UIKit)
import CoreGraphics
import XCTest

@testable import SurfacesVision

/// The rule that produces a row in a window the viewer resizes.
///
/// Held as a rule and never as a table. The web reference's own warning is the reason: "What a
/// surface has to match is the derivation below, not a rounded copy of one row of the table." A
/// test that asserted this surface answers 5 across at 1280 would pass on a build that answered 5
/// across at every width, and the one thing a window has to survive is being resized.
///
/// So every test below is a property, checked across the whole range of widths a window can
/// plausibly take, rather than a handful of points someone chose.
final class LayoutTests: XCTestCase {
    /// Every integer width a window might be dragged to. One is the unit because a window's width
    /// arrives in points from a drag, and a step function is at its most interesting at the step.
    private let widths = stride(from: CGFloat(480), through: 3000, by: 1)

    /// The tiles and the gutters between them fill the content box exactly, at every width.
    ///
    /// This is the one rule reused from `reference/netflix-on-the-web.md` and the only thing in
    /// this surface's geometry taken from a measurement of anything. A tile that is a rounded
    /// constant instead of this division leaves a sliver at the end of every row, which looks like
    /// nothing at all until the window is dragged.
    func testTilesAndGuttersFillTheContentBoxExactly() {
        for width in widths {
            let count = CGFloat(across(width))
            let filled = count * tileWide(width) + (count - 1) * gutter + 2 * pageEdge(width)
            XCTAssertEqual(
                filled, width, accuracy: 0.001,
                "at \(width) pt the row fills \(filled) pt, so it does not fill the window exactly"
            )
        }
    }

    /// A tile is never too small to look at.
    ///
    /// The floor is `Reach.tileFloor`, which is where the step function gets its steps: the count
    /// across goes up only while the tile that results still clears it. The one exception is stated
    /// rather than excused -- a window too narrow for a single floor-width tile and its margins
    /// draws one tile below the floor, because the alternative is drawing nothing.
    func testATileIsNeverTooSmallToLookAt() {
        for width in widths where across(width) > 1 {
            XCTAssertGreaterThanOrEqual(
                tileWide(width), Reach.tileFloor,
                "at \(width) pt the tile is \(tileWide(width)) pt, under the floor"
            )
        }
    }

    /// And never wider than the bound derived in `Sizes.swift`, wherever two or more fit.
    ///
    /// The bound is arithmetic, not taste: if the next count up would fall below the floor, the
    /// current one cannot exceed `1.5 * floor + gutter / 2`. Asserting it here is what turns that
    /// paragraph from a claim into something a change has to keep true.
    func testATileIsNeverWiderThanTheDerivedBound() {
        for width in widths where across(width) > 1 {
            XCTAssertLessThanOrEqual(
                tileWide(width), tileWidest,
                "at \(width) pt the tile is \(tileWide(width)) pt, over the bound the derivation gives"
            )
        }
    }

    /// Both ends of that band are actually reached, so the bounds above are describing this layout
    /// and not merely failing to constrain it.
    ///
    /// Without this, a build whose tile was always 240 pt would pass both bounds and have stopped
    /// responding to the window entirely.
    func testTheBandIsReachedAtBothEnds() {
        let tiles = widths.filter { across($0) > 1 }.map { tileWide($0) }
        XCTAssertEqual(tiles.min() ?? 0, Reach.tileFloor, accuracy: 0.5, "the floor is never reached")
        // One point of slack, and it is measured rather than generous: the widest tile any integer
        // width produces is 327.50 against a bound of 328.00, because the bound is the limit of a
        // strict inequality and no window width lands exactly on it.
        XCTAssertEqual(tiles.max() ?? 0, tileWidest, accuracy: 1, "the derived bound is never reached")
    }

    /// A wider window never shows fewer tiles.
    ///
    /// Not implied by anything above: the page margin steps up at 900 and 1400, and a margin that
    /// stepped too far could take a tile away as the window grew. That would be a window that shows
    /// less of the catalogue the larger it is made, which is the kind of thing nobody looks for.
    func testAWiderWindowNeverShowsFewerTiles() {
        var last = across(480)
        for width in widths {
            let now = across(width)
            XCTAssertGreaterThanOrEqual(
                now, last,
                "the window grew to \(width) pt and the row lost a tile, from \(last) across to \(now)"
            )
            last = now
        }
    }

    /// A tile is 16:9, the one tile shape both reference files measure independently.
    func testATileIsSixteenByNine() {
        for width in widths {
            XCTAssertEqual(
                tileWide(width) / tileHigh(width), 16.0 / 9.0, accuracy: 0.02,
                "at \(width) pt the tile is \(tileWide(width)) x \(tileHigh(width)), which is not 16:9"
            )
        }
    }

    /// The window this surface opens at shows a row of tiles rather than one or two enormous ones.
    ///
    /// The one place a specific width is named, and it is named because it is this surface's own
    /// choice rather than a measurement: if `windowWide` is changed, this is where the consequence
    /// of changing it shows up.
    func testTheWindowOpensOnAProperRow() {
        XCTAssertEqual(across(windowWide), 5, "the window this surface opens at no longer shows five tiles")
        XCTAssertGreaterThanOrEqual(tileWide(windowWide), Reach.tileFloor)
    }
}
#endif
