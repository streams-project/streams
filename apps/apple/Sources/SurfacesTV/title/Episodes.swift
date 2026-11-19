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
    let pictures: Pictures
    let episode: Episode
    let label: String
    let runtime: String?
    let on: Bool

    var body: some View {
        let shape = shapeOf(Round.card)
        ZStack(alignment: .topLeading) {
            ZStack(alignment: .bottomLeading) {
                Raised
                Art(pictures, wide(episode.images), box: CGSize(width: 456, height: 256))
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
                if let runtime { Text("(\(runtime))").brand(24).foregroundStyle(Muted) }
            }
            .frame(width: 590.ref, alignment: .leading)
            .offset(x: wordsX.ref, y: 44.ref)
        }
        .frame(width: 1920.ref, height: 256.ref, alignment: .topLeading)
        // The card draws its number, its name, what it is about and how long it runs: all of it is
        // words already, so the card gathers them rather than being given a label of its own. The
        // still behind them is decoration (`Art` marks itself) and the white outline that says
        // where the remote is cannot be seen, so `on` says it.
        .saysItsWords(on: on)
    }
}

private struct SeasonLine: View {
    let seen: Seen
    let season: SeasonSummary
    let chosen: Bool
    let on: Bool
    /// What the season whose episodes are shown is heard as, after its own words. From `strings/`.
    var chosenWord: String = ""

    var body: some View {
        let ink = on ? Back : Color.white
        HStack {
            Text(seen.say("title.season.number").fill("n", season.number))
                .brand(30).foregroundStyle(ink)
            Spacer()
            Text(seen.say("title.episodes.count").fill("n", season.episodeCount))
                .brand(24).foregroundStyle(ink)
        }
        .padding(.horizontal, 32.ref)
        .frame(width: 546.ref, height: 62.ref)
        .background(on ? Color.white : chosen ? Color.white.opacity(0.16) : .clear, in: Capsule())
        // The line draws the season's number and how many episodes it holds, so it gathers them.
        // Two plates are drawn here and they mean different things: a white one is where the remote
        // is (`on`), a dim one is the season whose episodes are shown (`chosen`). A person listening
        // sees neither, so the first is the selected trait and the second is read after the words.
        .saysItsWords(state: chosen ? chosenWord : "")
        .accessibilityAddTraits(on ? [.isSelected] : [])
    }
}

/// The episodes in hand, and the season the answer that brought them was for.
///
/// The two travel together because they are read together, and because the page used to read them
/// apart. The read in `.task(id:)` left the episodes on screen as they were until the new season
/// answered, so for the whole of a read -- as long as the link is slow -- the cards were the
/// *previous* season's names, stills and descriptions, under a column that already marked the new
/// season as chosen, with nothing on the page saying so. Every viewer saw that on every season
/// change. The rare tail of it was an address: a held Down walks the stale list's bound, and OK
/// then asked for `/watch/<title>/2/7` out of season 1's seven episodes, which the server answers
/// with no source and the viewer reads as "this title cannot play right now" on a title that plays
/// fine.
///
/// So the season travels with its episodes. `cardsOf` is what the page reads, and it hands back
/// nothing at all while the chosen season is being read; the address OK asks for names `season`,
/// the season the list in hand came from, so the number in the address and the card it was read
/// off can never be two different seasons. `season` is 0 before anything has arrived, which no
/// season number is, and `episodes` is empty there.
struct Arrived {
    var season = 0
    var episodes: [Episode] = []
}

/// The cards a season shows: the ones in hand when the answer that brought them was for this
/// season, and none while this season is being read.
///
/// This is the whole of the half that reached every viewer. A read takes as long as the link is
/// slow, and for all of it the page drew the *previous* season's names, stills and descriptions
/// under a column already marking the new season as chosen, because the pair in hand is only
/// replaced when the new answer lands. What is drawn now is what this family draws for any page
/// with nothing to show yet -- `Waiting` takes it from the measured reference: the destination
/// already lit, the body empty, no skeleton, no shimmer, no old content held.
///
/// Everything else on the page reads this and not the pair, so nothing can be walked to, lit or
/// played that is not drawn.
func cardsOf(_ arrived: Arrived, _ season: Int) -> [Episode] {
    arrived.season == season ? arrived.episodes : []
}

/// What OK on a card asks for: the episode under the remote, played from the season it came from.
/// Nothing when the remote is on no card, which is a season being read or one the viewer has
/// walked off the end of.
func watchAddress(_ path: String, _ arrived: Arrived, _ season: Int, _ card: Int) -> String? {
    guard let episode = cardsOf(arrived, season)[safe: card] else { return nil }
    return "\(Address.watch)\(path)/\(arrived.season)/\(episode.number)"
}

/// Where the remote is left on a column of `count` cards: never off either end of it, and on 0
/// when there are none.
///
/// The floor is the whole of it. A season being read is an empty list, whose last index is -1, and
/// a card at -1 is a page drawn a card low with nothing on it lit.
func cardIn(_ card: Int, _ count: Int) -> Int { max(0, min(card, count - 1)) }

/// One step down that column, or the same card at the end of it.
func cardBelow(_ card: Int, _ count: Int) -> Int { cardIn(card + 1, count) }

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
    @State private var arrived = Arrived()

    var body: some View {
        let seasons = data.seasons ?? []
        let path = data.title.rawValue.replacingOccurrences(of: ":", with: "/")
        let number = seasons[safe: chosen]?.number ?? 1
        // The cards of the season the page is on, which is none of them until it answers.
        let episodes = cardsOf(arrived, number)

        func press(_ word: Word) -> Bool {
            switch word {
            case .up: if inList { line = max(0, line - 1) } else { card = max(0, card - 1) }
            case .down: if inList { line = min(seasons.count - 1, line + 1) } else { card = cardBelow(card, episodes.count) }
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
                // The season the card came from, which `cardsOf` has already made the chosen
                // one: naming it here is what keeps the two from being able to disagree.
                } else if let address = watchAddress(path, arrived, number, card) {
                    shell.go(address)
                }
            case .back: leave()
            }
            return true
        }

        // What the remote is on: a season in the column on the left, or an episode card on the
        // right. A season line says how many episodes it holds, which is what a person choosing one
        // needs; an episode card says its number, its name and what it is about.
        let hereIs = inList
            ? sayBoth(
                seasons[safe: line].map { seen.say("title.season.number").fill("n", $0.number) },
                seasons[safe: line].map { seen.say("title.episodes.count").fill("n", $0.episodeCount) }
            )
            : sayBoth(
                episodes[safe: card].map { seen.say("title.episode").fill("n", $0.number) },
                episodes[safe: card]?.name
            )

        return ZStack(alignment: .topLeading) {
            Color.black
            // The facts follow the name, however many lines it takes; the seasons keep their place.
            VStack(alignment: .leading, spacing: 20.ref) {
                Text(data.name).brand(56, 800).lines(56, 62).foregroundStyle(.white).lineLimit(2)
                    .accessibilityAddTraits(.isHeader)
                Facts(data, seen.words)
            }
            .frame(width: 640.ref, alignment: .leading)
            .offset(x: pageEdge.ref, y: 70.ref)
            VStack(alignment: .leading, spacing: 0) {
                ForEach(Array(seasons.enumerated()), id: \.offset) { index, season in
                    SeasonLine(
                        seen: seen, season: season, chosen: index == chosen,
                        on: inList && index == line, chosenWord: seen.say("say.chosen")
                    )
                }
            }
            .offset(x: pageEdge.ref, y: 346.ref)
            ZStack(alignment: .topLeading) {
                ForEach(Array(episodes.enumerated()), id: \.element.number) { index, episode in
                    if index >= card - 1 && index <= card + near {
                        EpisodeCard(
                            pictures: shell.pictures,
                            episode: episode,
                            label: seen.say("title.episode").fill("n", episode.number),
                            runtime: episode.runtimeMin.map { seen.words.counted("title.length.minutes", $0) },
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
        .remote(says: hereIs) { press($0) }
        .onAppear {
            if let now = data.resume?.episode, let at = seasons.firstIndex(where: { $0.number == now.season }) {
                chosen = at
                line = at
            }
        }
        .task(id: number) {
            let brought = await shell.read("/v1/titles/\(path)/seasons/\(number)")
                .flatMap { try? JSONDecoder().decode(Season.self, from: Data($0.utf8)).episodes } ?? []
            // A season the viewer has already left. The read is not stopped by the cancel -- the
            // port's reply resumes it whenever it comes -- so an answer for the season before can
            // land after the answer for the one on screen, and without this it replaces it: the
            // chosen season then has no list at all and the cards stay empty until the viewer
            // changes season again.
            if Task.isCancelled { return }
            arrived = Arrived(season: number, episodes: brought)
            if !placed, let now = data.resume?.episode, number == now.season {
                card = brought.firstIndex { $0.number == now.episode } ?? 0
                placed = true
            } else {
                // Whatever a season answers with, the remote is left on a card that list has.
                card = cardIn(card, brought.count)
            }
        }
    }
}
