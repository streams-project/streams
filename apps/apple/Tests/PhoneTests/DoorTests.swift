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
