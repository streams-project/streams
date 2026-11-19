#if canImport(UIKit)
import Contract
import Foundation
import Heard
import Ports
import Stand
import SurfacesShared
import SwiftUI
import XCTest

@testable import SurfacesTV

/// The repository this test was compiled from, so the words are read out of `strings/en.json`
/// rather than copied here. A copy is a second home for the thing being checked and agrees with the
/// code forever.
private let repository = URL(fileURLWithPath: #filePath)
    .deletingLastPathComponent()  // TelevisionTests
    .deletingLastPathComponent()  // Tests
    .deletingLastPathComponent()  // apple
    .deletingLastPathComponent()  // apps
    .deletingLastPathComponent()  // the repository

private func strings() throws -> [String: String] {
    let data = try Data(contentsOf: repository.appendingPathComponent("strings/en.json"))
    return try XCTUnwrap(try JSONSerialization.jsonObject(with: data) as? [String: String])
}

/// The receiver `app/Remote.swift` puts over a screen, found in a drawn tree.
@MainActor
private func remoteOf(_ view: UIView) -> RemoteUIView? {
    if let found = view as? RemoteUIView { return found }
    for sub in view.subviews {
        if let found = remoteOf(sub) { return found }
    }
    return nil
}

/// Draws a screen in a window and presses the remote on it, as the system does.
///
/// A test that pressed a state machine underneath would pass for a screen whose `press` was never
/// wired to the receiver, or wired to the wrong word. `PageFocus` is private to `browse/Browse`
/// and cannot be reached from here at all, which is the right way round: what a press did is read
/// from the ports it called and from what a reader would then hear.
@MainActor
@discardableResult
private func pressing<V: SwiftUI.View>(
    _ view: V,
    _ words: [Word],
    settle: TimeInterval = 0.2
) throws -> [Heard] {
    let host = UIHostingController(rootView: view)
    let window = UIWindow(frame: CGRect(x: 0, y: 0, width: 1920, height: 1080))
    window.rootViewController = host
    window.isHidden = false
    window.makeKeyAndVisible()
    host.view.frame = CGRect(x: 0, y: 0, width: 1920, height: 1080)
    host.view.setNeedsLayout()
    host.view.layoutIfNeeded()
    RunLoop.current.run(until: Date().addingTimeInterval(settle))
    for word in words {
        let receiver = try XCTUnwrap(remoteOf(host.view), "the screen has no remote on it at all")
        _ = receiver.handler?(word)
        host.view.setNeedsLayout()
        host.view.layoutIfNeeded()
        RunLoop.current.run(until: Date().addingTimeInterval(settle))
    }
    let found = heard(in: host.view)
    window.isHidden = true
    return found
}

private func aTab(_ name: String, _ screen: ScreenName) -> NavItem {
    NavItem(id: name, label: name, screen: screen)
}

/// A page of rows as the server sends one, with one row on it so the page is not the empty one.
private func aPage() throws -> Contract.Screen {
    Contract.Screen(
        name: .home,
        sections: [
            Section(
                id: "r1",
                kind: .row,
                tiles: [Tile(name: "Dune", title: try TitleKey("movie:1"))],
                title: "Trending Now"
            )
        ],
        ttlS: 60
    )
}

@MainActor
private func aSeen(unread: Int, words: [String: String]) throws -> Seen {
    let seen = Seen()
    seen.words = words
    seen.active = "p1"
    seen.profiles = [Profile(hasPin: false, id: "p1", kids: false, main: true, name: "Ibrahim")]
    seen.nav = [aTab("Home", .home), aTab("Series", .series)]
    seen.screen = try aPage()
    seen.route = RouteTarget(screen: .home, target: .home)
    seen.unreadNotifications = unread
    return seen
}

/// The bell at the end of the top bar: the one way a viewer reaches the notifications feed on this
/// television, and the first television caller `say.notifications_unread` has ever had.
///
/// The feed itself has been routed by `app/Television.swift` since it landed and nothing sent a
/// viewer to it, so until this file the screen existed and could not be opened.
@MainActor
final class BellTests: XCTestCase {
    private let tv = CGSize(width: 1920, height: 1080)

    private func bar(focus: Int, unread: Int, bell: Bool = true, words: [String: String]) -> TopBar {
        TopBar(
            pictures: ToldPictures(),
            tabs: [aTab("Home", .home), aTab("Series", .series)],
            active: 0,
            focus: focus,
            person: Profile(hasPin: false, id: "p1", kids: false, main: true, name: "Ibrahim"),
            server: "",
            bell: bell,
            unread: unread,
            words: words
        )
    }

    // MARK: - What a reader is handed

    /// Nothing on the bar is reached and silent, at every place the remote can stand on it and with
    /// and without a count.
    ///
    /// The assertion this file is for. A control added to this bar with no words on it is the exact
    /// defect this project has found four times this week, and this test is over whatever the bar
    /// draws rather than over a list of its parts, so a control added later is in its scope without
    /// anybody remembering to put it there.
    func testNothingOnTheBarIsSilent() throws {
        let said = try strings()
        for focus in [whoTab, searchTab, 0, 1, bellTab] {
            for unread in [0, 4] {
                let found = heard(bar(focus: focus, unread: unread, words: said), size: tv)
                XCTAssertFalse(found.isEmpty, "the bar was reached by nothing at all")
                XCTAssertEqual(
                    silent(found), [],
                    "something on the bar is reached and says nothing, at place \(focus) with \(unread) waiting"
                )
                XCTAssertEqual(assetNames(found), [], "something on the bar reads as a picture's file name")
            }
        }
    }

    /// The bell says how many notices are waiting, in the words the strings give, and says the
    /// screen's own name when none are.
    ///
    /// The count is drawn on the bell and nowhere else, so this reading is the only way a person
    /// who cannot see it can have it at all. `say.notifications_unread` is in all eighteen
    /// languages and had no television caller on either family until this.
    func testTheBellSaysHowManyNoticesAreWaiting() throws {
        let said = try strings()
        let four = words(heard(bar(focus: bellTab, unread: 4, words: said), size: tv))
        XCTAssertTrue(
            four.contains(try XCTUnwrap(said["say.notifications_unread"]).fill("n", 4)),
            "the bell does not say the count the server sent: \(four)"
        )
        let none = words(heard(bar(focus: bellTab, unread: 0, words: said), size: tv))
        XCTAssertTrue(
            none.contains(try XCTUnwrap(said["notifications.title"])),
            "with nothing waiting the bell does not say the screen's own name: \(none)"
        )
        XCTAssertFalse(
            none.contains(try XCTUnwrap(said["say.notifications_unread"]).fill("n", 0)),
            "the bell says a count of none"
        )
    }

    /// A run that has not read its language yet leaves the bell unlabelled rather than saying the
    /// key or saying English. That is this surface's rule and it is also the first moment of every
    /// run, so it is the state a bell is most likely to be read in.
    func testABellWithNoWordsYetSaysNothingRatherThanTheKey() {
        let found = heard(bar(focus: bellTab, unread: 4, words: [:]), size: tv)
        XCTAssertEqual(
            words(found).filter { $0.contains("notification") }, [],
            "a key reached a viewer"
        )
    }

    /// The bell says whether the remote is on it, either way. A surface that paints its own focus
    /// has nothing else to tell a reader with.
    func testTheBellSaysWhetherTheRemoteIsOnIt() throws {
        let said = try strings()
        let label = try XCTUnwrap(said["say.notifications_unread"]).fill("n", 4)
        let on = heard(bar(focus: bellTab, unread: 4, words: said), size: tv)
        XCTAssertEqual(saying(on, label)?.isSelected, true, "the bell does not say the remote is on it")
        let away = heard(bar(focus: 0, unread: 4, words: said), size: tv)
        XCTAssertEqual(
            saying(away, label)?.isSelected, false,
            "a part the remote has left must say so, not fall silent"
        )
    }

    /// The count drawn on the bell is not a stop of its own, or a reader would hear the number
    /// twice and the second time with no noun on it.
    func testTheCountDrawnOnTheBellIsNotASecondStop() throws {
        let found = words(heard(bar(focus: bellTab, unread: 4, words: try strings()), size: tv))
        XCTAssertFalse(found.contains("4"), "the number alone is a stop")
    }

    /// The search screen's bar has no bell at all.
    ///
    /// Its key map walks the profile, search and the tabs and has no place for a fourth stop, so a
    /// bell drawn there would be a control no remote could reach. The other television's search
    /// screen draws the bar that has no bell, and the two match.
    func testTheSearchScreensBarHasNoBell() throws {
        let said = try strings()
        // The real screen and not a bar built here with `bell: false`: what is under test is that
        // `search/Search.swift` asks for no bell, and a bar handed the flag by the test would pass
        // whatever that file asked for.
        let seen = try aSeen(unread: 4, words: said)
        let found = words(
            heard(Search(shell: TellingShell(seen: seen), seen: seen, state: SearchState()), size: tv, settle: 0.4)
        )
        XCTAssertTrue(found.contains(try XCTUnwrap(said["nav.search"])), "the search screen's bar was not read at all: \(found)")
        // Search draws its bar with no count, so a bell there would say the screen's own name and
        // not the count. Both are refused: the first version of this test looked for the count only
        // and passed against a search screen that drew the bell.
        XCTAssertFalse(
            found.contains(try XCTUnwrap(said["say.notifications_unread"]).fill("n", 4)),
            "the search screen drew a bell with a count its key map cannot reach"
        )
        XCTAssertFalse(
            found.contains(try XCTUnwrap(said["notifications.title"])),
            "the search screen drew a bell its key map cannot reach"
        )
    }

    // MARK: - The key map, pressed through the receiver

    /// The bell is reachable from the page's own tab by pressing Right, and Ok on it opens the
    /// feed. Plainly: a viewer can now get there.
    func testTheBellIsReachedByPressingRightAndOpensTheFeed() throws {
        let seen = try aSeen(unread: 4, words: try strings())
        let shell = TellingShell(seen: seen)
        // Up from the first row to the bar, then right past both tabs onto the bell, then Ok.
        try pressing(Browse(shell: shell, seen: seen), [.up, .right, .right, .ok])
        XCTAssertEqual(shell.went, [Address.notifications], "Right and Ok did not open the feed")
    }

    /// The bell is the right edge of the bar: a Right pressed on it moves nothing, and in
    /// particular does not walk the remote to the profile at the far *left* of the same bar.
    ///
    /// That is not a hypothetical. The bell's place is -4, which satisfies `tab < tabs - 1` for
    /// every tab count there is, so a Right answered by that branch lands on -3. This is the test
    /// that holds the order of the branches.
    func testRightOnTheBellMovesNothing() throws {
        let seen = try aSeen(unread: 4, words: try strings())
        let shell = TellingShell(seen: seen)
        // Three more Rights than it takes to get there. Where it would land if the branch order
        // were wrong is the profile, whose Ok opens the menu and travels nowhere, so the feed
        // opening after them is the whole of the proof.
        try pressing(Browse(shell: shell, seen: seen), [.up, .right, .right, .right, .right, .right, .ok])
        XCTAssertEqual(shell.went, [Address.notifications], "a Right past the bell took the remote somewhere else")
    }

    /// Left from the bell comes back to the last tab, so a viewer can get out of the corner the
    /// same way they got into it.
    func testLeftFromTheBellComesBackToTheLastTab() throws {
        let seen = try aSeen(unread: 1, words: try strings())
        let shell = TellingShell(seen: seen)
        try pressing(Browse(shell: shell, seen: seen), [.up, .right, .right, .left, .ok])
        XCTAssertEqual(
            shell.went, [Address.of(ScreenName.series.rawValue)],
            "Left from the bell did not come back to the last tab"
        )
    }

    /// Back from the bell comes home to the page's own tab before it leaves the page, which is the
    /// branch this bar already had for a focus that wandered without choosing.
    func testBackFromTheBellComesHomeToThePagesTab() throws {
        let seen = try aSeen(unread: 1, words: try strings())
        let shell = TellingShell(seen: seen)
        try pressing(Browse(shell: shell, seen: seen), [.up, .right, .right, .back, .ok])
        XCTAssertEqual(
            shell.went, [Address.of(ScreenName.home.rawValue)],
            "Back from the bell did not come home to this page's tab"
        )
    }

    /// The count on the bar is the server's, and not a count this surface worked out for itself.
    ///
    /// The feed screen holds one page of the feed and knows how many of *those* are unread; a bar
    /// that used it would under-report every count past the first page and would disagree with the
    /// screen it opens. So the number drawn and said here moves with `seen.unreadNotifications` and
    /// with nothing else.
    func testTheCountComesFromTheServerAndNotFromTheSurface() throws {
        let said = try strings()
        for count in [1, 7, 23] {
            let found = words(heard(bar(focus: bellTab, unread: count, words: said), size: tv))
            XCTAssertTrue(
                found.contains(try XCTUnwrap(said["say.notifications_unread"]).fill("n", count)),
                "the bell said something other than the count the server gave: \(found)"
            )
        }
    }
}
#endif
