import Observation
import SwiftUI

private let key = 63
private let step = 66
private let columnsAcross = 10
private let buttons = 5

/// What a key does. A letter key carries its text.
private enum Press: Equatable {
    case letter(String), shift, marks, accents, space, erase, cancel, done
}

private struct Cap {
    let press: Press
    let column: Int
    let span: Int
    var word: String? = nil
    var icon: String? = nil
    /// Words for a key that draws a picture instead of a word: shift, space, erase. Nothing for a
    /// key that draws its own, which gathers it and so can never drift from what is seen. From
    /// `strings/`, handed down from the screen.
    var says: String? = nil
}

let letterKeys = ["1234567890", "qwertyuiop", "asdfghjkl-", "zxcvbnm'"]
private let symbolKeys = ["1234567890", "!@#$%^&*()", "_=+[]{};:\"", ",.?/\\|~"]
private let accentKeys = ["àáâäãåæçèé", "êëìíîïñòóô", "õöøùúûüýßÿ", "œšžćčđğ"]

private func line(_ letters: String, _ from: Int, _ upper: Bool) -> [Cap] {
    letters.enumerated().map { index, letter in
        let text = upper ? String(letter).uppercased() : String(letter)
        return Cap(press: .letter(text), column: from + index, span: 1, word: text)
    }
}

/// The keys of a page. `words` names only the three that draw a picture; every other key draws its
/// own text and is heard as that.
private func caps(
    _ page: [String], _ upper: Bool, _ cancel: String, _ done: String, _ words: [String: String] = [:]
) -> [[Cap]] {
    [
        line(page[0], 0, upper),
        line(page[1], 0, upper),
        line(page[2], 0, upper),
        [Cap(press: .shift, column: 0, span: 2, icon: "icon_shift", says: words["say.shift"] ?? "")] + line(page[3], 2, upper),
        [
            Cap(press: .marks, column: 0, span: 2, word: "!#$"),
            Cap(press: .accents, column: 2, span: 2, word: "àáâ"),
            Cap(press: .space, column: 4, span: 3, icon: "icon_space", says: words["say.space"] ?? ""),
            Cap(press: .erase, column: 7, span: 3, icon: "icon_erase", says: words["say.delete"] ?? ""),
        ],
        [Cap(press: .cancel, column: 0, span: buttons, word: cancel), Cap(press: .done, column: buttons, span: buttons, word: done)],
    ]
}

/// Where the viewer is on the keys, and the text so far.
@Observable
@MainActor
public final class KeyboardState {
    public var text: String
    var page = letterKeys
    var upper = false
    var row = 2
    var column = 4

    public init(_ start: String) {
        text = start
    }

    /// What the key the remote is on is heard as: the words of a picture key, or the text a key
    /// draws. The state is what knows where the remote is, so it is what answers, and a screen can
    /// ask without holding on to the view it drew.
    func hereIs(_ cancel: String, _ done: String, _ words: [String: String]) -> String {
        let rows = caps(page, upper, cancel, done, words)
        guard let line = rows[safe: row],
              let here = line.first(where: { column >= $0.column && column < $0.column + $0.span })
        else { return "" }
        let said = here.says ?? ""
        return said.isEmpty ? (here.word ?? "") : said
    }

    /// Answers a word from the remote. `finish` gets the text on Done, or nothing on Cancel.
    func press(_ word: Word, _ cancel: String, _ done: String, _ finish: (String?) -> Void) -> Bool {
        let rows = caps(page, upper, cancel, done)
        let here = rows[row].first { column >= $0.column && column < $0.column + $0.span }!
        switch word {
        case .up: if row > 0 { row -= 1 } else { return false }
        case .down: if row < rows.count - 1 { row += 1 }
        case .left: if here.column > 0 { column = here.column - 1 }
        case .right: if here.column + here.span < columnsAcross { column = here.column + here.span }
        case .back: finish(nil)
        case .ok:
            switch here.press {
            case .letter(let letter):
                text += letter
                upper = false
            case .shift: upper.toggle()
            case .marks: page = page == symbolKeys ? letterKeys : symbolKeys
            case .accents: page = page == accentKeys ? letterKeys : accentKeys
            case .space: text += " "
            case .erase: text = String(text.dropLast())
            case .cancel: finish(nil)
            case .done: finish(text)
            }
        }
        return true
    }
}

private struct CapView: View {
    let cap: Cap
    let row: Int
    let on: Bool

    var body: some View {
        let pill = cap.press == .cancel || cap.press == .done
        let top = pill ? 356 : row * step
        let high = pill ? 62 : key
        let left = pill ? (cap.press == .cancel ? 0 : 331) : cap.column * step
        let wide = pill ? 327 : cap.span * step - (step - key)
        let ink = on ? Back : Color.white
        let back = on ? Color.white : Color.white.opacity(0.16)
        ZStack {
            if pill { Capsule().fill(back) } else { shapeOf(Round.key).fill(back) }
            if let word = cap.word { Text(word).brand(26, pill ? 700 : 500).foregroundStyle(ink) }
            if let icon = cap.icon { Icon(icon, 34, tint: ink) }
        }
        .frame(width: CGFloat(wide).ref, height: CGFloat(high).ref)
        // A picture key takes the words it was given; a key that draws its own text falls through to
        // gathering it, which `says("")` does itself. `on` is the white cap, which is the only thing
        // saying where the remote is on a keyboard of sixty keys.
        .says(cap.says ?? "", on: on)
        .offset(x: CGFloat(left).ref, y: CGFloat(top).ref)
    }
}

/// A letter grid for a remote. Search and sign in share it as it stands.
///
/// `words` is for the three keys that draw a picture: shift, space and erase. Without them a person
/// signing in with a remote reaches three keys that read as nothing at all, and two of those are
/// the ones they need most.
struct Keyboard: View {
    let state: KeyboardState
    let cancel: String
    let done: String
    var words: [String: String] = [:]

    var body: some View {
        let rows = caps(state.page, state.upper, cancel, done, words)
        let shape = shapeOf(Round.panel)
        ZStack(alignment: .topLeading) {
            ForEach(Array(rows.enumerated()), id: \.offset) { row, line in
                ForEach(Array(line.enumerated()), id: \.offset) { _, cap in
                    CapView(cap: cap, row: row, on: row == state.row && state.column >= cap.column && state.column < cap.column + cap.span)
                }
            }
            .offset(x: 30.ref, y: 29.ref)
        }
        .frame(width: 722.ref, height: 476.ref, alignment: .topLeading)
        .background(Color.white.opacity(0.08), in: shape)
        .overlay(shape.stroke(Color.white.opacity(0.25), lineWidth: 1.ref))
    }
}
