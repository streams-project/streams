#if canImport(UIKit)
import Contract
import Heard
import Ports
import Stand
import SurfacesShared
import SwiftUI
import UIKit
import XCTest

@testable import SurfacesTV

/// The episodes page when two seasons are read at once and the older answer is the one that is late.
///
/// `TitleKeys.swift` holds the rules of the page as free functions. This holds the one thing a free
/// function cannot: what the page does with an answer that arrives after the viewer has moved on.
/// `.task(id:)` cancels the read of the season the viewer left, but the port's reply still resumes
/// it, so it ran on to the end and stored its list over the newer one.
///
/// Read through the accessibility tree, and that is not vacuous here, for the reason the other
/// screens are not: `Episodes` is a `ZStack`, not a `ScrollView`, so its cards are in the tree
/// (`FINDINGS.md` H6 is about the contents of a scroller). The test also asserts the positive case,
/// that the second season's names ARE heard once they land, so a screen that drew nothing at all
/// cannot pass the half that says the first season's names are not.
@MainActor
final class EpisodesRaceTests: XCTestCase {
    private let tv = CGSize(width: 1920, height: 1080)

    private func season(_ number: Int, _ names: [String]) -> String {
        let season = Season(
            episodes: names.enumerated().map { Episode(name: $1, number: $0 + 1) },
            number: number
        )
        return String(decoding: try! JSONEncoder().encode(season), as: UTF8.self)
    }

    private func series() -> TitleDetail {
        TitleDetail(
            name: "The Show",
            seasons: [SeasonSummary(episodeCount: 2, number: 1), SeasonSummary(episodeCount: 2, number: 2)],
            title: try! TitleKey("tv:5")
        )
    }

    private func cards(_ found: [Heard], _ names: [String]) -> [String] {
        names.filter { name in found.contains { $0.label.contains(name) } }
    }

    /// The page in a window, and the keys pressed on it through the receiver the system uses.
    private func open(_ shell: HeldShell) -> (host: UIHostingController<Episodes>, window: UIWindow) {
        let host = UIHostingController(rootView: Episodes(shell: shell, seen: shell.seen, data: series(), leave: {}))
        let window = UIWindow(frame: CGRect(origin: .zero, size: tv))
        window.rootViewController = host
        window.isHidden = false
        window.makeKeyAndVisible()
        host.view.frame = CGRect(origin: .zero, size: tv)
        host.view.setNeedsLayout()
        host.view.layoutIfNeeded()
        pump(0.2)
        return (host, window)
    }

    private func press(_ words: [Word], on host: UIViewController) throws {
        for word in words {
            let receiver = try XCTUnwrap(remoteOf(host.view), "the page has no remote on it")
            _ = receiver.handler?(word)
            host.view.setNeedsLayout()
            host.view.layoutIfNeeded()
            pump(0.1)
        }
        pump(0.2)
    }

    /// Left opens the seasons, Down moves to the second, OK chooses it.
    private let chooseSeasonTwo: [Word] = [.left, .down, .ok]

    private func aShell() -> HeldShell {
        let seen = Seen()
        seen.words = tvWords
        return HeldShell(seen: seen)
    }

    /// The half that reached every viewer: for the whole of a season's read the cards on screen were
    /// the season before's, under a column already marking the new one chosen. Pressed through the
    /// real receiver and read off the real tree, so a page that drew `arrived.episodes` in place of
    /// `cardsOf` fails here and not only in the rule's own test.
    func testWhileASeasonIsBeingReadNoCardOfTheSeasonBeforeIsDrawn() throws {
        let shell = aShell()
        shell.answers["/v1/titles/tv/5/seasons/1"] = season(1, ["First A", "First B"])
        let (host, window) = open(shell)
        XCTAssertEqual(cards(heard(in: host.view), ["First A", "First B"]), ["First A", "First B"], "season 1 answered, so its cards are drawn: without this nothing below means anything")

        try press(chooseSeasonTwo, on: host)
        XCTAssertEqual(shell.held.keys.sorted(), ["/v1/titles/tv/5/seasons/2"], "season 2 is being read")
        XCTAssertEqual(cards(heard(in: host.view), ["First A", "First B"]), [], "season 2 is being read and season 1's cards are still drawn")

        shell.release("/v1/titles/tv/5/seasons/2", season(2, ["Second A"]))
        pump(0.3)
        XCTAssertEqual(cards(heard(in: host.view), ["Second A", "First A"]), ["Second A"])
        window.isHidden = true
    }

    /// The rare tail: a held Down while the new season is read walked the stale list's bound, and OK
    /// asked for an episode the season does not have. Nothing plays while the season is read, and
    /// when its list lands the remote is on its first card, so OK asks for episode 1 of season 2.
    func testHoldingDownWhileASeasonIsReadPlaysNothingAndLeavesTheFirstCard() throws {
        let shell = aShell()
        shell.answers["/v1/titles/tv/5/seasons/1"] = season(1, (1...7).map { "Old \($0)" })
        let (host, window) = open(shell)
        try press(chooseSeasonTwo, on: host)
        XCTAssertEqual(shell.held.keys.sorted(), ["/v1/titles/tv/5/seasons/2"])

        try press(Array(repeating: .down, count: 6) + [.ok], on: host)
        XCTAssertEqual(shell.went, [], "OK on a season being read must play nothing: it asked for \(shell.went)")

        shell.release("/v1/titles/tv/5/seasons/2", season(2, (1...4).map { "New \($0)" }))
        pump(0.3)
        try press([.ok], on: host)
        XCTAssertEqual(
            shell.went, ["\(Address.watch)tv/5/2/1"],
            "the six Downs pressed while season 2 was read moved the remote on a list that was not season 2's"
        )
        window.isHidden = true
    }

    func testAnAnswerForTheSeasonBeforeDoesNotReplaceTheOneOnScreen() throws {
        let shell = aShell()
        shell.answers["/v1/titles/tv/5/seasons/2"] = season(2, ["Second A", "Second B"])
        // Season 1 is read first, on open, and its answer is held.
        let first = ["First A", "First B"]
        let second = ["Second A", "Second B"]
        let body = season(1, first)

        let (host, window) = open(shell)
        XCTAssertEqual(shell.held.keys.sorted(), ["/v1/titles/tv/5/seasons/1"], "season 1 is being read and has not answered")
        XCTAssertEqual(cards(heard(in: host.view), first + second), [], "nothing has arrived, so no card is drawn")

        try press(chooseSeasonTwo, on: host)
        XCTAssertEqual(
            cards(heard(in: host.view), second), second,
            "season 2 answered at once, so its cards are drawn: if they are not heard, nothing below means anything"
        )

        // Now the answer for season 1, late.
        shell.release("/v1/titles/tv/5/seasons/1", body)
        pump(0.3)
        let found = heard(in: host.view)
        XCTAssertEqual(cards(found, second), second, "a late answer for the season before replaced the season on screen")
        XCTAssertEqual(cards(found, first), [], "the season before is not what the chosen season shows")
        window.isHidden = true
    }

    private func pump(_ seconds: TimeInterval) {
        RunLoop.current.run(until: Date().addingTimeInterval(seconds))
    }
}

@MainActor
private func remoteOf(_ view: UIView) -> RemoteUIView? {
    if let found = view as? RemoteUIView { return found }
    for sub in view.subviews {
        if let found = remoteOf(sub) { return found }
    }
    return nil
}

/// A shell whose reads can be held back and let go in any order, which is the one thing a real
/// link does that a stand-in that answers at once never does.
@MainActor
private final class HeldShell: Shell {
    let seen: Seen
    var answers: [String: String] = [:]
    private(set) var held: [String: (Int, String) -> Void] = [:]
    private(set) var went: [String] = []
    let shown = ToldPictures()
    var pictures: Pictures { shown }

    init(seen: Seen) { self.seen = seen }

    func release(_ path: String, _ body: String) {
        held.removeValue(forKey: path)?(200, body)
    }

    var player: Player { fatalError("no screen under test plays") }
    var reel: Reel { fatalError("no screen under test reels") }
    func retry() {}
    func go(_ address: String) { went.append(address) }
    func swap(_ address: String) {}
    func back() {}
    func act(_ action: Action) {}
    func ask(_ path: String, reply: @escaping (Int, String) -> Void) {
        if let body = answers[path] { reply(200, body) } else { held[path] = reply }
    }
    func send(_ method: String, _ path: String, _ body: String, reply: @escaping (Int, String) -> Void) { reply(0, "") }
    func keep(_ profile: Profile) {}
    func keepAll(_ profiles: [Profile]) {}
    func speak(_ language: String?) {}
    func refreshNotifications() {}
}
#endif
