import Contract
import Ports
import SwiftUI

public let whoTab = -3
public let searchTab = -2

public func tabsOfNav(_ nav: [NavItem]) -> [NavItem] {
    return Array(nav.filter { $0.screen.rawValue != "genre" }.prefix(6))
}

public struct TopBar: View {
    public let tabs: [NavItem]
    public let active: Int
    public let focus: Int
    public let profile: Profile?
    public let brand: String

    public init(tabs: [NavItem], active: Int, focus: Int, profile: Profile?, brand: String) {
        self.tabs = tabs
        self.active = active
        self.focus = focus
        self.profile = profile
        self.brand = brand
    }

    public var body: some View {
        HStack(alignment: .center, spacing: 16.ref) {
            // Profile icon
            HStack(spacing: 8.ref) {
                let on = focus == whoTab
                ZStack {
                    Color.blue
                        .frame(width: 48.ref, height: 48.ref)
                        .clipShape(RoundedRectangle(cornerRadius: 8.ref))

                    Text(String(profile?.name.prefix(1) ?? "J").uppercased())
                        .font(.system(size: 24.refType, weight: .bold))
                        .foregroundStyle(.white)
                }
                .overlay(
                    RoundedRectangle(cornerRadius: 8.ref)
                        .stroke(on ? Color.white : Color.clear, lineWidth: 3.ref)
                )

                Image(systemName: "chevron.down")
                    .font(.system(size: 14.refType, weight: .bold))
                    .foregroundStyle(on ? .white : BrandTheme.muted)
            }
            .padding(.trailing, 24.ref)

            // Search icon
            let searchOn = focus == searchTab
            ZStack {
                Circle()
                    .fill(searchOn ? Color.white : Color.clear)
                    .frame(width: 48.ref, height: 48.ref)

                Image(systemName: "magnifyingglass")
                    .font(.system(size: 22.refType, weight: .bold))
                    .foregroundStyle(searchOn ? .black : .white)
            }
            .padding(.trailing, 16.ref)

            // Nav Tabs
            ForEach(Array(tabs.enumerated()), id: \.element.screen.rawValue) { index, tab in
                let on = focus == index
                let isActive = active == index

                Text(tab.label)
                    .font(.system(size: 26.refType, weight: .semibold))
                    .foregroundStyle(on ? .black : .white)
                    .padding(.horizontal, 28.ref)
                    .padding(.vertical, 14.ref)
                    .background(
                        on ? Color.white : (isActive ? Color.white.opacity(0.18) : Color.clear)
                    )
                    .clipShape(Capsule())
            }

            Spacer()

            // Brand mark top right
            Text(brand.isEmpty ? "Streams" : brand)
                .font(.system(size: 32.refType, weight: .heavy))
                .foregroundStyle(BrandTheme.accent)
        }
        .padding(.horizontal, 64.ref)
        .padding(.top, 40.ref)
        .frame(height: 100.ref)
    }
}
