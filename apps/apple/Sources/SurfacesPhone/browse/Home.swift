import Contract
import Ports
import SurfacesShared
import SwiftUI

/// Before the billboard's colours are read, the page wears our blue, darkened.
private let firstTint = Color(red: 0x1A / 255, green: 0x2A / 255, blue: 0x55 / 255)

/// The pages the chips offer, in the reference's order; the rest of the menu lives in the bar.
private let chipScreens = ["series", "films", "games", "new"]

/// A main page (Home, Series, Films): the pinned header, the billboard, the rows, and the floating
/// bar, laid out and behaving as the reference's. The page behind the header takes the billboard's
/// colour at the top and turns black as it scrolls; the chips slide away while the page scrolls down
/// and come back as soon as it scrolls up.
struct Home: View {
    let shell: Shell
    let seen: Seen

    @State private var tint: Color = firstTint
    @State private var chipsHidden: Bool = false
    @State private var scrollOffset: CGFloat = 0
    @State private var lastOffset: CGFloat = 0
    @State private var showCategories: Bool = false
    @State private var saved: Bool? = nil

    var body: some View {
        let words = seen.words
        let sections = seen.screen?.sections ?? []
        let lead = sections.first(where: { $0.kind == .hero })?.tiles?.first
        let rows = sections.filter { rowKinds.contains($0.kind) }
        let here = seen.route?.screen?.rawValue ?? "home"
        let onHome = here == "home" || here == "kids"
        let title = seen.nav.first(where: { $0.screen.rawValue == here && $0.genre == seen.route?.genre })?.label
            ?? seen.nav.first(where: { $0.screen.rawValue == here })?.label
            ?? words["app.title"] ?? "Streams"

        let inList = saved ?? false

        GeometryReader { proxy in
            let topInset = proxy.safeAreaInsets.top
            let atTop = scrollOffset > -24

            ZStack(alignment: .top) {
                // Background: billboard tint fading to Back
                VStack(spacing: 0) {
                    LinearGradient(
                        stops: [
                            .init(color: tint.opacity(0.95), location: 0.0),
                            .init(color: Back, location: 1.0),
                        ],
                        startPoint: .top,
                        endPoint: .bottom
                    )
                    .frame(height: topInset + 640)
                    Spacer()
                }
                .ignoresSafeArea()

                // Scroll content
                ScrollView {
                    VStack(spacing: 0) {
                        // Offset reporter
                        GeometryReader { geo in
                            Color.clear
                                .preference(
                                    key: ScrollOffsetKey.self,
                                    value: geo.frame(in: .named("homeScroll")).minY
                                )
                        }
                        .frame(height: 0)

                        // Top spacing under header
                        Spacer().frame(height: topInset + topBarHeight + chipRowHeight)

                        if let lead {
                            Billboard(
                                tile: lead,
                                words: words,
                                saved: inList,
                                play: {
                                    shell.go(lead.isSeries ? titleAddress(lead.title) : watchAddress(lead.title))
                                },
                                list: {
                                    saved = !inList
                                    shell.act(.command(ActionCommand(command: .list(CommandList(inList: !inList, title: lead.title)))))
                                },
                                open: {
                                    shell.go(titleAddress(lead.title))
                                },
                                tint: { newTint in
                                    withAnimation(.easeInOut(duration: 0.6)) {
                                        tint = newTint
                                    }
                                }
                            )

                            Spacer().frame(height: 16)
                        }

                        ForEach(Array(rows.enumerated()), id: \.element.id) { index, section in
                            SectionRow(
                                section: section,
                                words: words,
                                tallTiles: index == 0 && section.kind == .row,
                                open: { tile in
                                    shell.go(titleAddress(tile.title))
                                }
                            )
                        }

                        // Bottom spacing for bar
                        Spacer().frame(height: 120)
                    }
                }
                .coordinateSpace(name: "homeScroll")
                .onPreferenceChange(ScrollOffsetKey.self) { value in
                    let diff = value - lastOffset
                    if diff < -4 {
                        withAnimation(.easeInOut(duration: 0.22)) {
                            chipsHidden = true
                        }
                    } else if diff > 4 {
                        withAnimation(.easeInOut(duration: 0.22)) {
                            chipsHidden = false
                        }
                    }
                    lastOffset = value
                    scrollOffset = value
                }

                // Pinned Header
                VStack(spacing: 0) {
                    TopBar(
                        server: seen.server,
                        name: title,
                        back: onHome ? nil : { shell.back() },
                        bell: { shell.go(Address.notifications) }
                    )
                    .padding(.top, topInset)

                    ChipRow(
                        chips: makeChips(seen: seen, onHome: onHome) {
                            showCategories = true
                        },
                        shown: chipsHidden ? 0.0 : 1.0
                    )
                    .frame(height: chipRowHeight * (chipsHidden ? 0.0 : 1.0))
                    .clipped()
                }
                .background(atTop ? Color.clear : Back.opacity(0.97))
                .animation(.easeInOut(duration: 0.22), value: chipsHidden)
                .ignoresSafeArea(edges: .top)

                // Floating bottom bar
                VStack {
                    Spacer()
                    BottomBar(items: makeBarItems(shell: shell, seen: seen, onHome: onHome))
                        .padding(.bottom, 9)
                }
            }
            .confirmationDialog(
                seen.words["nav.categories"] ?? "Categories",
                isPresented: $showCategories,
                titleVisibility: .visible
            ) {
                let genres = seen.nav.filter { $0.screen.rawValue == "genre" }.sorted { $0.label < $1.label }
                ForEach(genres, id: \.id) { item in
                    Button(item.label) {
                        shell.go(Address.genre + (item.genre ?? ""))
                    }
                }
            }
        }
    }

    private func makeChips(seen: Seen, onHome: Bool, openCategories: @escaping () -> Void) -> [Chip] {
        var chips: [Chip] = []
        if onHome {
            for screen in chipScreens {
                if let item = seen.nav.first(where: { $0.screen.rawValue == screen }) {
                    chips.append(Chip(word: item.label, chosen: false) {
                        shell.go(tabAddress(screen))
                    })
                }
            }
        }
        chips.append(Chip(
            word: seen.words["nav.categories"] ?? "Categories",
            chosen: false,
            drops: true,
            go: openCategories
        ))
        return chips
    }

    private func makeBarItems(shell: Shell, seen: Seen, onHome: Bool) -> [BarItem] {
        var items: [BarItem] = []

        // Home
        let homeLabel = seen.nav.first(where: { $0.screen.rawValue == "home" })?.label ?? "Home"
        items.append(BarItem(icon: "icon_home", word: homeLabel, here: onHome) {
            shell.go(Address.home)
        })

        // Clips if offered
        if let clips = seen.nav.first(where: { $0.screen.rawValue == "clips" }) {
            items.append(BarItem(icon: "icon_clips", word: clips.label, here: false) {
                shell.go(Address.clips)
            })
        }

        // Search
        let searchLabel = seen.words["nav.search"] ?? "Search"
        items.append(BarItem(icon: "icon_search", word: searchLabel, here: false) {
            shell.go(Address.search)
        })

        // Mine with profile face
        let person = seen.profiles.first(where: { $0.id == seen.active }) ?? seen.profiles.first
        if let mine = seen.nav.first(where: { $0.screen.rawValue == "mine" }) {
            items.append(BarItem(
                icon: "icon_user",
                word: mine.label,
                here: false,
                face: person,
                server: seen.server
            ) {
                shell.go(Address.mine)
            })
        }

        return items
    }
}

private struct ScrollOffsetKey: PreferenceKey {
    static var defaultValue: CGFloat = 0
    static func reduce(value: inout CGFloat, nextValue: () -> CGFloat) {
        value = nextValue()
    }
}
