#if canImport(UIKit)
import Contract
import Heard
import Ports
import Stand
import SurfacesShared
import SwiftUI
import XCTest

@testable import SurfacesVision

/// Whether a viewer could actually look at the things on these screens.
///
/// This is the test the other two surfaces have no equivalent of, because neither has the problem.
/// On a television the app knows what is focused and draws an outline around it; on a phone a
/// finger lands where it is put. Here the system aims at whatever the eye is pointing at, the app
/// is never told where that is, and a target slightly too small does not fail visibly -- it fails
/// as the viewer looking at a thing and nothing happening. So "big enough" is not a detail of
/// styling, it is the only guarantee this surface can give, and it is read off the frames the
/// layout really produced rather than argued from the code.
///
/// Every size here comes from the accessibility tree, which is the only place a test can ask a
/// laid-out SwiftUI view what size it came out -- the reason `Tests/Heard` carries a frame at all.
/// That those frames are the control's and not its ink is itself something that had to be made
/// true: see the `accessibility` content shape in `app/Reach.swift`.
@MainActor
final class ReachTests: XCTestCase {
    /// Every control of a part, measured, with the count asserted first.
    ///
    /// The count comes first deliberately. A part that reached a reader as nothing at all would
    /// satisfy "every control is big enough" with no controls, which is exactly how a harness
    /// passes while proving nothing: a fix committed this week passed only because its harness
    /// reported a zero status-bar inset. So too few controls fails here, by name, before anything
    /// is measured.
    private func check(_ name: String, _ found: [Heard], least: Int) {
        let controls = found.filter(\.isButton)
        XCTAssertGreaterThanOrEqual(
            controls.count, least,
            "\(name) reached a reader with \(controls.count) controls, so measuring them proves nothing"
        )
        for control in controls {
            print("\(name) control:", control.label, control.frame.size)
            XCTAssertGreaterThanOrEqual(
                min(control.frame.width, control.frame.height), Reach.target,
                "\(name): \(control.label.replacingOccurrences(of: "\n", with: " ")) came out "
                    + "\(control.frame.size), under the minimum"
            )
        }
    }

    func testEveryControlOnTheNavBarIsBigEnoughToLookAt() {
        check("NavBar", heardAtLeast(aNavBar(), 5), least: 5)
    }

    func testEveryControlOnTheHeroIsBigEnoughToLookAt() {
        let shell = aBrowsingShell()
        check("Hero", heardWhenDrawn(aHero(shell), shell, pictures: 1), least: 2)
    }

    /// The title page's buttons and its three reactions. The reactions are where the small controls
    /// are: pictures with no words on them, which is the shape a target is most often too small in.
    func testEveryControlOnTheTitlePagePartsIsBigEnoughToLookAt() {
        check("TitleButtons", heardAtLeast(aTitleButtons(saved: false), 3), least: 3)
        check(
            "Reactions",
            heardAtLeast(Reactions(words: visionWords, chosen: .up, pick: { _ in }), 3),
            least: 3
        )
    }

    /// No two controls sit closer together than the guidance allows between their centres.
    ///
    /// Stated as its own test although the size rule implies it, because the implication holds only
    /// while both neighbours really are at least the minimum across -- which is the other test, and
    /// which a later change could break in one place without breaking the other. A gaze that cannot
    /// separate two targets lands on whichever the system guesses, and the viewer cannot correct it.
    func testNoTwoControlsAreCloserThanTheGuidanceAllows() {
        let parts: [(String, [Heard])] = [
            ("NavBar", heardAtLeast(aNavBar(), 5)),
            ("TitleButtons", heardAtLeast(aTitleButtons(saved: false), 3)),
            ("Reactions", heardAtLeast(Reactions(words: visionWords, chosen: nil, pick: { _ in }), 3)),
        ]
        for (name, found) in parts {
            let controls = found.filter(\.isButton)
            XCTAssertGreaterThanOrEqual(controls.count, 3, "\(name): too few controls to measure anything")
            for (index, one) in controls.enumerated() {
                for other in controls[(index + 1)...] {
                    let apart = hypot(one.frame.midX - other.frame.midX, one.frame.midY - other.frame.midY)
                    XCTAssertGreaterThanOrEqual(
                        apart, Reach.between,
                        "\(name): \(one.label) and \(other.label) have their centres \(apart) pt apart"
                    )
                }
            }
        }
    }

    /// A control given a word too short to fill the minimum on its own is grown until it does.
    ///
    /// The minimum is set on the layout and not on the hit area alone, which is the decision in
    /// `Reach.swift` worth holding: the system draws its highlight over the shape the view hands
    /// it, so an invisible margin would light up as part of the control. Asked of a single letter,
    /// which is the smallest a word can be, and of both ways a control can get its words.
    func testAControlTooSmallForItsWordsIsGrown() {
        for (name, found) in [
            ("gathered", heardAtLeast(Text("A").reachable(), 1)),
            ("labelled", heardAtLeast(Icon("icon_play", 18).reachable(says: "Play"), 1)),
        ] {
            XCTAssertEqual(found.count, 1, "\(name): one control should read as one thing")
            guard let box = found.first?.frame else { return XCTFail("\(name): no control at all") }
            XCTAssertGreaterThanOrEqual(box.width, Reach.target, "\(name): left \(box.width) pt wide")
            XCTAssertGreaterThanOrEqual(box.height, Reach.target, "\(name): left \(box.height) pt tall")
        }
    }

    /// A tile is at least twice the minimum tall, which is where ``Reach/tileFloor`` comes from,
    /// and is exactly the width the window rule says it should be.
    ///
    /// Asked of the tile the layout really produces at this surface's own window size rather than
    /// of the constant, so `Sizes.swift`'s arithmetic and `Rows.swift`'s drawing have to agree.
    func testATileClearsTwiceTheMinimumAndMatchesTheRule() {
        let shell = aBrowsingShell()
        let found = heardWhenDrawn(aTile(shell), shell, pictures: 1)
        guard let drawn = found.first(where: \.isButton) else {
            return XCTFail("no tile a reader can reach")
        }
        XCTAssertGreaterThanOrEqual(
            drawn.frame.height, 2 * Reach.target,
            "a tile came out \(drawn.frame.height) pt tall, under the floor its own width is derived from"
        )
        XCTAssertEqual(
            drawn.frame.width, tileWide(windowWide), accuracy: 1,
            "the tile drawn is \(drawn.frame.width) pt, not the width the window rule gives"
        )
        // Taller than its still, and that is the design rather than a slip: the title's name is
        // drawn under the picture at rest (`browse/Rows.swift`), so the control is the picture plus
        // that line. What has to match the window rule is the width; the height has to be at least
        // the still's, or no picture is being drawn at the size the rule asked for.
        XCTAssertGreaterThan(
            drawn.frame.height, tileHigh(windowWide),
            "the tile is exactly its picture's height, so it draws no name"
        )
    }

    /// Every control on the player clears the minimum too, including the bar.
    ///
    /// The bar is the one that would slip: it is a 6 pt capsule drawn inside a row of clocks, and a
    /// 6 pt control is one nobody can look at. What makes it reachable is the band around it, and
    /// this is what says the band is still there.
    func testEveryControlOnThePlayerIsBigEnoughToLookAt() {
        let shell = aWatchingShell()
        let bar = PositionBar(
            words: visionWords, atMs: shell.seen.positionMs, lengthMs: shell.seen.lengthMs,
            playing: true, flip: {}
        )
        check("PositionBar", heardAtLeast(bar, 1), least: 1)
    }
}
#endif
