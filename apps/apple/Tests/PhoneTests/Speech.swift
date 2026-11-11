#if canImport(UIKit)
import Heard
import SurfacesShared
import SwiftUI
import XCTest

@testable import SurfacesPhone

/// What the phone surface says to a person who cannot see it. Each test draws one real part of the
/// surface at phone size and reads back the tree VoiceOver would walk.
///
/// The phone surface does use system buttons, so SwiftUI derives a label for most of them from the
/// text inside. The gap is the buttons that carry a picture and no text: UIKit then falls back to the
/// name of the picture file, which is not words and not in any language.
@MainActor
final class SpeechTests: XCTestCase {
    private let phone = CGSize(width: 393, height: 852)

    private func bar(_ back: Bool, bell: Bool, unread: Int = 0) -> TopBar {
        TopBar(
            server: "",
            name: "Home",
            back: back ? {} : nil,
            bell: bell ? {} : nil,
            unreadCount: unread
        )
    }

    /// The bell and the back arrow carry no text, so they are the two buttons a screen reader cannot
    /// name unless someone names them.
    func testTheIconOnlyButtonsAreNamed() {
        let found = heard(bar(true, bell: true, unread: 3), size: phone)
        print("TopBar, back and bell:", found)
        XCTAssertEqual(assetNames(found), [], "a button is read as the name of its picture file")
        XCTAssertEqual(silent(found), [], "a button has nothing to read")
        XCTAssertTrue(found.contains { $0.isButton && !$0.label.isEmpty }, "the bar has a named button")
    }

    /// The brand's mark is decoration: the page's name is beside it and says the same thing.
    func testTheBrandMarkIsNotRead() {
        let found = heard(bar(false, bell: false), size: phone)
        print("TopBar, mark:", found)
        XCTAssertEqual(words(found), ["Home"], "the mark is decoration beside the page's name")
    }

    /// What the bell says has to carry how many are waiting, because the number is drawn on it.
    func testTheBellSaysHowManyAreWaiting() {
        let quiet = heard(bar(false, bell: true, unread: 0), size: phone)
        let busy = heard(bar(false, bell: true, unread: 3), size: phone)
        let quietBell = quiet.first { $0.isButton }?.label ?? ""
        let busyBell = busy.first { $0.isButton }?.label ?? ""
        print("bell, quiet:", quietBell, "busy:", busyBell)
        XCTAssertNotEqual(quietBell, "", "the bell has no words")
        XCTAssertNotEqual(busyBell, quietBell, "the count drawn on the bell is not spoken")
    }

    /// Every place on the foot bar reads as its own word, once, and the chosen one says it is chosen.
    func testTheFootBarReadsItsWordsOnce() {
        let items = [
            BarItem(icon: "icon_home", word: "Home", here: true, go: {}),
            BarItem(icon: "icon_search", word: "Search", here: false, go: {}),
        ]
        let found = heard(BottomBar(items: items), size: phone)
        print("BottomBar:", found)
        XCTAssertEqual(words(found), ["Home", "Search"], "each place reads as one word")
        XCTAssertEqual(found.filter { $0.kinds.contains("selected") }.count, 1, "the page in view is the chosen one")
    }

    /// A chip with a caret must not read the caret's file name after its word.
    func testTheChipsReadTheirWords() {
        let chips = [
            Chip(word: "Series", chosen: true, go: {}),
            Chip(word: "Categories", chosen: false, drops: true, go: {}),
        ]
        let found = heard(ChipRow(chips: chips, shown: 1), size: phone)
        print("ChipRow:", found)
        XCTAssertEqual(assetNames(found), [], "a chip reads the caret's file name")
        XCTAssertEqual(words(found), ["Series", "Categories"])
    }
}
#endif
