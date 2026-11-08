import Contract
import Ports
import SurfacesShared
import SwiftUI

/// Before the billboard's colours are read, the page wears our blue, darkened.
private let firstTint = Color(red: 0x1A / 255, green: 0x2A / 255, blue: 0x55 / 255)

/// The pages the chips offer, in the reference's order; the rest of the menu lives in the bar.
private let chipScreens = ["series", "films", "games", "new"]

/// The reference's muted grey for unselected genres in the Categories overlay.
private let mutedGrey = Color(red: 0xC0 / 255.0, green: 0xC0 / 255.0, blue: 0xC0 / 255.0)

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
    @State private var scrollPositions: [String: String] = [:]

    var body: some View {
        let words = seen.words
        let sections = seen.screen?.sections ?? []
        let lead = sections.first(where: { $0.kind == .hero })?.tiles?.first
        let rows = sections.filter { rowKinds.contains($0.kind) }
        let here = seen.route?.screen?.rawValue ?? "home"
        let onHome = here == "home" || here == "kids"
        let pickedGenre = here == "genre"
            ? seen.nav.first(where: { $0.screen.rawValue == "genre" && $0.genre == seen.route?.genre })?.label
            : nil
        let title = pickedGenre
            ?? seen.nav.first(where: { $0.screen.rawValue == here })?.label
            ?? words["app.title"] ?? "Streams"

        let screenKey = onHome ? "home" : "\(here):\(seen.route?.genre ?? "")"
        let scrollBinding = Binding<String?>(
            get: { scrollPositions[screenKey] },
            set: { scrollPositions[screenKey] = $0 }
        )

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
                            .id("billboard")

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
                            .id(section.id)
                        }

                        // Bottom spacing for bar
                        Spacer().frame(height: 120)
                    }
                }
                .coordinateSpace(name: "homeScroll")
                .scrollPosition(id: scrollBinding)
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
                        chips: makeChips(seen: seen, onHome: onHome, here: here, pickedGenre: pickedGenre) {
                            withAnimation(.easeInOut(duration: 0.25)) {
                                showCategories = true
                            }
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

                // Categories full-screen overlay (replacing DropdownMenu)
                if showCategories {
                    ZStack(alignment: .bottom) {
                        // Dimmed dark background (90% Back)
                        Back.opacity(0.90)
                            .ignoresSafeArea()
                            .onTapGesture {
                                withAnimation(.easeInOut(duration: 0.2)) {
                                    showCategories = false
                                }
                            }

                        // Scrollable list of genres
                        ScrollView {
                            VStack(spacing: 0) {
                                Spacer().frame(height: topInset + 40)

                                let locale = Locale(identifier: seen.language.isEmpty ? "en" : seen.language)
                                let genres = seen.nav
                                    .filter { $0.screen.rawValue == "genre" }
                                    .sorted {
                                        $0.label.compare($1.label, options: [.caseInsensitive, .diacriticInsensitive], locale: locale) == .orderedAscending
                                    }

                                ForEach(genres, id: \.id) { item in
                                    let isChosen = here == "genre" && item.genre == seen.route?.genre
                                    Button {
                                        withAnimation(.easeInOut(duration: 0.2)) {
                                            showCategories = false
                                        }
                                        shell.go(Address.genre + (item.genre ?? ""))
                                    } label: {
                                        Text(item.label)
                                            .brand(21, isChosen ? 700 : 500)
                                            .foregroundStyle(isChosen ? Color.white : mutedGrey)
                                            .multilineTextAlignment(.center)
                                            .frame(maxWidth: .infinity)
                                            .padding(.vertical, 12)
                                            .padding(.horizontal, 24)
                                    }
                                    .buttonStyle(.plain)
                                }

                                Spacer().frame(height: 120)
                            }
                        }

                        // Round white/grey close button (X) at bottom center (56 dp round circle)
                        Button {
                            withAnimation(.easeInOut(duration: 0.2)) {
                                showCategories = false
                            }
                        } label: {
                            ZStack {
                                RoundedRectangle(cornerRadius: min(Corner.button, 28))
                                    .fill(Color.white)
                                    .frame(width: 56, height: 56)

                                CloseX()
                                    .stroke(Back, style: StrokeStyle(lineWidth: 2.5, lineCap: .round))
                                    .frame(width: 20, height: 20)
                            }
                        }
                        .buttonStyle(.plain)
                        .padding(.bottom, 24)
                    }
                    .transition(.opacity)
                    .zIndex(10)
                }
            }
        }
    }

    private func makeChips(
        seen: Seen,
        onHome: Bool,
        here: String,
        pickedGenre: String?,
        openCategories: @escaping () -> Void
    ) -> [Chip] {
        if onHome {
            var chips: [Chip] = []
            for screen in chipScreens {
                if let item = seen.nav.first(where: { $0.screen.rawValue == screen }) {
                    chips.append(Chip(word: item.label, chosen: false) {
                        shell.go(tabAddress(screen))
                    })
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
        let label = pickedGenre ?? seen.words["nav.categories"] ?? "Categories"
        return [
            Chip(
                word: label,
                chosen: false,
                drops: true,
                go: openCategories
            )
        ]
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

private struct CloseX: Shape {
    func path(in rect: CGRect) -> Path {
        var path = Path()
        path.move(to: CGPoint(x: rect.minX, y: rect.minY))
        path.addLine(to: CGPoint(x: rect.maxX, y: rect.maxY))
        path.move(to: CGPoint(x: rect.minX, y: rect.maxY))
        path.addLine(to: CGPoint(x: rect.maxX, y: rect.minY))
        return path
    }
}

private struct ScrollOffsetKey: PreferenceKey {
    static var defaultValue: CGFloat = 0
    static func reduce(value: inout CGFloat, nextValue: () -> CGFloat) {
        value = nextValue()
    }
}
