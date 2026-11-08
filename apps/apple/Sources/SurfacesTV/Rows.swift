import Contract
import Ports
import SwiftUI

private let step: CGFloat = 328
private let ahead = 5
private let nextPosters = 6
private let slide = 0.24
private let fade = 0.2

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

    var body: some View {
        let shape = shapeOf(Round.card)
        ZStack(alignment: .bottomLeading) {
            Raised
            Art(wide(tile.images))
                .id(tile.title)
                .transition(.opacity.animation(.linear(duration: slide)))
            cardFoot.frame(height: 200.ref)
            TileBadges(tile, words).padding(.leading, 24.ref).padding(.bottom, 78.ref)
            Text(tile.name)
                .brand(44, 800)
                .foregroundStyle(.white)
                .lineLimit(1)
                .frame(width: 856.ref, alignment: .leading)
                .padding(.leading, 24.ref)
                .padding(.bottom, 16.ref)
        }
        .frame(width: 904.ref, height: 508.ref)
        .clipShape(shape)
        .overlay(shape.stroke(.white, lineWidth: 3.ref))
        .offset(x: pageEdge.ref, y: 200.ref)
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
        let tile = tiles[min(at, tiles.count - 1)]
        ZStack(alignment: .topLeading) {
            Text(section.title ?? "").brand(32, 700).foregroundStyle(.white).offset(x: pageEdge.ref, y: 146.ref)
            if at > 0 {
                Poster(tile: tiles[at - 1], words: words).opacity(0.55).offset(x: -264.ref, y: 202.ref)
            }
            // The poster under the card is drawn too, so a title leaving the card slides out from it.
            ForEach(at..<min(at + ahead + 1, tiles.count), id: \.self) { index in
                Poster(tile: tiles[index], words: words)
                    .offset(x: (pageEdge + 904 + 19 + CGFloat(index - 1 - at) * step).ref, y: 202.ref)
            }
            OpenCard(tile: tile, words: words)
            VStack(alignment: .leading, spacing: 0) {
                if let detail { Facts(detail, words) }
            }
            .offset(x: pageEdge.ref, y: 728.ref)
            Text(detail?.overview ?? "")
                .brand(30)
                .lines(30, 39)
                .foregroundStyle(.white.opacity(0.8))
                .lineLimit(3)
                .frame(width: 1000.ref, alignment: .leading)
                .offset(x: pageEdge.ref, y: 768.ref)
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
struct NextRow: View {
    let section: Contract.Section
    let high: Bool
    let words: [String: String]

    var body: some View {
        ZStack(alignment: .topLeading) {
            Text(section.title ?? "").brand(32, 700).foregroundStyle(.white)
                .offset(x: pageEdge.ref, y: (high ? 946 : 944).ref)
            ForEach(Array((section.tiles ?? []).prefix(nextPosters).enumerated()), id: \.offset) { index, tile in
                Poster(tile: tile, words: words)
                    .offset(x: (pageEdge + CGFloat(index) * step).ref, y: (high ? 1001 : 998).ref)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .opacity(0.5)
    }
}
