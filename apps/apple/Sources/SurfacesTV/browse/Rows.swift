import Contract
import CoreText
import Ports
import SwiftUI

private let step: CGFloat = 328
private let ahead = 5
private let nextPosters = 6
private let slide = 0.24
private let fade = 0.2

/// The open card, and the gutter between it and the poster beside it.
private let openWide: CGFloat = 904
private let openHigh: CGFloat = 508
private let gutter: CGFloat = 17

/// A ranked row keeps the same tile and widens the gutter, so each rank numeral has its own room:
/// pitch 457 instead of 328, and the row starts further in to leave the first numeral its place.
private let rankedStep: CGFloat = 457
private let rankedLead: CGFloat = 199
private let rankedGutter: CGFloat = 146

/// The numeral: the gutter it is drawn in, how tall its ink is, how far its right edge stands before
/// the tile it counts, how thick its outline is, and where its top sits.
private let rankWide: CGFloat = 146
private let rankHigh: CGFloat = 219
private let rankGap: CGFloat = 8
private let rankLine: CGFloat = 5
private let rankTop: CGFloat = 204

/// The curve every move on this page uses.
public func glide(_ seconds: Double) -> Animation {
    .timingCurve(0.22, 0.61, 0.2, 1, duration: seconds)
}

private let cardFoot = LinearGradient(colors: [.clear, .black.opacity(0.7)], startPoint: .top, endPoint: .bottom)

struct Poster: View {
    let tile: Tile
    let words: [String: String]

    var body: some View {
        ZStack(alignment: .bottomLeading) {
            Raised
            if let url = tall(tile.images) { Art(url) }
            TileBadges(tile, words).padding(12.ref)
        }
        .frame(width: 311.ref, height: 504.ref)
        .clipShape(shapeOf(Round.card))
    }
}

/// The card stays still and its art crossfades.
private struct OpenCard: View {
    let tile: Tile
    let words: [String: String]
    /// The title's own treatment, when the catalogue has one for it.
    let mark: String?
    let lead: CGFloat

    var body: some View {
        let shape = shapeOf(Round.card)
        ZStack(alignment: .bottomLeading) {
            Raised
            Art(wide(tile.images))
                .id(tile.title)
                .transition(.opacity.animation(.linear(duration: slide)))
            cardFoot.frame(height: 200.ref)
            // The reference carries the title's own treatment on the still and never a line of text
            // over it. The name is what is left when the catalogue has no treatment for this title.
            if let mark {
                Logo(url: mark, width: 305, height: 132)
                    .padding(.leading, 24.ref)
                    .padding(.bottom, 24.ref)
            } else {
                Text(tile.name)
                    .brand(44, 800)
                    .foregroundStyle(.white)
                    .lineLimit(1)
                    .frame(width: 856.ref, alignment: .leading)
                    .padding(.leading, 24.ref)
                    .padding(.bottom, 16.ref)
            }
        }
        .frame(width: openWide.ref, height: openHigh.ref)
        // A badge sits on the other corner from the treatment.
        .overlay(alignment: .bottomTrailing) {
            TileBadges(tile, words).padding(.trailing, 24.ref).padding(.bottom, 24.ref)
        }
        .clipShape(shape)
        .overlay(shape.stroke(.white, lineWidth: 3.ref))
        .offset(x: lead.ref, y: 200.ref)
    }
}

/// The rank a ranked row gives a tile: the one the answer carries, or its place in the row.
private func rankOf(_ tile: Tile, _ index: Int) -> Int {
    (tile.badges ?? []).compactMap(\.rank).first ?? index + 1
}

/// The same tile without the rank badge, for a row that draws the rank as a numeral instead.
private func unranked(_ tile: Tile) -> Tile {
    var plain = tile
    plain.badges = (tile.badges ?? []).filter { $0.kind != .ranked }
    return plain
}

/// A rank numeral, an outline with no fill, standing in the gutter before the tile it counts.
/// It is drawn under the tiles, so a number too wide for the gutter tucks behind the one before.
private struct RankMark: View {
    let rank: Int
    /// All the room there is to the left of the numeral's right edge. The brand's figures are wider
    /// than the reference's, so a wide one leans into the gutter behind the tile before it; one that
    /// would leave the screen altogether is drawn smaller instead.
    let room: CGFloat

    var body: some View {
        RankShape(text: number(rank), room: room)
            .stroke(.white.opacity(0.5), lineWidth: rankLine.ref)
            .frame(width: rankWide.ref, height: rankHigh.ref)
    }
}

/// The brand's own figures as one path, its ink as tall as the box and its right edge on the box's
/// right. A figure wider than the box hangs off its left.
private struct RankShape: Shape {
    let text: String
    let room: CGFloat

    func path(in rect: CGRect) -> Path {
        // Asking for the brand's text puts its letters in the hands of the process, by name.
        _ = BrandFont.of(10, 700)
        let font = CTFontCreateWithName("Inter-Regular" as CFString, 100, nil)
        var letters = Array(text.utf16)
        var glyphs = [CGGlyph](repeating: 0, count: letters.count)
        guard !letters.isEmpty,
              CTFontGetGlyphsForCharacters(font, &letters, &glyphs, letters.count) else { return Path() }
        var steps = [CGSize](repeating: .zero, count: glyphs.count)
        CTFontGetAdvancesForGlyphs(font, .horizontal, &glyphs, &steps, glyphs.count)
        let whole = CGMutablePath()
        var x: CGFloat = 0
        for (index, glyph) in glyphs.enumerated() {
            if let drawn = CTFontCreatePathForGlyph(font, glyph, nil) {
                whole.addPath(drawn, transform: CGAffineTransform(translationX: x, y: 0))
            }
            x += steps[index].width
        }
        let ink = whole.boundingBox
        guard ink.width > 0, ink.height > 0 else { return Path() }
        // Letters are built upwards, so the turn flips them as it sizes them; the outline is drawn
        // along the line, half of it outside, and the box holds all of it.
        let grow = min((rect.height - rankLine) / ink.height, (room - rankLine) / ink.width)
        var turn = CGAffineTransform(scaleX: grow, y: -grow)
        let sized = ink.applying(turn)
        turn = turn.concatenating(CGAffineTransform(
            translationX: rect.maxX - rankLine / 2 - sized.maxX,
            y: rect.minY + rankLine / 2 - sized.minY
        ))
        guard let turned = whole.copy(using: &turn) else { return Path() }
        return Path(turned)
    }
}

/// The row that owns the screen: its focused title opens wide with the next ones beside it.
struct OpenRow: View {
    let shell: Shell
    let words: [String: String]
    let store: DetailStore
    let section: Contract.Section
    let at: Int
    @State private var detail: TitleDetail?

    var body: some View {
        let tiles = section.tiles ?? []
        let at = min(at, tiles.count - 1)
        let tile = tiles[at]
        // A ranked row counts its tiles in the gutter, so it keeps a wider pitch and starts further in.
        let ranked = section.kind == .ranked
        let lead = ranked ? rankedLead : pageEdge
        let pitch = ranked ? rankedStep : step
        let open = lead + openWide + (ranked ? rankedGutter : gutter)
        func placeOf(_ index: Int) -> CGFloat { open + CGFloat(index - 1 - at) * pitch }
        return ZStack(alignment: .topLeading) {
            Text(section.title ?? "").brand(32, 700).foregroundStyle(.white).offset(x: pageEdge.ref, y: 146.ref)
            if ranked {
                ForEach(at..<min(at + ahead + 1, tiles.count), id: \.self) { index in
                    let edge = index == at ? lead : placeOf(index)
                    RankMark(rank: rankOf(tiles[index], index), room: edge - rankGap)
                        .offset(x: (edge - rankGap - rankWide).ref, y: rankTop.ref)
                }
            }
            if at > 0 {
                Poster(tile: ranked ? unranked(tiles[at - 1]) : tiles[at - 1], words: words)
                    .opacity(0.55)
                    .offset(x: (lead - pitch - 8).ref, y: 202.ref)
            }
            // The poster under the card is drawn too, so a title leaving the card slides out from it.
            ForEach(at..<min(at + ahead + 1, tiles.count), id: \.self) { index in
                Poster(tile: ranked ? unranked(tiles[index]) : tiles[index], words: words)
                    .offset(x: placeOf(index).ref, y: 202.ref)
            }
            OpenCard(
                tile: ranked ? unranked(tile) : tile,
                words: words,
                mark: logo((tile.images ?? []) + (detail?.images ?? [])),
                lead: lead
            )
            VStack(alignment: .leading, spacing: 0) {
                if let detail { Facts(detail, words) }
            }
            .offset(x: lead.ref, y: 728.ref)
            Text(detail?.overview ?? "")
                .brand(30)
                .lines(30, 39)
                .foregroundStyle(.white.opacity(0.8))
                .lineLimit(3)
                .frame(width: 1000.ref, alignment: .leading)
                .offset(x: lead.ref, y: 768.ref)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .animation(glide(slide), value: at)
        .task(id: tile.title) {
            if store.kept[tile.title] == nil { withAnimation(.linear(duration: fade)) { detail = nil } }
            let got = await detailOf(shell, store, tile.title)
            if !Task.isCancelled { withAnimation(.linear(duration: fade)) { detail = got } }
        }
    }
}

/// The next row, dim, cut by the screen's edge.
/// It shows where the viewer left it: the kept tile first, the one before it at the edge, as the open row does.
struct NextRow: View {
    let section: Contract.Section
    let high: Bool
    let words: [String: String]
    var at: Int = 0

    var body: some View {
        let tiles = section.tiles ?? []
        let top: CGFloat = high ? 1001 : 998
        ZStack(alignment: .topLeading) {
            Text(section.title ?? "").brand(32, 700).foregroundStyle(.white)
                .offset(x: pageEdge.ref, y: (high ? 946 : 944).ref)
            if at > 0, let before = tiles[safe: at - 1] {
                Poster(tile: before, words: words).offset(x: (pageEdge - step).ref, y: top.ref)
            }
            ForEach(Array(tiles.dropFirst(at).prefix(nextPosters).enumerated()), id: \.offset) { index, tile in
                Poster(tile: tile, words: words)
                    .offset(x: (pageEdge + CGFloat(index) * step).ref, y: top.ref)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .opacity(0.5)
    }
}
