//! What a page shows before it has anything.
//!
//! The port of `apps/android/surfaces/phone/src/main/kotlin/streams/phone/browse/Empty.kt`, and the
//! same reading of the reference it was written from: a row's name drawn as a plain bar and its
//! tiles as flat rounded rectangles at their final size, with no shimmer, no message, no retry and
//! no sign of waiting (`Empty.kt:24-33`, citing `netflix-reference/00-gaps.md:128-132`). That is
//! what is here and that is all that is here.
//!
//! This replaces a 44 pixel spinning ring, which is the one thing the reference does not draw.
//!
//! What crosses over as a number and what crosses over as a rule, stated because the difference is
//! the whole of the care here:
//!
//! * the count of rows is the phone's constant, because the derivation behind it needs a screen
//!   height and a row pitch measured on one device and the web has neither;
//! * the count of tiles in a row is the phone's **rule** applied to the browser's own width, which
//!   is what `Empty.kt:54` does with the phone's;
//! * every size - the tile, the gap, the row's gutter, the bar's height - is the web row's own,
//!   because the placeholder exists to be the same shape as the row that replaces it. A placeholder
//!   built from the phone's 119 x 166.6 dp portrait tile would jump the moment the answer arrived,
//!   since this surface draws a 16 : 9 tile of its own (`style.css`, `.card`).

use leptos::prelude::*;

use super::sizes::WidthClass;
use crate::state::Shared;

/// Rows a waiting page draws. `Empty.kt:36`, where the derivation is 936.6 dp of screen over a
/// 217.0 dp row pitch, which is 4.3, and a third of a row is not drawn.
///
/// Carried across rather than derived again: the division needs a measured screen and a measured
/// row pitch, and no capture of this project's own web rows exists to measure a pitch off. The test
/// below holds the count to the phone's own stated derivation, so the day that derivation changes
/// this fails rather than quietly disagreeing with it.
const WAITING_ROWS: usize = 4;

/// The tile's width and the gap between tiles, in CSS pixels, as `style.css` draws them: a tile is
/// `max(190px, (row - 40px) / 6)` wide and the gap is `--unit * 2`.
///
/// Held here as well as in the style sheet because the count of tiles a row holds is computed from
/// them, and the test below holds the two copies together.
const TILE_LEAST_WIDE: f64 = 190.0;
const TILES_ACROSS_ROW: f64 = 6.0;
const STRIP_SLACK: f64 = 40.0;
const TILE_GAP: f64 = 8.0;

/// A tile's width in a row this wide, in CSS pixels. The style sheet's own `flex` basis.
fn tile_wide(row_wide: f64) -> f64 {
    TILE_LEAST_WIDE.max((row_wide - STRIP_SLACK) / TILES_ACROSS_ROW)
}

/// Tiles a row holds: the screen less the row's start, over the tile pitch, rounded up - so the
/// last one is cut by the edge as the reference's is. `Empty.kt:54`, and `:78-82` for why it is
/// rounded up rather than down.
///
/// The arguments are the phone's, in its order, so the rule can be fed the phone's own numbers and
/// checked against the answer `Empty.kt:81` states for them.
#[must_use]
pub fn tiles_across(screen: f64, row_start: f64, tile: f64, gap: f64) -> usize {
    let pitch = tile + gap;
    if pitch <= 0.0 || screen <= row_start {
        return 1;
    }
    (((screen - row_start) / pitch).ceil() as usize).max(1)
}

/// How wide the window is, in CSS pixels. A width a browser will not give reads as the widest
/// class, which is the one that draws the fewest tiles, so a page that cannot measure itself draws
/// less rather than more.
fn window_wide() -> f64 {
    web_sys::window()
        .and_then(|window| window.inner_width().ok())
        .and_then(|width| width.as_f64())
        .filter(|width| width.is_finite() && *width > 0.0)
        .unwrap_or(TILE_LEAST_WIDE * TILES_ACROSS_ROW)
}

/// A page with nothing in it yet: names as bars, tiles flat at their final size. Nothing moves, as
/// nothing moves on the reference.
///
/// The whole block is one `aria-hidden` region with a `status` beside it. A screen reader otherwise
/// reads out four rows of empty boxes, which says less than nothing; what it is told instead is
/// that the page is loading, once, from `say.loading`.
#[component]
pub fn WaitingRows() -> impl IntoView {
    let shared = expect_context::<Shared>();
    let across = move || {
        let wide = window_wide();
        let edge = f64::from(WidthClass::of(wide).page_edge());
        let row = wide - edge * 2.0;
        tiles_across(wide, edge, tile_wide(row), TILE_GAP)
    };
    view! {
        <div class="waiting">
            <p class="waiting-said" role="status" aria-live="polite">
                {move || super::label_text(shared, "say.loading")}
            </p>
            <div aria-hidden="true">
                {move || (0..WAITING_ROWS).map(|_| view! {
                    <section class="row">
                        // An empty heading box, so the bar stands at the height of the name it
                        // replaces by construction rather than by a second number: one em inside
                        // the heading's own line box. `Empty.kt:38-45` measured that off the
                        // reference as a 30.8 dp bar against a 30 dp name box in a 34.3 dp block,
                        // which is the same statement.
                        <h2 class="bar-name"><span class="bar"></span></h2>
                        <div class="strip strip-flat">
                            {(0..across()).map(|_| view! { <div class="card"></div> }).collect_view()}
                        </div>
                    </section>
                }).collect_view()}
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::{
        STRIP_SLACK, TILE_GAP, TILE_LEAST_WIDE, TILES_ACROSS_ROW, WAITING_ROWS, tile_wide,
        tiles_across,
    };
    use crate::surfaces::tests_support::{english, numbers, one_number, source};

    const PHONE_EMPTY: &str =
        "apps/android/surfaces/phone/src/main/kotlin/streams/phone/browse/Empty.kt";
    const PHONE_ROWS: &str =
        "apps/android/surfaces/phone/src/main/kotlin/streams/phone/browse/Rows.kt";

    /// The rows drawn are the whole rows the phone's own derivation gives, and that derivation
    /// divides the two numbers it states.
    ///
    /// The shape `NumbersTest.kt:261` holds the phone to, from the other side: this one fails both
    /// when the phone's screen or pitch changes and when the count here stops being the answer.
    #[test]
    fn the_rows_drawn_are_the_whole_rows_the_phone_s_derivation_gives() {
        let empty = source(PHONE_EMPTY);
        let said = numbers(
            &[
                "Rows a phone screen holds: ",
                " dp of screen over a ",
                " dp row pitch is ",
                ".",
            ],
            &empty,
            "the waiting rows' derivation",
        );
        // The comment's own division, to the one digit it carries.
        let (screen, pitch, answer) = (said[0], said[1], said[2]);
        assert_eq!(
            (screen / pitch * 10.0).round() / 10.0,
            answer,
            "{screen} over {pitch} is not the {answer} the comment states"
        );
        assert_eq!(
            (screen / pitch).floor() as usize,
            WAITING_ROWS,
            "the rows drawn are the whole ones of {answer}"
        );
        // And the phone draws the same count, so the two surfaces wait alike.
        assert_eq!(
            one_number(&["WAITING_ROWS = ", "\n"], &empty, "the phone's row count"),
            WAITING_ROWS as f64
        );
    }

    /// The rule that counts a row's tiles gives the phone's answer when it is fed the phone's
    /// numbers.
    ///
    /// This is the one real derivation check on this file: `Empty.kt:78-82` states the whole of it
    /// in prose - the screen less the row's start, over the tile pitch, rounded up, which on the
    /// reference's phone is four, three full and a cut fourth against a measured 3.29 visible - and
    /// all four of those numbers live in the phone's source. A rule that rounded down, or divided
    /// by the tile without its gap, fails here.
    #[test]
    fn the_rule_that_counts_a_row_s_tiles_gives_the_phone_s_answer() {
        let empty = source(PHONE_EMPTY);
        let rows = source(PHONE_ROWS);
        let tile = numbers(
            &["Tiles on the reference: ", " x ", " dp ("],
            &rows,
            "the phone's tile",
        );
        let gaps = numbers(
            &["screen's edge (reference: ", " and ", " dp)"],
            &rows,
            "the phone's gaps",
        );
        let phone = numbers(
            &[
                "On the reference's ",
                " dp phone that is four -- three full and a cut fourth, which is the measured ",
                " visible",
            ],
            &empty,
            "the phone's own answer",
        );
        let (screen, visible) = (phone[0], phone[1]);
        // The gaps in the code are the measured ones rounded, which is what the phone draws with.
        let gap = gaps[0].round();
        let start = gaps[1].round();
        assert_eq!(
            tiles_across(screen, start, tile[0], gap),
            4,
            "{screen} less {start} over {} rounded up is not four",
            tile[0] + gap
        );
        // The stated 3.29 visible is the same division before rounding, to within the precision a
        // capture of a cut tile can carry.
        let derived = (screen - start) / (tile[0] + gap);
        assert!(
            (derived - visible).abs() < 0.05,
            "the rule gives {derived} visible against the measured {visible}"
        );
    }

    /// The rule is the same rule on this surface's own tile, which is a different tile.
    #[test]
    fn the_rule_counts_this_surface_s_own_tiles_too() {
        // A 1440 pixel window at the expanded 40 pixel edge: a 1360 pixel row, so a tile is
        // (1360 - 40) / 6 = 220, a pitch of 228, and (1440 - 40) / 228 rounded up is seven - six
        // full and a cut seventh, which is the row the style sheet lays out.
        let row = 1440.0 - 40.0 * 2.0;
        assert_eq!(tile_wide(row), 220.0);
        assert_eq!(tiles_across(1440.0, 40.0, tile_wide(row), TILE_GAP), 7);
        // A phone window, where the floor holds the tile and three fit with a fourth cut.
        let narrow = 390.0 - 12.0 * 2.0;
        assert_eq!(tile_wide(narrow), TILE_LEAST_WIDE);
        assert_eq!(tiles_across(390.0, 12.0, tile_wide(narrow), TILE_GAP), 2);
        // Nothing to divide by still draws a row, rather than none.
        assert_eq!(tiles_across(0.0, 0.0, 0.0, 0.0), 1);
    }

    /// The tile this file computes with is the tile the style sheet draws.
    ///
    /// Two copies of four numbers, one in Rust for the count and one in CSS for the layout. A page
    /// whose Rust and CSS disagreed would draw a placeholder row of a different length from the
    /// row that replaces it, which is the one thing a placeholder must not do.
    #[test]
    fn the_tile_this_file_computes_with_is_the_one_the_style_sheet_draws() {
        let css = source("apps/web/style.css");
        let floor = numbers(
            &["--tile-least: ", "px; --tile-across: ", ";"],
            &css,
            "the tile's floor and how many fill a row",
        );
        assert_eq!(floor[0], TILE_LEAST_WIDE);
        assert_eq!(floor[1], TILES_ACROSS_ROW);
        // The style sheet writes the slack and the gap in brand units; this file writes them in
        // pixels, and the unit is brand.json's own `spacing.unit`.
        let unit = one_number(
            &["\"unit\": ", ","],
            &source("brand.json"),
            "the brand's unit",
        );
        let slack = one_number(
            &["calc((100% - var(--unit) * ", ") / var(--tile-across))"],
            &css,
            "the row's slack",
        );
        assert_eq!(slack * unit, STRIP_SLACK);
        let gap = one_number(
            &[".strip { display: flex; gap: calc(var(--unit) * ", ")"],
            &css,
            "the gap between tiles",
        );
        assert_eq!(gap * unit, TILE_GAP);
    }

    #[test]
    fn the_only_words_this_screen_says_are_in_the_strings_file() {
        assert!(english().contains_key("say.loading"));
    }
}
