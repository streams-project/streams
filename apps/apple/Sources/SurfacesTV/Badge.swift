import Contract
import SwiftUI

private func badgeLabel(_ badge: Contract.Badge, _ words: [String: String]) -> String {
    switch badge.kind {
    case .new: return words["badge.new"] ?? ""
    case .ranked:
        guard let rank = badge.rank else { return "" }
        return (words["badge.ranked"] ?? "").replacingOccurrences(of: "{rank}", with: String(rank))
    default: return ""
    }
}

/// Draws `Tile.badges` (new, ranked) as small dark pills inside the tile at bottom left.
public struct TileBadges: View {
    public let tile: Tile
    public let words: [String: String]

    public init(_ tile: Tile, _ words: [String: String]) {
        self.tile = tile
        self.words = words
    }

    public var body: some View {
        let badges = tile.badges ?? []
        if !badges.isEmpty {
            HStack(spacing: 8.ref) {
                ForEach(Array(badges.enumerated()), id: \.offset) { _, badge in
                    Text(badgeLabel(badge, words))
                        .brand(22, 700)
                        .foregroundStyle(.white)
                        .lineLimit(1)
                        .padding(.horizontal, 12.ref)
                        .padding(.vertical, 4.ref)
                        .background(Color.black.opacity(0.72), in: RoundedRectangle(cornerRadius: 8.ref))
                }
            }
        }
    }
}
