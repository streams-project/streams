//! The width a page has to work with, by class, never by device name.
//!
//! The phone's `apps/android/surfaces/phone/src/main/kotlin/streams/phone/app/Sizes.kt`, number for
//! number: the two breakpoints at `Sizes.kt:18-19` and the three margins at `Sizes.kt:26-30`. That
//! file says "These equal Sizes.swift" and the preservation table of note 215 names the margins as
//! a thing that must not be lost, so this is the third copy and the tests below hold all three
//! together.
//!
//! A CSS pixel is read as a dp here, which is what the rest of this crate already does: `host.rs`
//! compares a viewport in CSS pixels against the same 600 the phone compares a width in dp
//! against. It holds because a mobile browser's layout viewport under `width=device-width` is the
//! screen in dp - the page's own `index.html` asks for exactly that - and it is stated here rather
//! than left as a coincidence between two files.

/// Where a window stops being compact and where it stops being medium. `Sizes.kt:18-19`.
const COMPACT_UNDER: f64 = 600.0;
const MEDIUM_UNDER: f64 = 840.0;

/// The side margin of a page by class: 12 on a phone, wider on tablets. `Sizes.kt:26-30`.
///
/// 12 is the row's own measured gutter (`browse/Rows.kt:89`) and 24 is twice it. **40 has no
/// derivation and no citation anywhere in this repository** - `NumbersTest.kt:285-291` says so of
/// the same number - so it is carried across unchanged rather than reasoned about, and the test
/// below holds it to the phone's copy and to its place in an increasing sequence.
const EDGE_COMPACT: u32 = 12;
const EDGE_MEDIUM: u32 = 24;
const EDGE_EXPANDED: u32 = 40;

/// The class of a window, as the phone names them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WidthClass {
    Compact,
    Medium,
    Expanded,
}

impl WidthClass {
    /// The class of a window this wide, in CSS pixels. `Sizes.kt:15-22`.
    #[must_use]
    pub fn of(wide: f64) -> Self {
        if wide < COMPACT_UNDER {
            Self::Compact
        } else if wide < MEDIUM_UNDER {
            Self::Medium
        } else {
            Self::Expanded
        }
    }

    /// The side margin of a page in this class. `Sizes.kt:26-30`.
    #[must_use]
    pub const fn page_edge(self) -> u32 {
        match self {
            Self::Compact => EDGE_COMPACT,
            Self::Medium => EDGE_MEDIUM,
            Self::Expanded => EDGE_EXPANDED,
        }
    }

    /// The word the contract's `Form` is named by for this class.
    ///
    /// The core keys its idle limit before the profile gate asks again on this one word
    /// (`platform/core/src/gate.rs:15-16`: two hours on a phone, forty minutes otherwise), so the
    /// class and the form are read off one pair of breakpoints rather than two that can drift. They
    /// did drift: this crate held 600 and 1024 against the phone's 600 and 840, and a window of
    /// exactly 600 was a phone here and medium there.
    #[must_use]
    pub const fn form(self) -> &'static str {
        match self {
            Self::Compact => "phone",
            Self::Medium => "tablet",
            Self::Expanded => "desktop",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        COMPACT_UNDER, EDGE_COMPACT, EDGE_EXPANDED, EDGE_MEDIUM, MEDIUM_UNDER, WidthClass,
    };
    use crate::surfaces::tests_support::{numbers, source};

    const PHONE_SIZES: &str =
        "apps/android/surfaces/phone/src/main/kotlin/streams/phone/app/Sizes.kt";

    #[test]
    fn a_window_is_in_the_class_its_width_puts_it_in() {
        assert_eq!(WidthClass::of(390.0), WidthClass::Compact);
        assert_eq!(WidthClass::of(599.0), WidthClass::Compact);
        // Exactly the breakpoint is the next class up, as `wide < 600` is on the phone.
        assert_eq!(WidthClass::of(600.0), WidthClass::Medium);
        assert_eq!(WidthClass::of(839.0), WidthClass::Medium);
        assert_eq!(WidthClass::of(840.0), WidthClass::Expanded);
        assert_eq!(WidthClass::of(1440.0), WidthClass::Expanded);
        // A width a browser would not give reads as the class with the shortest idle limit, which
        // is what `host.rs` relies on: an unknown device is never left open longer than a known one.
        assert_eq!(WidthClass::of(f64::INFINITY), WidthClass::Expanded);
    }

    /// The two breakpoints and the three margins are the phone's, read out of the phone's own file.
    ///
    /// The whole point of this test. `Sizes.kt:9` claims "These equal Sizes.swift" and
    /// `NumbersTest.kt:294` is the check of that claim; nothing tied the web to either, so the web
    /// drew a 48 pixel page edge from the brand's gutter and a 16 pixel one under 720, neither of
    /// which is any measurement of anything.
    #[test]
    fn the_breakpoints_and_margins_are_the_phone_s() {
        let kotlin = source(PHONE_SIZES);
        assert!(
            kotlin.contains("These equal Sizes.swift"),
            "the claim this test is the web's half of has gone from Sizes.kt"
        );
        let breaks = numbers(
            &[
                "wide < ",
                " -> WidthClass.COMPACT wide < ",
                " -> WidthClass.MEDIUM",
            ],
            &kotlin,
            "the phone's breakpoints",
        );
        assert_eq!(breaks, vec![COMPACT_UNDER, MEDIUM_UNDER]);
        let margins = numbers(
            &[
                "WidthClass.COMPACT -> ",
                " WidthClass.MEDIUM -> ",
                " WidthClass.EXPANDED -> ",
                "",
            ],
            &kotlin,
            "the phone's page margins",
        );
        assert_eq!(
            margins,
            vec![
                f64::from(EDGE_COMPACT),
                f64::from(EDGE_MEDIUM),
                f64::from(EDGE_EXPANDED)
            ]
        );
    }

    /// A phone's margin is the row's own measured gutter, a medium window's is twice it, and the
    /// three grow with the window. `NumbersTest.kt:321-324` holds the phone to the same three.
    #[test]
    fn the_margins_grow_with_the_window_from_the_row_s_own_gutter() {
        let rows =
            source("apps/android/surfaces/phone/src/main/kotlin/streams/phone/browse/Rows.kt");
        let gutter = numbers(
            &["screen's edge (reference: ", " and ", " dp)"],
            &rows,
            "the row's measured gaps",
        )[1];
        assert_eq!(
            f64::from(EDGE_COMPACT),
            gutter.round(),
            "a phone's margin is the measured {gutter} dp gutter rounded"
        );
        assert_eq!(EDGE_MEDIUM, 2 * EDGE_COMPACT);
        // The three margins and the two breakpoints grow with the window. Held as a sorted list
        // rather than as a chain of comparisons, because a comparison of two constants is a
        // comparison the compiler answers and clippy refuses.
        let margins = [EDGE_COMPACT, EDGE_MEDIUM, EDGE_EXPANDED];
        let mut rising = margins.to_vec();
        rising.sort_unstable();
        rising.dedup();
        assert_eq!(rising, margins.to_vec(), "the margins have to grow");
        let breaks = [COMPACT_UNDER, MEDIUM_UNDER];
        let mut order = breaks;
        order.sort_unstable_by(f64::total_cmp);
        assert_eq!(order, breaks, "and so do the breakpoints");
        assert_ne!(COMPACT_UNDER, MEDIUM_UNDER);
    }

    /// The style sheet draws the margin this file states, for all three classes.
    ///
    /// The numbers are in two places - here, for what a surface computes, and the style sheet, for
    /// what the browser draws - and a page whose Rust and whose CSS disagreed about its own margin
    /// would put a placeholder row's tiles at a different gutter from the real row that replaces
    /// them. So the style sheet is read as text and held to these three.
    #[test]
    fn the_style_sheet_draws_the_same_three_margins() {
        let css = source("apps/web/style.css");
        assert_eq!(
            numbers(
                &[
                    ":root { --edge: ",
                    "px; } @media (min-width: ",
                    "px) { :root { --edge: ",
                    "px; } } @media (min-width: ",
                    "px) { :root { --edge: ",
                    "px; } }",
                ],
                &css,
                "the page edge in the style sheet",
            ),
            vec![
                f64::from(EDGE_COMPACT),
                COMPACT_UNDER,
                f64::from(EDGE_MEDIUM),
                MEDIUM_UNDER,
                f64::from(EDGE_EXPANDED),
            ]
        );
    }
}
