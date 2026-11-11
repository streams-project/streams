import Contract
import Observation
import Ports
import SwiftUI

private let rise: CGFloat = 810
private let riseTime = 0.32

/// How near the last row the focus has to come before the rest of the page is read: two, meaning the
/// row the focus is on and the one drawn below it.
///
/// Derived from the band this surface draws, not from a fetch Netflix was seen to make. A row screen
/// always shows the next row's title and the top of its tiles under the focused row
/// (`notes/netflix-reference/tv/00-netflix-on-the-tv-measured.md` section 6b: "next row title
/// 948-978, next row tiles from 998 with 82 px visible"), and this page draws that row from
/// `focus.row + 1`. Reading on arrival at the very last row would therefore leave the viewer looking
/// at an empty band below the focus while the answer is in flight, so the read starts one row
/// earlier - the smallest window that keeps a row under the focus, and no prefetch beyond it.
///
/// Section 7 is not evidence for this number: it measures single steps on the profile gate, where
/// every step costs a fetch and nothing is prefetched, which says nothing about how a page of rows
/// is paged.
private let near = 2

private enum Where { case top, hero, row }

private let rowKinds: Set<SectionKind> = [.row, .ranked, .continue, .grid]

/// Where the viewer is on a page of rows. It knows nothing about drawing.
@Observable
@MainActor
private final class PageFocus {
    var rows: [Contract.Section]
    var tabs: Int
    var hero: Bool
    var place: Where
    var row = 0
    var tab: Int
    /// This page's own tab.
    let active: Int
    var button = 0
    /// Which way the last row change went, so the rows slide the right way.
    var way: CGFloat = 1
    /// The rows read past the first answer, the place the next read starts from, and whether one is
    /// already out. They sit with the focus, so a page returned to keeps the rows it had reached.
    var extra: [Contract.Section] = []
    var cursor: Cursor?
    var reading = false
    /// True once this page's first cursor has been taken from the answer.
    var seeded = false
    /// Where each row was left, by the row's id: the place and the title on it. A row the viewer comes
    /// back to, or sees below, opens there; a fresh answer that moved the title keeps the title.
    private var columns: [String: (at: Int, title: TitleKey?)] = [:]

    init(rows: [Contract.Section], tabs: Int, active: Int, hero: Bool) {
        self.rows = rows
        self.tabs = tabs
        self.hero = hero
        self.tab = active
        self.active = active
        self.place = hero || rows.isEmpty ? (hero ? .hero : .top) : .row
    }

    private var above: Where { hero ? .hero : .top }

    /// The tile a row was left on, or its first.
    func columnOf(_ index: Int) -> Int {
        guard let section = rows[safe: index], let kept = columns[section.id] else { return 0 }
        let tiles = section.tiles ?? []
        if tiles[safe: kept.at]?.title == kept.title { return kept.at }
        if let moved = tiles.firstIndex(where: { $0.title == kept.title }) { return moved }
        return min(kept.at, max(tiles.count - 1, 0))
    }

    var column: Int { columnOf(row) }

    private func keep(_ at: Int) {
        guard let section = rows[safe: row] else { return }
        columns[section.id] = (at, section.tiles?[safe: at]?.title)
    }

    private var lastColumn: Int { (rows.indices.contains(row) ? rows[row].tiles?.count ?? 0 : 0) - 1 }

    func press(_ word: Word) -> Bool {
        switch word {
        case .up:
            if place == .row && row > 0 { way = -1; row -= 1 }
            else if place == .row { place = above }
            else if place == .hero { place = .top }
            else { return false }
        case .down:
            if place == .top && hero { place = .hero }
            else if (place == .hero || place == .top) && !rows.isEmpty { place = .row; row = 0 }
            else if place == .row && row < rows.count - 1 { way = 1; row += 1 }
            else { return false }
        case .left:
            if place == .top && tab > 0 { tab -= 1 }
            else if place == .top && tab == 0 { tab = searchTab }
            else if place == .top && tab == searchTab { tab = whoTab }
            else if place == .hero { button = 0 }
            else if place == .row && column > 0 { keep(column - 1) }
            else { return false }
        case .right:
            if place == .top && tab == whoTab { tab = searchTab }
            else if place == .top && tab == searchTab { tab = 0 }
            else if place == .top && tab < tabs - 1 { tab += 1 }
            else if place == .hero { button = 1 }
            else if place == .row && column < lastColumn { keep(column + 1) }
            else { return false }
        case .back:
            switch place {
            case .row: place = above
            case .hero: place = .top
            // Focus wandered along the bar without choosing: Back brings it home to this page's tab
            // first, and only then leaves the page.
            case .top:
                if tab != active { tab = active } else { return false }
            }
        case .ok:
            return false
        }
        return true
    }
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

/// A page of rows. The parts draw, and this file only knows where the focus is.
public struct Browse: View {
    let shell: Shell
    let seen: Seen
    @State private var focus: PageFocus?
    @Environment(\.pages) private var memory
    @State private var store = DetailStore()
    @State private var shownRow = 0
    @State private var menu = false

    public init(shell: Shell, seen: Seen) {
        self.shell = shell
        self.seen = seen
    }

    public var body: some View {
        if let data = seen.screen, screenHere(seen) {
            page(data)
        } else {
            // The answer for this address is still on its way. The bar stands with the tab already
            // lit and the body stays black: no tiles of the tab before, and no skeleton either.
            waiting()
        }
    }

    private func waiting() -> some View {
        let current = seen.route?.screen?.rawValue ?? "home"
        let tabs = tabsOf(seen.nav)
        let active = max(0, tabs.firstIndex(where: { $0.screen.rawValue == current }) ?? 0)
        let person = seen.profiles.first(where: { $0.id == seen.active }) ?? seen.profiles.first
        return ZStack(alignment: .topLeading) {
            Color.black
            TopBar(tabs: tabs, active: active, focus: active, person: person, server: seen.server)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        // A screen that is not here yet answers no key. This is a choice, and it is stated as one:
        // the measurement covers what is drawn while a tab loads (section 5: the body is black for
        // 1.08-4.67 s, top bar standing, no skeleton) and nothing was measured about what the keys
        // do in that window. On tvOS a Back this app does not answer reaches the system and leaves
        // the app, so holding every key here costs the viewer a dead Back for as long as the body is
        // black, and letting Back through would drop them out of the app mid load.
        .remote { _ in true }
    }

    private func page(_ data: Contract.Screen) -> some View {
        let current = seen.route?.screen?.rawValue ?? "home"
        let tabs = tabsOf(seen.nav)
        let active = max(0, tabs.firstIndex(where: { $0.screen.rawValue == current }) ?? 0)
        let person = seen.profiles.first(where: { $0.id == seen.active }) ?? seen.profiles.first
        // A page is drawn from what was kept and then from the fresh answer. The focus belongs to the page.
        // Each page keeps where the viewer was for the run of the app, so coming back from a title lands
        // on the tile that opened it.
        let key = "\(seen.active ?? "") \(current) \(seen.route?.genre ?? "")"
        let kept = self.focus ?? (memory.pages[key] as? PageFocus)
        // Games has no rows yet, and the kept page behind it belongs to the tab before.
        // The answer's own rows come first, then the rows read from its cursor.
        let read = data.sections + (kept?.extra ?? [])
        let sections = current == "games" ? [] : read.filter { !($0.tiles ?? []).isEmpty }
        let rows = sections.filter { rowKinds.contains($0.kind) }
        let lead = sections.first(where: { $0.kind == .hero })?.tiles?.first
        let focus = kept ?? {
            let made = PageFocus(rows: rows, tabs: tabs.count, active: active, hero: lead != nil)
            memory.pages[key] = made
            return made
        }()
        focus.rows = rows
        focus.tabs = tabs.count
        focus.hero = lead != nil
        let inRow = focus.place == .row

        func choose() {
            if focus.place == .top && focus.tab == whoTab {
                menu = true
            } else if focus.place == .top && focus.tab == searchTab {
                shell.go(Address.search)
            } else if focus.place == .top {
                if tabs.indices.contains(focus.tab) { shell.go(Address.of(tabs[focus.tab].screen.rawValue)) }
            } else if focus.place == .row {
                if rows.indices.contains(focus.row), let tile = rows[focus.row].tiles?[safe: focus.column] {
                    shell.go(titleAddress(tile.title))
                }
            } else if let lead {
                if focus.button == 1 || lead.isSeries { shell.go(titleAddress(lead.title)) }
                else { shell.go(watchAddress(lead.title)) }
            }
        }

        func press(_ word: Word) -> Bool {
            if word == .ok { choose(); return true }
            if focus.press(word) { return true }
            // Back from the top of any page but Home goes Home. From Home it leaves the app.
            if word == .back && current != "home" { shell.go(Address.home); return true }
            return false
        }

        let way = focus.way
        return ZStack(alignment: .topLeading) {
            TopBar(tabs: tabs, active: active, focus: focus.place == .top ? focus.tab : -1, person: person, server: seen.server)
            if inRow {
                ZStack {
                    if rows.indices.contains(shownRow) {
                        OpenRow(
                            shell: shell, words: seen.words, store: store, section: rows[shownRow],
                            // A row sliding away keeps the tile it was left on.
                            at: focus.columnOf(shownRow)
                        )
                        .id(shownRow)
                        .transition(.asymmetric(
                            insertion: .offset(y: way * rise.ref).combined(with: .opacity),
                            removal: .offset(y: -way * rise.ref).combined(with: .opacity)
                        ))
                    }
                }
            } else if let lead {
                HeroCard(shell: shell, seen: seen, store: store, lead: lead, button: focus.place == .hero ? focus.button : -1)
            } else if rows.isEmpty {
                Text(seen.say(current == "games" ? "games.soon" : "mine.empty"))
                    .brand(32)
                    .foregroundStyle(Muted)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
            let below = inRow ? focus.row + 1 : 0
            if let next = rows[safe: below] {
                NextRow(section: next, high: !inRow, words: seen.words, at: focus.columnOf(below))
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .remote(on: !menu) { press($0) }
        .overlay {
            if menu { ProfileMenu(shell: shell, seen: seen, person: person) { menu = false } }
        }
        .onAppear {
            if self.focus == nil { self.focus = focus }
            shownRow = focus.row
            if !focus.seeded {
                focus.seeded = true
                focus.cursor = data.cursor
            }
            readOn(focus, rows.count)
        }
        .onChange(of: focus.row) { _, now in
            // The new direction is drawn first, then the rows move.
            DispatchQueue.main.async { withAnimation(glide(riseTime)) { shownRow = now } }
            readOn(focus, rows.count)
        }
    }

    /// The rest of the page. The server builds many more rows than one answer carries and hands back
    /// a cursor for the rest; without this read the rows past the first answer cannot be reached at
    /// all, because one row fills a television screen and the viewer walks down them one at a time.
    private func readOn(_ focus: PageFocus, _ count: Int) {
        guard focus.row >= count - near, let mark = focus.cursor, !focus.reading else { return }
        focus.reading = true
        Task { @MainActor in
            let answer = await shell.read("/v1/more/" + mark)
            focus.reading = false
            guard let answer,
                  let more = try? JSONDecoder().decode(More.self, from: Data(answer.utf8)) else { return }
            focus.extra += more.sections ?? []
            focus.cursor = more.cursor
        }
    }
}

/// Where the viewer was on each page of rows, for the run of the app.
@MainActor
public final class PageMemory {
    var pages: [String: AnyObject] = [:]
    public init() {}
}

private struct PagesKey: EnvironmentKey {
    @MainActor static let defaultValue = PageMemory()
}

extension EnvironmentValues {
    public var pages: PageMemory {
        get { self[PagesKey.self] }
        set { self[PagesKey.self] = newValue }
    }
}

extension Array {
    subscript(safe index: Int) -> Element? {
        indices.contains(index) ? self[index] : nil
    }
}
