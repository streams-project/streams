#if canImport(UIKit)
import Contract
import Heard
import Ports
import Stand
import SurfacesShared
import SwiftUI
import XCTest

@testable import SurfacesVision

/// What this surface says to a person who cannot see it.
///
/// Here from the first commit rather than after one. Four defects this week were screens whose
/// controls said nothing, all found after the fact, and the two families that already have this
/// test (`TelevisionTests/Speech.swift`, `PhoneTests/Speech.swift`) were both written over screens
/// that had been silent for weeks. There was no reason for a third surface to repeat that.
///
/// **Why these read parts and not whole screens**, which is not a convenience. Measured on the iOS
/// simulator on 2026-11-14: SwiftUI builds no accessibility nodes at all for the contents of a
/// `ScrollView` hosted in an offscreen window. `heard(ScrollView { Text("inside") })` returns the
/// empty list at every size tried from 390 x 800 to 1280 x 820 and at settles up to 2 s, while
/// `heard(VStack { Text("outside") })` returns the text at once. The tree under the hosting view
/// says it is not the walker's doing: the hosting view's one element is the `HostingScrollView`
/// itself, which answers nil to `accessibilityElements` and 0 to `accessibilityElementCount()`, so
/// there is nothing under it to find whichever way a container is asked. `Tests/Heard` cannot see
/// what was never built.
///
/// So `Browse` and `TitlePage`, which scroll, reach this harness as nothing at all, and every part
/// they are built from is read here on its own instead -- which is the shape the television
/// family's own speech tests already have. ``testAWholeScrollingScreenStillCannotBeRead`` is a
/// tripwire on that limitation: the day it fails, these can go back to reading whole screens.
@MainActor
final class SpeechTests: XCTestCase {
    /// Every control on the bar says what it is, and none of them reads as a picture's file name.
    ///
    /// Two halves, two different defects with one symptom. A silent element is one nobody gave
    /// words to; an element reading `icon_search` is one UIKit gave the name of its own asset to,
    /// which is not words and is in no language. `Tests/Heard` measured this family doing the
    /// second everywhere before anything here was labelled.
    func testNothingOnTheNavBarIsSilent() {
        let found = heardAtLeast(aNavBar(), 5)
        print("NavBar:", found.map { "\($0.label)|\($0.kinds)" })
        XCTAssertFalse(found.isEmpty, "the bar reached a screen reader as nothing at all")
        XCTAssertEqual(silent(found), [], "something on the bar is reached and says nothing")
        XCTAssertEqual(assetNames(found), [], "something on the bar reads out a picture's file name")
        XCTAssertEqual(
            words(found), ["Home", "Series", "Films", visionWords["nav.search"]!, "Ibrahim"],
            "the bar is not read as its tabs, then search, then who is watching"
        )
    }

    /// The hero names the title as a heading and offers two things to do about it.
    func testNothingOnTheHeroIsSilent() {
        let shell = aBrowsingShell()
        let found = heardWhenDrawn(aHero(shell), shell, pictures: 1)
        print("Hero:", found.map { "\($0.label)|\($0.kinds)" })
        XCTAssertFalse(found.isEmpty, "the hero reached a screen reader as nothing at all")
        XCTAssertEqual(silent(found), [], "something on the hero is reached and says nothing")
        XCTAssertEqual(assetNames(found), [], "something on the hero reads out a picture's file name")
        XCTAssertEqual(
            found.filter { $0.kinds.contains("header") }.map(\.label), ["Sunfall"],
            "the hero does not name its title as a heading"
        )
        XCTAssertEqual(
            found.filter(\.isButton).map(\.label),
            [visionWords["common.play"]!, visionWords["common.more_info"]!],
            "the hero's two buttons are not read as the words the strings give"
        )
    }

    /// A tile says the title's name, then what is drawn on it.
    ///
    /// The badge and the progress bar are drawn inside the tile's own control, so neither is a stop
    /// of its own; what they mean has to arrive as part of the tile's words or not at all.
    ///
    /// The expected words are built with `fill` rather than typed out, because `fill` wraps the
    /// value it inserts in the isolate characters U+2068 and U+2069, which is what keeps a number
    /// in its own direction inside a line of the other one. Typing the result out by hand drops
    /// them, and the test would then assert that this surface breaks bidirectional text.
    func testATileSaysItsNameAndWhatIsDrawnOnIt() {
        XCTAssertEqual(
            tileWords(tile("Salt Road", "tv:107", percent: 72), visionWords),
            "Salt Road\n" + visionWords["say.watched_percent"]!.fill("percent", 72),
            "a started title does not say how far through it the viewer is, in the words the strings give"
        )
        XCTAssertEqual(
            tileWords(tile("Glasshouse", "movie:105", badges: [Contract.Badge(kind: .ranked, rank: 1)]), visionWords),
            "Glasshouse\n" + visionWords["badge.ranked"]!.fill("rank", 1),
            "a ranked title does not say its place, in the words the strings give"
        )
        XCTAssertEqual(
            tileWords(tile("Nightbus", "movie:104"), visionWords), "Nightbus",
            "a plain tile says more than its name"
        )
    }

    /// A tile drawn for real is one thing a reader stops on, not four.
    ///
    /// The picture, the badge and the bar are all inside it and each would be a stop of its own if
    /// it were not marked. One tile, one stop, named by the title.
    func testATileIsOneStop() {
        let shell = aBrowsingShell()
        let found = heardWhenDrawn(aTile(shell, percent: 72), shell, pictures: 1)
        print("TileCard:", found.map { "\($0.label)|\($0.kinds)" })
        XCTAssertEqual(silent(found), [], "a tile is reached and says nothing")
        XCTAssertEqual(assetNames(found), [], "a tile reads out a picture's file name")
        XCTAssertEqual(
            found.filter(\.isButton).map(\.label),
            ["Salt Road\n" + visionWords["say.watched_percent"]!.fill("percent", 72)],
            "a tile is not read as one control carrying the title's words"
        )
        XCTAssertEqual(found.count, 1, "a tile reached a reader as more than one thing")
    }

    /// A row's title is a heading, so a reader can jump between rows instead of walking every tile
    /// of every one of them.
    ///
    /// The title is the one part of a row outside its scroller, which is why this can be asked of a
    /// real `Row` while the tiles in it have to be asked of a `TileCard`.
    func testARowTitleIsAHeading() {
        let shell = aBrowsingShell()
        let row = Row(
            pictures: shell.shown,
            section: section("picks", "Picked for You Today", .row, [tile("Nightbus", "movie:104")]),
            words: visionWords, width: windowWide, open: { _ in }
        )
        let found = heardAtLeast(row, 1)
        XCTAssertEqual(
            found.filter { $0.kinds.contains("header") }.map(\.label), ["Picked for You Today"],
            "a row's title is not a heading a reader can jump to"
        )
    }

    /// The buttons on a title page say what they do, and the one that changes meaning says the
    /// other thing when it means it.
    func testTheTitleButtonsSayWhatTheyDo() {
        let saved = heardAtLeast(aTitleButtons(saved: true), 3)
        let unsaved = heardAtLeast(aTitleButtons(saved: false), 3)
        print("TitleButtons:", unsaved.map { "\($0.label)|\($0.kinds)" })
        XCTAssertEqual(silent(unsaved), [], "a button on a title page is reached and says nothing")
        XCTAssertEqual(assetNames(unsaved), [], "a button on a title page reads out a picture's file name")
        XCTAssertEqual(
            words(unsaved),
            [
                visionWords["title.play_episode"]!.fill("n", 1),
                visionWords["title.episodes_more"]!,
                visionWords["list.add"]!,
            ],
            "the title page's buttons are not read as the words the strings give"
        )
        XCTAssertEqual(
            words(saved).last, visionWords["list.added"]!,
            "a title already on the list still offers to add it"
        )
    }

    /// The three reactions are three controls with three different words, and the chosen one says
    /// so apart from its name.
    ///
    /// Apart, because a reader says a label and a value in different places, and a state folded
    /// into the label is a state said twice. Which one is chosen is drawn as a filled picture,
    /// which is exactly the kind of thing a person listening cannot see.
    func testOnlyTheChosenReactionSaysItIsChosen() {
        let none = heardAtLeast(Reactions(words: visionWords, chosen: nil, pick: { _ in }), 3)
        let liked = heardAtLeast(Reactions(words: visionWords, chosen: .up, pick: { _ in }), 3)
        print("Reactions:", liked.map { "\($0.label)|\($0.kinds)|\($0.value)" })
        XCTAssertEqual(silent(none), [], "a reaction is reached and says nothing")
        XCTAssertEqual(assetNames(none), [], "a reaction reads out a picture's file name")
        XCTAssertEqual(
            words(none),
            [visionWords["player.down"]!, visionWords["player.up"]!, visionWords["player.love"]!],
            "the reactions are not read as the words the strings give"
        )
        XCTAssertEqual(
            none.filter { !$0.value.isEmpty }.map(\.label), [],
            "nothing was chosen and something says it was"
        )
        XCTAssertEqual(
            liked.filter { !$0.value.isEmpty }.map(\.label), [visionWords["player.up"]!],
            "the chosen reaction is not the one that says it is chosen"
        )
        XCTAssertEqual(
            saying(liked, visionWords["player.up"]!)?.value, visionWords["say.chosen"]!,
            "the chosen reaction does not say so in the words the strings give"
        )
    }

    /// Nothing claims to be selected, anywhere.
    ///
    /// This is the surface's own rule and not a tidiness check. The system moves focus by where the
    /// viewer looks and never tells the app, so the app has no idea what is selected; a control
    /// carrying the `selected` trait here would be announcing a focus it invented, and a person
    /// listening would be told they are somewhere they are not. `SurfacesShared`'s `says(_:on:)`
    /// and `saysItsWords(on:)` exist for the television and are never called in this module; this
    /// is what holds that.
    func testNothingClaimsToBeSelected() {
        let shell = aBrowsingShell()
        let parts: [(String, [Heard])] = [
            ("NavBar", heardAtLeast(aNavBar(), 5)),
            ("Hero", heardWhenDrawn(aHero(shell), shell, pictures: 1)),
            ("TitleButtons", heardAtLeast(aTitleButtons(saved: false), 3)),
            ("Reactions", heardAtLeast(Reactions(words: visionWords, chosen: .up, pick: { _ in }), 3)),
        ]
        for (name, found) in parts {
            XCTAssertFalse(found.isEmpty, "\(name) reached a screen reader as nothing at all")
            XCTAssertEqual(
                found.filter(\.isSelected).map(\.label), [],
                "\(name) says something is selected, on a surface that cannot know what is"
            )
        }
    }

    /// A tripwire on the limitation this whole file is shaped around, so it cannot be forgotten.
    ///
    /// Two halves, and the first one changed under this test on 2026-11-14, which is what a
    /// tripwire is for. Moving the navigation out of the scrolling content and into an ornament
    /// (`NavBar`) put it outside the scroller, and it is therefore now readable: the page that used
    /// to reach a reader as nothing at all now reaches it as five controls. That is the chrome
    /// decision proving itself from an unexpected direction -- chrome that costs the catalogue
    /// nothing also costs the tests nothing.
    ///
    /// What is still unreadable is everything **inside** the scroller: the hero, the rows and every
    /// tile in them. So the assertion is now the sharper one -- the chrome is read, the content is
    /// not -- and the day the second half changes, the part-by-part reads above can become
    /// whole-screen reads.
    ///
    /// The last assertion is what stops this passing for the wrong reason: a screen that never drew
    /// at all would also show no tiles, and would prove nothing about scrolling.
    func testOnlyTheChromeOfAScrollingScreenCanBeRead() {
        let shell = aBrowsingShell()
        let found = heard(Browse(shell: shell, seen: shell.seen), size: aWindow, settle: 1.0)
        print("Browse:", found.map(\.label))
        XCTAssertGreaterThan(
            shell.shown.asked.count, 0,
            "the screen did not draw at all, so this proves nothing about scrolling"
        )
        XCTAssertEqual(
            words(found),
            ["Home", "Series", "Films", visionWords["nav.search"]!, "Ibrahim"],
            "the page's chrome is not what is readable on a scrolling screen any more"
        )
        for inside in ["Sunfall", "Harbour Light", "Picked for You Today"] {
            XCTAssertFalse(
                words(found).contains { $0.contains(inside) },
                "\(inside) is inside the scroller and can now be read: the part-by-part reads "
                    + "above can become whole-screen reads"
            )
        }
    }

    /// A tile names its title with the name drawn on it, not only in what a reader is told.
    ///
    /// The rule this holds is in `browse/Rows.swift`: nothing on this surface is revealed on hover.
    /// A tile that carried its name only in its accessibility label, or only once the system lit
    /// it, would be a tile that tells a sighted person scanning the row nothing, and would charge
    /// somebody using Dwell Control a dwell to read a label. So the name has to be a drawn `Text`
    /// that is there at rest.
    ///
    /// Read by gathering: the tile is one control, so what a reader hears is assembled rather than
    /// listed, and the drawn name is proved by taking the words out of the tile's own label and
    /// asking whether the name is still in the tree.
    func testATileDrawsItsTitlesNameAtRest() {
        let shell = aBrowsingShell()
        let bare = TileCard(
            pictures: shell.shown, tile: tile("Salt Road", "tv:107"),
            words: visionWords, box: tileBox(windowWide), open: { _ in }
        )
        let found = heardWhenDrawn(bare, shell, pictures: 1)
        guard let drawn = found.first(where: \.isButton) else {
            return XCTFail("no tile a reader can reach")
        }
        XCTAssertTrue(
            drawn.label.contains("Salt Road"),
            "a tile does not carry its title's name"
        )
        // The name is drawn, so the tile is taller than its 16:9 still: that extra band is the
        // text, and a tile whose box equalled the still's would be one that draws no name.
        XCTAssertGreaterThan(
            drawn.frame.height, tileHigh(windowWide),
            "the tile is exactly its picture's height, so no name is drawn under it"
        )
    }

    /// The player says what each press does, and says where the film is.
    ///
    /// The bar is the one part of a player a person listening cannot read any other way: the clocks
    /// either side are text, and the bar between them holds all of the information and draws it as
    /// a shape. So it is one control, named by what pressing it does and valued by where the film
    /// is, with both sets of words from `strings/`.
    func testThePlayerSaysWhatItsPressesDoAndWhereTheFilmIs() {
        let shell = aWatchingShell()
        let bar = PositionBar(
            words: visionWords, atMs: shell.seen.positionMs, lengthMs: shell.seen.lengthMs,
            playing: true, flip: {}
        )
        let playing = heardAtLeast(bar, 1)
        print("PositionBar:", playing.map { "\($0.label)|\($0.value)" })
        XCTAssertEqual(silent(playing), [], "something on the player's bar says nothing")
        XCTAssertEqual(assetNames(playing), [], "something on the player's bar reads a file name")
        XCTAssertEqual(
            words(playing), [visionWords["say.pause"]!],
            "a playing bar does not offer to pause, in the words the strings give"
        )
        XCTAssertEqual(
            playing.first?.value,
            visionWords["say.position"]!.fill("at", "30:30").fill("length", "1:44:00"),
            "the bar does not say how far through the film it is, in the words the strings give"
        )
        let paused = heardAtLeast(
            PositionBar(
                words: visionWords, atMs: shell.seen.positionMs, lengthMs: shell.seen.lengthMs,
                playing: false, flip: {}
            ),
            1
        )
        XCTAssertEqual(
            words(paused), [visionWords["common.play"]!],
            "a paused bar does not offer to play"
        )
    }
}
#endif
