#if canImport(UIKit)
import Contract
import Heard
import Ports
import Stand
import SurfacesShared
import SwiftUI
import XCTest

@testable import SurfacesPhone

/// What the phone surface says to a person who cannot see it. Each test draws one real part of the
/// surface at phone size and reads back the tree VoiceOver would walk, and each is named after the
/// defect it keeps shut.
///
/// The phone surface does use system buttons, so SwiftUI derives a label for most of them from the
/// text inside. Two gaps are left by that, and both were measured on iPhone 17 (iOS 26.5) on
/// 2026-11-12 before any of this was labelled:
///  * a control that carries a picture and no text is read as the name of the picture's own asset
///    file - the brand's mark came out as "mark" - which is not words and is in no language;
///  * a control that carries a picture and the wrong text takes the wrong text: the bell was read
///    as the bare numeral "3", the count drawn on it, with the bell itself dropped.
///
/// What nothing here can read: anything inside a SwiftUI `ScrollView`. Measured the same day, a
/// test process with no window scene gets nothing from a scroll view's contents, lazy container or
/// not, so the chips, the tiles and the rows are hosted as the parts they are instead of through
/// the page that scrolls them. The page itself is still read, for the header and the bar that are
/// pinned outside the scroll.
@MainActor
final class SpeechTests: XCTestCase {
    private let phone = CGSize(width: 393, height: 852)
    /// A phone's width at a window tall enough to lay out a whole scrolling page. SwiftUI builds
    /// accessibility elements only for what is laid out inside the window, so a control below the
    /// fold is reached at no settle time at all (measured on iPhone 17, 2026-11-12). A test that
    /// has to hear the foot of a long page says so by hosting it at this size.
    private let tallPhone = CGSize(width: 393, height: 2400)

    private func bar(_ back: Bool, bell: Bool, unread: Int = 0) -> TopBar {
        TopBar(
            server: "",
            name: "Home",
            back: back ? {} : nil,
            bell: bell ? {} : nil,
            unreadCount: unread,
            words: phoneWords
        )
    }

    /// Every word heard came from `strings/` or from the catalogue, and none of it was written in
    /// the code. This is the rule `strings/` exists for, asserted rather than trusted: the label of
    /// every element is looked for among the values the surface was handed.
    private func notFromTheCode(_ found: [Heard], _ words: [String: String], names: [String] = []) -> [String] {
        // A label may be one of the given words, a name out of the catalogue, a `{...}` word with
        // its place filled, or two of those read one after the other.
        let given = Set(words.values) .union(names)
        let patterns = words.values.filter { $0.contains("{") }
        return found.flatMap { heard -> [String] in
            heard.label.split(separator: "\n").map(String.init)
        }.filter { part in
            if part.isEmpty || given.contains(part) { return false }
            if names.contains(where: { part.contains($0) }) { return false }
            // A filled word: the pattern's fixed text on both sides of its `{...}` is still there.
            return !patterns.contains { pattern in
                let fixed = pattern
                    .replacingOccurrences(of: "\\{[a-z_]+\\}", with: "\u{0001}", options: .regularExpression)
                    .split(separator: "\u{0001}")
                    .map(String.init)
                    .filter { !$0.isEmpty }
                return !fixed.isEmpty && fixed.allSatisfy { part.contains($0) }
            }
        }
    }

    // MARK: - The top bar

    /// The brand's mark is decoration: the page's name is beside it and says the same thing. Read
    /// as it stood, it was the name of its own asset file inside a button nobody can press.
    func testTheBrandMarkIsNotReadAsItsFileName() {
        let found = heard(bar(false, bell: false), size: phone)
        XCTAssertEqual(words(found), ["Home"], "the mark is decoration beside the page's name")
        XCTAssertFalse(words(found).contains("mark"), "the mark is read as the name of its asset file")
    }

    /// The back arrow carries no text, so it is the one control in the bar a screen reader cannot
    /// name unless someone names it, and its words come from `common.back`.
    func testTheBackArrowIsNamedFromTheStrings() {
        let found = heard(bar(true, bell: false), size: phone)
        XCTAssertEqual(assetNames(found), [], "a button is read as the name of its picture file")
        XCTAssertEqual(silent(found), [], "a button has nothing to read")
        XCTAssertEqual(
            found.first { $0.isButton }?.label,
            phoneWords["common.back"],
            "the arrow says what the strings call going back"
        )
        XCTAssertEqual(notFromTheCode(found, phoneWords, names: ["Home"]), [])
    }

    /// What the bell says has to carry how many are waiting, because the number is drawn on it -
    /// and it has to be the words for that, not the numeral on its own.
    func testTheBellSaysHowManyAreWaitingAndNotJustTheNumeral() {
        let quiet = heard(bar(false, bell: true, unread: 0), size: phone)
        let busy = heard(bar(false, bell: true, unread: 3), size: phone)
        let quietBell = quiet.first { $0.isButton }?.label
        let busyBell = busy.first { $0.isButton }?.label
        XCTAssertEqual(quietBell, phoneWords["notifications.title"], "a quiet bell says what it opens")
        XCTAssertEqual(
            busyBell,
            phoneWords["say.notifications_unread"]?.fill("n", 3),
            "a waiting bell says the count in words, from the strings"
        )
        XCTAssertNotEqual(busyBell, "3", "the bell is read as the numeral drawn on it")
        XCTAssertNotEqual(busyBell, quietBell, "the count drawn on the bell is not spoken")
    }

    // MARK: - The foot bar and the chips

    /// Every place on the foot bar reads as its own word, once, and the chosen one says it is
    /// chosen - because the plate under it is the only other thing that says so.
    func testTheFootBarReadsItsWordsOnceAndSaysWhichIsChosen() {
        // The words are taken from `phoneWords`, which holds the real values out of `strings/`, and
        // `nav.mine` is taken unfilled - "My {brand}". No source file in this project may spell the
        // product's name, which lives in `brand.json` alone (`launcher/tests/one_name.rs`), and
        // this test is about whether a place speaks rather than about which word it speaks.
        let items = [
            BarItem(icon: "icon_home", word: phoneWords["nav.home"]!, here: true, go: {}),
            BarItem(icon: "icon_search", word: phoneWords["nav.search"]!, here: false, go: {}),
            BarItem(
                icon: "icon_user", word: phoneWords["nav.mine"]!, here: false,
                face: Profile(hasPin: false, id: "p1", kids: false, main: true, name: "Ibrahim"),
                server: "", go: {}
            ),
        ]
        let found = heard(BottomBar(pictures: ToldPictures(), items: items), size: phone)
        XCTAssertEqual(words(found), items.map(\.word), "each place reads as one word, in order")
        XCTAssertEqual(
            found.filter { $0.kinds.contains("selected") }.count,
            items.filter(\.here).count,
            "the page in view is the chosen one, and it is the only one"
        )
        XCTAssertFalse(
            words(found).contains { $0.hasPrefix("I,") },
            "the profile's first letter is read before the word"
        )
    }

    /// A chip with a caret must not read the caret's file name after its word. The chip is hosted
    /// on its own because nothing inside the row's scroll view can be read.
    func testTheChipReadsItsWordAndNotItsCaret() {
        let shape = UnevenRoundedRectangle(
            topLeadingRadius: Corner.chip,
            bottomLeadingRadius: Corner.chip,
            bottomTrailingRadius: Corner.chipEnd,
            topTrailingRadius: Corner.chipEnd
        )
        let found = heard(
            ChipButton(
                chip: Chip(word: "Categories", chosen: false, drops: true, go: {}),
                shape: shape, high: 49, outline: false
            ),
            size: phone
        )
        XCTAssertEqual(words(found), ["Categories"], "a chip is its word and nothing else")
        XCTAssertEqual(assetNames(found), [], "a chip reads the caret's file name")
    }

    // MARK: - Home

    /// Every control the page pins outside its scroll view speaks, in words that came from the
    /// strings and not from the code.
    func testEveryControlOnHomeSpeaks() {
        let (shell, seen) = homeSeen()
        let found = heard(Home(shell: shell, seen: seen), size: phone, settle: 0.6)
        XCTAssertFalse(found.isEmpty, "nothing on Home was read at all")
        XCTAssertEqual(silent(found), [], "something on Home is reached and has nothing to read")
        XCTAssertEqual(assetNames(found), [], "something on Home is read as a picture's file name")
        XCTAssertEqual(
            found.filter { !$0.label.isEmpty }.count,
            found.count,
            "every element reached on Home carries words"
        )
        let names = (seen.screen?.sections ?? []).flatMap { ($0.tiles ?? []).map(\.name) }
        XCTAssertEqual(
            notFromTheCode(found, seen.words, names: names + ["Home"]),
            [],
            "Home says something that is in neither the strings nor the catalogue"
        )
    }

    /// The gate is the first screen of the app, and a locked profile has to say that it is locked:
    /// the padlock drawn under its name is the only other thing that says so.
    func testTheProfileGateSpeaks() {
        let (shell, seen) = homeSeen()
        seen.profiles = [
            Profile(hasPin: false, id: "p1", kids: false, main: true, name: "Ibrahim"),
            Profile(hasPin: true, id: "p2", kids: true, main: false, name: "Noor"),
        ]
        let found = heard(GateView(shell: shell, seen: seen), size: phone, settle: 0.6)
        XCTAssertEqual(silent(found), [], "something on the gate is reached and says nothing")
        XCTAssertEqual(assetNames(found), [], "something on the gate is read as a picture's file name")
        XCTAssertTrue(
            words(found).contains("Ibrahim"),
            "an unlocked profile is read as its own name"
        )
        XCTAssertTrue(
            words(found).contains(seen.words["say.profile_locked"]!.fill("name", "Noor")),
            "a locked profile says it is locked, in the words the strings give for it"
        )
        XCTAssertEqual(notFromTheCode(found, seen.words, names: ["Ibrahim", "Noor"]), [])
    }

    /// The sheet that asks for a PIN is a row of four boxes drawn over a field one point across.
    /// The boxes are a picture of what the field holds; the field is the control.
    func testThePinSheetAsksInWords() {
        let (shell, seen) = homeSeen()
        let person = Profile(hasPin: true, id: "p2", kids: false, main: false, name: "Noor")
        let found = heard(
            PinSheet(profile: person, shell: shell, seen: seen, onDismiss: {}),
            size: phone, settle: 0.6
        )
        XCTAssertEqual(silent(found), [], "something on the PIN sheet says nothing")
        XCTAssertTrue(
            words(found).contains(seen.words["say.pin"]!),
            "the field that takes the PIN says what it is for"
        )
        XCTAssertTrue(
            words(found).contains(seen.words["say.close"]!),
            "the control that closes the sheet is named"
        )
    }

    // MARK: - The billboard and the rows

    /// The card names the title once, and the two buttons inside it keep their own words.
    func testTheBillboardNamesTheTitleAndKeepsItsTwoButtons() {
        let (shell, seen) = homeSeen()
        let lead = (seen.screen?.sections ?? []).first { $0.kind == .hero }!.tiles!.first!
        let found = heard(
            Billboard(
                shell: shell, tile: lead, words: seen.words, saved: false,
                play: {}, list: {}, open: {}, tint: { _ in }
            ),
            size: phone, settle: 0.6
        )
        XCTAssertEqual(silent(found), [], "something on the billboard says nothing")
        XCTAssertEqual(assetNames(found), [], "the billboard reads a picture's file name")
        XCTAssertTrue(words(found).contains(lead.name), "the card is read as the title's name")
        XCTAssertTrue(words(found).contains(seen.words["common.play"]!), "the first button keeps its word")
        XCTAssertTrue(words(found).contains(seen.words["list.add"]!), "the second button keeps its word")
    }

    /// A tile is a picture with no words on it, and the title's name is what a reader has to hear.
    func testATileIsReadAsItsTitleAndNotAsItsPicture() {
        let (_, seen) = homeSeen()
        let tile = (seen.screen?.sections ?? []).first { $0.kind == .row }!.tiles!.first!
        let found = heard(
            TileCard(
                pictures: ToldPictures(), tile: tile, wide: tileWide, high: tileHigh, rank: nil,
                percent: nil, words: seen.words, open: {}
            ),
            size: phone
        )
        XCTAssertEqual(words(found), [tile.name], "a tile is the title's name and nothing else")
        XCTAssertEqual(assetNames(found), [])
    }

    /// A ranked tile says its place as well as its name, and the numeral beside it is not read -
    /// it is drawn as nine offset copies of the same figure and one more over them, so a reader
    /// that stopped on it would say the number ten times.
    func testARankedTileSaysItsPlaceAndTheNumeralIsNotRead() {
        let (_, seen) = homeSeen()
        let row = (seen.screen?.sections ?? []).first { $0.kind == .ranked }!
        let tile = row.tiles![1]
        let rank = tile.badges!.first!.rank!
        let found = heard(
            RankedTileCard(pictures: ToldPictures(), rank: rank, tile: tile, words: seen.words, open: {}),
            size: phone
        )
        XCTAssertEqual(
            words(found),
            [seen.words["say.ranked"]!.fill("rank", rank).fill("title", tile.name)],
            "a ranked tile is read once, as its place and its name, in the words the strings give"
        )
    }

    /// A continue tile draws how far through the title the viewer is as a bar across the picture,
    /// which is the one thing on it a person listening cannot see at all.
    func testAContinueTileSaysHowFarThroughItIs() {
        let (_, seen) = homeSeen()
        let tile = (seen.screen?.sections ?? []).first { $0.kind == .continue }!.tiles!.first!
        let found = heard(
            ContinueCard(
                pictures: ToldPictures(), tile: tile, words: seen.words, open: {}, openInfo: {},
                onRemove: {}
            ),
            size: phone
        )
        XCTAssertEqual(silent(found), [], "a control on a continue tile says nothing")
        XCTAssertEqual(assetNames(found), [], "a continue tile reads a picture's file name")
        XCTAssertTrue(
            words(found).contains(
                seen.words["say.watched"]!.fill("title", tile.name).fill("percent", tile.percent!)
            ),
            "the picture says the title and how much of it has been watched"
        )
        XCTAssertTrue(words(found).contains(seen.words["common.more_info"] ?? ""), "the info control is named")
        XCTAssertTrue(words(found).contains(seen.words["say.more_options"]!), "the menu control is named")
    }

    /// A row whose name opens the whole row is a control, and the chevron beside the name is the
    /// only thing on screen that says so. A row that opens nothing keeps its own name.
    func testARowThatOpensSaysSoAndOneThatDoesNotOnlyNamesItself() {
        let (_, seen) = homeSeen()
        let short = (seen.screen?.sections ?? []).first { $0.kind == .row }!
        var long = short
        long.cursor = "more"
        let shortHeard = heard(
            SectionRow(pictures: ToldPictures(), section: short, words: seen.words, open: { _ in }),
            size: phone
        )
        let longHeard = heard(
            SectionRow(pictures: ToldPictures(), section: long, words: seen.words, open: { _ in }),
            size: phone
        )
        XCTAssertEqual(words(shortHeard), [short.title!], "a row that opens nothing is its own name")
        XCTAssertEqual(
            words(longHeard),
            [seen.words["say.see_all"]!.fill("row", long.title!)],
            "a row that opens says so, in the words the strings give for it"
        )
    }

    // MARK: - The title's page

    /// The page a viewer reaches for a film. It had no labels at all until 2026-11-12 - 705 lines
    /// and nothing - so this asks the plain question first: does everything reached on it speak.
    ///
    /// What it reaches is the three controls pinned above the scroll, and that is all: see the note
    /// below this test. So a green run here says the bar is sound and says nothing at all about the
    /// rest of the page, which is most of it.
    func testEveryControlOnTheTitlePageSpeaks() {
        let (shell, seen) = titleSeen()
        let found = heard(Title(shell: shell, seen: seen), size: phone, settle: 0.6)
        XCTAssertFalse(found.isEmpty, "nothing on the title page was read at all")
        XCTAssertEqual(silent(found), [], "something on the title page is reached and says nothing")
        XCTAssertEqual(assetNames(found), [], "something is read as a picture's file name")
        XCTAssertEqual(
            notFromTheCode(found, seen.words, names: ["The Deep"] + (seen.detail?.genres ?? [])),
            [],
            "the title page says something that is in neither the strings nor the catalogue"
        )
    }

    /// Three of the page's controls draw a picture and no text at all: the arrow, the cast button
    /// and the close cross. The cross is two lines on a canvas, so it had nothing to read even
    /// before anything was named as its file.
    func testTheTitlePagesPictureOnlyControlsAreNamed() {
        let (shell, seen) = titleSeen()
        let found = heard(Title(shell: shell, seen: seen), size: phone, settle: 0.6)
        for key in ["common.back", "say.cast", "say.close"] {
            XCTAssertTrue(
                words(found).contains(seen.words[key]!),
                "the page has no control saying what the strings call \(key)"
            )
        }
    }

    // Three more of this page's controls want tests and cannot have them from here: the synopsis
    // that folds, the Rate control that carries the rating already given, and the two tabs. All
    // three are inside the page's `ScrollView`, and nothing inside one reaches the accessibility
    // tree in a test process with no window scene.
    //
    // Measured on iPhone 17 on 2026-11-12, and measured rather than assumed because the first
    // attempt assumed otherwise: hosting the page at 393x2400 instead of 393x852 changes nothing,
    // and neither does a stub that answers its requests at once instead of never. Both pages, a
    // film's and a series', reach exactly `Back`, `Play on another device` and `Close` - the three
    // controls pinned outside the scroll - and nothing else, at 0.6 s settle.
    //
    // The fix is the one the predecessor used for the chips and the tiles: lift each part into a
    // view of its own so a test can host it directly. That is a change to the shape of
    // `title/Title.swift`, whose body nests these controls eight levels deep, so it is left as its
    // own piece of work rather than rushed in alongside the labelling.

    // MARK: - Notifications

    /// A notice that has not been read is marked with an eight point dot and nothing else, which is
    /// the one thing on the row a person listening cannot see.
    func testAnUnreadNoticeSaysItHasNotBeenRead() {
        let (shell, seen) = homeSeen()
        let found = heard(NotificationsView(shell: shell, seen: seen), size: phone, settle: 0.4)
        // The page asks the server for its notices and this stub answers nothing, so what is read
        // here is the bar and the empty line under it. Both must still speak.
        XCTAssertEqual(silent(found), [], "something on the notifications page says nothing")
        XCTAssertEqual(assetNames(found), [], "the bell is read as the name of its asset file")
        XCTAssertTrue(
            words(found).contains(seen.words["common.back"]!),
            "the way back off the page is not named"
        )
    }

    // MARK: - Decoration

    /// A picture carries no words unless it is given them. This is as much of the work as labelling
    /// a control: an unmarked picture here was read out as the name of its own asset file.
    func testEveryPictureIsDecorationUntilItIsGivenWords() {
        let silentOnes: [(String, AnyView)] = [
            ("Icon", AnyView(Icon("icon_search", 34))),
            ("Mark", AnyView(Mark(60))),
            ("Art", AnyView(Art(ToldPictures(), "https://example.invalid/a.jpg"))),
        ]
        for (name, view) in silentOnes {
            XCTAssertEqual(heard(view, size: phone), [], "\(name) is read although nothing gave it words")
        }
        let spoken = heard(Icon("icon_search", 34, says: phoneWords["nav.search"]!), size: phone)
        XCTAssertEqual(
            words(spoken), [phoneWords["nav.search"]!],
            "a picture given words is read as those words"
        )
        XCTAssertEqual(assetNames(spoken), [])
    }
}
#endif
