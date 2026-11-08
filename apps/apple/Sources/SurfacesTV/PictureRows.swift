import Observation
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
    let row: PictureRow
    let index: Int
    let focus: PictureFocus
    let server: String

    var body: some View {
        let at = focus.column(index)
        let shape = RoundedRectangle(cornerRadius: 16.ref)
        let top = CGFloat(index) * down
        ZStack(alignment: .topLeading) {
            Text(row.label).brand(30).foregroundStyle(.white).multilineTextAlignment(.center).lineLimit(2)
                .frame(width: 300.ref)
                .offset(x: 90.ref, y: (top + 94).ref)
            ForEach(Array(row.pictures.indices.clamped(to: max(0, at - 1)..<(at + ahead + 1))), id: \.self) { picture in
                Art(server + row.pictures[picture])
                    .frame(width: tile.ref, height: tile.ref)
                    .clipShape(shape)
                    .overlay(shape.stroke(.white, lineWidth: index == focus.row && picture == at ? 4.ref : 0))
                    .offset(x: (firstX + CGFloat(picture - at) * across).ref, y: top.ref)
            }
        }
        .animation(glide(move), value: at)
    }
}

/// Rows of pictures by theme, each scrolling sideways, with a label at the left.
struct PictureRows: View {
    let rows: [PictureRow]
    let focus: PictureFocus
    let server: String

    var body: some View {
        ZStack(alignment: .topLeading) {
            ForEach(Array(rows.enumerated()), id: \.offset) { index, row in
                OneRow(row: row, index: index, focus: focus, server: server)
            }
        }
        .offset(y: (firstY - CGFloat(focus.row) * down).ref)
        .animation(glide(move), value: focus.row)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }
}
