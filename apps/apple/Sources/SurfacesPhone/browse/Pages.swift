import Contract
import Ports
import SwiftUI

/// The main pages under the bottom bar. The lead's M4 builds Home and pages to the
/// reference; this skeleton lists the rows so every layer is proven end to end.
struct Pages: View {
    let shell: Shell
    let seen: Seen

    var body: some View {
        GeometryReader { box in
            let edge = pageEdge(box.size.width)
            ZStack(alignment: .bottom) {
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 20) {
                        ForEach(seen.screen?.sections ?? [], id: \.id) { section in
                            if let title = section.title { Text(title).brand(18, 700).padding(.horizontal, edge) }
                            ScrollView(.horizontal, showsIndicators: false) {
                                LazyHStack(spacing: 6) {
                                    ForEach(section.tiles ?? [], id: \.title) { tile in
                                        Art(tall(tile.images))
                                            .frame(width: 119, height: 170)
                                            .clipShape(RoundedRectangle(cornerRadius: Corner.card))
                                    }
                                }
                                .padding(.horizontal, edge)
                            }
                        }
                    }
                    .padding(.vertical, 16)
                    .padding(.bottom, 96)
                }
                BottomBar(items: bar(seen)).padding(.horizontal, edge).padding(.bottom, 8)
            }
        }
    }

    private func bar(_ seen: Seen) -> [BarItem] {
        let tabs = seen.nav.filter { $0.screen != .genre }.prefix(3)
        return tabs.map { item in
            BarItem(icon: "icon_home", word: item.label, here: seen.active == item.id) {
                shell.go(Address.home)
            }
        }
    }
}
