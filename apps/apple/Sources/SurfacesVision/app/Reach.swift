import SwiftUI

/// What a viewer does on this surface, and the assumption it breaks.
///
/// Both of this family's other surfaces assume the app decides where attention goes, or that
/// nothing decides.
///
/// The television decides. `SurfacesTV/app/Remote.swift` turns an arrow key into a `Word`, and
/// every television screen keeps the answer as plain state: `PageFocus` holds a row and a column,
/// `Summary` holds which line of the menu is open, `KeyboardState` holds a row and a column of
/// keys. The screen paints a white outline wherever its own integer says, and one box per screen
/// tells a screen reader what that integer now means.
///
/// The phone does not decide, because nothing is focused: a finger arrives where it arrives and
/// there is no state before the press at all.
///
/// Here the system decides, and the app is not told. The viewer looks at a thing and the system
/// lights it; the app hears about a pinch and never about a direction, and -- by design, for the
/// viewer's privacy -- never about where the eyes are. That inverts the television's model rather
/// than adjusting it, so none of it is carried over: there is no `Word` in this module, no key
/// map, and no integer anywhere that means "where the viewer is".
///
/// Two consequences are worth writing down because they are easy to get wrong by habit.
///
/// `says(_:on:)` and `saysItsWords(on:)` from `SurfacesShared/Says.swift` are never called in this
/// module. Those add the `selected` trait from the surface's own idea of focus, which is the right
/// answer on a television whose outline is painted and wrong here: the app would be announcing a
/// selection it invented, in a place where the real one is known only to the system.
///
/// And there is no screen-level announcement box. The television needs one because its parts are
/// not system-focusable, so a screen reader following the remote has nothing to follow; here every
/// target is a real control the system can focus on its own, and a box repeating what the focused
/// control already says would be a second voice.
///
/// What is left for the app is the whole of this file: make every target big enough and far enough
/// from its neighbours to be looked at, give it a shape the system can light, and give it words.
/// ``SwiftUI/View/reachable(says:state:corner:)`` does those four together, so that a control
/// cannot end up with three of them.
///
/// ## Eye and pinch is not the only input, and designing for it alone excludes people
///
/// This is the part of the surface most easily skipped, so it is written down with the rest of the
/// design rather than left for a pass afterwards. A person using this platform may be using:
///
/// - **VoiceOver**, hearing the screen and activating whatever its cursor is on. Covered by every
///   control carrying words from `strings/` and by nothing being announced that the app invented;
///   `Tests/VisionTests/Speech.swift` holds both.
/// - **Dwell Control**, which activates a target by resting on it. A dwell cannot hold a pinch and
///   move, so **nothing on this surface may need a drag**: there is no scrub-by-drag in the player
///   (`watch/Watch.swift` argues it), no swipe that does something a press cannot, and no control
///   that reveals itself only while held.
/// - **Pointer Control**, aiming with a head or a wrist instead of the eyes. Coarser than a gaze,
///   which is what ``target`` and ``between`` are really sized for.
/// - **A hardware keyboard or trackpad**, where the system moves focus itself.
///
/// All four are served by one decision rather than four: **every target is a standard `Button`**.
/// A `Button` is activated by VoiceOver, by a dwell, by a pointer and by a keyboard without this
/// module knowing that any of them exist. A custom gesture recogniser reading a pinch would have
/// worked for eye-and-pinch alone and silently excluded the rest, and it would have been the
/// invented gesture this surface refuses anyway.
///
/// The second decision is in `browse/Rows.swift`: **nothing is revealed on hover.** A tile draws
/// its title's name at rest rather than on focus, because on a surface whose pointer is the gaze,
/// anything shown on hover is shown only to the one thing being looked at -- useless to somebody
/// scanning the page, and a paid dwell for somebody using Dwell Control just to read a label.
public enum Reach {
    /// The smallest a thing the viewer looks at and pinches may be, in points.
    ///
    /// **Apple's published visionOS guidance, not a measurement taken here.** It puts the minimum
    /// at 60 pt and asks for 60 pt between the centres of neighbouring targets. Nothing on this
    /// machine can check it against a device: there is no Vision Pro here, and no visionOS
    /// simulator either (`xcrun simctl list runtimes` on 2026-11-14 answered iOS 26.5 and tvOS
    /// 26.5 and nothing else). It is written once, here, so that the day it is measured on a
    /// device there is one number to change and not sixty call sites.
    ///
    /// It is not the television's number wearing a different coat. A television target is large
    /// because the viewer is three metres away; this one is large because the eye cannot be aimed
    /// finely and the app is never told where it is pointing, so a target that is nearly big
    /// enough fails silently rather than visibly.
    public static let target: CGFloat = 60

    /// The smallest gap allowed between the centres of two neighbouring targets, in points.
    ///
    /// The same 60 pt, from the same guidance. There is deliberately no separate padding constant
    /// derived from it: two neighbouring targets that are each at least ``target`` across already
    /// have their centres at least that far apart, even touching. So the spacing rule is a
    /// consequence of the size rule rather than a second thing to remember, and `Tests/VisionTests`
    /// asserts it of the real laid-out frames instead of trusting the argument.
    public static let between: CGFloat = 60

    /// The narrowest a tile in a row may be drawn, in points. **This surface's own, derived.**
    ///
    /// A tile is a picture in 16:9 (the one tile shape both references agree on, measured
    /// independently: `reference/netflix-on-the-tv.md` reads 906 x 510 = 1.776 for a focused
    /// television tile, and `reference/netflix-on-the-web.md` reads the image inside a web tile as
    /// exactly 16:9). It is also a target, so its short side is what has to clear ``target``.
    ///
    /// Clearing it by exactly one would be a tile whose whole height is the smallest thing the
    /// guidance allows, with a badge and a progress bar drawn inside it. So the short side is held
    /// to two targets, and the width follows from the shape. That factor of two is a judgement and
    /// not a measurement, and it is the only arbitrary number in this file.
    public static let tileFloor: CGFloat = (2 * target) * 16 / 9
}

extension View {
    /// A thing the viewer looks at and pinches, whose own words are drawn inside it: a tab, a
    /// button with a word on it, a line of a menu.
    ///
    /// Use this and not ``reachable(says:state:corner:)`` wherever a word is already on screen, for
    /// the reason `Says.swift` gives: the label then cannot drift from what the viewer sees.
    public func reachable(corner: CGFloat = Round.card) -> some SwiftUI.View {
        modifier(Reachable(corner: corner, words: "", state: "", gathered: true))
    }

    /// The same, for a target with no words drawn on it: a tile, a face, an icon-only control.
    ///
    /// `says` is words a person hears, so it comes from `strings/` or from the title's own name in
    /// the answer, and is never an English literal. `state` is how the thing stands, read after
    /// the label -- "In My List", "72% watched" -- and is from `strings/` too.
    public func reachable(says: String, state: String = "", corner: CGFloat = Round.card) -> some SwiftUI.View {
        modifier(Reachable(corner: corner, words: says, state: state, gathered: false))
    }
}

/// Big enough, shaped, lit, and spoken: the four things a target on this surface needs.
///
/// The minimum is set on the layout and not only on the hit area, which is the one decision in here
/// that costs something. A hit area larger than the drawing is the usual trick on a touch surface,
/// and it is wrong here twice over: the system draws the highlight over the shape this view hands
/// it, so an invisible margin would light up as part of the control; and the spacing rule above is
/// about the distance between centres, which an invisible margin does not change. So a control too
/// small to look at grows on the screen, where the designer can see that it did.
private struct Reachable: ViewModifier {
    let corner: CGFloat
    let words: String
    let state: String
    /// True when the words are drawn inside and should be gathered rather than set.
    let gathered: Bool

    func body(content: Content) -> some SwiftUI.View {
        let shape = RoundedRectangle(cornerRadius: corner, style: .continuous)
        return content
            .frame(minWidth: Reach.target, minHeight: Reach.target)
            // The shape assistive technology is told the control occupies, and
            // without it this whole file's guarantee does not reach the one place that can check
            // it. Measured on the iOS simulator on 2026-11-14: a one-letter control inside
            // `.frame(minWidth: 60, minHeight: 60)` reports an accessibility frame of 11.3 x 20.3
            // -- the ink of the letter -- because the frame modifier centres its content and the
            // element follows the content. With the kind added it reports 60 x 60. The control was
            // always 60 pt to a finger or a gaze; what changed is that it is now 60 pt to VoiceOver
            // and to `Tests/VisionTests/Reach.swift`, which is the difference between a rule and a
            // rule somebody can check.
            .contentShape(.accessibility, shape)
            .modifier(Lit(shape: shape))
            .modifier(Spoken(words: words, state: state, gathered: gathered))
    }
}

/// The shape the system lights under a gaze or a pointer, where there is one.
///
/// Left out, the effect is drawn around whatever the view's bounds happen to be, which for a
/// clipped picture is a rectangle with square corners over a card with round ones.
///
/// Guarded, and the guard is not about visionOS. This module is in a package that declares tvOS, so
/// `-only-testing:TelevisionTests` still compiles it for tvOS -- and `hoverEffect` is tvOS 18, while
/// this family's floor is tvOS 17. Without this, adding the window surface turned the television
/// target red: measured on 2026-11-16, `check_apple.py` reported TelevisionTests running no test at
/// all with `'hoverEffect' is only available in tvOS 18.0 or newer`, while VisionTests passed 25.
/// A television has neither a pointer nor a gaze, so there is nothing for the effect to do there;
/// what it would have cost is the other surface's whole test target.
private struct Lit<S: Shape>: ViewModifier {
    let shape: S

    func body(content: Content) -> some SwiftUI.View {
        #if os(tvOS)
        content
        #else
        content.contentShape(.hoverEffect, shape).hoverEffect()
        #endif
    }
}

/// The words, in the one vocabulary this family has for them (`SurfacesShared/Says.swift`).
///
/// An unset state is left unset rather than set to nothing: an empty accessibility value is a
/// control a reader stops on and announces a blank for, and `words[...]` is empty for as long as a
/// run has not read its language yet, which is every run's first moment.
private struct Spoken: ViewModifier {
    let words: String
    let state: String
    let gathered: Bool

    @ViewBuilder
    func body(content: Content) -> some SwiftUI.View {
        if gathered {
            if state.isEmpty { content.saysItsWords() } else { content.saysItsWords(state: state) }
        } else {
            if state.isEmpty { content.says(words) } else { content.says(words, state: state) }
        }
    }
}
