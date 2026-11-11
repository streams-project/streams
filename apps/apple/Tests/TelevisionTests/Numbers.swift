#if canImport(UIKit)
import SwiftUI
import XCTest

@testable import SurfacesTV

/// Numbers that were derived from a measurement rather than measured. Each test here states the
/// derivation and checks the number against it, so a number that was transcribed by hand cannot
/// quietly drift from the reason it has that value. Pinning a derived number to itself would prove
/// nothing.
final class NumbersTests: XCTestCase {
    /// The read of the rest of a page starts `near` rows from the end.
    ///
    /// Derivation (`browse/Browse.swift` and the band in
    /// `notes/netflix-reference/tv/00-netflix-on-the-tv-measured.md` section 6b, "next row title
    /// 948-978, next row tiles from 998 with 82 px visible"): a row screen always shows exactly one
    /// row under the focused one, and `Browse.swift` draws that row from `focus.row + 1`. The window
    /// has to cover the focused row and every row the viewer can already see under it, so it is
    /// one plus the number of rows drawn below the focus.
    func testTheReadWindowCoversTheRowsTheViewerCanAlreadySee() {
        XCTAssertEqual(rowsBelowFocus, 1, "the surface draws one row under the focus")
        XCTAssertEqual(near, rowsBelowFocus + 1, "the window is the focused row plus the rows under it")
    }

    /// The treatment box on a focused row card.
    ///
    /// Derivation (`browse/Rows.swift`): no measurement gives a size for the treatment on a focused
    /// still, so the box is this surface's own hero box scaled down by the width the card has against
    /// the width the hero has, in one ratio both ways so the box the treatment is fitted into keeps
    /// its shape. A box scaled by one ratio has the shape of the box it came from, which a pair of
    /// rounded numbers does not.
    func testTheTreatmentBoxKeepsTheHeroBoxShape() {
        XCTAssertEqual(openMarkScale, openWide / heroWide, "the ratio is the card's width against the hero's")
        XCTAssertEqual(openMarkWide, heroMark.width * openMarkScale, accuracy: 0.001)
        XCTAssertEqual(openMarkHigh, heroMark.height * openMarkScale, accuracy: 0.001)
        XCTAssertEqual(
            openMarkWide / openMarkHigh,
            heroMark.width / heroMark.height,
            accuracy: 0.001,
            "one ratio both ways means the scaled box has the hero box's shape"
        )
    }

    /// The hero's own width is the page less its two margins, not a number of its own.
    func testTheHeroIsThePageLessItsMargins() {
        XCTAssertEqual(heroWide, pageWide - 2 * pageEdge)
    }
}
#endif
