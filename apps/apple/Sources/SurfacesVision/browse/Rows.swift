import Contract
import Ports
import SwiftUI

/// One tile in a row: a 16:9 still, its badges, a progress bar when the viewer has started it, and
/// **the title's name, drawn, always**.
///
/// The name is the correction worth writing down. Both references put nothing but artwork in a
/// closed tile and reveal the title's name on focus or hover -- the television swaps in a
/// landscape still with the treatment composited on it, and the web page opens a card 556 ms after
/// the pointer arrives (`reference/netflix-on-the-web.md`). Neither is available here, and the
/// second is not merely unavailable but wrong: **on a surface whose pointer is the viewer's gaze,
/// anything revealed on hover is revealed only to whatever is being looked at.** A person scanning
/// a page learns nothing from the other eleven tiles, and a person using Dwell Control or Pointer
/// Control pays a dwell to read a label. So the name is drawn on every tile at rest.
///
/// It costs a line of text under each still and it is the right trade: it is also what makes the
/// row legible to somebody who cannot aim their eyes at all.
///
/// The tile is a control and nothing inside it is. Art marks itself, the badges and the bar are
/// decoration, and the drawn name is gathered rather than spoken twice, because a reader stopping
/// four times on one tile has heard a list instead of a tile. What it says is ``tileWords(_:_:)``.
///
/// No `on` and no focus state. See ``Reach``.
struct TileCard: SwiftUI.View {
    let pictures: Pictures
    let tile: Tile
    let words: [String: String]
    let box: CGSize
    let open: (TitleKey) -> Void

    var body: some SwiftUI.View {
        Button {
            open(tile.title)
        } label: {
            VStack(alignment: .leading, spacing: 8) {
                ZStack(alignment: .bottomLeading) {
                    if let url = wide(tile.images) {
                        Art(pictures, url, box: box)
                    }
                    VStack(alignment: .leading, spacing: 6) {
                        TileBadges(tile, words)
                        if let percent = tile.percent, percent > 0 {
                            Progress(percent: percent, width: box.width - 16)
                        }
                    }
                    .padding(8)
                }
                .frame(width: box.width, height: box.height)
                .plate(Round.card)
                .clipShape(RoundedRectangle(cornerRadius: Round.card, style: .continuous))
                // Drawn at rest, never on hover. See this type's own note.
                Text(tile.name)
                    .brand(15, 600)
                    .foregroundStyle(Ink)
                    .lineLimit(2, reservesSpace: true)
                    .multilineTextAlignment(.leading)
                    .frame(width: box.width, alignment: .leading)
            }
        }
        .buttonStyle(.plain)
        .reachable(says: tileWords(tile, words), corner: Round.card)
    }
}

/// How far through a title the viewer is, drawn as a bar.
///
/// Decoration, and that is not an omission: the percentage is already in the tile's words through
/// ``tileWords(_:_:)``, where it is read after the name, so a reader hears it once.
private struct Progress: SwiftUI.View {
    let percent: Int
    let width: CGFloat

    var body: some SwiftUI.View {
        let share = max(0, min(100, CGFloat(percent))) / 100
        return ZStack(alignment: .leading) {
            Capsule().fill(.white.opacity(0.3)).frame(width: width, height: 4)
            Capsule().fill(Join).frame(width: width * share, height: 4)
        }
        .decoration()
    }
}

/// One row: its title, and the tiles that run off the side of the window.
///
/// The row scrolls and nothing is pinned. That is the television's one fact that cannot be carried
/// over -- `reference/netflix-on-the-tv.md` records the focused row held in a fixed y 199-708 band
/// in 24 of 24 sampled frames, which only means anything where the app moves the focus itself. Here
/// the viewer looks wherever they look, so there is no band to hold anything in, and a page that
/// scrolled itself under a gaze it cannot see would move the thing being looked at.
struct Row: SwiftUI.View {
    let pictures: Pictures
    let section: Contract.Section
    let words: [String: String]
    let width: CGFloat
    let open: (TitleKey) -> Void
    /// Runs when the last tile of this row is drawn, so a page can read the rest of itself.
    var reached: (() -> Void)?

    var body: some SwiftUI.View {
        let tiles = section.tiles ?? []
        let box = tileBox(width)
        return VStack(alignment: .leading, spacing: titleGap) {
            Text(section.title ?? "")
                .brand(19, 700)
                .foregroundStyle(Ink)
                .accessibilityAddTraits(.isHeader)
                .padding(.horizontal, pageEdge(width))
            ScrollView(.horizontal) {
                LazyHStack(spacing: gutter) {
                    ForEach(Array(tiles.enumerated()), id: \.offset) { index, tile in
                        TileCard(pictures: pictures, tile: tile, words: words, box: box, open: open)
                            .onAppear { if index == tiles.count - 1 { reached?() } }
                    }
                }
                // The row's tiles start on the page margin, as everything else on the page does,
                // and the scroller itself runs to both edges so a tile slides in from outside the
                // window rather than appearing at the margin.
                .padding(.horizontal, pageEdge(width))
            }
            .scrollIndicators(.hidden)
            // The highlight the system draws sits slightly outside the tile, so a row that clipped
            // its own contents would cut the top and bottom off whatever is being looked at.
            .scrollClipDisabled()
        }
    }
}
