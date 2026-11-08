import SwiftUI

private let across = 6
private let step = 66
private let cell = 62

/// The English letters, which every profile can switch to.
let englishLetters = "abcdefghijklmnopqrstuvwxyz1234567890"

/// The keys of the grid: letters six across under a top line of space, delete and, when the
/// profile's language has letters of its own, the globe that switches between them and English.
/// These equal LetterKeys in LetterGrid.kt.
struct LetterKeys: Equatable {
    let letters: [Character]
    let globe: Bool
    let mirrored: Bool

    init(_ letters: String, globe: Bool, mirrored: Bool = false) {
        self.letters = Array(letters)
        self.globe = globe
        self.mirrored = mirrored
    }

    /// The letter at a place, or nil. A left to right alphabet inside a right to left screen fills
    /// each line from the left (`mirrored`), so English still reads a to z.
    func at(_ line: Int, _ column: Int) -> Character? {
        guard line >= 1 else { return nil }
        let index = (line - 1) * across + (mirrored ? across - 1 - column : column)
        return letters.indices.contains(index) ? letters[index] : nil
    }

    /// Lines of the grid, the top line included.
    var lines: Int { 1 + (letters.count + across - 1) / across }

    /// The top line split into its keys, as column ranges: space, globe, delete.
    var parts: [ClosedRange<Int>] { globe ? [0...1, 2...3, 4...5] : [0...2, 3...5] }

    /// The height of the grid in reference pixels.
    var high: Int { lines * step }
}

/// What a key on the top line does.
enum TopKey { case space, globe, delete }

/// A place on the letter grid. Line 0 holds space, the globe and delete.
struct LetterPlace: Equatable {
    var line = 1
    var column = 0

    /// The top line key at this place, or nil on a letter.
    func top(_ keys: LetterKeys) -> TopKey? {
        guard line == 0 else { return nil }
        let part = keys.parts.firstIndex { $0.contains(column) } ?? 0
        if part == 0 { return .space }
        return keys.globe && part == 1 ? .globe : .delete
    }

    func move(_ word: Word, _ keys: LetterKeys) -> LetterPlace? {
        func fits(_ place: LetterPlace) -> Bool { place.line == 0 || keys.at(place.line, place.column) != nil }
        var next = self
        switch word {
        case .up:
            guard line > 0 else { return nil }
            next.line -= 1
        case .down:
            next.line += 1
            if !fits(next) {
                guard line + 1 < keys.lines else { return nil }
                // On a short last line, Down lands on its nearest letter.
                let near = (0..<across).filter { fits(LetterPlace(line: line + 1, column: $0)) }
                    .min { abs($0 - column) < abs($1 - column) }
                guard let near else { return nil }
                next.column = near
            }
        case .left:
            if line == 0 {
                let part = keys.parts.firstIndex { $0.contains(column) } ?? 0
                guard part > 0 else { return nil }
                next.column = keys.parts[part - 1].lowerBound
            } else {
                next.column -= 1
                guard column > 0, fits(next) else { return nil }
            }
        case .right:
            if line == 0 {
                let part = keys.parts.firstIndex { $0.contains(column) } ?? 0
                guard part < keys.parts.count - 1 else { return nil }
                next.column = keys.parts[part + 1].lowerBound
            } else {
                next.column += 1
                guard column < across - 1, fits(next) else { return nil }
            }
        default:
            return nil
        }
        return next
    }

    /// Applies this key to the text. The globe is the screen's to handle.
    func press(_ text: String, _ keys: LetterKeys) -> String {
        switch top(keys) {
        case .space: return text + " "
        case .delete: return String(text.dropLast())
        case .globe: return text
        case nil:
            return keys.at(line, column).map { text + String($0) } ?? text
        }
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
    let keys: LetterKeys

    var body: some View {
        ZStack(alignment: .topLeading) {
            ForEach(Array(keys.parts.enumerated()), id: \.offset) { part, columns in
                let icon = keys.globe ? ["icon_space", "icon_globe", "icon_erase"][part] : (part == 0 ? "icon_space" : "icon_erase")
                Cell(left: columns.lowerBound * step, top: 0, wide: columns.count * step - 4,
                     on: focus?.line == 0 && columns.contains(focus?.column ?? -1)) {
                    Icon(icon, 34, tint: $0)
                }
            }
            ForEach(1..<keys.lines, id: \.self) { line in
                ForEach(0..<across, id: \.self) { column in
                    if let letter = keys.at(line, column) {
                        Cell(left: column * step, top: line * step, wide: cell, on: focus == LetterPlace(line: line, column: column)) {
                            Text(String(letter)).brand(30).foregroundStyle($0)
                        }
                    }
                }
            }
        }
    }
}
