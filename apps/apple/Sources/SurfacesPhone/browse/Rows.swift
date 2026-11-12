import Contract
import Ports
import SurfacesShared
import SwiftUI

/// The rows a page shows, in the server's order.
public let rowKinds: Set<SectionKind> = [.row, .ranked, .continue, .grid]

/// Tiles on the reference: 119 x 166.6 pt (1 : 1.400), every row the same. The first row is not
/// taller (phone/00-netflix-phone-measurements.md section 3, 2026-11-11).
public let tileWide: CGFloat = 119
public let tileHigh: CGFloat = 166.6

/// Between tiles, and the row's start from the screen's edge (reference: 5.9 and 12.1 pt).
private let tileGap: CGFloat = 6
private let rowStart: CGFloat = 12

/// One row: its name over a sideways list of tiles (reference: the name 34.3 pt high, bold, 18 pt; 4 pt
/// to the tiles; 12 pt to the next row). Every row draws the same tile.
/// Top 10 row shows large outlined rank numerals beside each tile. Continue Watching shows progress
/// and action buttons. Row title tap opens the row as a full page grid when it has more tiles.
public struct SectionRow: View {
    public let pictures: Pictures
    public let section: Contract.Section
    public let words: [String: String]
    public let open: (Tile) -> Void
    public var openInfo: ((Tile) -> Void)? = nil
    public var onRemove: ((Tile) -> Void)? = nil
    public var openGrid: ((Contract.Section) -> Void)? = nil

    @State private var hiddenTitles: Set<String> = []
    @State private var gridOpen: Bool = false
    @Environment(\.layoutDirection) private var layoutDirection

    public init(
        pictures: Pictures,
        section: Contract.Section,
        words: [String: String],
        open: @escaping (Tile) -> Void,
        openInfo: ((Tile) -> Void)? = nil,
        onRemove: ((Tile) -> Void)? = nil,
        openGrid: ((Contract.Section) -> Void)? = nil
    ) {
        self.pictures = pictures
        self.section = section
        self.words = words
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
                                .truncationMode(.tail)
                            if hasMore {
                                Text(layoutDirection == .rightToLeft ? "‹" : "›")
                                    .brand(18, 700)
                                    .foregroundStyle(.white.opacity(0.7))
                            }
                        }
                        .padding(.horizontal, rowStart)
                        .frame(height: 34, alignment: .leading)
                    }
                    .buttonStyle(.plain)
                    .disabled(!hasMore)
                    // A row whose name opens the whole row is a control, and the chevron beside the
                    // name is the only thing on screen saying so, so it is given words that say it.
                    // A row that opens nothing keeps its own name and nothing more.
                    .modifier(RowTitleSays(
                        says: hasMore ? (words["say.see_all"] ?? "").fill("row", title) : ""
                    ))
                }

                ScrollView(.horizontal, showsIndicators: false) {
                    LazyHStack(spacing: tileGap) {
                        ForEach(Array(tiles.enumerated()), id: \.element.title.rawValue) { index, tile in
                            switch section.kind {
                            case .ranked:
                                let rank = tile.badges?.first(where: { $0.rank != nil })?.rank ?? (index + 1)
                                RankedTileCard(
                                    pictures: pictures,
                                    rank: rank,
                                    tile: tile,
                                    words: words,
                                    open: { open(tile) }
                                )
                            case .continue:
                                ContinueCard(
                                    pictures: pictures,
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
                                    pictures: pictures,
                                    tile: tile,
                                    wide: tileWide,
                                    high: tileHigh,
                                    rank: rank,
                                    percent: nil,
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
                    pictures: pictures,
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

/// Words for a row's name when it opens the row, and nothing added when it does not.
private struct RowTitleSays: ViewModifier {
    let says: String

    func body(content: Content) -> some View {
        if says.isEmpty {
            content.saysItsWords()
        } else {
            content.says(says)
        }
    }
}

/// Top 10 row item: draws a large outlined rank numeral (80-100 pt high) beside the tile,
/// with the poster tile overlapping the numeral on top. Digits follow the language.
///
/// Internal rather than private so a test can host one: nothing inside a SwiftUI `ScrollView`
/// reaches the accessibility tree in a test process with no scene (measured 2026-11-12), and every
/// tile on this surface is drawn inside one.
struct RankedTileCard: View {
    let pictures: Pictures
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
            // The numeral is drawn as nine offset copies of the same figure plus a tenth over them,
            // so read as it stands it would be said ten times. It is decoration: the place is part
            // of what the tile itself says, through `say.ranked`.
            RankNumeral(text: rankText, width: numeralWide, height: numeralHigh)
                .decoration()

            // Poster tile at trailing, on top of numeral
            HStack {
                Spacer()
                TileCard(
                    pictures: pictures,
                    tile: tile,
                    wide: tileWide,
                    high: tileHigh,
                    rank: nil,
                    percent: nil,
                    words: words,
                    says: (words["say.ranked"] ?? "").fill("rank", rank).fill("title", tile.name),
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
///
/// Internal rather than private for the same reason as `RankedTileCard`.
struct ContinueCard: View {
    let pictures: Pictures
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
                    Art(pictures, tall(tile.images))
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
            // The bar drawn across the picture is how far through the title the viewer is, and it
            // is the one thing on this tile a person listening cannot see at all.
            .says(
                percent > 0
                    ? (words["say.watched"] ?? "").fill("title", tile.name).fill("percent", percent)
                    : tile.name
            )

            // Control strip under the art
            HStack {
                Button(action: openInfo) {
                    Icon("icon_info", 20, tint: .white)
                        .frame(width: 34, height: 34)
                }
                .buttonStyle(.plain)
                .says(words["common.more_info"] ?? "")

                Spacer()

                Menu {
                    Button(role: .destructive, action: onRemove) {
                        Text(words["row.continue.remove"] ?? "Remove from row")
                    }
                } label: {
                    Icon("icon_more", 20, tint: .white)
                        .frame(width: 34, height: 34)
                }
                .says(words["say.more_options"] ?? "")
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
///
/// A tile draws a picture and sometimes a badge over it, and no words of its own. The title's name
/// is the tile: it is the one thing a reader has to hear, and it comes from the answer, so it needs
/// no key. `says` is for a row that has more to say about the tile than its name - its place in a
/// Top 10, how much of it has been watched - and those words do come from `strings/`.
///
/// Internal rather than private for the same reason as `RankedTileCard`.
struct TileCard: View {
    let pictures: Pictures
    let tile: Tile
    let wide: CGFloat
    let high: CGFloat
    let rank: Int?
    let percent: Int?
    let words: [String: String]
    var says: String = ""
    let open: () -> Void

    var body: some View {
        let shape = RoundedRectangle(cornerRadius: Corner.card)

        Button(action: open) {
            ZStack(alignment: .topLeading) {
                Art(pictures, SurfacesShared.tall(tile.images))
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
            .frame(width: wide, height: high)
            .background(Raised)
            .clipShape(shape)
        }
        .buttonStyle(.plain)
        .says(says.isEmpty ? tile.name : says)
    }
}

/// Full-page grid opened when row title is tapped.
private struct FullPageGrid: View {
    let pictures: Pictures
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
                    .says(words["say.close"] ?? "")

                    Text(title)
                        .brand(20, 700)
                        .foregroundStyle(.white)
                        .lineLimit(1)
                        .accessibilityAddTraits(.isHeader)

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
                                        Art(pictures, url)
                                    }
                                }
                                .aspectRatio(tileWide / tileHigh, contentMode: .fit)
                                .clipShape(RoundedRectangle(cornerRadius: Corner.card))
                            }
                            .buttonStyle(.plain)
                            .says(tile.name)
                        }
                    }
                    .padding(.horizontal, rowStart)
                    .padding(.vertical, 8)
                }
            }
        }
    }
}
