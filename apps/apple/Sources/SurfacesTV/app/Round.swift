import SwiftUI

/// How much larger a corner is on a television than on a phone, in reference pixels per brand unit.
private let tvScale: CGFloat = 1.5

/// The brand's corners on a television, by role, in reference pixels. Every rounded part takes one
/// of these, so brand.json changes them all at once. These equal Round.kt.
enum Round {
    /// Posters, stills, episode pictures, the picture grid.
    static let card = (Corner.card * tvScale).rounded()

    /// The hero, the landing cards, menus, panels.
    static let panel = (Corner.panel * tvScale).rounded()

    /// Fields, boxes of text, the code plate, PIN boxes.
    static let field = (Corner.field * tvScale).rounded()

    /// Keys and letters: smaller than a field.
    static let key = (field * 2 / 3).rounded(.down)

    /// Badges and small labels.
    static let chip = (field / 2).rounded(.down)

    /// A corner that sits inside another, `inset` from its edge, so the two curves share a centre.
    static func inside(_ outer: CGFloat, _ inset: CGFloat) -> CGFloat { max(outer - inset, chip) }

    /// A profile's face, its focus ring and the add tile: the corner is a share of the size.
    static func face(_ size: CGFloat) -> CGFloat { size * (card * 100 / 130).rounded(.down) / 100 }
}

/// The reference screen this surface is drawn on, in reference pixels: 1920 x 1080, the size every
/// television frame in `notes/netflix-reference/tv` was measured at.
let pageWide: CGFloat = 1920
let pageHigh: CGFloat = 1080

/// The page margin on a television, left and right, in reference pixels. Pages line up their edges on it.
let pageEdge: CGFloat = 72

/// Pages without the top bar (settings, help, profile edits) keep their words on this column.
let subEdge: CGFloat = 130

/// A rounded rectangle with a corner in reference pixels.
func shapeOf(_ corner: CGFloat) -> RoundedRectangle { RoundedRectangle(cornerRadius: corner.ref) }
