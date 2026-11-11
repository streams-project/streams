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

/// How many rows before the end of a page the next part is asked for. Netflix's web home appends
/// its next block "roughly two viewports of remaining scroll" before the end; a phone viewport is
/// about four rows of 217, so two viewports is about eight. Three is the careful end of that.
private let aheadRows = 3

/// The last offset seen, held outside view state: it changes on every scrolled frame, and a page
/// that re-reads its own body that often draws nothing new for the cost.
@MainActor
private final class Drift {
    var last: CGFloat = 0
}

/// A main page (Home, Series, Films): the pinned header, the billboard, the rows, and the floating
/// bar, laid out and behaving as the reference's. The page behind the header takes the billboard's
/// colour at the top and turns black as it scrolls; the chips slide away while the page scrolls down
/// and come back as soon as it scrolls up.
struct Home: View {
    let shell: Shell
    let seen: Seen

    @State private var tint: Color = firstTint
    @State private var chipsHidden: Bool = false
    @State private var headerSolid: Bool = false
    @State private var showCategories: Bool = false
    @State private var saved: Bool? = nil
    @State private var asking: Bool = false
    @State private var drift = Drift()

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
            ?? words["app.title"] ?? seen.brand

        let inList = saved ?? false

        GeometryReader { proxy in
            let topInset = proxy.safeAreaInsets.top
            // The page rests with its first pixel under the chip row, as the reference's does
            // (phone/00-netflix-phone-measurements.md section 3: billboard y 453 = status bar 132 +
            // header expanded 321, which is 125.3 dp and is this header's own 52 + 73). Only the
            // header's own height is held back here, because the safe area is already in the scroll
            // view's insets: the rig read the content resting at 62 before this margin and at 187
            // after it, 62 + 125.
            let headerHigh = topBarHeight + chipRowHeight

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
                        // Before iOS 18 the offset is read from the content's own geometry. The reader
                        // stands outside the lazy stack: as the lazy stack's first child it is thrown
                        // away the moment it scrolls out of sight, and the reading falls back to the
                        // key's nought mid scroll, which puts the chips back and takes the header's
                        // colour away with nothing to report afterwards.
                        if #unavailable(iOS 18.0) {
                            GeometryReader { geo in
                                Color.clear
                                    .preference(
                                        key: ScrollOffsetKey.self,
                                        value: geo.frame(in: .named("homeScroll")).minY
                                    )
                            }
                            .frame(height: 0)
                        }

                        LazyVStack(spacing: 0) {
                            if let lead {
                                Billboard(
                                    shell: shell,
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

                                Color.clear.frame(height: 16)
                            }

                            ForEach(Array(rows.enumerated()), id: \.element.id) { index, section in
                                SectionRow(
                                    section: section,
                                    words: words,
                                    open: { tile in
                                        shell.go(titleAddress(tile.title))
                                    },
                                    openInfo: { tile in
                                        shell.go(titleAddress(tile.title))
                                    },
                                    onRemove: { tile in
                                        shell.act(.command(ActionCommand(command: .hide(CommandHide(title: tile.title)))))
                                    }
                                )
                                .id(section.id)
                                .onAppear {
                                    if index >= rows.count - aheadRows { readMore() }
                                }
                            }

                            // Bottom spacing for bar
                            Color.clear.frame(height: 120)
                        }
                    }
                }
                .coordinateSpace(name: "homeScroll")
                .contentMargins(.top, headerHigh, for: .scrollContent)
                .modifier(Drifted { moved($0) })
                .onPreferenceChange(ScrollOffsetKey.self) { value in
                    // Only the reader above writes this, and it is installed only before iOS 18;
                    // from 18 the scroll view's own geometry is read instead. Without this the one
                    // delivery of the key's default on a version that has no reader would report the
                    // page back at its top.
                    guard #unavailable(iOS 18.0) else { return }
                    // The content rests below the header, not at the reader's own nought: the safe
                    // area inset the scroll view is given, plus the top margin held back for the
                    // header. Both readings came off the rig - the reader rested at 62 before this
                    // page held a margin back, and at 62 + 125 = 187 after it - so the distance
                    // scrolled is measured from there and is nought at rest.
                    moved(topInset + headerHigh - value)
                }

                // Pinned Header
                VStack(spacing: 0) {
                    TopBar(
                        server: seen.server,
                        name: title,
                        back: onHome ? nil : { shell.back() },
                        bell: { shell.go(Address.notifications) },
                        unreadCount: seen.unreadNotifications
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
                .background(headerSolid ? Back.opacity(0.97) : Color.clear)
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

    /// The page moved. `down` is how far it has been pulled up from its own top, nought at rest.
    /// Only the two things the header draws from are written, and only when they turn over, so a
    /// scrolled frame costs no re-reading of this page.
    private func moved(_ down: CGFloat) {
        let solid = down > 24
        if solid != headerSolid { headerSolid = solid }

        let step = down - drift.last
        drift.last = down
        if step > 4, !chipsHidden {
            withAnimation(.easeInOut(duration: 0.22)) { chipsHidden = true }
        } else if step < -4, chipsHidden {
            withAnimation(.easeInOut(duration: 0.22)) { chipsHidden = false }
        }
    }

    /// The next rows of this page, through the cursor the screen carries. Without this the page
    /// stops at the rows the first answer held and the rest of what the server built is unreachable.
    private func readMore() {
        guard !asking,
              let screen = seen.screen,
              let cursor = screen.cursor,
              !cursor.isEmpty else { return }
        let name = screen.name
        asking = true
        shell.ask("/v1/more/" + cursor) { status, text in
            Task { @MainActor in
                asking = false
                guard (200...299).contains(status),
                      var screen = seen.screen,
                      screen.name == name,
                      screen.cursor == cursor,
                      let more = try? JSONDecoder().decode(More.self, from: Data(text.utf8))
                else { return }
                let gained = more.sections ?? []
                screen.sections.append(contentsOf: gained)
                // An answer with no rows in it ends the page, whatever cursor it carries, so a
                // page that has run out cannot ask again for ever.
                screen.cursor = gained.isEmpty ? nil : more.cursor
                seen.screen = screen
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

/// How far the page has been pulled up, from the scroll view itself. Reading it from the content's
/// own geometry does not work on 26.5, the one version this was tried on: a scrolled page reports
/// the same number it reports at rest, so the chips never left and the header never took its colour.
/// Which version it broke on was not measured, so the older reader below is kept for the floor the
/// package still declares rather than on a claim about where the break began.
private struct Drifted: ViewModifier {
    let moved: (CGFloat) -> Void

    func body(content: Content) -> some View {
        // tvOS is named as well as iOS because the package declares a tvOS floor too, and `*` there
        // means that floor: without it this file cannot be compiled for tvOS at all.
        if #available(iOS 18.0, tvOS 18.0, *) {
            content.onScrollGeometryChange(for: CGFloat.self) { geometry in
                geometry.contentOffset.y + geometry.contentInsets.top
            } action: { _, now in
                moved(now)
            }
        } else {
            content
        }
    }
}


private struct ScrollOffsetKey: PreferenceKey {
    static var defaultValue: CGFloat = 0
    static func reduce(value: inout CGFloat, nextValue: () -> CGFloat) {
        value = nextValue()
    }
}
