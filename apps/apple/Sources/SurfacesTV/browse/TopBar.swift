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
            .brand(26, 500)
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
    /// The words for the two controls here that draw no text: who is watching, and search.
    var words: [String: String] = [:]

    var body: some View {
        let searchOn = focus == searchTab
        let whoOn = focus == whoTab
        let searchHere = active == searchTab
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
            HStack(spacing: 8.ref) {
                ForEach(Array(tabs.enumerated()), id: \.offset) { index, item in
                    Tab(item: item, active: index == active, on: index == focus)
                }
            }
            .offset(x: 609.ref, y: (barMid - barHigh / 2).ref)
            Mark(barHigh).offset(x: (1920 - pageEdge - barHigh).ref, y: (barMid - barHigh / 2).ref)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }
}
