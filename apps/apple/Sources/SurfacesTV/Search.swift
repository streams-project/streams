import Contract
import Observation
import Ports
import SwiftUI

private let settle: UInt64 = 350
private let offeredCount = 12
private let genreCount = 6

enum Zone { case top, letters, genres, results }

/// What a search holds. It outlives the page, so Back from a title finds it as it was left.
@Observable
@MainActor
public final class SearchState {
    var text = ""
    var offered: [Tile] = []
    var results: [Tile] = []
    var zone = Zone.letters
    var place = LetterPlace()
    var tab = searchTab
    var genre = 0
    var result = 0

    public init() {}
}

private func tilesOf(_ text: String?, _ limit: Int) -> [Tile] {
    guard let text, let screen = try? JSONDecoder().decode(Contract.Screen.self, from: Data(text.utf8)) else { return [] }
    var seen = Set<TitleKey>()
    return Array(screen.sections.flatMap { $0.tiles ?? [] }.filter { seen.insert($0.title).inserted }.prefix(limit))
}

/// Search: letters and genres on the left, posters on the right.
public struct Search: View {
    let shell: Shell
    let seen: Seen
    let state: SearchState

    public init(shell: Shell, seen: Seen, state: SearchState) {
        self.shell = shell
        self.seen = seen
        self.state = state
    }

    public var body: some View {
        let tabs = tabsOf(seen.nav)
        let genres = Array(seen.nav.filter { $0.screen == .genre }.prefix(genreCount))
        let person = seen.profiles.first { $0.id == seen.active } ?? seen.profiles.first
        let shown = state.text.trimmingCharacters(in: .whitespaces).isEmpty ? state.offered : state.results

        func press(_ word: Word) -> Bool {
            switch state.zone {
            case .top:
                switch word {
                case .left: if state.tab == 0 { state.tab = searchTab } else if state.tab > 0 { state.tab -= 1 }
                case .right: if state.tab == searchTab { state.tab = 0 } else if state.tab < tabs.count - 1 { state.tab += 1 }
                case .down: state.zone = .letters
                case .ok: if let tab = tabs[safe: state.tab] { shell.go(Address.of(tab.screen.rawValue)) }
                // From the bar Back returns to the letters first; from the letters it leaves.
                case .back:
                    state.tab = searchTab
                    state.zone = .letters
                case .up: break
                }
            case .letters:
                switch word {
                case .ok: state.text = state.place.press(state.text)
                case .back: shell.back()
                default:
                    if let next = state.place.move(word) {
                        state.place = next
                    } else if word == .up {
                        state.zone = .top
                    } else if word == .down && !genres.isEmpty {
                        state.zone = .genres
                    } else if word == .right && !shown.isEmpty {
                        state.zone = .results
                    }
                }
            case .genres:
                switch word {
                case .up: if state.genre > 0 { state.genre -= 1 } else { state.zone = .letters }
                case .down: if state.genre < genres.count - 1 { state.genre += 1 }
                case .right: if !shown.isEmpty { state.zone = .results }
                case .ok: if let id = genres[safe: state.genre]?.genre { shell.go(Address.genre + id) }
                case .back: shell.back()
                case .left: break
                }
            case .results:
                if word == .ok {
                    if let tile = shown[safe: state.result] { shell.go(titleAddress(tile.title)) }
                } else if word == .back {
                    state.zone = .letters
                } else if word == .left && state.result % postersAcross == 0 {
                    state.zone = .letters
                } else {
                    state.result = gridMove(state.result, word, shown.count - 1)
                }
            }
            return true
        }

        return ZStack(alignment: .topLeading) {
            PosterGrid(tiles: shown, focus: state.zone == .results ? state.result : -1, left: 566, top: 232, words: seen.words)
            TopBar(tabs: tabs, active: searchTab, focus: state.zone == .top ? state.tab : -1, person: person, server: seen.server)
            LetterGrid(focus: state.zone == .letters ? state.place : nil).offset(x: pageEdge.ref, y: 160.ref)
            Menu(lines: genres.map { MenuLine(word: $0.label) }, focus: state.zone == .genres ? state.genre : -1, width: 440)
                .offset(x: (pageEdge - menuWords).ref, y: 636.ref)
            Text(state.text.trimmingCharacters(in: .whitespaces).isEmpty ? seen.say("search.recommended") : state.text)
                .brand(40).foregroundStyle(.white).lineLimit(1)
                .frame(width: 1280.ref, alignment: .leading)
                .offset(x: 566.ref, y: 150.ref)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .remote { press($0) }
        .task {
            if state.offered.isEmpty { state.offered = tilesOf(await shell.read("/v1/screens/home"), offeredCount) }
        }
        .task(id: state.text) {
            let typed = state.text.trimmingCharacters(in: .whitespaces)
            guard !typed.isEmpty else {
                state.results = []
                return
            }
            try? await Task.sleep(nanoseconds: settle * 1_000_000)
            if Task.isCancelled { return }
            let query = typed.addingPercentEncoding(withAllowedCharacters: .alphanumerics) ?? ""
            state.results = tilesOf(await shell.read("/v1/search?q=\(query)"), .max)
            state.result = 0
        }
    }
}
