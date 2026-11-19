#if canImport(UIKit)
import Contract
import Heard
import Ports
import Stand
import SurfacesShared
import SwiftUI
import UIKit
import XCTest

@testable import SurfacesPhone

@MainActor
final class DoorTests: XCTestCase {
    private let phone = CGSize(width: 393, height: 852)

    func testAccountsRequiredHidesHomeButtonOnPhoneDoor() {
        let shell = PhoneDoorShell()
        shell.seen.words = phoneWords
        shell.seen.accountsRequired = true

        let found = heard(Door(shell: shell, seen: shell.seen), size: phone)
        XCTAssertEqual(silent(found), [])
        XCTAssertEqual(assetNames(found), [])
        XCTAssertTrue(
            words(found).contains(phoneWords["door.sign_in"]!),
            "Sign In is still shown when accounts are required"
        )
        XCTAssertFalse(
            words(found).contains(phoneWords["nav.home"]!),
            "nav.home must be hidden when accounts are required"
        )
    }

    func testOpenServerShowsHomeButtonOnPhoneDoorAndNavigatesHome() throws {
        let shell = PhoneDoorShell()
        shell.seen.words = phoneWords
        shell.seen.accountsRequired = false

        let found = heard(Door(shell: shell, seen: shell.seen), size: phone)
        XCTAssertEqual(silent(found), [])
        XCTAssertEqual(assetNames(found), [])
        XCTAssertTrue(words(found).contains(phoneWords["door.sign_in"]!))
        let homeButton = try XCTUnwrap(
            saying(found, phoneWords["nav.home"]!),
            "nav.home must be shown when accounts are not required"
        )
        XCTAssertTrue(homeButton.isButton)
        XCTAssertGreaterThanOrEqual(homeButton.frame.height, 52)

        XCTAssertTrue(
            activate(Door(shell: shell, seen: shell.seen), size: phone, label: phoneWords["nav.home"]!),
            "nav.home button could not be activated"
        )
        XCTAssertEqual(shell.went, [Address.home])
    }

    /// What the server's hello says reaches the door: the same `Seen.take` the host calls when a
    /// hello arrives, driven with the flag true, false and absent. Absent is the contract's "accounts
    /// are required", so only an explicit false shows Home to somebody with no account.
    func testTheHelloDecidesWhetherAVisitorIsOfferedHome() throws {
        for (flag, offered) in [(true, false), (false, true), (nil, false)] as [(Bool?, Bool)] {
            let shell = PhoneDoorShell()
            shell.seen.words = phoneWords
            shell.seen.take(try aHello(accountsRequired: flag))
            let found = heard(Door(shell: shell, seen: shell.seen), size: phone)
            XCTAssertTrue(words(found).contains(phoneWords["door.sign_in"]!), "Sign In is always there (flag \(String(describing: flag)))")
            XCTAssertEqual(
                words(found).contains(phoneWords["nav.home"]!), offered,
                "accounts_required \(String(describing: flag)): Home offered must be \(offered)"
            )
        }
    }

    /// Before any hello has arrived the door must not offer Home: a visitor is let in only by a
    /// server that said so.
    func testBeforeAnyHelloTheDoorAsksForAnAccount() {
        let shell = PhoneDoorShell()
        shell.seen.words = phoneWords
        XCTAssertTrue(shell.seen.accountsRequired)
        XCTAssertFalse(words(heard(Door(shell: shell, seen: shell.seen), size: phone)).contains(phoneWords["nav.home"]!))
    }
}

/// The hello fixture the contract tests hold (`platform/contract/fixtures`), with its one feature
/// set the way the test says. Read from the file, so the hello is decoded off the wire shape a
/// server sends and not built by hand, and `nil` really leaves the key out of the JSON.
private func aHello(accountsRequired: Bool?) throws -> Hello {
    let repository = URL(fileURLWithPath: #filePath)
        .deletingLastPathComponent()  // PhoneTests
        .deletingLastPathComponent()  // Tests
        .deletingLastPathComponent()  // apple
        .deletingLastPathComponent()  // apps
        .deletingLastPathComponent()  // the repository
    let file = repository.appendingPathComponent("platform/contract/fixtures/2026-11-05/valid/Hello.basic.json")
    var json = try XCTUnwrap(try JSONSerialization.jsonObject(with: Data(contentsOf: file)) as? [String: Any])
    var features = try XCTUnwrap(json["features"] as? [String: Any])
    if let accountsRequired { features["accounts_required"] = accountsRequired } else { features.removeValue(forKey: "accounts_required") }
    json["features"] = features
    return try JSONDecoder().decode(Hello.self, from: JSONSerialization.data(withJSONObject: json))
}

@MainActor
private func activate<V: SwiftUI.View>(_ view: V, size: CGSize, label: String) -> Bool {
    _ = heard(view, size: size)
    let host = UIHostingController(rootView: view)
    let window = UIWindow(frame: CGRect(origin: .zero, size: size))
    window.rootViewController = host
    window.isHidden = false
    window.makeKeyAndVisible()
    host.view.frame = CGRect(origin: .zero, size: size)
    host.view.setNeedsLayout()
    host.view.layoutIfNeeded()
    RunLoop.current.run(until: Date().addingTimeInterval(0.1))
    let target = findAccessibilityElement(in: host.view, label: label)
    let activated = target?.accessibilityActivate() ?? false
    RunLoop.current.run(until: Date().addingTimeInterval(0.05))
    window.isHidden = true
    return activated
}

@MainActor
private func findAccessibilityElement(in node: AnyObject, label: String) -> NSObject? {
    guard let obj = node as? NSObject else { return nil }
    if obj.isAccessibilityElement && obj.accessibilityLabel == label {
        return obj
    }
    if let elements = obj.accessibilityElements {
        for child in elements {
            if let found = findAccessibilityElement(in: child as AnyObject, label: label) {
                return found
            }
        }
    }
    if let view = obj as? UIView {
        for sub in view.subviews {
            if let found = findAccessibilityElement(in: sub, label: label) {
                return found
            }
        }
    }
    return nil
}

@MainActor
private final class PhoneDoorShell: Shell {
    let seen = Seen()
    let shown = ToldPictures()
    private(set) var went: [String] = []

    var pictures: Pictures { shown }
    var player: Player { fatalError("no screen under test plays") }
    var reel: Reel { fatalError("no screen under test reels") }
    func retry() {}
    func go(_ address: String) { went.append(address) }
    func swap(_ address: String) { went.append(address) }
    func back() {}
    func act(_ action: Action) {}
    func ask(_ path: String, reply: @escaping (Int, String) -> Void) { reply(0, "") }
    func send(_ method: String, _ path: String, _ body: String, reply: @escaping (Int, String) -> Void) { reply(0, "") }
    func keep(_ profile: Profile) {}
    func keepAll(_ profiles: [Profile]) {}
    func speak(_ language: String?) {}
    func refreshNotifications() {}
}
#endif
