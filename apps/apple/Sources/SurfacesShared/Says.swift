import SwiftUI

/// What a screen reader is handed. The same six jobs as the android family's
/// `surfaces/shared/.../Says.kt`, in SwiftUI idiom, so the two families do not grow two different
/// vocabularies for one problem.
///
/// Measured on this family on 2026-11-12, before any of this existed: `rg
/// "accessibilityLabel|accessibilityElement|accessibilityValue"` over `apps/apple/Sources` matched
/// nothing, against 99 call sites on android. So nothing on either Apple surface was labelled.
/// SwiftUI does derive a label from a `Text` inside a `Button`, which is why much of the phone
/// surface spoke already; what it also does, measured through `Tests/Heard`, is read a picture out
/// as the name of its own asset file - `Icon("icon_search", 34)` reads "icon_search", `Mark(60)`
/// reads "mark" - which is not words and is in no language.
///
/// A television is where this matters most and is least often done. Both Apple surfaces draw their
/// own focus from plain state (`SurfacesTV/app/Remote.swift`), so the toolkit holds no per-control
/// focus for a screen reader to follow; and a remote gives a person no way to explore a screen by
/// touch, so an unlabelled control there is unreachable rather than merely unlabelled.
///
/// The words are never written here. They arrive from `strings/{lang}.json` through `seen.words`,
/// or from the catalogue, where a title's name is already in the viewer's language. A key the
/// strings do not hold leaves the part unlabelled rather than putting English in the code.
extension View {
    /// A control whose own words are drawn inside it. Everything under it is read as one thing, so
    /// the word beside the icon becomes the button's name.
    ///
    /// Use this and not ``says(_:)`` wherever a word is already on screen: the label then cannot
    /// drift from what the viewer sees, and it needs no key of its own.
    public func saysItsWords() -> some View {
        accessibilityElement(children: .combine).accessibilityAddTraits(.isButton)
    }

    /// The same, for a control that also stands one of two ways and says so with something that is
    /// not words - a dot, a tick, a filled bar. `state` is those words, from `strings/`, read after
    /// the control's own.
    public func saysItsWords(state: String) -> some View {
        saysItsWords().accessibilityValue(state)
    }

    /// A control with no words on it: an icon, a picture, a tile. `label` is words a person hears,
    /// so it comes from `strings/` or from the title's own name, and is never an English literal.
    ///
    /// An empty label falls through to ``saysItsWords()`` rather than being set. Measured on
    /// iPhone 17 on 2026-11-12: `.accessibilityLabel("")` leaves a node a reader stops on with
    /// nothing to read, which is worse than one it falls through to the contents of - and
    /// `words[...]` is empty for as long as a run has not read its language yet.
    @ViewBuilder
    public func says(_ label: String) -> some View {
        if label.isEmpty {
            saysItsWords()
        } else {
            accessibilityElement(children: .ignore)
                .accessibilityLabel(label)
                .accessibilityAddTraits(.isButton)
        }
    }

    /// A control that is on or off - in My List or not, a reminder set or not. `state` is the words
    /// for which of the two it is now, again from `strings/`, and is read after the label.
    public func says(_ label: String, state: String) -> some View {
        says(label).accessibilityValue(state)
    }

    /// A switch. The toolkit's own `Toggle` already says which way it stands and that it is a
    /// switch; it does not say what it switches, because the words for that are drawn beside it as
    /// a row of text the switch is no part of. `label` is those words, from `strings/`.
    @ViewBuilder
    public func saysSwitch(_ label: String) -> some View {
        if label.isEmpty {
            self
        } else {
            accessibilityLabel(label).accessibilityAddTraits(.isToggle)
        }
    }

    /// A picture that carries no meaning: the art inside a tile that already says the title's name,
    /// a gradient, a plate, a brand mark beside the page's own name.
    ///
    /// Marking it is as much of the work as labelling a control. An unmarked decoration is one more
    /// thing a reader stops on, and on this family it does not even stop on nothing: it reads the
    /// asset's file name aloud.
    public func decoration() -> some View {
        accessibilityHidden(true)
    }
}

/// Where this family differs from the android one, and why.
///
/// Android keeps two files: six helpers in `shared/.../Says.kt` for the phone, and five more in
/// `tv/.../Spoken.kt` for the television, because its two surfaces are separate Gradle modules and
/// the television's focus is painted rather than held by the toolkit. This family has one file,
/// because `SurfacesShared` is `@_exported` into both surfaces (`SurfacesTV/app/Shared.swift`,
/// `SurfacesPhone/app/Shared.swift`): a second copy of the same vocabulary per surface would be the
/// divergence, not the sharing. So the television's two extra jobs are the two `on:` forms below,
/// and they live here beside the rest.
extension View {
    /// A part the remote can stop on, whose own drawn words are its label: a pill, a tab, a key, a
    /// line of a menu. `on` is what a screen reader says as "selected", which on a surface whose
    /// focus is painted is the only thing that tells a person where they are.
    public func saysItsWords(on: Bool) -> some View {
        saysItsWords().accessibilityAddTraits(on ? [.isSelected] : [])
    }

    /// A part the remote can stop on with no words drawn on it: a poster, a face, a rating. `label`
    /// comes from `strings/` or from the catalogue, and `on` says whether the remote is on it now.
    public func says(_ label: String, on: Bool) -> some View {
        says(label).accessibilityAddTraits(on ? [.isSelected] : [])
    }
}

/// Two things a person hears one after the other, as one label: a title and the row it sits in, a
/// name and its state. The break is a line end rather than a word or a comma, so no language's
/// punctuation or word order is decided in the code. Mirrors `sayBoth` in android's `Spoken.kt`.
public func sayBoth(_ first: String?, _ second: String?) -> String {
    [first, second].compactMap { $0 }.filter { !$0.isEmpty }.joined(separator: "\n")
}
