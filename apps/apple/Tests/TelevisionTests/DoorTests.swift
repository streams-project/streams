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

@MainActor
final class DoorTests: XCTestCase {
    private let tv = CGSize(width: 1920, height: 1080)

    func testAccountsRequiredHidesLookRoundAndRoutesToSignIn() throws {
        let shell = TVDoorShell(titles: sampleTitles())
        shell.seen.words = tvWords
        shell.seen.accountsRequired = true

        let state = openedState()
        let door = Door(shell: shell, seen: shell.seen, state: state)

        let initial = try drive(door, [])
        let buttons = initial.filter(\.isButton)
        XCTAssertFalse(
            words(buttons).contains(tvWords["nav.home"]!),
            "nav.home must not appear on the showreel card when accounts are required"
        )
        XCTAssertEqual(saying(buttons, tvWords["door.join"]!)?.isSelected, true)

        // With accounts required, Right from Get Started moves directly to the first title card.
        let afterRight = try drive(door, [.right])
        XCTAssertEqual(saying(afterRight.filter(\.isButton), tvWords["door.watch"]!)?.isSelected, true)

        // On an empty trending list, Right from Get Started stays on Get Started and OK opens Sign In.
        let emptyShell = TVDoorShell(titles: [])
        emptyShell.seen.words = tvWords
        emptyShell.seen.accountsRequired = true
        try drive(Door(shell: emptyShell, seen: emptyShell.seen, state: openedState()), [.right, .ok])
        XCTAssertEqual(emptyShell.went, [Address.sign_in])
    }

    func testOpenServerAddsLookRoundAndOpensHomeOnOk() throws {
        let shell = TVDoorShell(titles: sampleTitles())
        shell.seen.words = tvWords
        shell.seen.accountsRequired = false

        let door = Door(shell: shell, seen: shell.seen, state: openedState())

        // Right from Get Started focuses nav.home (.lookRound).
        let onLookRound = try drive(door, [.right])
        let lookRoundButton = try XCTUnwrap(
            saying(onLookRound.filter(\.isButton), tvWords["nav.home"]!),
            "nav.home must appear when accounts are not required"
        )
        XCTAssertTrue(lookRoundButton.isSelected, "Right from Get Started must select nav.home")

        // OK on .lookRound goes to Address.home.
        try drive(Door(shell: shell, seen: shell.seen, state: openedState()), [.right, .ok])
        XCTAssertEqual(shell.went, [Address.home])

        // Left from .lookRound returns to Get Started, and OK opens Sign In.
        let backShell = TVDoorShell(titles: sampleTitles())
        backShell.seen.words = tvWords
        backShell.seen.accountsRequired = false
        let backToJoin = try drive(Door(shell: backShell, seen: backShell.seen, state: openedState()), [.right, .left])
        XCTAssertEqual(saying(backToJoin.filter(\.isButton), tvWords["door.join"]!)?.isSelected, true)
        try drive(Door(shell: backShell, seen: backShell.seen, state: openedState()), [.right, .left, .ok])
        XCTAssertEqual(backShell.went, [Address.sign_in])

        // Right past .lookRound enters title card 1 (.watch).
        let cardShell = TVDoorShell(titles: sampleTitles())
        cardShell.seen.words = tvWords
        cardShell.seen.accountsRequired = false
        let onCard = try drive(Door(shell: cardShell, seen: cardShell.seen, state: openedState()), [.right, .right])
        XCTAssertEqual(saying(onCard.filter(\.isButton), tvWords["door.watch"]!)?.isSelected, true)
    }

    func testFlippingAccountsRequiredTrueSettlesLookRoundBackToJoin() throws {
        let shell = TVDoorShell(titles: sampleTitles())
        shell.seen.words = tvWords
        shell.seen.accountsRequired = false

        let door = Door(shell: shell, seen: shell.seen, state: openedState())
        let host = UIHostingController(rootView: door)
        let window = UIWindow(frame: CGRect(origin: .zero, size: tv))
        window.rootViewController = host
        window.isHidden = false
        window.makeKeyAndVisible()
        host.view.frame = CGRect(origin: .zero, size: tv)
        host.view.setNeedsLayout()
        host.view.layoutIfNeeded()
        RunLoop.current.run(until: Date().addingTimeInterval(0.15))

        let receiver = try XCTUnwrap(remoteOf(host.view))
        _ = receiver.handler?(.right)
        RunLoop.current.run(until: Date().addingTimeInterval(0.05))
        XCTAssertEqual(saying(heard(in: host.view).filter(\.isButton), tvWords["nav.home"]!)?.isSelected, true)

        shell.seen.accountsRequired = true
        host.view.setNeedsLayout()
        host.view.layoutIfNeeded()
        RunLoop.current.run(until: Date().addingTimeInterval(0.15))

        let afterFlip = heard(in: host.view).filter(\.isButton)
        XCTAssertFalse(words(afterFlip).contains(tvWords["nav.home"]!))
        XCTAssertEqual(saying(afterFlip, tvWords["door.join"]!)?.isSelected, true)

        let updatedReceiver = try XCTUnwrap(remoteOf(host.view))
        _ = updatedReceiver.handler?(.ok)
        RunLoop.current.run(until: Date().addingTimeInterval(0.05))
        XCTAssertEqual(shell.went, [Address.sign_in])
        window.isHidden = true
    }

    @discardableResult
    private func drive(_ door: Door, _ wordsToPress: [Word], settle: TimeInterval = 0.15) throws -> [Heard] {
        let host = UIHostingController(rootView: door)
        let window = UIWindow(frame: CGRect(origin: .zero, size: tv))
        window.rootViewController = host
        window.isHidden = false
        window.makeKeyAndVisible()
        host.view.frame = CGRect(origin: .zero, size: tv)
        host.view.setNeedsLayout()
        host.view.layoutIfNeeded()
        RunLoop.current.run(until: Date().addingTimeInterval(settle))
        for word in wordsToPress {
            let receiver = try XCTUnwrap(remoteOf(host.view), "the landing has no remote receiver")
            _ = receiver.handler?(word)
            host.view.setNeedsLayout()
            host.view.layoutIfNeeded()
            RunLoop.current.run(until: Date().addingTimeInterval(0.05))
        }
        let found = heard(in: host.view)
        window.isHidden = true
        return found
    }

    private func openedState() -> OpeningState {
        let state = OpeningState()
        state.over()
        return state
    }

    private func sampleTitles() -> [TitleDetail] {
        [
            TitleDetail(
                images: [Contract.Image(role: .backdrop, url: "https://example.invalid/back1.jpg")],
                name: "North Road",
                overview: "A title card.",
                title: try! TitleKey("movie:1"),
                year: 2025
            )
        ]
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

@MainActor
private final class TVDoorShell: Shell {
    let seen = Seen()
    let shown = ToldPictures(.refuse("no test here draws pictures"))
    private let noReel = SilentReel()
    private let titles: [TitleDetail]
    private(set) var went: [String] = []

    init(titles: [TitleDetail]) {
        self.titles = titles
    }

    var pictures: Pictures { shown }
    var player: Player { fatalError("no screen under test plays") }
    var reel: Reel { noReel }
    func retry() {}
    func go(_ address: String) { went.append(address) }
    func swap(_ address: String) { went.append(address) }
    func back() {}
    func act(_ action: Action) {}
    func ask(_ path: String, reply: @escaping (Int, String) -> Void) {
        guard path == "/v1/public/trending" else { return reply(0, "") }
        let body = String(data: try! JSONEncoder().encode(PublicTitles(titles: titles)), encoding: .utf8)!
        reply(200, body)
    }
    func send(_ method: String, _ path: String, _ body: String, reply: @escaping (Int, String) -> Void) { reply(0, "") }
    func keep(_ profile: Profile) {}
    func keepAll(_ profiles: [Profile]) {}
    func speak(_ language: String?) {}
    func refreshNotifications() {}
}

private final class SilentReel: Reel {
    func picture(_ context: Any, url: String, loop: Bool, ready: @escaping () -> Void, ended: @escaping () -> Void) -> Any {
        DispatchQueue.main.async { ready() }
        return UIView()
    }
    func play(_ picture: Any, _ on: Bool) {}
    func release(_ picture: Any) {}
    func hold(_ still: Bool) {}
}
#endif
