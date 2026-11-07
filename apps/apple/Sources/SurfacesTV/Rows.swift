import Contract
import Ports
import SwiftUI

public struct TileCard: View {
    public let tile: Tile
    public let focused: Bool
    public let rank: Int?

    public init(tile: Tile, focused: Bool, rank: Int? = nil) {
        self.tile = tile
        self.focused = focused
        self.rank = rank
    }

    public var body: some View {
        ZStack(alignment: .topTrailing) {
            Art(posterImage(tile.images), contentMode: .fill)
                .frame(width: 250.ref, height: 360.ref)
                .clipped()
                .background(BrandTheme.plate)
                .clipShape(RoundedRectangle(cornerRadius: 12.ref))

            // Badges
            if let r = rank {
                RankedBadge(rank: r)
                    .padding(8.ref)
            } else if tile.badges?.contains(where: { $0.kind == .new }) == true {
                NewBadge()
                    .padding(8.ref)
            }
        }
        .frame(width: 250.ref, height: 360.ref)
        .overlay(
            RoundedRectangle(cornerRadius: 12.ref)
                .stroke(focused ? Color.white : Color.clear, lineWidth: 5.ref)
        )
        .scaleEffect(focused ? 1.08 : 1.0)
        .animation(.easeOut(duration: 0.15), value: focused)
    }
}

public struct SectionRow: View {
    public let section: Contract.Section
    public let rowIndex: Int
    public let focusedRow: Int
    public let focusedColumn: Int

    public init(section: Contract.Section, rowIndex: Int, focusedRow: Int, focusedColumn: Int) {
        self.section = section
        self.rowIndex = rowIndex
        self.focusedRow = focusedRow
        self.focusedColumn = focusedColumn
    }

    public var body: some View {
        VStack(alignment: .leading, spacing: 18.ref) {
            // Row Title
            if let title = section.title, !title.isEmpty {
                Text(title)
                    .font(.system(size: 34.refType, weight: .bold))
                    .foregroundStyle(.white)
                    .padding(.leading, 75.ref)
            }

            // Tiles row
            ScrollViewReader { proxy in
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 24.ref) {
                        ForEach(Array((section.tiles ?? []).enumerated()), id: \.element.title.rawValue) { colIndex, tile in
                            let on = rowIndex == focusedRow && colIndex == focusedColumn
                            let rank = section.kind == .ranked ? colIndex + 1 : nil
                            TileCard(tile: tile, focused: on, rank: rank)
                                .id(colIndex)
                        }
                    }
                    .padding(.horizontal, 75.ref)
                    .padding(.vertical, 16.ref)
                }
                .onChange(of: focusedColumn) { _, newCol in
                    if rowIndex == focusedRow {
                        withAnimation {
                            proxy.scrollTo(newCol, anchor: .center)
                        }
                    }
                }
            }
        }
    }
}
