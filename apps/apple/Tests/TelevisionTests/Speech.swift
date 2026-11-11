#if canImport(UIKit)
import Heard
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

    /// A ticked line says it is ticked, rather than drawing a tick no one can hear.
    func testTickedLineSaysItIsChosen() {
        let found = heard(TickGrid(words: ["English", "Arabic"], ticked: 1, focus: 0), size: tv)
        print("TickGrid:", found)
        XCTAssertEqual(words(found), ["English", "Arabic"])
        XCTAssertEqual(found.filter { $0.kinds.contains("selected") }.count, 1, "the ticked line is the chosen one")
    }

    /// The page title of a settings style page is a heading, so a screen reader can jump to it.
    func testPageHeadIsAHeading() {
        let found = heard(PageHead(title: "Settings", help: "Choose what this device does."), size: tv)
        print("PageHead:", found)
        XCTAssertEqual(words(found), ["Settings", "Choose what this device does."])
        XCTAssertTrue(found.first?.kinds.contains("header") == true, "a page title is a heading")
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
