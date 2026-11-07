import Contract
import Ports
import SwiftUI

private enum PageZone {
    case top
    case hero
    case row
}

public struct Browse: View {
    public let shell: Shell
    public let seen: Seen

    @State private var zone: PageZone = .hero
    @State private var tabIndex: Int = 0
    @State private var heroButton: Int = 0
    @State private var rowIndex: Int = 0
    @State private var columns: [Int: Int] = [:]
    @State private var heroDetail: TitleDetail? = nil

    public init(shell: Shell, seen: Seen) {
        self.shell = shell
        self.seen = seen
    }

    private var navTabs: [NavItem] {
        tabsOfNav(seen.nav)
    }

    private var activeTabIndex: Int {
        guard let currentScreen = seen.route?.screen?.rawValue else { return 0 }
        return navTabs.firstIndex(where: { $0.screen.rawValue == currentScreen }) ?? 0
    }

    private var leadHero: Tile? {
        seen.screen?.sections.first(where: { $0.kind == .hero })?.tiles?.first
    }

    private var contentRows: [Contract.Section] {
        (seen.screen?.sections ?? []).filter {
            $0.kind == .row || $0.kind == .ranked || $0.kind == .continue || $0.kind == .grid
        }
    }

    private var currentColumn: Int {
        columns[rowIndex] ?? 0
    }

    public var body: some View {
        ZStack(alignment: .top) {
            BrandTheme.back.ignoresSafeArea()

            ScrollViewReader { scrollProxy in
                ScrollView(.vertical, showsIndicators: false) {
                    VStack(alignment: .leading, spacing: 36.ref) {
                        // Top spacer for TopBar
                        Spacer().frame(height: 120.ref)

                        // Hero card
                        if let hero = leadHero {
                            HeroCard(
                                lead: hero,
                                button: zone == .hero ? heroButton : -1,
                                words: seen.words,
                                detail: heroDetail
                            )
                            .padding(.horizontal, 75.ref)
                            .id("hero")
                        }

                        // Content rows
                        ForEach(Array(contentRows.enumerated()), id: \.offset) { idx, section in
                            SectionRow(
                                section: section,
                                rowIndex: idx,
                                focusedRow: zone == .row ? rowIndex : -1,
                                focusedColumn: zone == .row ? currentColumn : -1
                            )
                            .id("row-\(idx)")
                        }

                        Spacer().frame(height: 100.ref)
                    }
                }
                .onChange(of: rowIndex) { _, newRow in
                    if zone == .row {
                        withAnimation {
                            scrollProxy.scrollTo("row-\(newRow)", anchor: .center)
                        }
                    }
                }
                .onChange(of: zone) { _, newZone in
                    if newZone == .hero {
                        withAnimation {
                            scrollProxy.scrollTo("hero", anchor: .top)
                        }
                    }
                }
            }

            // TopBar floating on top
            TopBar(
                tabs: navTabs,
                active: activeTabIndex,
                focus: zone == .top ? tabIndex : -100,
                profile: seen.profiles.first(where: { $0.id == seen.active }),
                brand: seen.brand
            )
        }
        .task(id: leadHero?.title.rawValue) {
            guard let title = leadHero?.title.rawValue else { return }
            let path = "/v1/titles/" + title.replacingOccurrences(of: ":", with: "/")
            if let text = await shell.read(path) {
                heroDetail = try? JSONDecoder().decode(TitleDetail.self, from: Data(text.utf8))
            }
        }
        .remote { word in
            press(word)
        }
    }

    private func press(_ word: Word) -> Bool {
        switch zone {
        case .top:
            switch word {
            case .left:
                if tabIndex > whoTab { tabIndex -= 1 }
                return true
            case .right:
                if tabIndex < navTabs.count - 1 { tabIndex += 1 }
                return true
            case .down:
                if leadHero != nil {
                    zone = .hero
                    heroButton = 0
                } else if !contentRows.isEmpty {
                    zone = .row
                    rowIndex = 0
                }
                return true
            case .ok:
                if tabIndex >= 0, let item = navTabs[safe: tabIndex] {
                    shell.go(Address.of(item.screen.rawValue))
                }
                return true
            case .back:
                shell.back()
                return true
            default:
                return true
            }

        case .hero:
            switch word {
            case .up:
                zone = .top
                tabIndex = activeTabIndex
                return true
            case .down:
                if !contentRows.isEmpty {
                    zone = .row
                    rowIndex = 0
                }
                return true
            case .left:
                if heroButton > 0 { heroButton -= 1 }
                return true
            case .right:
                if heroButton < 1 { heroButton += 1 }
                return true
            case .ok:
                guard let hero = leadHero else { return true }
                let keyPath = hero.title.rawValue.replacingOccurrences(of: ":", with: "/")
                if heroButton == 0 {
                    shell.go(Address.watch + keyPath)
                } else {
                    shell.go(Address.title + keyPath)
                }
                return true
            case .back:
                shell.back()
                return true
            }

        case .row:
            let currentRowTiles = contentRows.get(safe: rowIndex)?.tiles ?? []
            switch word {
            case .up:
                if rowIndex > 0 {
                    rowIndex -= 1
                } else if leadHero != nil {
                    zone = .hero
                    heroButton = 0
                } else {
                    zone = .top
                    tabIndex = activeTabIndex
                }
                return true
            case .down:
                if rowIndex < contentRows.count - 1 {
                    rowIndex += 1
                }
                return true
            case .left:
                if currentColumn > 0 {
                    columns[rowIndex] = currentColumn - 1
                }
                return true
            case .right:
                if currentColumn < currentRowTiles.count - 1 {
                    columns[rowIndex] = currentColumn + 1
                }
                return true
            case .ok:
                if let tile = currentRowTiles.get(safe: currentColumn) {
                    let keyPath = tile.title.rawValue.replacingOccurrences(of: ":", with: "/")
                    shell.go(Address.title + keyPath)
                }
                return true
            case .back:
                zone = .top
                tabIndex = activeTabIndex
                return true
            }
        }
    }
}

extension Array {
    fileprivate func get(safe index: Int) -> Element? {
        return indices.contains(index) ? self[index] : nil
    }
}
