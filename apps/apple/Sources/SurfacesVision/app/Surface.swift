import SwiftUI

/// What this surface is drawn on, and how far forward each thing stands.
///
/// Both of the other surfaces in this family paint a dark plate and then paint on it: `Back` is
/// `#0B0B10`, `Plate` is `#16161D`, `Raised` is `#24242E`, and every word on both is literal white
/// (`SurfacesShared/Theme.swift`). That is right for a television in a dim room and for a phone
/// whose screen is the only light. It is wrong here, twice.
///
/// It is wrong because there is nothing to paint on. A window here hangs in the viewer's own room
/// and the system draws its material; the room shows through it. Filling that with `#0B0B10` makes
/// an opaque black slab floating in a living room -- it throws away the one thing that makes a
/// window in space readable, which is that the system continuously adjusts its material to the
/// light actually falling on it. And it is wrong because white ink assumes a dark plate: on glass
/// over a bright room, white on white is what the system's own semantic colours exist to prevent.
///
/// So on this platform the plate is the system's and the ink is semantic. The brand does not
/// disappear -- `Accent` and `Join` still mark what they always marked, artwork still fills a tile,
/// and the brand's font and corners are unchanged. What goes is the slab and the literal white.
///
/// Nothing here is checked by anything. There is no visionOS simulator on this machine and no
/// device, so how any of this reads against a real room is the first thing a device has to settle.

/// The ink of a word that carries the sentence.
public var Ink: Color {
    #if os(visionOS)
    .primary
    #else
    .white
    #endif
}

/// The ink of a word beside it: a fact, a count, a caption.
public var InkSoft: Color {
    #if os(visionOS)
    .secondary
    #else
    .white.opacity(0.75)
    #endif
}

extension SwiftUI.View {
    /// What a panel stands on: the system's glass here, the brand's raised plate elsewhere.
    ///
    /// One helper rather than a material at each draw site, so the day a device says the glass is
    /// wrong there is one place to change.
    @ViewBuilder
    public func plate(_ corner: CGFloat) -> some SwiftUI.View {
        #if os(visionOS)
        glassBackgroundEffect(in: .rect(cornerRadius: corner))
        #else
        background(Raised, in: RoundedRectangle(cornerRadius: corner, style: .continuous))
        #endif
    }

    /// Stands this view forward of what is behind it.
    ///
    /// **Depth is layout here, not decoration**, and this is the part of the television's model
    /// that genuinely has somewhere else to go. A television expands its focused tile to 2.91x the
    /// width of a closed one (`reference/netflix-on-the-tv.md`) and pins the focused row to a fixed
    /// band, because a flat panel has no way to bring a thing nearer and must spend screen area
    /// instead. That expansion costs it a row: the same measurement records 0 rows fully visible on
    /// its first screen. Here the z axis does that job and costs no area at all.
    ///
    /// What this modifier does **not** do is move anything when the viewer looks at it. It cannot:
    /// the app is never told where the eyes are, and that is the platform working as intended. The
    /// lift under a gaze is the system's own hover effect, drawn out of this process, and
    /// reimplementing it would be both impossible and the "invented gesture" this surface refuses.
    /// What the app sets is each layer's **resting** depth, so that the system's lift has somewhere
    /// to rise from and a sheet reads as being over a page rather than cut into it.
    ///
    /// On every other platform this compiles for it is nothing, which also keeps the layout the
    /// tests measure identical on both.
    @ViewBuilder
    public func forward(_ depth: CGFloat) -> some SwiftUI.View {
        #if os(visionOS)
        offset(z: depth)
        #else
        self
        #endif
    }
}

/// How far forward each layer rests, in points. **This surface's own**, unmeasured, and small on
/// purpose: these are separations meant to be felt rather than seen, and a device is the only place
/// to find out whether they are.
public enum Depth {
    /// A sheet or a panel over the page: a title's facts, a menu.
    public static let panel: CGFloat = 16

    /// The controls over a playing picture, which have to read as being in front of it rather than
    /// composited into it.
    public static let controls: CGFloat = 24
}
