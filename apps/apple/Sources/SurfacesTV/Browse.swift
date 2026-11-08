import Contract
import Observation
import Ports
import SwiftUI

private let rise: CGFloat = 810
private let riseTime = 0.32

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
    private var columns: [Int: Int] = [:]

    init(rows: [Contract.Section], tabs: Int, active: Int, hero: Bool) {
        self.rows = rows
        self.tabs = tabs
        self.hero = hero
        self.tab = active
        self.active = active
        self.place = hero || rows.isEmpty ? (hero ? .hero : .top) : .row
    }

    private var above: Where { hero ? .hero : .top }

    var column: Int { columns[row] ?? 0 }

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
            else if place == .row && column > 0 { columns[row] = column - 1 }
            else { return false }
        case .right:
            if place == .top && tab == whoTab { tab = searchTab }
            else if place == .top && tab == searchTab { tab = 0 }
            else if place == .top && tab < tabs - 1 { tab += 1 }
            else if place == .hero { button = 1 }
            else if place == .row && column < lastColumn { columns[row] = column + 1 }
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
        if let data = seen.screen {
            page(data)
        }
    }

    private func page(_ data: Contract.Screen) -> some View {
        let current = seen.route?.screen?.rawValue ?? "home"
        // Games has no rows yet, and the kept page behind it belongs to the tab before.
        let sections = current == "games" ? [] : data.sections.filter { !($0.tiles ?? []).isEmpty }
        let rows = sections.filter { rowKinds.contains($0.kind) }
        let lead = sections.first(where: { $0.kind == .hero })?.tiles?.first
        let tabs = tabsOf(seen.nav)
        let active = max(0, tabs.firstIndex(where: { $0.screen.rawValue == current }) ?? 0)
        let person = seen.profiles.first(where: { $0.id == seen.active }) ?? seen.profiles.first
        // A page is drawn from what was kept and then from the fresh answer. The focus belongs to the page.
        // Each page keeps where the viewer was for the run of the app, so coming back from a title lands
        // on the tile that opened it.
        let key = "\(seen.active ?? "") \(current) \(seen.route?.genre ?? "")"
        let focus = self.focus ?? (memory.pages[key] as? PageFocus) ?? {
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
                            at: shownRow == focus.row ? focus.column : 0
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
            if let next = inRow ? rows[safe: focus.row + 1] : rows.first {
                NextRow(section: next, high: !inRow, words: seen.words)
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
        }
        .onChange(of: focus.row) { _, now in
            // The new direction is drawn first, then the rows move.
            DispatchQueue.main.async { withAnimation(glide(riseTime)) { shownRow = now } }
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
