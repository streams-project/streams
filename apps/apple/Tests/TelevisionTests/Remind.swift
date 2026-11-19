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

private let repo = URL(fileURLWithPath: #filePath)
    .deletingLastPathComponent()
    .deletingLastPathComponent()
    .deletingLastPathComponent()
    .deletingLastPathComponent()
    .deletingLastPathComponent()

private func said() throws -> [String: String] {
    let data = try Data(contentsOf: repo.appendingPathComponent("strings/en.json"))
    return try XCTUnwrap(try JSONSerialization.jsonObject(with: data) as? [String: String])
}

@MainActor
private func remoteOf(_ view: UIView) -> RemoteUIView? {
    if let found = view as? RemoteUIView { return found }
    for sub in view.subviews {
        if let found = remoteOf(sub) { return found }
    }
    return nil
}

@MainActor
@discardableResult
private func pressing<V: SwiftUI.View>(
    _ view: V,
    _ keys: [Word],
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
    for key in keys {
        let receiver = try XCTUnwrap(remoteOf(host.view), "the screen has no remote on it at all")
        _ = receiver.handler?(key)
        host.view.setNeedsLayout()
        host.view.layoutIfNeeded()
        RunLoop.current.run(until: Date().addingTimeInterval(settle))
    }
    let found = heard(in: host.view)
    window.isHidden = true
    return found
}

/// A title, here or still to come.
private func aTitle(coming day: String?, reminded: Bool = false) throws -> TitleDetail {
    TitleDetail(
        availableFrom: try day.map { try CalendarDay($0) },
        name: "Dune",
        overview: "A story.",
        reminderSet: reminded,
        title: try TitleKey("movie:1")
    )
}

@MainActor
private func aSeen(_ data: TitleDetail, words: [String: String]) throws -> Seen {
    let seen = Seen()
    seen.words = words
    seen.active = "p1"
    seen.detail = data
    seen.route = RouteTarget(target: .title, title: data.title)
    return seen
}

/// The reminder panel reached from the title page.
///
/// `reminder/Reminder.swift` and its `Coming` were written, tested and left unrouted: nothing on
/// this surface opened the panel, so a viewer looking at a title that had not arrived had no way to
/// ask to be told. These are the tests for the way in.
@MainActor
final class RemindTests: XCTestCase {
    private let tv = CGSize(width: 1920, height: 1080)

    /// A title that has not arrived has a line on its menu that says so, and a title already here
    /// does not.
    ///
    /// `available_from` is the server's own answer to "is it here", and it is the condition the
    /// phone's button hangs on and the one `Coming(_:)` states.
    func testOnlyATitleStillToComeOffersAReminder() throws {
        let book = try said()
        let remind = try XCTUnwrap(book["remind.me"])
        let coming = try aTitle(coming: "2026-12-01")
        let here = try aTitle(coming: nil)

        let comingSeen = try aSeen(coming, words: book)
        let onComing = heard(
            TitlePage(shell: TellingShell(seen: comingSeen), seen: comingSeen),
            size: tv, settle: 0.4
        )
        XCTAssertTrue(
            words(onComing).contains(remind),
            "a title that has not arrived does not offer a reminder: \(words(onComing))"
        )

        let hereSeen = try aSeen(here, words: book)
        let onHere = heard(
            TitlePage(shell: TellingShell(seen: hereSeen), seen: hereSeen),
            size: tv, settle: 0.4
        )
        XCTAssertFalse(
            words(onHere).contains(remind),
            "a title already here offers to remind the viewer about it"
        )
    }

    /// Ok on that line opens the panel, Ok on the panel's first line tells the server, and Ok on
    /// its second line comes back to the title. Plainly: a viewer can get there and get back.
    func testTheLineOpensThePanelAndTheSecondLineComesBack() throws {
        let book = try said()
        let data = try aTitle(coming: "2026-12-01")
        let seen = try aSeen(data, words: book)
        let shell = TellingShell(seen: seen)
        // The menu opens on Play; the reminder is the last line, under Play and My List.
        let onPanel = try pressing(TitlePage(shell: shell, seen: seen), [.down, .down, .ok])
        let askWord = try XCTUnwrap(book["remind.me"])
        XCTAssertTrue(
            words(onPanel).contains(where: { $0.contains(askWord) }),
            "the panel did not open: \(words(onPanel))"
        )
        XCTAssertEqual(shell.acted.count, 0, "opening the panel told the server something")

        // The panel opens on the line that leaves, so Up reaches the one that turns.
        let shell2 = TellingShell(seen: seen)
        try pressing(TitlePage(shell: shell2, seen: seen), [.down, .down, .ok, .up, .ok])
        XCTAssertEqual(shell2.reminders, [true], "the panel did not tell the server to set the reminder")

        // And down again onto the line that leaves, which comes back to the title's own menu.
        let shell3 = TellingShell(seen: seen)
        let back = try pressing(TitlePage(shell: shell3, seen: seen), [.down, .down, .ok, .ok])
        XCTAssertEqual(shell3.acted.count, 0, "the line that leaves told the server something")
        XCTAssertTrue(
            words(back).contains(try XCTUnwrap(book["common.play"])),
            "the panel did not come back to the title: \(words(back))"
        )
        XCTAssertEqual(shell3.backs, 0, "leaving the panel left the title page as well")
    }

    /// The line that opened the panel says what the viewer did there when they come back to it.
    ///
    /// Without this, a viewer sets a reminder, presses the line that leaves, and reads "Remind Me"
    /// on the title they have just asked about -- because `seen.detail` is still the answer the
    /// server gave before the command. The page keeps the answer the viewer gave, which is the
    /// shape `in_list` already has on this page.
    func testTheLineSaysReminderSetAfterTheViewerSetsIt() throws {
        let book = try said()
        let data = try aTitle(coming: "2026-12-01")
        let seen = try aSeen(data, words: book)
        let shell = TellingShell(seen: seen)
        let back = try pressing(TitlePage(shell: shell, seen: seen), [.down, .down, .ok, .up, .ok, .down, .ok])
        XCTAssertEqual(shell.reminders, [true])
        XCTAssertTrue(
            words(back).contains(try XCTUnwrap(book["remind.set"])),
            "the line still offers to set a reminder that is already set: \(words(back))"
        )
    }

    /// Nothing on the title page with a reminder line on it is reached in silence.
    func testNothingOnATitleThatIsComingIsSilent() throws {
        let book = try said()
        let data = try aTitle(coming: "2026-12-01")
        let seen = try aSeen(data, words: book)
        let found = heard(TitlePage(shell: TellingShell(seen: seen), seen: seen), size: tv, settle: 0.4)
        XCTAssertEqual(silent(found), [], "something on the title page says nothing")
        XCTAssertEqual(assetNames(found), [], "something reads as a picture's file name")
    }

    /// And nothing on the panel it opens.
    func testNothingOnThePanelReachedFromTheTitleIsSilent() throws {
        let book = try said()
        let data = try aTitle(coming: "2026-12-01")
        let seen = try aSeen(data, words: book)
        let found = try pressing(TitlePage(shell: TellingShell(seen: seen), seen: seen), [.down, .down, .ok])
        XCTAssertEqual(silent(found), [], "something on the panel says nothing")
        // Silence is the absence of a node, and a line drawn with no words has no node to be found
        // silent. So the two lines are asked for by name: the first version of this test passed
        // against a panel whose leaving line was empty.
        let heardWords = words(found)
        for key in ["remind.me", "common.cancel"] {
            XCTAssertTrue(
                heardWords.contains(where: { $0.contains(book[key] ?? "\u{0}") }),
                "the panel does not say \(key): \(heardWords)"
            )
        }
    }
}
#endif
