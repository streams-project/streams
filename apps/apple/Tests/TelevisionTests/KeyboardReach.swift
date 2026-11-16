#if canImport(UIKit)
import Heard
import SwiftUI
import XCTest

@testable import SurfacesTV

/// The stop where this keyboard's key map could be reached from, pressed on the two pages whose
/// fourth row is short.
///
/// The twin of `apps/android/surfaces/tv/src/test/kotlin/streams/tv/KeyboardReachTest.kt`, written
/// a day later and for a worse reason: the Kotlin and the Java were fixed on 2026-11-13 and this
/// surface was not, so between then and 2026-11-14 a crash that two families had already closed
/// stayed open on every tvOS device.
///
/// Two of the three pages have a fourth row that does not cover every column. The letters page's is
/// `zxcvbnm'`, eight keys from column 2, which reaches column 9; the symbols page's and the accented
/// page's are seven each, which reach column 8. So on either of those a remote standing on column 9
/// and pressed Down lands on a row with no key at that column, and `press` then asked
/// `first(where:)` for a key that is not there and force-unwrapped the answer.
///
/// It is reachable in eleven presses from where the keyboard opens, on every screen that draws one
/// -- sign in, add profile, edit profile and the lock steps -- so the shortest path to it ends with
/// a viewer typing a password and the screen closing under them. These tests press exactly that.
///
/// Found by neither family's tests. Found while porting this screen to a third surface
/// (`apps/xbox/surfaces/remote/keys.cpp`), whose own test walks every key from every key in all four
/// directions rather than aiming at the stop that is known to be bad.
@MainActor
final class KeyboardReachTests: XCTestCase {
    private func keys() -> KeyboardState { KeyboardState("") }

    private func say(_ keys: KeyboardState, _ words: Word...) {
        for word in words { _ = keys.press(word, "cancel", "done") { _ in } }
    }

    private func say(_ keys: KeyboardState, _ word: Word, times: Int) {
        for _ in 0..<times { _ = keys.press(word, "cancel", "done") { _ in } }
    }

    /// To the marks key on the modifier row, which swaps the symbols page in.
    private func toSymbols(_ keys: KeyboardState) {
        say(keys, .down, .down, .left, .left, .left, .left, .ok)
    }

    /// The far right of a short row holds the remote instead of dropping it.
    ///
    /// Before the fix this did not fail an assertion -- it trapped on a nil force-unwrap, which on a
    /// device is the process ending. The remote lands on the last key of the row it arrived at,
    /// which is where a viewer coming down the right hand edge expects to be.
    func testTheRightHandEdgeOfAShortRowHoldsTheRemote() {
        let keys = keys()
        toSymbols(keys)
        say(keys, .up, .up, .up, .up)
        say(keys, .right, times: 9)
        XCTAssertEqual(keys.row, 0)
        XCTAssertEqual(keys.column, 9)
        say(keys, .down, .down, .down)
        XCTAssertEqual(keys.row, 3)
        say(keys, .ok)
        XCTAssertEqual(
            keys.text, "~",
            "the last key of the short row, which is the one under the remote")
    }

    /// The same on the accented page, whose fourth row is short for the same reason.
    func testTheAccentedPageHoldsItToo() {
        let keys = keys()
        say(keys, .down, .down, .left, .left, .left, .left, .right, .ok)
        say(keys, .up, .up, .up, .up)
        say(keys, .right, times: 9)
        say(keys, .down, .down, .down, .ok)
        XCTAssertEqual(keys.text, "ğ")
    }

    /// And the letters page, whose fourth row does reach column 9, is unchanged by the fix.
    func testTheLettersPageReachesItsLastKeyAsItAlwaysDid() {
        let keys = keys()
        say(keys, .up, .up)
        say(keys, .right, times: 9)
        // Three rows down, not two: row 3 is the ragged one, and on THIS page it is
        // `zxcvbnm'` from column 2, eight keys reaching column 9 exactly. That is why the
        // letters page never crashed and the other two did.
        say(keys, .down, .down, .down, .ok)
        XCTAssertEqual(keys.text, "'")
    }

    /// What a person who cannot see the screen is told, at the stop that used to trap.
    ///
    /// This test failed when it was first written, and the failure was the point: the fix above had
    /// been applied to `press` alone, so the spoken answer still returned nothing at the four
    /// ragged stops. Safe and wrong is not better than unsafe and right -- it is the same defect
    /// with the noise turned off.
    ///
    /// `hereIs` guarded with `rows[safe:]` the whole time this file force-unwrapped eight lines
    /// below it, so the two answers disagreed about whether the question has an answer at all. They
    /// are asserted together here so they cannot drift apart again: the spoken key and the pressed
    /// key are the same key.
    func testTheSpokenKeyAndThePressedKeyAreTheSameKeyAtTheRaggedStop() {
        let keys = keys()
        toSymbols(keys)
        say(keys, .up, .up, .up, .up)
        say(keys, .right, times: 9)
        say(keys, .down, .down, .down)
        let spoken = keys.hereIs("cancel", "done", [:])
        XCTAssertFalse(spoken.isEmpty, "the stop has a key, so it has something to say")
        let selected = heard(
            Keyboard(state: keys, cancel: "cancel", done: "done"),
            size: CGSize(width: 1920, height: 1080)
        ).filter(\.isSelected).map(\.label)
        XCTAssertEqual(selected, [spoken], "the key that lights is the key that speaks")
        say(keys, .ok)
        XCTAssertEqual(keys.text, spoken, "what it said it was, and what it typed")
    }

    /// Every reachable stop across all three pages, both cases, six rows and ten columns lights a
    /// key (`lit(row) >= 0`), and at the four ragged stops it lights the last key of the row.
    func testEveryReachableStopLightsAKey() {
        var seen = Set<String>()
        var queue: [[Word]] = [[]]
        seen.insert("\(keys().page[3]):false:2:4")
        let all: [Word] = [.up, .down, .left, .right, .ok, .back]
        var stops = 0
        var unlit = 0
        var ragged = 0
        var wrongLit = 0
        var head = 0
        while head < queue.count {
            let path = queue[head]
            head += 1
            let from = keys()
            for step in path { _ = from.press(step, "cancel", "done") { _ in } }
            stops += 1
            let at = from.lit(from.row)
            if at < 0 { unlit += 1 }
            if from.row == 3 && from.column == 9 && from.page[3].count == 7 {
                ragged += 1
                if at != 7 { wrongLit += 1 }
            }
            for word in all {
                let next = keys()
                for step in path { _ = next.press(step, "cancel", "done") { _ in } }
                _ = next.press(word, "cancel", "done") { _ in }
                if next.lit(next.row) < 0 { unlit += 1 }
                let key = "\(next.page[3]):\(next.upper):\(next.row):\(next.column)"
                if seen.insert(key).inserted { queue.append(path + [word]) }
            }
        }
        XCTAssertEqual(stops, 360)
        XCTAssertEqual(unlit, 0)
        XCTAssertEqual(ragged, 4)
        XCTAssertEqual(wrongLit, 0)
    }
}
#endif
