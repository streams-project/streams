import Contract
import Ports
import SwiftUI

/// What has been read already, for this run of the screen.
@MainActor
public final class DetailStore {
    var kept: [TitleKey: TitleDetail] = [:]
    public init() {}
}

extension TitleDetail {
    var isSeries: Bool { title.rawValue.hasPrefix("tv:") }
    var path: String { title.rawValue.replacingOccurrences(of: ":", with: "/") }
}

/// The word on the first button, which says what pressing it will do. Lifted from
/// `SurfacesTV/title/TitlePage.swift`, where the rule was worked out: it is a question about the
/// contract -- film or series, resumed or not -- and the answer is the same whatever is being
/// looked at or pressed.
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

/// The three things a viewer can say about a title.
private let reactions: [(rating: Rating, icon: String, key: String)] = [
    (.down, "icon_thumb_down", "player.down"),
    (.up, "icon_thumb_up", "player.up"),
    (.love, "icon_love", "player.love"),
]

/// One title in a window: its artwork, its facts, what to do about it, and what else is like it.
///
/// The content follows `SurfacesTV/title/TitlePage.swift` and the input follows nothing of it. The
/// television page is a menu whose lines are walked with Up and Down and whose ratings are a strip
/// walked with Left and Right, all of it held in two integers. Here every one of those is a control
/// in its own right, laid out so it can be looked at, and the page holds no idea of where anybody
/// is.
public struct TitlePage: SwiftUI.View {
    let shell: Shell
    let seen: Seen
    @State private var saved = false
    @State private var rating: Rating?
    @State private var shown: TitleKey?

    public init(shell: Shell, seen: Seen) {
        self.shell = shell
        self.seen = seen
    }

    public var body: some SwiftUI.View {
        GeometryReader { space in
            if let data = seen.detail, data.title == seen.route?.title {
                page(data, space.size.width)
                    .onChange(of: data.title, initial: true) { _, now in
                        if shown != now {
                            shown = now
                            rating = data.rating
                            saved = data.inList == true
                        }
                    }
            }
        }
    }

    private func page(_ data: TitleDetail, _ width: CGFloat) -> some SwiftUI.View {
        let inset = pageEdge(width)
        let words = seen.words
        return ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                Backdrop(pictures: shell.pictures, data: data, width: width)
                VStack(alignment: .leading, spacing: 14) {
                    Text(data.name)
                        .brand(30, 800)
                        .lines(30, 36)
                        .foregroundStyle(Ink)
                        .lineLimit(2)
                        .accessibilityAddTraits(.isHeader)
                    Facts(data, words)
                    Text(data.overview ?? "")
                        .brand(16)
                        .lines(16, 23)
                        .foregroundStyle(InkSoft)
                        .frame(maxWidth: 640, alignment: .leading)
                    TitleButtons(
                        data: data, words: words, playing: playWord(data, seen), saved: saved,
                        play: { play(data) },
                        episodes: { shell.go(titleAddress(data.title)) },
                        list: {
                            saved.toggle()
                            shell.act(.command(ActionCommand(command: .list(CommandList(inList: saved, title: data.title)))))
                        }
                    )
                    Reactions(words: words, chosen: rating) { picked in
                        rating = rating == picked ? nil : picked
                        shell.act(.command(ActionCommand(command: .rate(CommandRate(rating: rating, title: data.title)))))
                    }
                }
                .padding(20)
                .plate(Round.panel)
                // The facts stand forward of the still behind them: the panel reads as being over
                // the page rather than cut into it, and it costs no area to say so. See `Surface`.
                .forward(Depth.panel)
                .padding(.horizontal, inset)
                if let like = data.moreLikeThis, !like.isEmpty {
                    Row(
                        pictures: shell.pictures,
                        section: Contract.Section(
                            id: "more-like", kind: .row, tiles: like, title: words["title.more_like"]
                        ),
                        words: words,
                        width: width,
                        open: { shell.go(titleAddress($0)) }
                    )
                }
            }
            .padding(.vertical, 24)
        }
    }

    private func play(_ data: TitleDetail) {
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
}

/// The artwork behind a title, and the shade that keeps words readable over it.
///
/// Decoration, all of it: the title is named, described and listed below, so a picture that
/// announced itself would say the same thing again. The treatment is drawn over the still when the
/// catalogue has one, and it carries no words either, for the same reason.
private struct Backdrop: SwiftUI.View {
    let pictures: Pictures
    let data: TitleDetail
    let width: CGFloat

    var body: some SwiftUI.View {
        let inset = pageEdge(width)
        let box = CGSize(width: width - 2 * inset, height: (width * heroRatio).rounded())
        return ZStack(alignment: .bottomLeading) {
            if let url = wide(data.images) {
                Art(pictures, url, box: box)
            }
            LinearGradient(colors: [.clear, .black.opacity(0.6)], startPoint: .top, endPoint: .bottom)
        }
        .frame(width: box.width, height: box.height)
        .plate(Round.panel)
        .clipShape(RoundedRectangle(cornerRadius: Round.panel, style: .continuous))
        .padding(.horizontal, inset)
        .accessibilityHidden(true)
    }
}

/// Play, episodes, and whether the title is on the viewer's list.
///
/// Each one is a control that draws its own word, so each gathers rather than being told what to
/// say. The list button is the one that changes meaning, and it changes its drawn word with it,
/// which is why it needs no state of its own either.
///
/// Its own view rather than a method on the page, so a test can draw it. A whole screen cannot be
/// read on this family: anything inside a `ScrollView` reaches `Tests/Heard` as nothing at all,
/// measured on 2026-11-14 and written up in `Tests/VisionTests/Speech.swift`. So the parts have to
/// be reachable one at a time, the way the television's are.
struct TitleButtons: SwiftUI.View {
    let data: TitleDetail
    let words: [String: String]
    /// What the first button says, which the page works out from the contract.
    let playing: String
    let saved: Bool
    let play: () -> Void
    let episodes: () -> Void
    let list: () -> Void

    var body: some SwiftUI.View {
        HStack(spacing: 12) {
            Button(action: play) {
                WordButton(word: playing, icon: "icon_play", dark: true)
            }
            .buttonStyle(.plain)
            .reachable(corner: Round.field)
            if !(data.seasons ?? []).isEmpty {
                Button(action: episodes) {
                    WordButton(word: words["title.episodes_more"] ?? "", icon: "icon_episodes", dark: false)
                }
                .buttonStyle(.plain)
                .reachable(corner: Round.field)
            }
            Button(action: list) {
                WordButton(
                    word: words[saved ? "list.added" : "list.add"] ?? "",
                    icon: saved ? "icon_check" : "icon_plus",
                    dark: false
                )
            }
            .buttonStyle(.plain)
            .reachable(corner: Round.field)
        }
    }
}

/// The three reactions, each its own target.
///
/// Which one the viewer has chosen is drawn as a filled picture and said as the control's state,
/// because a filled picture is the kind of thing a person listening cannot see -- the same split
/// the television's settings rows make between what a control is and how it stands. The word for
/// the state comes from `strings/`, never from here.
struct Reactions: SwiftUI.View {
    let words: [String: String]
    let chosen: Rating?
    let pick: (Rating) -> Void

    var body: some SwiftUI.View {
        HStack(spacing: 12) {
            ForEach(reactions, id: \.key) { choice in
                let on = chosen == choice.rating
                Button {
                    pick(choice.rating)
                } label: {
                    Icon(on ? choice.icon + "_filled" : choice.icon, 24, tint: on ? .white : Ink)
                        // The minimum is applied here as well as by `reachable`, and this is the
                        // one case where that is not a second home for it: what is drawn behind
                        // the picture has to fill the target, or the system lights a 60 pt shape
                        // over a 32 pt plate. Everywhere the control draws no background of its
                        // own, the modifier is the only place the minimum appears.
                        .frame(width: Reach.target, height: Reach.target)
                        .background(
                            on ? AnyShapeStyle(Accent) : AnyShapeStyle(Material.thin),
                            in: RoundedRectangle(cornerRadius: Round.field, style: .continuous)
                        )
                }
                .buttonStyle(.plain)
                .reachable(
                    says: words[choice.key] ?? "",
                    state: on ? (words["say.chosen"] ?? "") : "",
                    corner: Round.field
                )
            }
        }
    }
}
