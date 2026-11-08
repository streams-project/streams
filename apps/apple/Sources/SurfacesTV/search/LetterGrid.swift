import SwiftUI

private let across = 7
private let step = 66
private let cell = 62

/// The English letters, which every profile can switch to. Each alphabet fills whole lines of seven
/// with the marks titles use, so no line ends half empty: 26 letters and ' and - make four lines.
let englishLetters = "abcdefghijklmnopqrstuvwxyz'-"

/// Behind the 123 key: digits and the marks titles use, two whole lines of seven.
let digitKeys = "1234567890-&:."

/// What a key on the top line does.
enum TopKey: CaseIterable { case space, digits, globe, delete }

/// The keys of the grid: letters seven across under a top line of space, 123 (digits), the globe
/// (when the profile's language has letters of its own) and delete. These equal LetterKeys in
/// LetterGrid.kt.
struct LetterKeys: Equatable {
    let letters: [Character]
    let globe: Bool
    let mirrored: Bool
    /// What the 123 key says: 123, or the alphabet's first letters while the digits show.
    let switchWord: String

    init(_ letters: String, globe: Bool, mirrored: Bool = false, switchWord: String = "123") {
        self.letters = Array(letters)
        self.globe = globe
        self.mirrored = mirrored
        self.switchWord = switchWord
    }

    /// The letter at a place, or nil. A left to right set inside a right to left screen fills each
    /// line from the left (`mirrored`), so English and digits still read from the left.
    func at(_ line: Int, _ column: Int) -> Character? {
        guard line >= 1 else { return nil }
        let index = (line - 1) * across + (mirrored ? across - 1 - column : column)
        return letters.indices.contains(index) ? letters[index] : nil
    }

    /// Lines of the grid, the top line included.
    var lines: Int { 1 + (letters.count + across - 1) / across }

    /// The top line's keys, in order.
    var tops: [TopKey] { globe ? TopKey.allCases : TopKey.allCases.filter { $0 != .globe } }

    /// The top line split into its keys, as column ranges.
    var parts: [ClosedRange<Int>] { globe ? [0...1, 2...3, 4...4, 5...6] : [0...2, 3...4, 5...6] }

    /// The height of the grid in reference pixels.
    var high: Int { lines * step }
}

/// A place on the letter grid. Line 0 holds space, 123, the globe and delete.
struct LetterPlace: Equatable {
    var line = 1
    var column = 0

    /// The top line key at this place, or nil on a letter.
    func top(_ keys: LetterKeys) -> TopKey? {
        guard line == 0 else { return nil }
        return keys.tops[keys.parts.firstIndex { $0.contains(column) } ?? 0]
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
                // On a short line, Down lands on its nearest letter.
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

    /// Applies this key to the text. The 123 key and the globe are the screen's to handle.
    func press(_ text: String, _ keys: LetterKeys) -> String {
        switch top(keys) {
        case .space: return text + " "
        case .delete: return String(text.dropLast())
        case .globe, .digits: return text
        case nil: return keys.at(line, column).map { text + number($0) } ?? text
        }
    }
}

private struct Cell<Content: View>: View {
    let left: Int
    let top: Int
    let wide: Int
    let on: Bool
    @ViewBuilder let content: (Color) -> Content

    var body: some View {
        content(on ? Back : Color.white.opacity(0.75))
            .frame(width: CGFloat(wide).ref, height: CGFloat(cell).ref)
            .background(on ? Color.white : Color.white.opacity(0.1), in: shapeOf(Round.key))
            .offset(x: CGFloat(left).ref, y: CGFloat(top).ref)
    }
}

/// Letters seven across under the top line. `focus` is nil when the grid has no focus.
struct LetterGrid: View {
    let focus: LetterPlace?
    let keys: LetterKeys

    var body: some View {
        ZStack(alignment: .topLeading) {
            ForEach(Array(keys.parts.enumerated()), id: \.offset) { part, columns in
                Cell(left: columns.lowerBound * step, top: 0, wide: columns.count * step - 4,
                     on: focus?.line == 0 && columns.contains(focus?.column ?? -1)) { ink in
                    switch keys.tops[part] {
                    case .space: Icon("icon_space", 34, tint: ink)
                    case .digits: Text(keys.switchWord).brand(26).foregroundStyle(ink).lineLimit(1)
                    case .globe: Icon("icon_globe", 34, tint: ink)
                    case .delete: Icon("icon_erase", 34, tint: ink)
                    }
                }
            }
            ForEach(1..<keys.lines, id: \.self) { line in
                ForEach(0..<across, id: \.self) { column in
                    if let letter = keys.at(line, column) {
                        Cell(left: column * step, top: line * step, wide: cell, on: focus == LetterPlace(line: line, column: column)) {
                            Text(number(letter)).brand(30).foregroundStyle($0)
                        }
                    }
                }
            }
        }
    }
}
