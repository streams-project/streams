import Observation
import Ports
import SwiftUI

private let tile: CGFloat = 243
private let across: CGFloat = 270
private let down: CGFloat = 311
private let firstX: CGFloat = 395
private let firstY: CGFloat = 260
private let move = 0.24
private let ahead = 6

/// A themed row of pictures, each with where it sits on the server.
struct PictureRow {
    let label: String
    let pictures: [String]
}

/// Which picture has focus.
@Observable
@MainActor
final class PictureFocus {
    var row = 0
    private var columns: [Int: Int] = [:]

    func column(_ of: Int) -> Int { columns[of] ?? 0 }

    func press(_ word: Word, _ rows: [PictureRow]) -> Bool {
        let at = column(row)
        switch word {
        case .up: if row > 0 { row -= 1 }
        case .down: if row < rows.count - 1 { row += 1 }
        case .left: if at > 0 { columns[row] = at - 1 }
        case .right: if at < rows[row].pictures.count - 1 { columns[row] = at + 1 }
        default: return false
        }
        return true
    }

    func chosen(_ rows: [PictureRow]) -> String? {
        rows[safe: row]?.pictures[safe: column(row)]
    }
}

private struct OneRow: View {
    let pictures: Pictures
    let row: PictureRow
    let index: Int
    let focus: PictureFocus
    let server: String
    /// `say.avatar_in_set`, for a picture that has no words of its own at all.
    let pattern: String

    var body: some View {
        let at = focus.column(index)
        let shape = shapeOf(Round.card)
        let top = CGFloat(index) * down
        ZStack(alignment: .topLeading) {
            // The row's theme, from the server, drawn once at the left. It names the pictures
            // beside it, and each of those says which of the set it is, so the label is read here
            // as the heading over them.
            Text(row.label).brand(30).foregroundStyle(.white).multilineTextAlignment(.center).lineLimit(2)
                .frame(width: 300.ref)
                .accessibilityAddTraits(.isHeader)
                .offset(x: 90.ref, y: (top + 94).ref)
            ForEach(Array(row.pictures.indices.clamped(to: max(0, at - 1)..<(at + ahead + 1))), id: \.self) { picture in
                // A picture of a face and nothing else: no words anywhere on it, and no name for it
                // in the answer either, so without this a person choosing a profile picture reaches
                // a grid of stops that all read as nothing. `say.avatar_in_set` is the words this
                // project already has for the case - its place in the set, and which set.
                Art(
                    pictures,
                    server + row.pictures[picture],
                    says: pattern.isEmpty
                        ? ""
                        : pattern.fill("n", picture + 1).fill("set", row.label)
                )
                    .frame(width: tile.ref, height: tile.ref)
                    .clipShape(shape)
                    .overlay(shape.stroke(.white, lineWidth: index == focus.row && picture == at ? 4.ref : 0))
                    .accessibilityAddTraits(index == focus.row && picture == at ? [.isSelected] : [])
                    .offset(x: (firstX + CGFloat(picture - at) * across).ref, y: top.ref)
            }
        }
        .animation(glide(move), value: at)
    }
}

/// Rows of pictures by theme, each scrolling sideways, with a label at the left.
///
/// `pattern` is `say.avatar_in_set` from `strings/`, handed in by the screen: these pictures are the
/// clearest case on the television of a control with no words available anywhere, since neither the
/// picture nor the answer that lists it carries a name.
struct PictureRows: View {
    let pictures: Pictures
    let rows: [PictureRow]
    let focus: PictureFocus
    let server: String
    var pattern: String = ""

    /// What the picture the remote is on is heard as, for the screen that holds these rows.
    func hereIs() -> String {
        guard !pattern.isEmpty, let row = rows[safe: focus.row] else { return "" }
        return pattern.fill("n", focus.column(focus.row) + 1).fill("set", row.label)
    }

    var body: some View {
        ZStack(alignment: .topLeading) {
            ForEach(Array(rows.enumerated()), id: \.offset) { index, row in
                OneRow(pictures: pictures, row: row, index: index, focus: focus, server: server, pattern: pattern)
            }
        }
        .offset(y: (firstY - CGFloat(focus.row) * down).ref)
        .animation(glide(move), value: focus.row)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }
}
