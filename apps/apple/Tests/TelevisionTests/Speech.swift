#if canImport(UIKit)
import Heard
import Ports
import Stand
import SurfacesShared
import SwiftUI
import XCTest

@testable import SurfacesTV

/// What the television surface says to a person who cannot see it. Every test here draws one real
/// part of the surface and reads back the tree an assistive technology would walk.
///
/// The surface draws itself and takes the remote in `app/Remote.swift`, so nothing on it is a system
/// control. A part that reads as nothing, or reads as the name of a picture file, is a defect whatever
/// any other app does: note 207 found the Android family with zero text nodes on either surface, and
/// our own capture found Netflix's television app exposing one node for the whole interface.
@MainActor
final class SpeechTests: XCTestCase {
    private let tv = CGSize(width: 1920, height: 1080)

    /// A large button reads as its word and says it is a button.
    func testPillSaysItsWordAndIsAButton() {
        let found = heard(Pill("Play", on: true, icon: "icon_play"), size: tv)
        print("Pill:", found)
        XCTAssertEqual(words(found), ["Play"], "a button should read as one thing: its word")
        XCTAssertTrue(found.first?.isButton == true, "a button should say it is a button")
        XCTAssertTrue(found.first?.kinds.contains("selected") == true, "the focused button is the selected one")
    }

    /// An icon inside a button that already carries the word must not be read again, and must never
    /// be read as its file name.
    func testNoPartReadsAsAnAssetName() {
        let parts: [(String, [Heard])] = [
            ("Pill", heard(Pill("Play", on: true, icon: "icon_play"), size: tv)),
            ("Menu", heard(Menu(lines: [MenuLine(word: "Episodes", icon: "icon_episodes")], focus: 0), size: tv)),
            ("TickGrid", heard(TickGrid(words: ["English"], ticked: 0, focus: 0), size: tv)),
            ("Icon", heard(Icon("icon_search", 34), size: tv)),
            ("Mark", heard(Mark(60), size: tv)),
        ]
        for (name, found) in parts {
            print("\(name):", found)
            XCTAssertEqual(assetNames(found), [], "\(name) reads a picture's file name out loud")
        }
    }

    /// A line of a menu is one thing with one word, not a pile of pictures and text.
    func testMenuLineIsOneButtonPerLine() {
        let lines = [MenuLine(word: "Episodes", icon: "icon_episodes"), MenuLine(word: "More Like This")]
        let found = heard(Menu(lines: lines, focus: 0), size: tv)
        print("Menu:", found)
        XCTAssertEqual(words(found), ["Episodes", "More Like This"])
        XCTAssertTrue(found.allSatisfy(\.isButton), "every menu line is a button")
    }

    /// A ticked line says it is ticked, rather than drawing a tick no one can hear - and says it
    /// apart from where the remote is, because the grid draws two marks that mean two different
    /// things and a person listening sees neither.
    ///
    /// This test replaces one that asserted the opposite of its own name: it counted the `selected`
    /// trait and called the result the ticked line, when on this grid `selected` is the line the
    /// remote is on. It passed with the tick unread, because `Heard` carried no value to read it
    /// from (`Tests/Heard/Heard.swift`). Both the harness and the test were wrong together, which is
    /// how a green test came to check nothing.
    func testTickedLineSaysItIsChosenAndTheFocusedLineSaysSoSeparately() {
        let chosen = "Selected"
        let found = heard(
            TickGrid(words: ["English", "Arabic"], ticked: 1, focus: 0, chosenWord: chosen),
            size: tv
        )
        print("TickGrid:", found)
        XCTAssertEqual(words(found), ["English", "Arabic"])

        let english = saying(found, "English")
        let arabic = saying(found, "Arabic")
        XCTAssertEqual(arabic?.value, chosen, "the ticked line does not say it is the chosen one")
        XCTAssertEqual(english?.value, "", "a line that is not ticked says it is chosen")
        XCTAssertEqual(english?.isSelected, true, "the line the remote is on does not say so")
        XCTAssertEqual(arabic?.isSelected, false, "a line the remote is not on says the remote is on it")
    }

    /// The tick and the focus are read from two different places, so neither can stand in for the
    /// other. Asked of one grid where they fall on the same line and one where they do not.
    func testTheTickAndTheRemoteAreReadApart() {
        let chosen = "Selected"
        let apart = heard(
            TickGrid(words: ["English", "Arabic"], ticked: 1, focus: 0, chosenWord: chosen),
            size: tv
        )
        let together = heard(
            TickGrid(words: ["English", "Arabic"], ticked: 1, focus: 1, chosenWord: chosen),
            size: tv
        )
        XCTAssertEqual(
            apart.filter(\.isSelected).map(\.label), ["English"],
            "the remote is on the first line and nothing else is selected"
        )
        XCTAssertEqual(
            together.filter(\.isSelected).map(\.label), ["Arabic"],
            "the remote is on the second line and nothing else is selected"
        )
        // Named rather than compared to each other: two empty lists are equal, so comparing the two
        // grids alone would pass for a grid that had stopped saying which line is ticked at all.
        XCTAssertEqual(
            apart.filter { !$0.value.isEmpty }.map(\.label), ["Arabic"],
            "the ticked line is not the one that says it is ticked"
        )
        XCTAssertEqual(
            together.filter { !$0.value.isEmpty }.map(\.label), ["Arabic"],
            "moving the remote changed which line says it is ticked"
        )
    }

    /// A grid given no word for the ticked state says nothing for it rather than an empty value a
    /// reader would stop on. The words arrive from the server, so a run that has not read its
    /// language yet has none, and that must not read as a control with a blank state.
    func testAGridWithNoWordForTheTickSaysNothingForIt() {
        let found = heard(TickGrid(words: ["English", "Arabic"], ticked: 1, focus: 0), size: tv)
        XCTAssertEqual(states(found), ["", ""], "an unset state word was read out as an empty value")
        XCTAssertEqual(silent(found), [], "a line of the grid has nothing to read")
    }

    /// The page title of a settings style page is a heading, so a screen reader can jump to it.
    func testPageHeadIsAHeading() {
        let found = heard(PageHead(title: "Settings", help: "Choose what this device does."), size: tv)
        print("PageHead:", found)
        XCTAssertEqual(words(found), ["Settings", "Choose what this device does."])
        XCTAssertTrue(found.first?.kinds.contains("header") == true, "a page title is a heading")
    }

    // MARK: - The two keyboards a remote types on

    /// The letter grid's top line is three pictures and one word. The pictures are the keys a person
    /// typing with a remote needs most - space and delete - and read as they stood they were three
    /// stops with nothing to read, which is a keyboard nobody can type on.
    func testTheLetterGridsPictureKeysSpeak() {
        let keys = LetterKeys("abc", globe: true)
        let found = heard(LetterGrid(focus: LetterPlace(line: 0, column: 0), keys: keys, words: tvWords), size: tv)
        print("LetterGrid:", found)
        XCTAssertEqual(silent(found), [], "a key of the grid has nothing to read")
        XCTAssertEqual(assetNames(found), [], "a key is read as the name of its picture file")
        for word in ["say.space", "say.letters", "say.delete"] {
            XCTAssertTrue(
                words(found).contains(tvWords[word]!),
                "the grid has no key saying what the strings call \(word)"
            )
        }
        // The letters themselves are drawn, so they are gathered rather than given words.
        XCTAssertTrue(words(found).contains("a"), "a letter key does not read as its letter")
    }

    /// Only the key the remote is on says it is selected, because a white cap is the only thing
    /// that says so and a person listening cannot see it.
    func testOnlyTheKeyTheRemoteIsOnSaysSo() {
        let keys = LetterKeys("abc", globe: true)
        let onSpace = heard(LetterGrid(focus: LetterPlace(line: 0, column: 0), keys: keys, words: tvWords), size: tv)
        let onLetter = heard(LetterGrid(focus: LetterPlace(line: 1, column: 1), keys: keys, words: tvWords), size: tv)
        XCTAssertEqual(
            onSpace.filter(\.isSelected).map(\.label), [tvWords["say.space"]!],
            "the remote is on space and nothing else says it is"
        )
        XCTAssertEqual(
            onLetter.filter(\.isSelected).map(\.label), ["b"],
            "the remote is on the second letter and nothing else says it is"
        )
    }

    /// The keyboard that signs in draws sixty keys, three of them pictures, and all three speak:
    /// `say.shift` landed on 2026-11-12, so the key that stayed silent while this test could name
    /// only the other two is checked with them now, and nothing on the keyboard is reached silently.
    func testTheSignInKeyboardsPictureKeysSpeak() {
        let state = KeyboardState("")
        let found = heard(Keyboard(state: state, cancel: "Back", done: "Next", words: tvWords), size: tv)
        XCTAssertEqual(assetNames(found), [], "a key is read as the name of its picture file")
        XCTAssertEqual(silent(found), [], "a key on the keyboard is reached and says nothing")
        for word in ["say.space", "say.delete", "say.shift"] {
            XCTAssertTrue(
                words(found).contains(tvWords[word]!),
                "the keyboard has no key saying what the strings call \(word)"
            )
        }
        XCTAssertTrue(words(found).contains("q"), "a letter key does not read as its letter")
        XCTAssertTrue(words(found).contains("Next"), "the key that finishes does not keep its word")
    }

    /// The keyboard state works out for itself which of its keys the remote is on, because it is
    /// the part that knows. The screen says that aloud, and nothing else on the screen can. It
    /// lives on the state rather than the view so a screen can ask without holding the view it drew.
    func testTheKeyboardSaysWhichKeyTheRemoteIsOn() {
        let state = KeyboardState("")
        // Row 2, column 4 is where the keys open: the letter g on the third line.
        XCTAssertEqual(state.hereIs("Back", "Next", tvWords), "g", "the keyboard does not say which letter the remote is on")
        state.row = 4
        state.column = 4
        XCTAssertEqual(
            state.hereIs("Back", "Next", tvWords), tvWords["say.space"]!,
            "the keyboard does not say the space key in words"
        )
        state.column = 7
        XCTAssertEqual(
            state.hereIs("Back", "Next", tvWords), tvWords["say.delete"]!,
            "the keyboard does not say the delete key in words"
        )
    }

    // MARK: - The player

    /// The bar is the one part of the player a person listening cannot read any other way: the two
    /// clocks are text, but the bar between them holds the whole of the information and says
    /// nothing. So the four pieces are one control - named by what Ok does, valued by where the
    /// film is - and the words for both come from `strings/`.
    func testThePlayerBarSaysWhatOkDoesAndWhereTheFilmIs() {
        let seen = Seen()
        seen.words = tvWords
        seen.lengthMs = 3_600_000
        let playing = heard(Bar(seen: seen, at: 60_000, playing: true, on: true), size: tv)
        let paused = heard(Bar(seen: seen, at: 60_000, playing: false, on: true), size: tv)
        print("Bar:", playing)
        XCTAssertEqual(words(playing), [tvWords["say.pause"]!], "a playing bar does not offer to pause")
        XCTAssertEqual(words(paused), [tvWords["common.play"]!], "a paused bar does not offer to play")
        XCTAssertNotEqual(
            words(playing), words(paused),
            "the bar reads the same whether the film is playing or stopped"
        )
        let said = tvWords["say.position"]!.fill("at", "1:00").fill("length", "1:00:00")
        XCTAssertEqual(
            playing.first?.value, said,
            "the bar does not say how far through the film it is, in the words the strings give"
        )
    }

    /// A bar with no length yet says where it is and nothing about an end it does not know.
    func testAPlayerBarWithNoLengthSaysNoEnd() {
        let seen = Seen()
        seen.words = tvWords
        seen.lengthMs = 0
        let found = heard(Bar(seen: seen, at: 30_000, playing: true, on: false), size: tv)
        XCTAssertEqual(silent(found), [], "the bar has nothing to read")
        XCTAssertEqual(found.first?.value, "0:30", "a bar with no length should still say where it is")
        XCTAssertEqual(found.first?.isSelected, false, "a bar the remote is not on says the remote is on it")
    }

    // MARK: - Who is watching

    /// Only the profile in focus draws its name, so a row that is not in focus has nothing on it a
    /// reader could gather: the row carries the name itself, and a locked one says it is locked.
    func testAProfileRowIsNamedWhetherOrNotTheRemoteIsOnIt() {
        let locked = person("Noor", "p2", hasPin: true)
        let away = heard(
            ProfileRow(
                pictures: ToldPictures(), person: person("Ibrahim", "p1"), on: false, pencil: false, server: "",
                pencilWord: tvWords["profile.edit"]!,
                lockedWord: tvWords["say.profile_locked"]!.fill("name", "Ibrahim")
            ),
            size: tv
        )
        let here = heard(
            ProfileRow(
                pictures: ToldPictures(), person: locked, on: true, pencil: false, server: "",
                pencilWord: tvWords["profile.edit"]!,
                lockedWord: tvWords["say.profile_locked"]!.fill("name", locked.name)
            ),
            size: tv
        )
        XCTAssertEqual(words(away), ["Ibrahim"], "a row the remote is not on is not read as its name")
        XCTAssertEqual(away.first?.isSelected, false, "a row the remote is not on says it is selected")
        XCTAssertEqual(
            words(here), [tvWords["say.profile_locked"]!.fill("name", "Noor")],
            "a locked profile does not say it is locked, in the words the strings give"
        )
        XCTAssertEqual(here.first?.isSelected, true, "the row the remote is on does not say so")
    }

    /// Left puts the pencil on the profile, which changes what Ok does rather than where the remote
    /// is. That is drawn as a pencil over the picture and is said as the row's state.
    func testTheProfileRowSaysWhenAPressWouldEditInstead() {
        func row(_ pencil: Bool) -> [Heard] {
            heard(
                ProfileRow(
                    pictures: ToldPictures(), person: person("Ibrahim", "p1"), on: true, pencil: pencil,
                    server: "",
                    pencilWord: tvWords["profile.edit"]!, lockedWord: ""
                ),
                size: tv
            )
        }
        XCTAssertEqual(row(true).first?.value, tvWords["profile.edit"]!, "the pencil says nothing")
        XCTAssertEqual(row(false).first?.value, "", "a row with no pencil says there is one")
    }

    // MARK: - Choosing a profile picture

    /// A grid of faces with no words anywhere: not on the pictures, and not in the answer that
    /// lists them either. Read as they stood, every one of them was a stop that said nothing, so a
    /// person choosing a profile picture had a screenful of silence. `say.avatar_in_set` is the
    /// words this project already had for the case and nobody had wired them.
    func testEveryProfilePictureSaysWhichOfTheSetItIs() {
        let rows = [PictureRow(label: "Cats", pictures: ["/a.jpg", "/b.jpg", "/c.jpg"])]
        let focus = PictureFocus()
        let found = heard(
            PictureRows(
                pictures: ToldPictures(), rows: rows, focus: focus, server: "https://example.invalid",
                pattern: tvWords["say.avatar_in_set"]!
            ),
            size: tv
        )
        print("PictureRows:", found)
        XCTAssertEqual(silent(found), [], "a profile picture is reached and says nothing")
        XCTAssertEqual(assetNames(found), [], "a picture is read as the name of its file")
        let said = tvWords["say.avatar_in_set"]!
        XCTAssertTrue(
            words(found).contains(said.fill("n", 1).fill("set", "Cats")),
            "the first picture does not say which of the set it is"
        )
        XCTAssertTrue(
            words(found).contains(said.fill("n", 3).fill("set", "Cats")),
            "the third picture does not say which of the set it is"
        )
        XCTAssertEqual(
            found.filter(\.isSelected).count, 1,
            "the picture the remote is on is the selected one, and it is the only one"
        )
    }

    /// Given no pattern - a run that has not read its language yet - the pictures go from the tree
    /// rather than becoming a screenful of stops with nothing to read.
    func testProfilePicturesWithNoWordsAreNotSilentStops() {
        let rows = [PictureRow(label: "Cats", pictures: ["/a.jpg", "/b.jpg"])]
        let found = heard(
            PictureRows(
                pictures: ToldPictures(), rows: rows, focus: PictureFocus(),
                server: "https://example.invalid"
            ),
            size: tv
        )
        XCTAssertEqual(silent(found), [], "an unworded picture is reached and says nothing")
        XCTAssertEqual(words(found), ["Cats"], "only the row's own theme should be left to read")
    }

    // MARK: - The profile card's menu

    /// A line of the card draws a face, a word and sometimes a line under it. The face is decoration
    /// and the words are gathered, so the line reads as one thing and not as three.
    ///
    /// This screen also shows what the remote box costs. `remote(says:)` makes the screen box an
    /// accessibility element carrying what the remote is on (`app/Remote.swift`), so a whole screen
    /// has one extra node that repeats whatever the focused control already says - here
    /// "Ibrahim\nSwitch Profiles" from the box beside "Ibrahim, Switch Profiles" from the line. The
    /// two differ only in the join: `sayBoth` uses a line end so no language's punctuation is
    /// decided in code, while SwiftUI's own gathering uses a comma.
    ///
    /// Kept rather than removed, and the reason is an assumption this process cannot test: on a
    /// surface that paints its own focus, that box is the one thing the system really focuses, so it
    /// is probably also where VoiceOver's cursor sits and the only node whose label updating makes
    /// the screen followable. Whether the duplicate is worth that is the owner's call, and it is in
    /// the report. The test states the duplicate rather than working around it, so a change to the
    /// mechanism shows up here.
    func testAProfileMenuLineReadsAsOneThing() {
        let seen = Seen()
        seen.words = tvWords
        let me = person("Ibrahim", "p1")
        let found = heard(
            ProfileMenu(shell: NoTellShell(seen: seen), seen: seen, person: me, leave: {}),
            size: tv
        )
        print("ProfileMenu:", found)
        XCTAssertEqual(silent(found), [], "a line of the card says nothing")
        XCTAssertEqual(assetNames(found), [], "a line is read as the name of a picture file")
        XCTAssertFalse(
            words(found).contains { $0.hasPrefix("I\n") || $0.hasPrefix("I,") },
            "the profile's first letter is read before its name"
        )
        XCTAssertEqual(
            found.filter(\.isSelected).count, 1,
            "the line the remote is on is the only selected one"
        )
        // The lines themselves are buttons; the one node that is not is the remote's box.
        let lines = found.filter(\.isButton)
        XCTAssertEqual(
            words(lines), ["Ibrahim, Switch Profiles", tvWords["menu.help"]!],
            "the card's lines are not read as one word each, in order"
        )
        XCTAssertEqual(
            found.filter { !$0.isButton }.count, 1,
            "the remote's box is the one node on the screen that is not a control"
        )
    }

    // MARK: - Settings

    /// A settings row draws two things that mean two things: what the setting is, and how it stands.
    /// Gathering them would read the value into the name and then again as the value.
    func testASettingsRowSaysItsNameAndItsValueApart() {
        let row = SettingsRow(
            tvWords["settings.autoplay_next"]!, value: tvWords["settings.on"]!, action: .flipAutoplayNext
        )
        let found = heard(SettingsRowItem(row: row, on: true), size: tv)
        print("SettingsRowItem:", found)
        XCTAssertEqual(words(found), [tvWords["settings.autoplay_next"]!], "the row is not read as its name")
        XCTAssertEqual(found.first?.value, tvWords["settings.on"]!, "the row does not say how it stands")
        XCTAssertEqual(found.first?.isSelected, true, "the row the remote is on does not say so")
        XCTAssertFalse(
            found.first!.label.contains(tvWords["settings.on"]!),
            "the value was read into the name, so a reader says it twice"
        )
    }

    /// Nothing a screen reader stops on may be silent.
    func testNothingIsSilent() {
        let parts: [(String, [Heard])] = [
            ("Pill", heard(Pill("More Info", on: false), size: tv)),
            ("Menu", heard(Menu(lines: [MenuLine(word: "Episodes")], focus: -1), size: tv)),
            ("PageHead", heard(PageHead(title: "Settings", help: "Help."), size: tv)),
        ]
        for (name, found) in parts {
            XCTAssertEqual(silent(found), [], "\(name) has an element with nothing to read")
        }
    }
}
#endif
