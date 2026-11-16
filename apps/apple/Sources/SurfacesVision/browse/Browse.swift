import Contract
import Observation
import Ports
import SwiftUI

/// Which section kinds this surface draws as a row of tiles. A kind it does not know falls through
/// to nothing rather than to a guess.
private let rowKinds: Set<SectionKind> = [.row, .ranked, .continue, .grid]

/// True when the screen the shell holds is the one this address asks for.
///
/// Until it is, the answer for the page the viewer has just left is still in hand, and drawing it
/// would show them the page they left. The same question the television asks in
/// `SurfacesTV/help/Problem.swift`; it is four lines about the contract and not about either
/// surface, and the two cannot share it until there is somewhere shared to put it.
@MainActor
func screenHere(_ seen: Seen) -> Bool {
    let wanted: String? = switch seen.route?.target {
    case .home: "home"
    case .kidsHome: "kids"
    default: seen.route?.screen?.rawValue
    }
    guard let name = seen.screen?.name.rawValue else { return false }
    return wanted == nil || name == wanted
}

/// The tabs that fit the bar. Genres live on their own pages.
func tabsOf(_ nav: [NavItem]) -> [NavItem] {
    Array(nav.filter { $0.screen != .genre })
}

func titleAddress(_ key: TitleKey) -> String {
    Address.title + key.rawValue.replacingOccurrences(of: ":", with: "-")
}

func watchAddress(_ key: TitleKey) -> String {
    Address.watch + key.rawValue.replacingOccurrences(of: ":", with: "/")
}

extension Tile {
    var isSeries: Bool { title.rawValue.hasPrefix("tv:") }
}

/// The rows read past the first answer, and where the next read starts.
///
/// The television keeps this with its focus, because the focus is what says the viewer is near the
/// end. Here it hangs off the page, and what says the viewer is near the end is a row drawing its
/// last tile.
@Observable
@MainActor
final class MoreRows {
    var extra: [Contract.Section] = []
    var cursor: Cursor?
    var reading = false
    var seeded = false
}

/// Where the viewer is, where else they can go, and who they are watching as.
///
/// **Not a band across the page.** The television spends its top 96 px of 1080 on this bar, with a
/// 53 px content band inside it (`reference/netflix-on-the-tv.md`), because a television has one
/// plane and chrome has to live on it. Here an ornament is placed by the system *outside* the
/// window's bounds, so the same controls cost the catalogue nothing, and what is inside the window
/// is then all content -- which is what the viewer resized the window to see.
///
/// It is anchored to the **bottom**, and that is a reach decision rather than a style one. A window
/// is put at about eye height and stays there for the length of a film, while hands rest low; a
/// pinch is made where the hand is comfortable, not where the eye is pointing. Chrome at the top of
/// a window the viewer has pushed away is chrome at the far end of a reach. A device has to confirm
/// it, and the reasoning is written here so it can be argued with rather than guessed at.
///
/// On every other platform this module compiles for the same bar is drawn inline at the foot of the
/// window, because ornaments do not exist there -- and because that is the only place on this
/// machine where its controls can be laid out and measured at all. The ornament itself has never
/// been drawn by anything.
struct NavBar: SwiftUI.View {
    let tabs: [NavItem]
    let active: String
    let person: Profile?
    let words: [String: String]
    let go: (String) -> Void

    var body: some SwiftUI.View {
        HStack(spacing: 8) {
            ForEach(tabs, id: \.id) { item in
                let here = item.screen.rawValue == active
                Button {
                    go(Address.of(item.screen.rawValue))
                } label: {
                    Text(item.label)
                        .brand(16, here ? 700 : 500)
                        .foregroundStyle(here ? Ink : InkSoft)
                        .padding(.horizontal, 18)
                }
                .buttonStyle(.plain)
                .reachable(corner: Round.field)
            }
            Spacer(minLength: 0)
            Button {
                go(Address.search)
            } label: {
                Icon("icon_search", 22, tint: Ink)
            }
            .buttonStyle(.plain)
            .reachable(says: words["nav.search"] ?? "", corner: Round.field)
            if let person {
                Button {
                    go(Address.profiles)
                } label: {
                    Face(name: person.name)
                }
                .buttonStyle(.plain)
                .reachable(says: person.name, corner: Round.field)
            }
        }
    }
}

/// A profile's picture, which on this surface is the first letter of the name on a plate.
///
/// Decoration inside a control that is already named with the whole name, so a reader never hears
/// the letter before the name -- the mistake `SurfacesTV`'s own profile menu test is written
/// against.
struct Face: SwiftUI.View {
    let name: String

    var body: some SwiftUI.View {
        ZStack {
            RoundedRectangle(cornerRadius: Round.field, style: .continuous).fill(Accent)
            Text(name.prefix(1).uppercased()).brand(20, 700).foregroundStyle(.white)
        }
        .frame(width: 40, height: 40)
        .decoration()
    }
}

/// The card at the top of a page of rows: one title, large, with two things to do about it.
struct Hero: SwiftUI.View {
    let pictures: Pictures
    let tile: Tile
    let words: [String: String]
    let width: CGFloat
    let go: (String) -> Void

    var body: some SwiftUI.View {
        let inset = pageEdge(width)
        let box = CGSize(width: width - 2 * inset, height: (width * heroRatio).rounded())
        return ZStack(alignment: .bottomLeading) {
            if let url = wide(tile.images) {
                Art(pictures, url, box: box)
            }
            // A scrim over artwork, which is content and not chrome: the words below have to stay
            // readable over whatever still the catalogue sent, and that is true on any plate.
            LinearGradient(colors: [.clear, .black.opacity(0.75)], startPoint: .top, endPoint: .bottom)
                .decoration()
            VStack(alignment: .leading, spacing: 14) {
                Text(tile.name)
                    .brand(34, 800)
                    .foregroundStyle(.white)
                    .lineLimit(2)
                    .accessibilityAddTraits(.isHeader)
                HStack(spacing: 12) {
                    Button {
                        go(tile.isSeries ? titleAddress(tile.title) : watchAddress(tile.title))
                    } label: {
                        WordButton(word: words["common.play"] ?? "", icon: "icon_play", dark: true)
                    }
                    .buttonStyle(.plain)
                    .reachable(corner: Round.field)
                    Button {
                        go(titleAddress(tile.title))
                    } label: {
                        WordButton(word: words["common.more_info"] ?? "", icon: "icon_info", dark: false)
                    }
                    .buttonStyle(.plain)
                    .reachable(corner: Round.field)
                }
            }
            .padding(24)
        }
        .frame(width: box.width, height: box.height)
        .plate(Round.panel)
        .clipShape(RoundedRectangle(cornerRadius: Round.panel, style: .continuous))
        .padding(.horizontal, inset)
    }
}

/// A word with a picture beside it, inside a control that carries both. Named `WordButton` and not
/// `Label` so it cannot shadow SwiftUI's own type of that name inside this module.
///
/// The icon is never given words: the word is drawn next to it, the control gathers the two, and an
/// icon that spoke as well would say the same thing twice -- or, unmarked, would be read out as the
/// name of its own asset file, which is what `Tests/Heard` measured this family doing before
/// anything was labelled.
struct WordButton: SwiftUI.View {
    let word: String
    let icon: String
    let dark: Bool

    var body: some SwiftUI.View {
        HStack(spacing: 8) {
            Icon(icon, 18, tint: dark ? .white : Ink)
            Text(word).brand(16, 700).foregroundStyle(dark ? .white : Ink)
        }
        .padding(.horizontal, 22)
        // Applied here as well as by `reachable`, for the one reason that is not duplication: the
        // plate drawn behind the word has to fill the target, or the system lights a shape larger
        // than the thing it appears to be lighting.
        .frame(minHeight: Reach.target)
        // The primary action keeps the brand's accent; everything beside it is the system's
        // material. Literal white on white is what this replaces: a white plate reads as a plate on
        // a television in a dim room and as nothing at all on glass over a bright one.
        .background(
            dark ? AnyShapeStyle(Accent) : AnyShapeStyle(Material.thin),
            in: RoundedRectangle(cornerRadius: Round.field, style: .continuous)
        )
    }
}

/// A page of rows in a window.
///
/// What is the television's, kept: the content. Rows of tiles under a card, each row a theme with a
/// title, the catalogue paged by a cursor. What is the television's and left behind: all of the
/// input. There is no `PageFocus` here, no `press(_:)`, no row index and no column -- the page
/// draws everything it has and the viewer looks at whichever part of it they want.
public struct Browse: SwiftUI.View {
    let shell: Shell
    let seen: Seen
    @State private var more = MoreRows()

    public init(shell: Shell, seen: Seen) {
        self.shell = shell
        self.seen = seen
    }

    public var body: some SwiftUI.View {
        GeometryReader { space in
            let width = space.size.width
            if let data = seen.screen, screenHere(seen) {
                page(data, width)
            } else {
                // The answer for this address is on its way. The bar is not drawn either, because
                // it is drawn from the same answer; what stands is the one word the strings have
                // for it, which is a thing to read rather than a screen of nothing.
                Text(seen.say("say.loading"))
                    .brand(17)
                    .foregroundStyle(InkSoft)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        }
    }

    private func page(_ data: Contract.Screen, _ width: CGFloat) -> some SwiftUI.View {
        let current = seen.route?.screen?.rawValue ?? "home"
        let tabs = tabsOf(seen.nav)
        let person = seen.profiles.first(where: { $0.id == seen.active }) ?? seen.profiles.first
        let read = data.sections + more.extra
        let sections = read.filter { !($0.tiles ?? []).isEmpty }
        let rows = sections.filter { rowKinds.contains($0.kind) }
        let lead = sections.first(where: { $0.kind == .hero })?.tiles?.first

        let bar = NavBar(tabs: tabs, active: current, person: person, words: seen.words, go: shell.go)
        return ScrollView {
            LazyVStack(alignment: .leading, spacing: rowGap) {
                if let lead {
                    Hero(pictures: shell.pictures, tile: lead, words: seen.words, width: width, go: shell.go)
                }
                if rows.isEmpty {
                    Text(seen.say(current == "games" ? "games.soon" : "mine.empty"))
                        .brand(17)
                        .foregroundStyle(InkSoft)
                        .padding(.horizontal, pageEdge(width))
                }
                ForEach(Array(rows.enumerated()), id: \.offset) { index, section in
                    Row(
                        pictures: shell.pictures,
                        section: section,
                        words: seen.words,
                        width: width,
                        open: { shell.go(titleAddress($0)) },
                        reached: index == rows.count - 1 ? { readOn() } : nil
                    )
                }
            }
            .padding(.vertical, 24)
        }
        .modifier(Chrome(bar: bar, inset: pageEdge(width)))
        .onAppear {
            if !more.seeded {
                more.seeded = true
                more.cursor = data.cursor
            }
        }
    }

    /// The rest of the page. The server builds more rows than one answer carries and hands back a
    /// cursor for them; this reads the next lot when the viewer reaches the end of the last row.
    private func readOn() {
        guard let mark = more.cursor, !more.reading else { return }
        more.reading = true
        Task { @MainActor in
            let answer = await shell.read("/v1/more/" + mark)
            more.reading = false
            guard let answer,
                  let got = try? JSONDecoder().decode(More.self, from: Data(answer.utf8)) else { return }
            more.extra += got.sections ?? []
            more.cursor = got.cursor
        }
    }
}


/// Where the page's chrome goes: outside the window on this platform, at the foot of it elsewhere.
///
/// One modifier rather than an `#if` inside the page, so there is a single place that knows the
/// difference and the page reads the same on both. `NavBar`'s own note argues the bottom anchor.
private struct Chrome<Bar: SwiftUI.View>: ViewModifier {
    let bar: Bar
    let inset: CGFloat

    func body(content: Content) -> some SwiftUI.View {
        #if os(visionOS)
        content.ornament(attachmentAnchor: .scene(.bottom)) {
            bar.padding(.horizontal, 20).padding(.vertical, 12).glassBackgroundEffect()
        }
        #else
        VStack(spacing: 0) {
            content
            bar.padding(.horizontal, inset).padding(.vertical, 12)
        }
        #endif
    }
}
