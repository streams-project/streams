import Contract
import Ports
import SurfacesShared
import SwiftUI

/// The rows a page shows, in the server's order.
let rowKinds: Set<SectionKind> = [.row, .ranked, .continue, .grid]

/// Tiles on the reference: standard 119 x 166.6 pt (5 : 7), the first row's tall panels 184.6 x 369.2 pt.
private let tileWide: CGFloat = 119
private let tileHigh: CGFloat = 166.6
private let tallWide: CGFloat = 184.6
private let tallHigh: CGFloat = 369.2

/// Between tiles, and the row's start from the screen's edge (reference: 5.9 and 12.1 pt).
private let tileGap: CGFloat = 6
private let rowStart: CGFloat = 12

/// One row: its name over a sideways list of tiles (reference: the name 34.3 pt high, bold, 18 pt; 4 pt
/// to the tiles; 12 pt to the next row). The page's first row shows tall panels, as the reference's.
struct SectionRow: View {
    let section: Contract.Section
    let words: [String: String]
    let tallTiles: Bool
    let open: (Tile) -> Void

    var body: some View {
        let tiles = section.tiles ?? []
        if !tiles.isEmpty {
            VStack(alignment: .leading, spacing: 4) {
                if let title = section.title, !title.isEmpty {
                    Text(title)
                        .brand(18, 700)
                        .foregroundStyle(.white)
                        .lineLimit(1)
                        .padding(.horizontal, rowStart)
                        .frame(height: 34, alignment: .leading)
                }

                ScrollView(.horizontal, showsIndicators: false) {
                    LazyHStack(spacing: tileGap) {
                        ForEach(tiles, id: \.title.rawValue) { tile in
                            let rank = tile.badges?.first(where: { $0.rank != nil })?.rank
                            TileCard(
                                tile: tile,
                                wide: tallTiles ? tallWide : tileWide,
                                high: tallTiles ? tallHigh : tileHigh,
                                rank: section.kind == .ranked ? rank : nil,
                                percent: section.kind == .continue ? tile.percent : nil,
                                words: words
                            ) {
                                open(tile)
                            }
                        }
                    }
                    .padding(.horizontal, rowStart)
                }
            }
            .padding(.bottom, 12)
        }
    }
}

/// One tile: the title's tall picture, rounded by the brand. A ranked row marks
/// the rank in the top corner and Continue Watching shows how far the viewer is, both in our blue
/// where the reference uses red.
private struct TileCard: View {
    let tile: Tile
    let wide: CGFloat
    let high: CGFloat
    let rank: Int?
    let percent: Int?
    let words: [String: String]
    let open: () -> Void

    var body: some View {
        let shape = RoundedRectangle(cornerRadius: Corner.card)

        Button(action: open) {
            ZStack(alignment: .topLeading) {
                Art(tall(tile.images))
                    .frame(width: wide, height: high)
                    .clipped()

                if let rank {
                    let text = (words["badge.ranked"] ?? "#{rank}").fill("rank", rank)
                    Text(text)
                        .brand(12, 700)
                        .foregroundStyle(.white)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 2)
                        .background(Accent, in: RoundedRectangle(cornerRadius: 6))
                        .padding(6)
                }

                if let percent {
                    let ratio = CGFloat(max(0, min(100, percent))) / 100.0
                    VStack {
                        Spacer()
                        ZStack(alignment: .leading) {
                            Rectangle()
                                .fill(Color.white.opacity(0.3))
                                .frame(height: 3)
                            Rectangle()
                                .fill(Accent)
                                .frame(width: wide * ratio, height: 3)
                        }
                    }
                }
            }
            .frame(width: wide, height: high)
            .background(Raised)
            .clipShape(shape)
        }
        .buttonStyle(.plain)
    }
}
