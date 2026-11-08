import Contract
import Ports
import SwiftUI

private let cardX: CGFloat = 776
private let cardY: CGFloat = 140
private let cardStep: CGFloat = 284
private let wordsX: CGFloat = 1268
private let move = 0.24
private let near = 4

private let cardFoot = LinearGradient(
    stops: [.init(color: .clear, location: 0.5), .init(color: .black.opacity(0.7), location: 1)],
    startPoint: .top, endPoint: .bottom
)

private struct EpisodeCard: View {
    let episode: Episode
    let label: String
    let on: Bool

    var body: some View {
        let shape = shapeOf(Round.card)
        ZStack(alignment: .topLeading) {
            ZStack(alignment: .bottomLeading) {
                Raised
                Art(wide(episode.images))
                cardFoot
                Text(label).brand(24, 500).foregroundStyle(.white).padding(.leading, 20.ref).padding(.bottom, 14.ref)
            }
            .frame(width: 456.ref, height: 256.ref)
            .clipShape(shape)
            .overlay(shape.stroke(.white, lineWidth: on ? 4.ref : 0))
            .offset(x: cardX.ref)
            VStack(alignment: .leading, spacing: 8.ref) {
                Text(episode.name).brand(30, 700).foregroundStyle(.white).lineLimit(1)
                Text(episode.overview ?? "").brand(24).lines(24, 32).foregroundStyle(.white.opacity(0.8)).lineLimit(3)
                if let minutes = episode.runtimeMin { Text("(\(minutes)m)").brand(24).foregroundStyle(Muted) }
            }
            .frame(width: 590.ref, alignment: .leading)
            .offset(x: wordsX.ref, y: 44.ref)
        }
        .frame(width: 1920.ref, height: 256.ref, alignment: .topLeading)
    }
}

private struct SeasonLine: View {
    let seen: Seen
    let season: SeasonSummary
    let chosen: Bool
    let on: Bool

    var body: some View {
        let ink = on ? Back : Color.white
        HStack {
            Text(seen.say("title.season.number").fill("n", String(season.number)))
                .brand(30).foregroundStyle(ink)
            Spacer()
            Text(seen.say("title.episodes.count").fill("n", String(season.episodeCount)))
                .brand(24).foregroundStyle(ink)
        }
        .padding(.horizontal, 32.ref)
        .frame(width: 546.ref, height: 62.ref)
        .background(on ? Color.white : chosen ? Color.white.opacity(0.16) : .clear, in: Capsule())
    }
}

/// A series' seasons on the left, and that season's episodes as cards on the right.
struct Episodes: View {
    let shell: Shell
    let seen: Seen
    let data: TitleDetail
    let leave: () -> Void
    // The page opens on the season and episode the viewer is on, so Play next is one press away.
    @State private var chosen = 0
    @State private var inList = false
    @State private var line = 0
    @State private var card = 0
    @State private var placed = false
    @State private var episodes: [Episode] = []

    var body: some View {
        let seasons = data.seasons ?? []
        let path = data.title.rawValue.replacingOccurrences(of: ":", with: "/")
        let number = seasons[safe: chosen]?.number ?? 1

        func press(_ word: Word) -> Bool {
            switch word {
            case .up: if inList { line = max(0, line - 1) } else { card = max(0, card - 1) }
            case .down: if inList { line = min(seasons.count - 1, line + 1) } else { card = max(0, min(episodes.count - 1, card + 1)) }
            // From the cards Left opens the seasons at the one shown; from the seasons it goes back.
            case .left:
                if inList { leave() } else {
                    inList = true
                    line = chosen
                }
            case .right: inList = false
            case .ok:
                if inList {
                    chosen = line
                    card = 0
                    inList = false
                } else if let episode = episodes[safe: card] {
                    shell.go("\(Address.watch)\(path)/\(number)/\(episode.number)")
                }
            case .back: leave()
            }
            return true
        }

        return ZStack(alignment: .topLeading) {
            Color.black
            // The facts follow the name, however many lines it takes; the seasons keep their place.
            VStack(alignment: .leading, spacing: 20.ref) {
                Text(data.name).brand(56, 800).lines(56, 62).foregroundStyle(.white).lineLimit(2)
                Facts(data, seen.words)
            }
            .frame(width: 640.ref, alignment: .leading)
            .offset(x: pageEdge.ref, y: 70.ref)
            VStack(alignment: .leading, spacing: 0) {
                ForEach(Array(seasons.enumerated()), id: \.offset) { index, season in
                    SeasonLine(seen: seen, season: season, chosen: index == chosen, on: inList && index == line)
                }
            }
            .offset(x: pageEdge.ref, y: 346.ref)
            ZStack(alignment: .topLeading) {
                ForEach(Array(episodes.enumerated()), id: \.element.number) { index, episode in
                    if index >= card - 1 && index <= card + near {
                        EpisodeCard(
                            episode: episode,
                            label: seen.say("title.episode").fill("n", String(episode.number)),
                            on: !inList && index == card
                        )
                        .offset(y: (CGFloat(index) * cardStep).ref)
                    }
                }
            }
            .offset(y: (cardY - CGFloat(card) * cardStep).ref)
            .animation(glide(move), value: card)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .remote { press($0) }
        .onAppear {
            if let now = data.resume?.episode, let at = seasons.firstIndex(where: { $0.number == now.season }) {
                chosen = at
                line = at
            }
        }
        .task(id: number) {
            episodes = await shell.read("/v1/titles/\(path)/seasons/\(number)")
                .flatMap { try? JSONDecoder().decode(Season.self, from: Data($0.utf8)).episodes } ?? []
            if !placed, let now = data.resume?.episode, seasons[safe: chosen]?.number == now.season {
                card = episodes.firstIndex { $0.number == now.episode } ?? 0
                placed = true
            }
        }
    }
}
