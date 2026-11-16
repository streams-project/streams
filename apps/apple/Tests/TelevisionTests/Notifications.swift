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

/// The repository this test was compiled from, so a test can read the files the surface cites:
/// `strings/en.json`, which holds the words, and `reference/netflix-on-the-tv.md`, which holds the
/// measurements. Both are read rather than copied, because a copy is a second home for the thing
/// being checked and would agree with the code forever.
private let repository = URL(fileURLWithPath: #filePath)
    .deletingLastPathComponent()  // TelevisionTests
    .deletingLastPathComponent()  // Tests
    .deletingLastPathComponent()  // apple
    .deletingLastPathComponent()  // apps
    .deletingLastPathComponent()  // the repository

private func text(_ path: String) throws -> String {
    try String(contentsOf: repository.appendingPathComponent(path), encoding: .utf8)
}

private func strings() throws -> [String: String] {
    let data = try Data(contentsOf: repository.appendingPathComponent("strings/en.json"))
    return try XCTUnwrap(try JSONSerialization.jsonObject(with: data) as? [String: String])
}

private func aNotice(
    _ id: String,
    _ name: String,
    kind: NoticeKind = .coming,
    on when: String = "2026-12-01",
    unread: Bool = true,
    about title: String = "movie:1",
    art: [Contract.Image]? = nil
) throws -> NoticeItem {
    NoticeItem(
        date: try CalendarDay(when),
        id: id,
        kind: kind,
        tile: Tile(images: art, name: name, title: try TitleKey(title)),
        unread: unread
    )
}

/// A feed as the server sends one: the wire text, so the screen's own decode is under test and not
/// stepped over.
private func asWire(_ feed: Notifications) throws -> String {
    String(decoding: try JSONEncoder().encode(feed), as: UTF8.self)
}

/// A `Shell` that answers what it was given and keeps everything a screen asked it to do.
///
/// `NoTellShell` answers nothing and records nothing, which is what a reading of a tree wants. A
/// key map is the other half: what a press *did* is a call on this port, and nowhere else.
@MainActor
final class TellingShell: Shell {
    let seen: Seen
    /// What `GET` answers, by path. A path not in here answers 404, as a server would.
    var answers: [String: String] = [:]
    let shown = ToldPictures()
    var pictures: Pictures { shown }

    private(set) var asked: [String] = []
    private(set) var went: [String] = []
    private(set) var sent: [(method: String, path: String, body: String)] = []
    private(set) var acted: [Action] = []
    private(set) var backs = 0
    private(set) var refreshed = 0

    init(seen: Seen, answers: [String: String] = [:]) {
        self.seen = seen
        self.answers = answers
    }

    /// Which way each reminder command this screen sent was turning.
    var reminders: [Bool] {
        acted.compactMap { action in
            guard case let .command(what) = action, case let .remind(remind) = what.command else { return nil }
            return remind.on
        }
    }

    var player: Player { fatalError("no screen under test plays") }
    var reel: Reel { fatalError("no screen under test reels") }
    func retry() {}
    func go(_ address: String) { went.append(address) }
    func swap(_ address: String) { went.append(address) }
    func back() { backs += 1 }
    func act(_ action: Action) { acted.append(action) }
    func ask(_ path: String, reply: @escaping (Int, String) -> Void) {
        asked.append(path)
        if let body = answers[path] { reply(200, body) } else { reply(404, "") }
    }
    func send(_ method: String, _ path: String, _ body: String, reply: @escaping (Int, String) -> Void) {
        sent.append((method, path, body))
        reply(200, "")
    }
    func keep(_ profile: Profile) {}
    func keepAll(_ profiles: [Profile]) {}
    func speak(_ language: String?) {}
    func refreshNotifications() { refreshed += 1 }
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
/// The one thing `heard` cannot answer. A screen's key map reaches its parts through the receiver
/// in `app/Remote.swift`, and a test that pressed the state machine underneath would pass for a
/// screen whose `press` was never wired to it, or wired to the wrong word. Pressing through the
/// receiver is what proves the wiring, and what a press did is read from the ports it called.
///
/// The receiver is looked up again before every word, because a press can replace the screen and
/// with it the view that was holding the handler.
/// What a reader would find on the screen once the keys have been pressed, so the one walker in
/// `Tests/Heard` answers both questions and a test can ask where a line ended up after a move.
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

private func aFeed(_ items: [NoticeItem], cursor: String? = nil, more: Bool = false) -> Notifications {
    Notifications(cursor: cursor, hasMore: more, items: items)
}

private func aComingTitle(_ name: String = "Dune", day: String? = "2026-12-01", reminded: Bool = false) throws -> TitleDetail {
    TitleDetail(
        availableFrom: try day.map { try CalendarDay($0) },
        name: name,
        reminderSet: reminded,
        title: try TitleKey("movie:1")
    )
}

/// The feed and the question: the two screens the television did not have, asked of their own state
/// rather than of a running app.
@MainActor
final class NoticesTests: XCTestCase {
    private let tv = CGSize(width: 1920, height: 1080)

    // MARK: - The numbers, each against what it is made of

    /// The feed begins under the page head, by the measured gap. Pinning 203 to itself would pass
    /// forever and catch nothing; this fails the day the head moves.
    func testTheFeedBeginsUnderThePageHead() {
        XCTAssertEqual(PageHead.under, PageHead.helpAt + PageHead.helpType, "the head ends where its last line ends")
        XCTAssertEqual(Spot.first, PageHead.under + Spot.gap, "the first line does not start under the head")
    }

    /// How many lines fit is the canvas divided by the pitch, and the pitch is the still plus the
    /// gap. A change to any of the three moves the answer.
    func testHowManyLinesFitIsTheCanvasDividedByThePitch() {
        XCTAssertEqual(Spot.step, Spot.stillHigh + Spot.gap, "the pitch is not the still plus the gap")
        XCTAssertEqual(
            Spot.visible, Int((pageHigh - Spot.first) / Spot.step),
            "how many lines fit is not the canvas divided by the pitch"
        )
        XCTAssertEqual(Spot.reach, Spot.visible, "the read ahead is not one screenful")
    }

    /// Every measured number on this screen is the reference's own, read out of the file the code
    /// cites rather than trusted to a comment.
    func testTheMeasuredNumbersAreTheOnesTheReferenceHolds() throws {
        let measured = try text("reference/netflix-on-the-tv.md")
        XCTAssertTrue(
            measured.contains("**\(Int(Spot.stillWide)) x \(Int(Spot.stillHigh))"),
            "the still is not the size the reference measured for a picture with words beside it"
        )
        XCTAssertTrue(
            measured.contains("gap \(Int(Spot.gap)),"),
            "the gap is not the one the reference measured between a picture and the words by it"
        )
        // Read off the line that measures the outline, and not from anywhere in the file: "2 px"
        // is in this file as well, as the tolerance a surface is held to, so a looser check would
        // pass for an outline of the wrong thickness.
        let outline = try XCTUnwrap(
            measured.split(separator: "\n").first { $0.contains("focus outline") },
            "the reference no longer measures the focus outline"
        )
        XCTAssertTrue(
            outline.contains("\(Int(Spot.edge)) px"),
            "the focus outline is not the thickness the reference measured"
        )
    }

    // MARK: - The key map

    /// The ends stop. Nothing on this surface wraps, and a press that moved nothing must say so,
    /// because a move is the one thing that redraws.
    func testTheEndsStopAndSayNothingMoved() throws {
        var spot = Spot(aFeed([try aNotice("1", "Dune"), try aNotice("2", "Arrival")]))
        XCTAssertFalse(spot.press(.up), "the first line moved up")
        XCTAssertEqual(spot.at, 0)
        XCTAssertTrue(spot.press(.down))
        XCTAssertEqual(spot.at, 1)
        XCTAssertFalse(spot.press(.down), "the last line moved down")
        XCTAssertEqual(spot.at, 1, "the end of the list wrapped round to the top")
    }

    /// An empty feed has nowhere to go and nothing to open.
    func testAnEmptyFeedMovesNowhere() {
        var spot = Spot(aFeed([]))
        XCTAssertFalse(spot.press(.down))
        XCTAssertFalse(spot.press(.up))
        XCTAssertNil(spot.chosen, "an empty feed offered a notice to open")
    }

    /// Left and Back leave; Right does nothing at all. None of the three is a move.
    func testLeftAndBackLeaveAndRightDoesNot() throws {
        var spot = Spot(aFeed([try aNotice("1", "Dune")]))
        XCTAssertTrue(spot.leaves(.left), "Left does not leave a one column page")
        XCTAssertTrue(spot.leaves(.back), "Back does not leave")
        XCTAssertFalse(spot.leaves(.right), "Right leaves the screen")
        XCTAssertFalse(spot.leaves(.ok), "Ok leaves the screen")
        for word in [Word.left, .back, .right, .ok] {
            XCTAssertFalse(spot.press(word), "\(word) reported that something moved")
        }
    }

    /// The focus is kept by the notice's id. The feed is the one list in this app that grows at the
    /// top, so a focus kept by number slides onto the notice above the one being read.
    func testANoticeArrivingAtTheTopDoesNotMoveTheFocus() throws {
        let dune = try aNotice("1", "Dune")
        let arrival = try aNotice("2", "Arrival")
        var spot = Spot(aFeed([dune, arrival]))
        XCTAssertTrue(spot.press(.down))
        XCTAssertEqual(spot.chosen?.id, "2")

        spot.arrived(aFeed([try aNotice("3", "Sicario"), dune, arrival]))
        XCTAssertEqual(spot.chosen?.id, "2", "a notice arriving at the top moved the focus off the one being read")
        XCTAssertEqual(spot.at, 2, "the focus did not follow its notice down the list")
    }

    /// A notice that is gone leaves the focus at the same place in the new list, clamped to its
    /// end, which is the nearest thing to where the viewer was looking.
    func testANoticeThatIsGoneLeavesTheFocusWhereItWas() throws {
        var spot = Spot(aFeed([try aNotice("1", "Dune"), try aNotice("2", "Arrival")]))
        XCTAssertTrue(spot.press(.down))
        spot.arrived(aFeed([try aNotice("3", "Sicario")]))
        XCTAssertEqual(spot.at, 0, "the focus was left past the end of a shorter feed")
        XCTAssertEqual(spot.chosen?.id, "3")
    }

    /// The focus walks down to the last line wholly on the screen and the list scrolls from there.
    func testTheListStandsStillUntilTheFocusPassesTheLastWholeLine() throws {
        let many = try (1...8).map { try aNotice("\($0)", "Title \($0)") }
        var spot = Spot(aFeed(many))
        for line in 0..<Spot.visible {
            XCTAssertEqual(spot.scrolled, 0, "the list slid up with line \(line) still on the screen")
            XCTAssertTrue(spot.press(.down))
        }
        XCTAssertEqual(spot.scrolled, 1, "the list did not slide up when the focus passed the last whole line")
        XCTAssertTrue(spot.press(.down))
        XCTAssertEqual(spot.scrolled, 2, "the list slid by something other than one line")
    }

    /// The next page is asked for a screenful from the end, and never when the server says there is
    /// none. A feed with nothing in it asks whatever the server last said, because it can only fill
    /// by asking.
    func testTheNextPageIsAskedForOneScreenfulFromTheEnd() throws {
        XCTAssertFalse(
            Spot(aFeed([])).wantsMore,
            "a feed the server has given nothing more of asks for a page that does not exist"
        )
        XCTAssertTrue(
            Spot(aFeed([], cursor: "c", more: true)).wantsMore,
            "a screen the server says has more, and nothing on it yet, never asks"
        )

        let many = try (1...8).map { try aNotice("\($0)", "Title \($0)") }
        var ended = Spot(aFeed(many, cursor: nil, more: false))
        for _ in 0..<7 { _ = ended.press(.down) }
        XCTAssertFalse(ended.wantsMore, "a feed the server says has ended still asks for more")

        var going = Spot(aFeed(many, cursor: "c", more: true))
        XCTAssertFalse(going.wantsMore, "the first line of eight asked for the next page already")
        while going.at < many.count - 1 - Spot.reach { XCTAssertTrue(going.press(.down)) }
        XCTAssertTrue(going.wantsMore, "the focus reached one screenful from the end without asking")
    }

    /// A page joins on the end, and what is left over - the cursor and the flag - comes from the
    /// later answer, because that is what describes what has not arrived.
    func testAPageJoinsOnTheEndAndWhatIsLeftComesFromTheLaterAnswer() throws {
        var spot = Spot(aFeed([try aNotice("1", "Dune")], cursor: "c1", more: true))
        spot.andThen(aFeed([try aNotice("2", "Arrival")], cursor: nil, more: false))
        XCTAssertEqual(spot.feed.items.map(\.id), ["1", "2"], "the page did not join on the end")
        XCTAssertFalse(spot.feed.hasMore, "the feed still claims a page the later answer says is not there")
        XCTAssertEqual(spot.chosen?.id, "1", "joining a page moved the focus")
    }

    /// Reading a notice reads everything above it, which is what `POST /v1/notifications/read`
    /// means, and leaves the ones under it alone.
    func testReadingANoticeReadsEverythingAboveIt() throws {
        var spot = Spot(
            aFeed([
                try aNotice("1", "Dune"),
                try aNotice("2", "Arrival"),
                try aNotice("3", "Sicario"),
            ])
        )
        XCTAssertEqual(spot.unread, 3)
        spot.readTo("2")
        XCTAssertEqual(spot.feed.items.map(\.unread), [false, false, true], "the wrong notices were marked read")
        XCTAssertEqual(spot.unread, 1)
        spot.readTo("nothing like this")
        XCTAssertEqual(spot.unread, 1, "an id this feed does not hold changed what has been read")
    }

    // MARK: - The words

    /// The five keys are the contract's own five kinds, and a kind from a newer server has no words
    /// rather than words this build invented for it.
    func testTheFiveKindsAreTheContractsOwnFiveKeys() throws {
        let said = try strings()
        for kind in [NoticeKind.coming, .reminderLive, .newArrival, .newSeason, .newEpisode] {
            XCTAssertEqual(noticeKey(kind), "notification.\(kind.rawValue)")
            XCTAssertNotNil(said[noticeKey(kind)], "strings/en.json has no words for \(kind.rawValue)")
        }
        XCTAssertEqual(
            noticeKey(NoticeKind(rawValue: "something_this_build_has_never_heard_of")), "",
            "a kind this build does not know was given a key, so a viewer would be shown its wire value"
        )
    }

    /// Every key these two screens ask for is a key the strings hold. A key spelled wrong draws
    /// nothing at all on this surface, which is silent rather than wrong, and this is what catches
    /// it.
    func testEveryKeyTheseScreensAskForIsInTheStrings() throws {
        let said = try strings()
        let asked = [
            "notifications.title", "notifications.empty", "say.notifications_unread", "say.unread",
            "say.loading", "remind.me", "remind.set", "gate.coming", "common.cancel",
        ]
        for key in asked {
            XCTAssertNotNil(said[key], "strings/en.json has no \(key), so the screen asking for it draws nothing")
        }
    }

    /// What a person hears on a notice: what kind it is, the title's name, the day, and whether it
    /// has been read - in that order, joined by a line end so no language's word order is decided
    /// in the code.
    func testWhatAReaderHearsOnANotice() throws {
        let said = try strings()
        let spot = Spot(aFeed([try aNotice("1", "Dune", kind: .reminderLive)]))
        XCTAssertEqual(
            spot.says(said),
            "\(said["notification.reminder_live"]!)\nDune\n2026-12-01\n\(said["say.unread"]!)"
        )

        var read = spot
        read.readTo("1")
        XCTAssertEqual(
            read.says(said), "\(said["notification.reminder_live"]!)\nDune\n2026-12-01",
            "a notice already read is still said to be unread"
        )
    }

    /// A reader that arrives on a screen with no stops would otherwise be handed silence, so an
    /// empty feed says the screen's own name and that there is nothing on it.
    func testAnEmptyFeedSaysTheScreensNameAndThatItIsEmpty() throws {
        let said = try strings()
        XCTAssertEqual(
            Spot(aFeed([])).says(said),
            "\(said["notifications.title"]!)\n\(said["notifications.empty"]!)"
        )
    }

    /// A run that has not read its language yet has no words, and what it says then is the title's
    /// name and the day - never a key, and never English.
    func testARunWithNoWordsYetSaysOnlyWhatTheServerSent() throws {
        let spot = Spot(aFeed([try aNotice("1", "Dune")]))
        XCTAssertEqual(spot.says([:]), "Dune\n2026-12-01", "a screen with no words read a key out loud")
    }

    // MARK: - The question with two answers

    /// It opens on the line that leaves: a panel that opens on the line that does the thing turns
    /// one stray Ok into an action nobody asked for.
    func testTheQuestionOpensOnTheLineThatLeaves() {
        XCTAssertEqual(Question().at, Question.close, "the question opens on the line that answers it")
    }

    /// Its ends stop too, and a press that moved nothing says so.
    func testTheQuestionsEndsStop() {
        var ask = Question()
        XCTAssertEqual(ask.press(.down), .nothing, "the second line moved down")
        XCTAssertEqual(ask.press(.up), .moved)
        XCTAssertEqual(ask.at, Question.yes)
        XCTAssertEqual(ask.press(.up), .nothing, "the first line moved up")
    }

    /// Ok answers on the first line and leaves on the second; Left and Back leave from either;
    /// Right does nothing.
    func testOkAnswersOnTheFirstLineAndLeavesOnTheSecond() {
        var ask = Question()
        XCTAssertEqual(ask.press(.ok), .close, "Ok on the line that leaves did something else")
        XCTAssertEqual(ask.press(.right), .nothing, "Right did something on a panel with one column")
        _ = ask.press(.up)
        XCTAssertEqual(ask.press(.ok), .yes, "Ok on the first line did not answer the question")
        XCTAssertEqual(ask.press(.left), .close, "Left did not leave")
        XCTAssertEqual(ask.press(.back), .close, "Back did not leave")
    }

    /// The settings confirm panel asks with this same type, so the two cannot drift apart. The
    /// check is on the source because the drift it guards against is somebody writing the four
    /// branches out again in `Settings`.
    func testTheSettingsConfirmPanelAsksWithTheSameType() throws {
        let settings = try text("apps/apple/Sources/SurfacesTV/settings/Settings.swift")
        XCTAssertTrue(settings.contains("question.press(word)"), "Settings no longer asks its question through Question")
        XCTAssertTrue(settings.contains("QuestionPanel("), "Settings no longer draws the shared panel")
        XCTAssertFalse(
            settings.contains("confirmAt"),
            "Settings has a second copy of where the remote is on a two line question"
        )
    }

    // MARK: - The reminder

    /// A title that is already here has nothing to be reminded of, so there is no question to ask
    /// about it.
    func testATitleThatIsAlreadyHereIsNotComing() throws {
        XCTAssertNil(Coming(try aComingTitle(day: nil)), "a title with no day offered a reminder")
        XCTAssertNil(Coming(try aComingTitle("")), "a title with no name offered a reminder")
        XCTAssertNotNil(Coming(try aComingTitle()), "a coming title has no reminder to set")
    }

    /// The line says Remind Me until the reminder is set, and Reminder Set after, which is the
    /// phone's own pair.
    func testTheLineSaysRemindMeUntilTheReminderIsSet() throws {
        let off = try XCTUnwrap(Coming(try aComingTitle(reminded: false)))
        let on = try XCTUnwrap(Coming(try aComingTitle(reminded: true)))
        XCTAssertEqual(off.turnKey, "remind.me")
        XCTAssertEqual(on.turnKey, "remind.set")
    }

    /// The day is dropped into the sentence with the bidirectional isolates around it. Without them
    /// a date in an Arabic line comes out in the wrong order on screen.
    func testTheDayIsFilledWithTheIsolatesRoundIt() throws {
        let said = try strings()
        let coming = try XCTUnwrap(Coming(try aComingTitle()))
        let line = coming.comingWords(said)
        XCTAssertTrue(line.contains("\u{2068}2026-12-01\u{2069}"), "the day was filled in without its isolates")
        XCTAssertFalse(line.contains("{date}"), "the day was not filled in at all")
        XCTAssertEqual(coming.comingWords([:]), "", "a run with no words drew a bare date")
    }

    /// The title is said at both stops on purpose: a reader that moved to Cancel and heard only
    /// "Cancel" would have lost what was being cancelled.
    func testTheTitleIsSaidAtBothStops() throws {
        let said = try strings()
        let coming = try XCTUnwrap(Coming(try aComingTitle()))
        let onTurn = coming.says(said, at: Question.yes)
        let onClose = coming.says(said, at: Question.close)
        XCTAssertEqual(onTurn, "Dune\n\(coming.comingWords(said))\n\(said["remind.me"]!)")
        XCTAssertEqual(onClose, "Dune\n\(coming.comingWords(said))\n\(said["common.cancel"]!)")
        XCTAssertTrue(onClose.hasPrefix("Dune"), "the line that leaves does not say what is being cancelled")
    }

    // MARK: - What the tree says

    /// A line of the feed is one stop carrying the whole notice, with whether it has been read as
    /// its state and the remote's place as its own trait. Four stops saying a fragment each is what
    /// this is instead of.
    func testANoticeLineIsOneStopCarryingTheWholeNotice() throws {
        let said = try strings()
        let art = [Contract.Image(role: .backdrop, url: "https://example.invalid/a.jpg")]
        let found = heard(
            NoticeLine(
                pictures: ToldPictures(),
                notice: try aNotice("1", "Dune", kind: .coming, art: art),
                on: true,
                words: said
            ),
            size: tv
        )
        print("NoticeLine:", found)
        XCTAssertEqual(found.count, 1, "a notice is read as \(found.count) stops rather than one")
        let line = try XCTUnwrap(found.first)
        for piece in [said["notification.coming"]!, "Dune", "2026-12-01"] {
            XCTAssertTrue(line.label.contains(piece), "the line does not say \(piece)")
        }
        XCTAssertEqual(line.value, said["say.unread"]!, "the line does not say it has not been read")
        XCTAssertTrue(line.isSelected, "the line the remote is on does not say so")
        XCTAssertEqual(silent(found), [], "something on the line is reached and says nothing")
        XCTAssertEqual(assetNames(found), [], "the line reads a picture's file name out loud")
    }

    /// A notice already read says nothing about being unread, and a line the remote is not on does
    /// not claim the remote.
    func testAReadNoticeSaysNothingAboutBeingUnread() throws {
        let said = try strings()
        let found = heard(
            NoticeLine(
                pictures: ToldPictures(),
                notice: try aNotice("1", "Dune", unread: false),
                on: false,
                words: said
            ),
            size: tv
        )
        XCTAssertEqual(found.first?.value, "", "a notice already read still says it has not been")
        XCTAssertEqual(found.first?.isSelected, false, "a line the remote is not on says it is selected")
    }

    /// A kind this build has no words for draws no chip and is not said, and the notice still says
    /// what happened and when.
    func testAKindThisBuildDoesNotKnowIsLeftOutRatherThanShown() throws {
        let said = try strings()
        let strange = NoticeKind(rawValue: "a_kind_from_a_newer_server")
        let found = heard(
            NoticeLine(
                pictures: ToldPictures(),
                notice: try aNotice("1", "Dune", kind: strange),
                on: false,
                words: said
            ),
            size: tv
        )
        let line = try XCTUnwrap(found.first)
        XCTAssertFalse(line.label.contains("a_kind_from_a_newer_server"), "a wire value was read out to a viewer")
        XCTAssertTrue(line.label.contains("Dune"), "the notice stopped saying which title it is about")
    }

    /// The whole screen, with a shell that answers nothing: it says it is empty rather than looking
    /// like a page that is still loading, and nothing on it is reached in silence.
    func testNothingOnTheNotificationsScreenIsSilent() throws {
        let seen = Seen()
        seen.words = try strings()
        let found = heard(Notices(shell: NoTellShell(seen: seen), seen: seen), size: tv, settle: 0.6)
        print("Notices:", found)
        XCTAssertEqual(silent(found), [], "something on the notifications screen is reached and says nothing")
        XCTAssertEqual(assetNames(found), [], "something on the screen reads as a picture's file name")
        XCTAssertTrue(
            words(found).contains(seen.words["notifications.title"]!),
            "the screen does not say its own name"
        )
    }

    /// The feed the server sent is what the screen draws: the decode, the lines, the one line the
    /// remote is on, and the count under the page's own name.
    ///
    /// Every other reading here is of one line or of the empty screen, and an empty screen is the
    /// one state a feed that never decoded would also reach - so without this the screen could stop
    /// understanding the server's answer and nothing would say so.
    func testTheFeedTheServerSentIsWhatTheScreenDraws() throws {
        let seen = Seen()
        seen.words = try strings()
        let shell = TellingShell(
            seen: seen,
            answers: [
                "/v1/notifications": try asWire(
                    aFeed([
                        try aNotice("1", "Dune", kind: .reminderLive),
                        try aNotice("2", "Arrival", unread: false),
                    ])
                )
            ]
        )
        let found = heard(Notices(shell: shell, seen: seen), size: tv, settle: 0.6)
        print("Notices, filled:", found)
        XCTAssertEqual(silent(found), [], "something on the filled screen is reached and says nothing")
        let lines = found.filter(\.isButton)
        XCTAssertEqual(lines.count, 2, "the screen drew \(lines.count) lines for a feed of two")
        XCTAssertTrue(lines[0].label.contains("Dune"), "the first line is not the first notice sent")
        XCTAssertTrue(lines[1].label.contains("Arrival"), "the second line is not the second notice sent")
        XCTAssertEqual(
            lines.filter(\.isSelected).map { $0.label.contains("Dune") }, [true],
            "the remote is not on the newest notice, alone"
        )
        XCTAssertTrue(
            words(found).contains(seen.words["say.notifications_unread"]!.fill("n", 1)),
            "the page head does not say how many notices have not been read"
        )
    }

    /// Where the lines really are on screen, before and after the list slides.
    ///
    /// `scrolled` is a number, and a screen that offset by it the wrong way, or not at all, would
    /// keep answering that number correctly: the arithmetic and the drawing are two things and only
    /// one of them was under test. This reads the frames an assistive technology reports, which is
    /// the one place a test can ask a laid out SwiftUI view where it ended up.
    func testTheLinesLandWhereTheGeometrySaysAndSlideUnderTheHead() throws {
        let seen = Seen()
        seen.words = try strings()
        let many = try (1...8).map { try aNotice("\($0)", "Title \($0)", about: "movie:\($0)") }
        let shell = TellingShell(seen: seen, answers: ["/v1/notifications": try asWire(aFeed(many))])
        let screen = Notices(shell: shell, seen: seen)

        let drawn = try pressing(screen, [], settle: 0.6)
        print("Notices, frames:", drawn.map { ($0.label.prefix(14), $0.frame) })
        let still = drawn.filter(\.isButton)
        XCTAssertEqual(still.count, many.count, "the screen drew \(still.count) lines for a feed of eight")

        // Two lines the remote is not on, so neither frame carries the focus outline.
        XCTAssertEqual(
            still[2].frame.origin.y - still[1].frame.origin.y, Spot.step, accuracy: 0.5,
            "one line to the next is not the still plus the gap"
        )
        // The line the remote is on is outset by half the outline, because a stroke is drawn
        // astride the edge it follows. Measured here rather than assumed: it is what says the
        // outline is drawn at all, and at the measured thickness.
        XCTAssertEqual(
            still[1].frame.origin.y - still[0].frame.origin.y, Spot.step + Spot.edge / 2, accuracy: 0.5,
            "the line the remote is on is not outlined, or not at the measured thickness"
        )
        // Where the feed begins, read against the page head's own drawn place rather than against
        // the page's top. A screen hosted by itself is pushed down by tvOS's safe area - measured
        // at 60 points on this simulator - because `app/Television.swift` is what calls
        // `ignoresSafeArea`, and a test that pinned an absolute y would be asserting the harness.
        let headTitle = try XCTUnwrap(
            drawn.first { $0.kinds.contains("header") },
            "the page head is not in the tree to measure against"
        )
        XCTAssertEqual(
            still[1].frame.origin.y - headTitle.frame.origin.y,
            (Spot.first + Spot.step) - PageHead.titleAt,
            accuracy: 1,
            "the feed does not begin under the page head by the measured gap"
        )

        // Down to the last line that is wholly on the screen: nothing has moved yet.
        // The last line is never the one the remote is on in any of the three readings, so its
        // frame carries no outline and the three are comparable.
        let last = many.count - 1
        let held = try pressing(screen, Array(repeating: Word.down, count: Spot.visible - 1), settle: 0.6)
            .filter(\.isButton)
        XCTAssertEqual(
            held[last].frame.origin.y, still[last].frame.origin.y, accuracy: 0.5,
            "the list slid up with the last whole line still to be reached"
        )

        // One more, and the list slides by exactly one line.
        let slid = try pressing(screen, Array(repeating: Word.down, count: Spot.visible), settle: 0.6)
            .filter(\.isButton)
        print("Notices, slid:", slid.map { ($0.label.prefix(14), $0.frame) })
        XCTAssertEqual(
            still[last].frame.origin.y - slid[last].frame.origin.y, Spot.step, accuracy: 0.5,
            "the list did not slide up by exactly one line when the focus passed the last whole line"
        )
        let onIt = try XCTUnwrap(slid.first(where: \.isSelected), "nothing says where the remote is")
        XCTAssertTrue(
            onIt.frame.maxY <= slid[last].frame.maxY,
            "the line the remote is on is below the lines under it"
        )
    }

    /// Ok on a notice that has not been read tells the server so, asks for the count to be read
    /// again, and opens the title - in that order, and all three.
    func testOkOnAnUnreadNoticeTellsTheServerAndOpensTheTitle() throws {
        let seen = Seen()
        seen.words = try strings()
        let shell = TellingShell(
            seen: seen,
            answers: [
                "/v1/notifications": try asWire(
                    aFeed([
                        try aNotice("1", "Dune", about: "movie:1"),
                        try aNotice("2", "Arrival", about: "movie:2"),
                    ])
                )
            ]
        )
        try pressing(Notices(shell: shell, seen: seen), [.down, .ok])
        XCTAssertEqual(shell.sent.map(\.path), ["/v1/notifications/read"], "the notice was not marked read")
        XCTAssertEqual(shell.sent.first?.method, "POST")
        XCTAssertEqual(
            shell.sent.first?.body, #"{"up_to":"2"}"#,
            "the id told to the server is not the notice the remote was on"
        )
        XCTAssertEqual(shell.refreshed, 1, "the unread count was not read again")
        XCTAssertEqual(
            shell.went, [titleAddress(try TitleKey("movie:2"))],
            "Ok did not open the title the notice is about"
        )
    }

    /// A notice already read opens its title and tells the server nothing: the call means "and
    /// everything above this", and sending it again for a line already read would mark later
    /// arrivals read behind the viewer's back.
    func testOkOnANoticeAlreadyReadTellsTheServerNothing() throws {
        let seen = Seen()
        seen.words = try strings()
        let shell = TellingShell(
            seen: seen,
            answers: ["/v1/notifications": try asWire(aFeed([try aNotice("1", "Dune", unread: false)]))]
        )
        try pressing(Notices(shell: shell, seen: seen), [.ok])
        XCTAssertEqual(shell.sent.count, 0, "a notice already read was marked read again")
        XCTAssertEqual(shell.went.count, 1, "Ok did not open the title")
    }

    /// The next page is read when the focus comes within a screenful of the end, and what comes
    /// back is joined on the end of what is already drawn.
    ///
    /// The whole of the paging path: `wantsMore` is a number's answer and was tested as one, while
    /// nothing said the screen ever acts on it, asks at the cursor the server gave, or draws what
    /// comes back. A remote has no moment where a row comes into view, so if this is not wired
    /// there is nothing else to fall back on - the feed simply stops at its first page.
    func testTheNextPageIsReadWhenTheFocusNearsTheEndAndIsJoinedOn() throws {
        let seen = Seen()
        seen.words = try strings()
        let first = try (1...8).map { try aNotice("\($0)", "Title \($0)", about: "movie:\($0)") }
        let second = try (9...11).map { try aNotice("\($0)", "Title \($0)", about: "movie:\($0)") }
        let pages = [
            "/v1/notifications": try asWire(aFeed(first, cursor: "c1", more: true)),
            "/v1/notifications?cursor=c1": try asWire(aFeed(second)),
        ]
        // Each session gets its own shell, because each draws the screen afresh and so reads the
        // first page afresh.
        let early = TellingShell(seen: seen, answers: pages)
        let reaching = TellingShell(seen: seen, answers: pages)

        // Still further than a screenful from the end: nothing is asked for.
        let near = first.count - 1 - Spot.reach
        let start = try pressing(
            Notices(shell: early, seen: seen),
            Array(repeating: Word.down, count: near - 1),
            settle: 0.6
        )
        XCTAssertEqual(early.asked, ["/v1/notifications"], "the screen asked for a page it did not need yet")
        XCTAssertEqual(start.filter(\.isButton).count, first.count)

        // One line further, and the next page is read at the cursor the server gave, once, and
        // joined on the end of what is drawn.
        let after = try pressing(
            Notices(shell: reaching, seen: seen),
            Array(repeating: Word.down, count: near),
            settle: 0.6
        )
        XCTAssertEqual(
            reaching.asked, ["/v1/notifications", "/v1/notifications?cursor=c1"],
            "the next page was not read at the cursor the server gave, or was read more than once"
        )
        XCTAssertEqual(
            after.filter(\.isButton).count, first.count + second.count,
            "the page that came back is not drawn under the page that was there"
        )
    }

    /// Left leaves the screen, from the first line, which is the whole of the Left decision and is
    /// what a one column page on this surface does.
    func testLeftLeavesTheNotificationsScreen() throws {
        let seen = Seen()
        seen.words = try strings()
        let shell = TellingShell(
            seen: seen,
            answers: ["/v1/notifications": try asWire(aFeed([try aNotice("1", "Dune")]))]
        )
        try pressing(Notices(shell: shell, seen: seen), [.left])
        XCTAssertEqual(shell.backs, 1, "Left did not leave a one column page")
        XCTAssertEqual(shell.went, [], "Left opened something")
        XCTAssertEqual(shell.sent.count, 0, "Left told the server something")
    }

    /// The reminder sends the command for the way it is turning, each time it is pressed, and the
    /// line it leaves behind says which way that was.
    func testTheReminderTellsTheServerWhichWayItIsTurning() throws {
        let seen = Seen()
        seen.words = try strings()
        let shell = TellingShell(seen: seen)
        let coming = try XCTUnwrap(Coming(try aComingTitle()))
        try pressing(
            Reminder(shell: shell, seen: seen, coming: coming, leave: {}),
            [.up, .ok, .ok]
        )
        XCTAssertEqual(
            shell.reminders, [true, false],
            "the two presses did not set the reminder and then take it off again"
        )
        XCTAssertEqual(shell.backs, 0, "turning the reminder left the screen")
    }

    /// The line that leaves says nothing to the server. A panel that opens on the line that acts
    /// would make one stray Ok a reminder nobody asked for, and this is the other half of that.
    func testTheLineThatLeavesTellsTheServerNothing() throws {
        let seen = Seen()
        seen.words = try strings()
        let shell = TellingShell(seen: seen)
        let coming = try XCTUnwrap(Coming(try aComingTitle()))
        final class Count { var left = 0 }
        let leaving = Count()
        try pressing(
            Reminder(shell: shell, seen: seen, coming: coming, leave: { leaving.left += 1 }),
            [.ok]
        )
        XCTAssertEqual(leaving.left, 1, "Ok on the line that leaves did not leave")
        XCTAssertEqual(shell.acted.count, 0, "Ok on the line that leaves set a reminder")
    }

    /// The reminder panel: the question is a heading, the two lines are its answers, and only the
    /// line the remote is on says so.
    func testTheReminderPanelIsTwoLinesAndOnlyOneOfThemIsTheRemotesPlace() throws {
        let seen = Seen()
        seen.words = try strings()
        let coming = try XCTUnwrap(Coming(try aComingTitle()))
        let found = heard(
            Reminder(shell: NoTellShell(seen: seen), seen: seen, coming: coming, leave: {}),
            size: tv
        )
        print("Reminder:", found)
        XCTAssertEqual(silent(found), [], "something on the reminder screen is reached and says nothing")
        XCTAssertEqual(assetNames(found), [], "something on the screen reads as a picture's file name")
        let lines = found.filter(\.isButton)
        XCTAssertEqual(
            words(lines), [seen.words["remind.me"]!, seen.words["common.cancel"]!],
            "the panel's two answers are not the reminder and Cancel, in that order"
        )
        XCTAssertEqual(
            lines.filter(\.isSelected).map(\.label), [seen.words["common.cancel"]!],
            "the line the remote opens on is not the one that leaves"
        )
        XCTAssertTrue(
            found.contains { $0.kinds.contains("header") && $0.label == "Dune" },
            "the title is not a heading a reader can jump to"
        )
    }
}
#endif
