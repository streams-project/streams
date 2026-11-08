import Contract
import Ports
import SurfacesShared
import SwiftUI

/// The rows a page shows, in the server's order.
public let rowKinds: Set<SectionKind> = [.row, .ranked, .continue, .grid]

/// Tiles on the reference: standard 119 x 166.6 pt (5 : 7), the first row's tall panels 184.6 x 369.2 pt.
public let tileWide: CGFloat = 119
public let tileHigh: CGFloat = 166.6
public let tallWide: CGFloat = 184.6
public let tallHigh: CGFloat = 369.2

/// Between tiles, and the row's start from the screen's edge (reference: 5.9 and 12.1 pt).
private let tileGap: CGFloat = 6
private let rowStart: CGFloat = 12

/// One row: its name over a sideways list of tiles (reference: the name 34.3 pt high, bold, 18 pt; 4 pt
/// to the tiles; 12 pt to the next row). The page's first row shows tall panels, as the reference's.
/// Top 10 row shows large outlined rank numerals beside each tile. Continue Watching shows progress
/// and action buttons. Row title tap opens the row as a full page grid when it has more tiles.
public struct SectionRow: View {
    public let section: Contract.Section
    public let words: [String: String]
    public let tallTiles: Bool
    public let open: (Tile) -> Void
    public var openInfo: ((Tile) -> Void)? = nil
    public var onRemove: ((Tile) -> Void)? = nil
    public var openGrid: ((Contract.Section) -> Void)? = nil

    @State private var hiddenTitles: Set<String> = []
    @State private var gridOpen: Bool = false

    public init(
        section: Contract.Section,
        words: [String: String],
        tallTiles: Bool,
        open: @escaping (Tile) -> Void,
        openInfo: ((Tile) -> Void)? = nil,
        onRemove: ((Tile) -> Void)? = nil,
        openGrid: ((Contract.Section) -> Void)? = nil
    ) {
        self.section = section
        self.words = words
        self.tallTiles = tallTiles
        self.open = open
        self.openInfo = openInfo
        self.onRemove = onRemove
        self.openGrid = openGrid
    }

    public var body: some View {
        let rawTiles = section.tiles ?? []
        let tiles = rawTiles.filter { !hiddenTitles.contains($0.title.rawValue) }
        let hasMore = section.cursor != nil || rawTiles.count > 10

        if !tiles.isEmpty {
            VStack(alignment: .leading, spacing: 4) {
                if let title = section.title, !title.isEmpty {
                    Button {
                        if hasMore {
                            if let openGrid {
                                openGrid(section)
                            } else {
                                gridOpen = true
                            }
                        }
                    } label: {
                        HStack(spacing: 6) {
                            Text(title)
                                .brand(18, 700)
                                .foregroundStyle(.white)
                                .lineLimit(1)
                            if hasMore {
                                Text("›")
                                    .brand(18, 700)
                                    .foregroundStyle(.white.opacity(0.7))
                            }
                        }
                        .padding(.horizontal, rowStart)
                        .frame(height: 34, alignment: .leading)
                    }
                    .buttonStyle(.plain)
                    .disabled(!hasMore)
                }

                ScrollView(.horizontal, showsIndicators: false) {
                    LazyHStack(spacing: tileGap) {
                        ForEach(Array(tiles.enumerated()), id: \.element.title.rawValue) { index, tile in
                            switch section.kind {
                            case .ranked:
                                let rank = tile.badges?.first(where: { $0.rank != nil })?.rank ?? (index + 1)
                                RankedTileCard(
                                    rank: rank,
                                    tile: tile,
                                    words: words,
                                    open: { open(tile) }
                                )
                            case .continue:
                                ContinueCard(
                                    tile: tile,
                                    words: words,
                                    open: { open(tile) },
                                    openInfo: { (openInfo ?? open)(tile) },
                                    onRemove: {
                                        hiddenTitles.insert(tile.title.rawValue)
                                        onRemove?(tile)
                                    }
                                )
                            default:
                                let rank = tile.badges?.first(where: { $0.rank != nil })?.rank
                                TileCard(
                                    tile: tile,
                                    wide: tallTiles ? tallWide : tileWide,
                                    high: tallTiles ? tallHigh : tileHigh,
                                    rank: rank,
                                    percent: nil,
                                    isTall: tallTiles,
                                    words: words,
                                    open: { open(tile) }
                                )
                            }
                        }
                    }
                    .padding(.horizontal, rowStart)
                }
            }
            .padding(.bottom, 12)
            .fullScreenCover(isPresented: $gridOpen) {
                FullPageGrid(
                    title: section.title ?? "",
                    tiles: tiles,
                    words: words,
                    onClose: { gridOpen = false },
                    open: open
                )
            }
        }
    }
}

/// Top 10 row item: draws a large outlined rank numeral (80-100 pt high) beside the tile,
/// with the poster tile overlapping the numeral on top. Digits follow the language.
private struct RankedTileCard: View {
    let rank: Int
    let tile: Tile
    let words: [String: String]
    let open: () -> Void

    var body: some View {
        let isDouble = rank >= 10
        let itemWide: CGFloat = tileWide + (isDouble ? 34 : 26)
        let numeralWide: CGFloat = isDouble ? 68 : 48
        let numeralHigh: CGFloat = 92
        let rankText = number(rank)

        ZStack(alignment: .bottomLeading) {
            // Large outlined numeral at leading
            RankNumeral(text: rankText, width: numeralWide, height: numeralHigh)

            // Poster tile at trailing, on top of numeral
            HStack {
                Spacer()
                TileCard(
                    tile: tile,
                    wide: tileWide,
                    high: tileHigh,
                    rank: nil,
                    percent: nil,
                    isTall: false,
                    words: words,
                    open: open
                )
            }
            .frame(width: itemWide, height: tileHigh)
        }
        .frame(width: itemWide, height: tileHigh)
    }
}

/// A large outlined numeral with dark interior and grey/white stroke.
private struct RankNumeral: View {
    let text: String
    let width: CGFloat
    let height: CGFloat

    var body: some View {
        ZStack {
            // Outline effect
            ForEach([-1.8, 0, 1.8], id: \.self) { dx in
                ForEach([-1.8, 0, 1.8], id: \.self) { dy in
                    if dx != 0 || dy != 0 {
                        Text(verbatim: text)
                            .font(.system(size: 90, weight: .black))
                            .foregroundStyle(Color(white: 0.62))
                            .offset(x: dx, y: dy)
                    }
                }
            }
            // Dark interior
            Text(verbatim: text)
                .font(.system(size: 90, weight: .black))
                .foregroundStyle(Color(white: 0.12))
        }
        .frame(width: width, height: height, alignment: .leading)
    }
}

/// Continue Watching tile: art with progress bar, plus info and menu buttons under the art.
private struct ContinueCard: View {
    let tile: Tile
    let words: [String: String]
    let open: () -> Void
    let openInfo: () -> Void
    let onRemove: () -> Void

    var body: some View {
        let shape = RoundedRectangle(cornerRadius: Corner.card)
        let percent = tile.percent ?? 0
        let ratio = CGFloat(max(0, min(100, percent))) / 100.0

        VStack(spacing: 0) {
            // Artwork with progress bar
            Button(action: open) {
                ZStack(alignment: .bottomLeading) {
                    Art(tall(tile.images))
                        .frame(width: tileWide, height: 138)
                        .clipped()

                    // Progress bar
                    ZStack(alignment: .leading) {
                        Rectangle()
                            .fill(Color.white.opacity(0.3))
                            .frame(height: 3)
                        Rectangle()
                            .fill(Accent)
                            .frame(width: tileWide * ratio, height: 3)
                    }
                }
            }
            .buttonStyle(.plain)

            // Control strip under the art
            HStack {
                Button(action: openInfo) {
                    Icon("icon_info", 20, tint: .white)
                        .frame(width: 34, height: 34)
                }
                .buttonStyle(.plain)

                Spacer()

                Menu {
                    Button(role: .destructive, action: onRemove) {
                        Text(words["row.continue.remove"] ?? "Remove from row")
                    }
                } label: {
                    Icon("icon_more", 20, tint: .white)
                        .frame(width: 34, height: 34)
                }
            }
            .padding(.horizontal, 4)
            .frame(width: tileWide, height: 38)
            .background(Raised)
        }
        .frame(width: tileWide)
        .background(Raised)
        .clipShape(shape)
    }
}

/// One tile: the title's tall picture, rounded by the brand.
/// Tall first row tiles show the TOP 10 corner badge and lower caption chip.
private struct TileCard: View {
    let tile: Tile
    let wide: CGFloat
    let high: CGFloat
    let rank: Int?
    let percent: Int?
    let isTall: Bool
    let words: [String: String]
    let open: () -> Void

    var body: some View {
        let shape = RoundedRectangle(cornerRadius: Corner.card)

        Button(action: open) {
            ZStack(alignment: .topLeading) {
                Art(SurfacesShared.tall(tile.images))
                    .frame(width: wide, height: high)
                    .clipped()

                if isTall {
                    // "TOP 10" corner badge on tall first row tile
                    if rank != nil || tile.badges?.contains(where: { $0.rank != nil || $0.kind == .ranked }) == true {
                        Text(words["badge.top_10"] ?? "TOP 10")
                            .brand(11, 700)
                            .foregroundStyle(.white)
                            .padding(.horizontal, 7)
                            .padding(.vertical, 4)
                            .background(
                                Accent,
                                in: UnevenRoundedRectangle(
                                    topLeadingRadius: Corner.card,
                                    bottomLeadingRadius: 0,
                                    bottomTrailingRadius: 6,
                                    topTrailingRadius: 0
                                )
                            )
                    }

                    // Caption chip on lower edge ("New Episode", "Watch Now")
                    let caption = (tile.isSeries || tile.badges?.contains(where: { $0.kind == .new }) == true)
                        ? (words["badge.new_episode"] ?? words["badge.new"] ?? "New Episode")
                        : (words["badge.watch_now"] ?? "Watch Now")

                    VStack {
                        Spacer()
                        HStack {
                            Spacer()
                            Text(caption)
                                .brand(11, 700)
                                .foregroundStyle(.white)
                                .padding(.horizontal, 8)
                                .padding(.vertical, 3)
                                .background(Color.black.opacity(0.75), in: RoundedRectangle(cornerRadius: 4))
                            Spacer()
                        }
                        .padding(.bottom, 10)
                    }
                } else {
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

                    if tile.badges?.contains(where: { $0.kind == .new }) == true {
                        Text(words["badge.new"] ?? "New")
                            .brand(10, 700)
                            .foregroundStyle(.white)
                            .padding(.horizontal, 5)
                            .padding(.vertical, 2)
                            .background(Accent, in: RoundedRectangle(cornerRadius: 4))
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
            }
            .frame(width: wide, height: high)
            .background(Raised)
            .clipShape(shape)
        }
        .buttonStyle(.plain)
    }
}

/// Full-page grid opened when row title is tapped.
private struct FullPageGrid: View {
    let title: String
    let tiles: [Tile]
    let words: [String: String]
    let onClose: () -> Void
    let open: (Tile) -> Void

    private let columns = [
        GridItem(.flexible(), spacing: tileGap),
        GridItem(.flexible(), spacing: tileGap),
        GridItem(.flexible(), spacing: tileGap),
    ]

    var body: some View {
        ZStack {
            Back.ignoresSafeArea()

            VStack(spacing: 0) {
                // Top bar
                HStack(spacing: 8) {
                    Button(action: onClose) {
                        Icon("icon_back", 24, tint: .white)
                            .frame(width: 48, height: 44)
                    }
                    .buttonStyle(.plain)

                    Text(title)
                        .brand(20, 700)
                        .foregroundStyle(.white)
                        .lineLimit(1)

                    Spacer()
                }
                .padding(.horizontal, 4)
                .frame(height: 52)

                // 3-column vertical grid
                ScrollView {
                    LazyVGrid(columns: columns, spacing: tileGap) {
                        ForEach(tiles, id: \.title.rawValue) { tile in
                            Button {
                                onClose()
                                open(tile)
                            } label: {
                                ZStack {
                                    Raised
                                    if let url = tall(tile.images) {
                                        Art(url)
                                    }
                                }
                                .aspectRatio(tileWide / tileHigh, contentMode: .fit)
                                .clipShape(RoundedRectangle(cornerRadius: Corner.card))
                            }
                            .buttonStyle(.plain)
                        }
                    }
                    .padding(.horizontal, rowStart)
                    .padding(.vertical, 8)
                }
            }
        }
    }
}
