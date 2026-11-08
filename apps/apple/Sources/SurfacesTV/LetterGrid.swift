import SwiftUI

private let across = 6
private let step = 66
private let cell = 62
private let letters = Array("abcdefghijklmnopqrstuvwxyz1234567890")

/// How many lines the grid has, with space and delete above the letters.
let letterLines = 7

/// A place on the letter grid. Line 0 holds space and delete.
struct LetterPlace: Equatable {
    var line = 1
    var column = 0

    func move(_ word: Word) -> LetterPlace? {
        var next = self
        switch word {
        case .up: if line > 0 { next.line -= 1 } else { return nil }
        case .down: if line < letterLines - 1 { next.line += 1 } else { return nil }
        case .left:
            if line == 0 && column >= across / 2 { next.column = 0 }
            else if line > 0 && column > 0 { next.column -= 1 }
            else { return nil }
        case .right:
            if line == 0 && column < across / 2 { next.column = across / 2 }
            else if line > 0 && column < across - 1 { next.column += 1 }
            else { return nil }
        default: return nil
        }
        return next
    }

    /// Applies this key to the text.
    func press(_ text: String) -> String {
        if line == 0 && column < across / 2 { return text + " " }
        if line == 0 { return String(text.dropLast()) }
        return text + String(letters[(line - 1) * across + column])
    }
}

private struct Cell<Content: View>: View {
    let left: Int
    let top: Int
    let wide: Int
    let on: Bool
    let content: (Color) -> Content

    var body: some View {
        content(on ? Back : Color.white.opacity(0.75))
            .frame(width: CGFloat(wide).ref, height: CGFloat(cell).ref)
            .background(on ? Color.white : Color.white.opacity(0.1), in: shapeOf(Round.key))
            .offset(x: CGFloat(left).ref, y: CGFloat(top).ref)
    }
}

/// Letters and digits, six across. `focus` is nil when the grid has no focus.
struct LetterGrid: View {
    let focus: LetterPlace?

    var body: some View {
        let half = across / 2 * step
        ZStack(alignment: .topLeading) {
            Cell(left: 0, top: 0, wide: half - 4, on: focus?.line == 0 && (focus?.column ?? 0) < across / 2) {
                Icon("icon_space", 34, tint: $0)
            }
            Cell(left: half, top: 0, wide: half - 4, on: focus?.line == 0 && (focus?.column ?? 0) >= across / 2) {
                Icon("icon_erase", 34, tint: $0)
            }
            ForEach(Array(letters.enumerated()), id: \.offset) { index, letter in
                let line = index / across + 1
                let column = index % across
                Cell(left: column * step, top: line * step, wide: cell, on: focus == LetterPlace(line: line, column: column)) {
                    Text(String(letter)).brand(30).foregroundStyle($0)
                }
            }
        }
    }
}
