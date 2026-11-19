import Contract
import Ports
import SwiftUI

/// The bar's centre line and the height of its round parts: face, search, tabs and mark share them.
private let barMid: CGFloat = 68
private let barHigh: CGFloat = 64
private let barFace: CGFloat = 48

/// The place of the search button, to the left of the first tab.
let searchTab = -2

/// The place of the profile, to the left of search.
let whoTab = -3

/// The place of the bell, to the **right** of the last tab.
///
/// The phone's bell is at the trailing end of its header (`SurfacesPhone/browse/Header.swift`) and
/// so is the browser's; this bar's trailing end is the brand's mark, so the bell takes the place
/// beside it. Putting it between the face and search was rejected: it would move search and the
/// tabs, whose x's are transcribed coordinates and are not this pass's to move.
///
/// Negative like the other two because it is not a tab, and -4 and not a number past the last tab
/// because a place past the last tab collides with a real tab on a bar that gained one. `PageFocus`
/// answers it before `tab < tabs - 1`, which -4 satisfies for every tab count there is.
let bellTab = -4

/// The gap between two round parts of this bar: the tabs' own spacing.
private let barGap: CGFloat = 8

/// Where the bell sits: one round part and one gap in from the mark, which is drawn at
/// `1920 - pageEdge - barHigh`. A derivation over numbers that already had a home, so moving the
/// mark moves the bell with it.
let bellX = 1920 - pageEdge - barHigh - barGap - barHigh

/// The glyph inside the bell: the size this bar already draws the search glass at.
let bellGlyph: CGFloat = 34

/// The plate the count sits on, at the bell's trailing top corner: half a round part high.
///
/// The phone caps its count at "9+" because its badge is 18 dp wide. That cap is a width this bar
/// does not have -- there is a whole bar height to draw in, and a television is read from three
/// metres, where a number small enough to need capping is a number nobody can read. So the count is
/// drawn as the server sent it, in the language's own digits, and the phone's 9 is not copied here.
let bellCountHigh = barHigh / 2

/// And the size this bar writes in, which a tab's label is already drawn at.
let barType: CGFloat = 26

/// What the bell says: how many notices are waiting, or the screen's own name when none are.
///
/// The same two keys, in the same order, as every other bell this project has -- the phone's
/// `Header.swift`, the Android phone's `Header.kt` and the browser's `bell_says`. Both are in all
/// eighteen languages; neither is new. `say.notifications_unread` had no television caller on
/// either family until this, which is the one thing a bell is for: the count is drawn on it, so a
/// person who cannot see it has nowhere else to get it.
///
/// Filled through `fill`, which wraps the number in the bidirectional isolates and writes it in the
/// language's own digits, and not by a plain replacement. A run that has not read its language yet
/// holds neither key and says nothing, which is this surface's rule: never English, never the key.
func bellSays(_ words: [String: String], unread: Int) -> String {
    guard unread > 0, let pattern = words["say.notifications_unread"], !pattern.isEmpty else {
        return words["notifications.title"] ?? ""
    }
    return pattern.fill("n", unread)
}

/// The tabs that fit the bar. Genres live on their own pages.
func tabsOf(_ nav: [NavItem]) -> [NavItem] {
    Array(nav.filter { $0.screen != .genre }.prefix(6))
}

private struct Tab: View {
    let item: NavItem
    let active: Bool
    let on: Bool

    var body: some View {
        let back: Color = on ? .white : active ? .white.opacity(0.16) : .clear
        Text(item.label)
            .brand(barType, 500)
            .foregroundStyle(on ? Back : .white)
            .padding(.horizontal, 32.ref)
            .frame(height: barHigh.ref)
            .background(back, in: Capsule())
            // A tab draws its own name. `on` is where the remote is, which is drawn as a white
            // pill; `active` is the page in view, drawn as a dim one. Only the first is said,
            // because that is the one a person pressing keys needs to follow.
            .saysItsWords(on: on)
    }
}

/// Who is watching, search, the tabs and the mark, at the top of every signed in page.
struct TopBar: View {
    let pictures: Pictures
    let tabs: [NavItem]
    let active: Int
    let focus: Int
    let person: Profile?
    let server: String
    /// Whether this bar has a bell at all.
    ///
    /// A page of rows does; the search screen does not, and that is a decision rather than an
    /// oversight. The search screen's key map (`search/Search.swift`) walks its bar between the
    /// profile, search and the tabs and has no place for a fourth stop, so a bell drawn there would
    /// be a control no remote could reach -- the exact defect this surface keeps finding. The other
    /// television's search screen draws a bar with no bell for the same reason, and the two match.
    var bell: Bool = true
    /// How many notices are waiting, for the bell. The server's own number, off
    /// `seen.unreadNotifications`; never counted from the feed the notifications screen holds,
    /// because that screen holds one page of it.
    var unread: Int = 0
    /// The words for the three controls here that draw no text of their own: who is watching,
    /// search, and the bell.
    var words: [String: String] = [:]

    var body: some View {
        let searchOn = focus == searchTab
        let whoOn = focus == whoTab
        let searchHere = active == searchTab
        let bellOn = focus == bellTab
        ZStack(alignment: .topLeading) {
            // Who is watching: a face, which is a picture, and a caret, which is a picture of the
            // menu it opens. The profile's own name is the only words there are for it, and it
            // comes from the answer rather than from `strings/`.
            HStack(spacing: 0) {
                Face(pictures, person, barFace, server)
                    .overlay(shapeOf(Round.face(barFace)).stroke(.white, lineWidth: whoOn ? 3.ref : 0))
                Icon("icon_caret_down", 22)
            }
            .says(person?.name ?? "", on: whoOn)
            .offset(x: pageEdge.ref, y: (barMid - barFace / 2).ref)
            Circle()
                .fill(searchOn ? Color.white : searchHere ? Color.white.opacity(0.16) : .clear)
                .frame(width: barHigh.ref, height: barHigh.ref)
                .overlay(Icon("icon_search", 34, tint: searchOn ? Back : .white))
                .says(words["nav.search"] ?? "", on: searchOn)
                .offset(x: 539.ref, y: (barMid - barHigh / 2).ref)
            HStack(spacing: barGap.ref) {
                ForEach(Array(tabs.enumerated()), id: \.offset) { index, item in
                    Tab(item: item, active: index == active, on: index == focus)
                }
            }
            .offset(x: 609.ref, y: (barMid - barHigh / 2).ref)
            // The bell: a picture and a count, and neither is words, so what it says is the only
            // thing that makes it reachable at all. No dim plate for "the page in view" the way
            // search has one: the notifications screen is not one of this bar's pages, so there is
            // no state in which the bell is the page the viewer is on.
            if bell {
            Circle()
                .fill(bellOn ? Color.white : .clear)
                .frame(width: barHigh.ref, height: barHigh.ref)
                .overlay(Icon("icon_bell", bellGlyph, tint: bellOn ? Back : .white))
                .overlay(alignment: .topTrailing) {
                    if unread > 0 {
                        // Decoration: the bell itself says the count in words, so a reader that
                        // stopped on the number as well would hear it twice, the second time with
                        // no noun on it.
                        Text(number(unread))
                            .brand(barType, 800)
                            .foregroundStyle(.white)
                            .padding(.horizontal, barGap.ref)
                            .frame(height: bellCountHigh.ref)
                            .background(Accent, in: Capsule())
                            .accessibilityHidden(true)
                    }
                }
                .says(bellSays(words, unread: unread), on: bellOn)
                .offset(x: bellX.ref, y: (barMid - barHigh / 2).ref)
            }
            Mark(barHigh).offset(x: (1920 - pageEdge - barHigh).ref, y: (barMid - barHigh / 2).ref)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }
}
