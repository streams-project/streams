import Contract
import SwiftUI

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
            .frame(height: 62.ref)
            .background(back, in: Capsule())
    }
}

/// Who is watching, search, the tabs and the mark, at the top of every signed in page.
struct TopBar: View {
    let tabs: [NavItem]
    let active: Int
    let focus: Int
    let person: Profile?
    let server: String

    var body: some View {
        let searchOn = focus == searchTab
        let whoOn = focus == whoTab
        let searchHere = active == searchTab
        ZStack(alignment: .topLeading) {
            HStack(spacing: 0) {
                Face(person, 48, 8, server)
                    .overlay(RoundedRectangle(cornerRadius: 8.ref).stroke(.white, lineWidth: whoOn ? 3.ref : 0))
                Icon("icon_caret_down", 22)
            }
            .offset(x: 68.ref, y: 46.ref)
            Circle()
                .fill(searchOn ? Color.white : searchHere ? Color.white.opacity(0.16) : .clear)
                .frame(width: 64.ref, height: 64.ref)
                .overlay(Icon("icon_search", 34, tint: searchOn ? Back : .white))
                .offset(x: 539.ref, y: 35.ref)
            HStack(spacing: 8.ref) {
                ForEach(Array(tabs.enumerated()), id: \.offset) { index, item in
                    Tab(item: item, active: index == active, on: index == focus)
                }
            }
            .offset(x: 609.ref, y: 36.ref)
            Mark(64).offset(x: 1796.ref, y: 40.ref)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }
}
