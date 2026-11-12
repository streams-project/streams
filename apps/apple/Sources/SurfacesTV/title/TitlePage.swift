import Contract
import Ports
import SwiftUI

private let side = LinearGradient(
    stops: [.init(color: .black.opacity(0.82), location: 0), .init(color: .black.opacity(0.12), location: 0.62)],
    startPoint: .leading, endPoint: .trailing
)
private let foot = LinearGradient(
    stops: [.init(color: .clear, location: 0.42), .init(color: Back, location: 0.92)],
    startPoint: .top, endPoint: .bottom
)

private enum Part { case reactions, menu }
enum TitleView { case title, episodes, more }
private enum Line { case play, episodes, more, list }

extension TitleDetail {
    var isSeries: Bool { title.rawValue.hasPrefix("tv:") }
    var path: String { title.rawValue.replacingOccurrences(of: ":", with: "/") }
}

/// The artwork and its shades, behind a title's pages. Decoration: the title is named, described
/// and listed on top of it, so a screen reader that announced the picture as well would say it
/// twice. `Art` marks itself; the two shades are nothing at all.
struct TitleBackdrop: View {
    let pictures: Pictures
    let data: TitleDetail

    var body: some View {
        ZStack {
            Art(pictures, wide(data.images))
            side
            foot
        }
        .accessibilityHidden(true)
    }
}

/// The first line of the page says what pressing it will do.
@MainActor
private func playWord(_ data: TitleDetail, _ seen: Seen) -> String {
    let resume = data.resume
    let episode = resume?.episode
    let key: String
    if !data.isSeries {
        key = resume != nil ? "title.resume" : "common.play"
    } else if episode != nil && (resume?.positionS ?? 0) > 0 {
        key = "title.continue_episode"
    } else {
        key = "title.play_episode"
    }
    return seen.say(key).fill("n", episode?.episode ?? 1)
}

/// One title: its facts, three ratings, and a menu of what to do next.
public struct TitlePage: View {
    let shell: Shell
    let seen: Seen
    // A new title starts at its summary, and the line opened there is kept for coming back to it.
    @State private var view = TitleView.title
    @State private var line = 0
    @State private var shown: TitleKey?

    public init(shell: Shell, seen: Seen) {
        self.shell = shell
        self.seen = seen
    }

    public var body: some View {
        if let data = seen.detail, data.title == seen.route?.title {
            Group {
                switch view {
                case .episodes: Episodes(shell: shell, seen: seen, data: data) { view = .title }
                case .more: MoreLike(shell: shell, seen: seen, data: data) { view = .title }
                case .title: Summary(shell: shell, seen: seen, data: data, line: $line) { view = $0 }
                }
            }
            .onChange(of: data.title, initial: true) { _, now in
                if shown != now {
                    shown = now
                    view = .title
                    line = 0
                }
            }
        }
    }
}

private struct Summary: View {
    let shell: Shell
    let seen: Seen
    let data: TitleDetail
    @Binding var line: Int
    let open: (TitleView) -> Void
    @State private var part = Part.menu
    @State private var reaction = 1
    @State private var rating: Rating?
    @State private var saved = false

    var body: some View {
        var lines: [Line] = [.play]
        if !(data.seasons ?? []).isEmpty { lines.append(.episodes) }
        if !(data.moreLikeThis ?? []).isEmpty { lines.append(.more) }
        lines.append(.list)
        let row = min(line, lines.count - 1)

        func play() {
            if !data.isSeries {
                shell.go(Address.watch + data.path)
                return
            }
            if let next = data.resume?.episode {
                shell.go("\(Address.watch)\(data.path)/\(next.season)/\(next.episode)")
                return
            }
            let number = data.seasons?.first?.number ?? 1
            Task {
                let season = await shell.read("/v1/titles/\(data.path)/seasons/\(number)")
                    .flatMap { try? JSONDecoder().decode(Season.self, from: Data($0.utf8)) }
                shell.go("\(Address.watch)\(data.path)/\(number)/\(season?.episodes.first?.number ?? 1)")
            }
        }

        func rate() {
            let asked = reactionAt(reaction)
            rating = rating == asked ? nil : asked
            shell.act(.command(ActionCommand(command: .rate(CommandRate(rating: rating, title: data.title)))))
        }

        func toggleList() {
            saved.toggle()
            shell.act(.command(ActionCommand(command: .list(CommandList(inList: saved, title: data.title)))))
        }

        func press(_ word: Word) -> Bool {
            switch word {
            case .up:
                if part == .menu && row > 0 { line = row - 1 } else if part == .menu { part = .reactions }
            case .down:
                if part == .reactions { part = .menu } else if row < lines.count - 1 { line = row + 1 }
            // The menu sits at the left edge: Left from it goes back, as Back does.
            case .left:
                if part == .reactions && reaction > 0 { reaction -= 1 } else if part == .menu { shell.back() }
            case .right:
                if part == .reactions && reaction < reactionCount - 1 { reaction += 1 }
            case .ok:
                if part == .reactions { rate(); return true }
                switch lines[row] {
                case .play: play()
                case .episodes: open(.episodes)
                case .list: toggleList()
                case .more: open(.more)
                }
            // From the ratings Back returns to the menu first, then leaves the title. On Apple TV a
            // Back no page uses leaves the app, so the page goes back itself.
            case .back:
                if part == .reactions { part = .menu } else { shell.back() }
            }
            return true
        }

        let menuLines = lines.map { kind -> MenuLine in
            switch kind {
            case .play: return MenuLine(word: playWord(data, seen), icon: "icon_play")
            case .episodes: return MenuLine(word: seen.say("title.episodes_more"), icon: "icon_episodes")
            case .more: return MenuLine(word: seen.say("title.more_like"), icon: "icon_grid")
            case .list:
                return MenuLine(word: seen.say(saved ? "list.added" : "list.add"), icon: saved ? "icon_check" : "icon_plus")
            }
        }
        let logoUrl = (data.images ?? []).first { $0.role == .logo }?.url
        // What the remote is on: a line of the menu, or one of the three reactions. The screen box
        // is the one thing the system really focuses here, so this is what a person hears as they
        // press keys.
        let here = part == .reactions
            ? reactionWord(reaction, seen.words)
            : sayBoth(menuLines[safe: row]?.word, data.name)

        return ZStack(alignment: .topLeading) {
            TitleBackdrop(pictures: shell.pictures, data: data)
            ZStack(alignment: .bottomLeading) {
                if let logoUrl {
                    Logo(pictures: shell.pictures, url: logoUrl, width: 600, height: 200, says: data.name)
                } else {
                    Text(data.name).brand(76, 800).lines(76, 84).foregroundStyle(.white).lineLimit(2)
                        .accessibilityAddTraits(.isHeader)
                }
            }
            .frame(width: 900.ref, height: 200.ref, alignment: .bottomLeading)
            .offset(x: pageEdge.ref, y: 26.ref)
            Facts(data, seen.words).offset(x: pageEdge.ref, y: 244.ref)
            Text(data.overview ?? "")
                .brand(28).lines(28, 38)
                .foregroundStyle(.white.opacity(0.85))
                .lineLimit(4)
                .frame(width: 820.ref, alignment: .leading)
                .offset(x: pageEdge.ref, y: 300.ref)
            // The reactions sit over the menu icons: both centre on 122.
            Reactions(
                chosen: rating, focus: part == .reactions ? reaction : -1,
                left: pageEdge + 18, top: 504, words: seen.words
            )
            Menu(lines: menuLines, focus: part == .menu ? row : -1, width: 570).offset(x: pageEdge.ref, y: 581.ref)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .remote(says: here) { press($0) }
        .onChange(of: data.title, initial: true) { _, _ in
            rating = data.rating
            saved = data.inList == true
        }
    }
}
