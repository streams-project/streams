import Contract
import SwiftUI

/// Posters in one line of the grid.
let postersAcross = 4

private let wide: CGFloat = 296
private let high: CGFloat = 414
private let gapX: CGFloat = 10
private let gapY: CGFloat = 10
private let move = 0.24

/// Moves a place in the grid. Answers the new place, or the same one at an edge.
func gridMove(_ at: Int, _ word: Word, _ last: Int) -> Int {
    switch word {
    case .up: return at >= postersAcross ? at - postersAcross : at
    case .down: return at + postersAcross <= last ? at + postersAcross : at
    case .left: return at % postersAcross > 0 ? at - 1 : at
    case .right: return at % postersAcross < postersAcross - 1 && at < last ? at + 1 : at
    default: return at
    }
}

/// Portrait posters, four across. `focus` is -1 when the grid has no focus.
struct PosterGrid: View {
    let tiles: [Tile]
    let focus: Int
    let left: CGFloat
    let top: CGFloat
    let words: [String: String]

    var body: some View {
        let line = max(focus, 0) / postersAcross
        let shape = shapeOf(Round.card)
        ZStack(alignment: .topLeading) {
            ForEach(Array(tiles.enumerated()), id: \.offset) { index, tile in
                let row = index / postersAcross
                if row >= line - 1 && row <= line + 2 {
                    ZStack(alignment: .bottomLeading) {
                        Raised
                        Art(tall(tile.images))
                        TileBadges(tile, words).padding(12.ref)
                    }
                    .frame(width: wide.ref, height: high.ref)
                    .clipShape(shape)
                    .overlay(shape.stroke(.white, lineWidth: index == focus ? 4.ref : 0))
                    // The focus is drawn as a white outline, which says nothing to a person
                    // listening, so the poster the remote is on says it is the selected one.
                    .says(tile.name, on: index == focus)
                    .offset(
                        x: (left + CGFloat(index % postersAcross) * (wide + gapX)).ref,
                        y: (CGFloat(row) * (high + gapY)).ref
                    )
                }
            }
        }
        .offset(y: (top - CGFloat(line) * (high + gapY)).ref)
        .animation(glide(move), value: line)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }
}
