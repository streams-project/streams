import Contract
import SwiftUI

private func badgeLabel(_ badge: Contract.Badge, _ words: [String: String]) -> String {
    switch badge.kind {
    case .new: return words["badge.new"] ?? ""
    case .ranked:
        guard let rank = badge.rank else { return "" }
        return (words["badge.ranked"] ?? "").fill("rank", rank)
    default: return ""
    }
}

/// `Tile.badges` drawn as small dark pills inside a tile.
///
/// Every badge is drawn inside the tile's own control, so these are never targets of their own and
/// never carry ``Reach/target``: a 60 pt pill inside a 225 pt tile would be a quarter of the
/// picture. They are gathered into the tile's words instead, which is what ``words(_:)`` below is
/// for -- the tile sets its own label from the title's name and whatever these say, rather than
/// letting a reader stop on a pill drawn on top of a control it is part of.
///
/// A ranked row is where this surface parts from the television deliberately. The television draws
/// the rank as a large outlined numeral standing in a widened gutter, which costs it 146 reference
/// pixels of row (`reference/netflix-on-the-tv.md`, "Top 10 rows"): on a 1920 screen that is 7.6%
/// of the width, and in a window opening at ``windowWide`` the same arrangement would eat an eighth
/// of the row to show a number the tile is already carrying as a badge. So the badge is what is
/// drawn, and the row keeps one pitch whatever kind it is.
public struct TileBadges: SwiftUI.View {
    public let tile: Tile
    public let words: [String: String]

    public init(_ tile: Tile, _ words: [String: String]) {
        self.tile = tile
        self.words = words
    }

    public var body: some SwiftUI.View {
        let badges = (tile.badges ?? []).map { badgeLabel($0, words) }.filter { !$0.isEmpty }
        if !badges.isEmpty {
            HStack(spacing: 6) {
                ForEach(Array(badges.enumerated()), id: \.offset) { _, word in
                    Text(word)
                        .brand(12, 700)
                        .foregroundStyle(.white)
                        .lineLimit(1)
                        .padding(.horizontal, 8)
                        .padding(.vertical, 3)
                        .background(Color.black.opacity(0.72), in: RoundedRectangle(cornerRadius: Round.chip))
                }
            }
            // Drawn inside a control that speaks for all of it. See the type's own note.
            .decoration()
        }
    }
}

/// What a tile is called when a reader stops on it: its name, then whatever its badges say, then
/// how far through it the viewer is.
///
/// Every piece comes from the answer or from `strings/`; nothing here is an English literal. The
/// join is a line end rather than a comma, so no language's punctuation is decided in this file --
/// the same bargain `sayBoth` makes in `SurfacesShared/Says.swift`, which this uses.
public func tileWords(_ tile: Tile, _ words: [String: String]) -> String {
    var said = tile.name
    for badge in tile.badges ?? [] {
        let word = badgeLabel(badge, words)
        if !word.isEmpty { said = sayBoth(said, word) }
    }
    if let percent = tile.percent, percent > 0 {
        said = sayBoth(said, (words["say.watched_percent"] ?? "").fill("percent", percent))
    }
    return said
}
