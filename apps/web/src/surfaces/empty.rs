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
//! * the count of tiles in a row is the web's own, because this is the one place where the two
//!   references disagree. `Empty.kt:54` divides and rounds **up**, so the last tile is cut by the
//!   screen's edge, which is what the phone's reference draws. `reference/netflix-on-the-web.md`
//!   measured nine widths from 570 to 1920 and cuts a tile at none of them: the count across steps
//!   with the window instead, and the tile is what is left over. So this file steps rather than
//!   rounds up, and the steps are held to that reference's own table by the tests below;
//! * every size - the tile, the gap, the row's gutter, the bar's height - is the web row's own,
//!   because the placeholder exists to be the same shape as the row that replaces it. A placeholder
//!   built from the phone's 119 x 166.6 dp portrait tile would jump the moment the answer arrived,
//!   since this surface draws a 16 : 9 tile of its own (`style.css`, `.card`).

use leptos::prelude::*;

use crate::state::Shared;

/// Rows a waiting page draws. `Empty.kt:36`, where the derivation is 936.6 dp of screen over a
/// 217.0 dp row pitch, which is 4.3, and a third of a row is not drawn.
///
/// Carried across rather than derived again: the division needs a measured screen and a measured
/// row pitch, and no capture of this project's own web rows exists to measure a pitch off. The test
/// below holds the count to the phone's own stated derivation, so the day that derivation changes
/// this fails rather than quietly disagreeing with it.
const WAITING_ROWS: usize = 4;

/// Where the count of tiles across a row steps, and what it steps to: a window at least this wide
/// holds that many. `style.css` carries the same six as media queries and the test below reads
/// them out of it, so the placeholder row and the row that replaces it are the same length.
///
/// Every one of them is a width `reference/netflix-on-the-web.md` states a count at. The three
/// above 1024 are the queries it names in the page it measured, and 600 is the step it bracketed
/// to 1 px: 2 across at 599 and 3 at 600, which is also where this page's own margin changes
/// class. 840 is the one step nothing has bracketed - the reference has 3 across at 768 and 4 at
/// 1024, and anything in between satisfies both - so it sits on the other boundary this page
/// already has, `sizes.rs`'s 840, rather than on a number nothing measured.
///
/// Two across is the narrowest row and not a floor under one: the reference measured 2 across at
/// nineteen widths from 160 to 599 and never falls to one tile.
const ACROSS_STEPS: [(f64, usize); 6] = [
    (0.0, 2),
    (600.0, 3),
    (840.0, 4),
    (1280.0, 5),
    (1600.0, 6),
    (1920.0, 7),
];

/// Tiles a row holds in a window this wide: the step above, and nothing else.
///
/// There is no division here and no rounding, which is the whole of the change of 2026-11-13: a
/// count that is a step and a tile that is the remainder cannot leave a fraction of a tile drawn,
/// and a count divided out of a floored tile did, at three of the nine widths the reference
/// measured. A width that is not a number reads as the narrowest step, which is the fewest tiles.
#[must_use]
pub fn tiles_across(wide: f64) -> usize {
    let mut across = ACROSS_STEPS[0].1;
    for (from, count) in ACROSS_STEPS {
        if wide >= from {
            across = count;
        }
    }
    across
}

/// How wide the window is, in CSS pixels: the document element's own width, which is what the
/// style sheet's `min-width` queries are matched against. `window.innerWidth` counts a classic
/// scrollbar that those queries do not, and a count read from it would be a step ahead of the
/// style sheet for the fifteen or so pixels above each one.
///
/// A window that cannot be measured reads as 0, which is the step that draws the fewest tiles, so
/// a page that cannot measure itself draws less rather than more.
fn window_wide() -> f64 {
    web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.document_element())
        .map(|root| f64::from(root.client_width()))
        .filter(|wide| wide.is_finite() && *wide > 0.0)
        .unwrap_or(0.0)
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
    let across = move || tiles_across(window_wide());
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
    use super::{ACROSS_STEPS, WAITING_ROWS, tiles_across};
    use crate::surfaces::sizes::WidthClass;
    use crate::surfaces::tests_support::{english, flat, numbers, one_number, source};

    const PHONE_EMPTY: &str =
        "apps/android/surfaces/phone/src/main/kotlin/streams/phone/browse/Empty.kt";

    /// What the web surface is held to, measured live on 2026-11-13.
    const WEB_REFERENCE: &str = "reference/netflix-on-the-web.md";

    /// Its two tables of rows, by the heading each one sits under, and the widths each states a
    /// row for. 570 and 320 appear in more than one table in that file, so a row is always read
    /// out of the section it is being asked about rather than out of the whole file.
    const WIDE_TABLE: &str = "## The one rule that produces a row";
    const MEASURED_WIDE: [f64; 9] = [
        570.0, 768.0, 1024.0, 1280.0, 1440.0, 1536.0, 1600.0, 1800.0, 1920.0,
    ];
    const NARROW_TABLE: &str = "### The one rule still holds";
    const MEASURED_NARROW: [f64; 19] = [
        160.0, 200.0, 240.0, 280.0, 300.0, 320.0, 360.0, 375.0, 390.0, 414.0, 430.0, 480.0, 540.0,
        570.0, 575.0, 580.0, 590.0, 595.0, 599.0,
    ];

    /// The gap between two tiles, in CSS pixels, as `style.css` draws it: `--unit * 2`. The
    /// reference measured the same 8 at every one of its nine widths, which
    /// `the_one_rule_gives_the_reference_s_own_tile` reads out of it rather than trusting this.
    const TILE_GAP: f64 = 8.0;

    /// The row a strip lays its tiles in: the window less its two margins.
    fn row_wide(wide: f64) -> f64 {
        wide - f64::from(WidthClass::of(wide).page_edge()) * 2.0
    }

    /// A tile's width in a window this wide, in CSS pixels.
    ///
    /// The one rule of the reference, and the mirror of what `.strip` computes in
    /// `style.css` - the style sheet is where that width lives, because the browser is what draws
    /// it, and nothing in this crate needs the number at run time. The two are held together as
    /// text by `the_tile_this_file_computes_with_is_the_one_the_style_sheet_draws`, so a floor put
    /// back in front of this division fails there, and a floor put back in front of that one fails
    /// here in `no_window_draws_part_of_a_tile`.
    fn tile_wide(wide: f64) -> f64 {
        let across = tiles_across(wide) as f64;
        (row_wide(wide) - (across - 1.0) * TILE_GAP) / across
    }

    /// One section of the reference: from its heading to the next heading of the same depth.
    fn section(reference: &str, heading: &str) -> String {
        let at = reference
            .find(heading)
            .unwrap_or_else(|| panic!("{heading} has gone from the reference"));
        let rest = &reference[at + heading.len()..];
        let depth = heading
            .split(' ')
            .next()
            .expect("a heading opens with its hashes");
        let end = rest.find(&format!("\n{depth} ")).unwrap_or(rest.len());
        rest[..end].to_owned()
    }

    /// The page margin, the count across and the tile width one table states at this width. Its
    /// rows read `| 1920 | 60 | 7 | 250.3 x 140.5 | ...`.
    fn reference_row(table: &str, wide: f64) -> (f64, f64, f64) {
        let head = format!("| {wide} | ");
        let read = numbers(
            &[&head, " | ", " | ", " x "],
            table,
            "the reference's row for this width",
        );
        (read[0], read[1], read[2])
    }

    /// Every width the reference states a row for, with the count across it measured there.
    fn reference_counts(reference: &str) -> Vec<(f64, f64)> {
        let mut all = Vec::new();
        for (heading, widths) in [
            (WIDE_TABLE, &MEASURED_WIDE[..]),
            (NARROW_TABLE, &MEASURED_NARROW[..]),
        ] {
            let table = section(reference, heading);
            for wide in widths {
                all.push((*wide, reference_row(&table, *wide).1));
            }
        }
        all
    }

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

    /// No window draws part of a tile.
    ///
    /// The defect this file was changed for on 2026-11-13, stated as the thing under test: a row
    /// whose last tile is cut in half tells a reader the row scrolls, and this surface said that
    /// by accident at 570, 768 and 1024, where a 190 px floor held the tile wider than the row
    /// could divide. `reference/netflix-on-the-web.md` cuts a tile at none of the nine widths it
    /// measured.
    ///
    /// Swept rather than sampled, because a floor cuts a tile over a band of widths and not at a
    /// round one: the band this surface had was every width under 1140, which no table of nine
    /// would have to contain. 160 is the narrowest the reference reaches and 2560 is above the
    /// widest.
    #[test]
    fn no_window_draws_part_of_a_tile() {
        for step in 160..=2560 {
            let wide = f64::from(step);
            let row = row_wide(wide);
            let tile = tile_wide(wide);
            assert!(tile > 0.0, "a {wide} px window draws a {tile} px tile");
            // What the row holds between its margins: the row plus one gap, over a tile's pitch.
            // A whole number here is a row that ends on a tile; anything else ends inside one.
            let fits = (row + TILE_GAP) / (tile + TILE_GAP);
            assert!(
                (fits - fits.round()).abs() < 1e-9,
                "a {wide} px window fits {fits} tiles, so it draws part of one"
            );
            assert_eq!(
                fits.round() as usize,
                tiles_across(wide),
                "a {wide} px window fits {fits} tiles and draws {} of them",
                tiles_across(wide)
            );
        }
    }

    /// The count across is the reference's count at every width the reference measured.
    ///
    /// Twenty-eight widths from 160 to 1920, read out of the reference's own two tables rather
    /// than copied into an expectation here, so the day that file is measured again this fails
    /// rather than quietly disagreeing with it. It was measured again on the afternoon of
    /// 2026-11-13, with nineteen widths under 600, and this is what read them.
    #[test]
    fn the_count_across_is_the_reference_s_count_at_every_width_it_measured() {
        let reference = source(WEB_REFERENCE);
        for (wide, across) in reference_counts(&reference) {
            assert_eq!(
                tiles_across(wide) as f64,
                across,
                "the reference holds {across} tiles across a {wide} px window"
            );
        }
    }

    /// The one rule gives the reference's own tile width when it is fed the reference's own page
    /// margin, at all nine widths, inside the tolerance that file sets for itself.
    ///
    /// This is the derivation check: a rule that divided by the tile without its gutter, or took
    /// one margin out of the window instead of two, or counted a gutter per tile instead of one
    /// between each pair, fails here against nine measured numbers. The gutter is read out of the
    /// reference too, so the 8 this file computes with is that file's 8.
    ///
    /// What it does not say is that this surface draws the reference's tile: it draws the same
    /// rule on its own page margin, which is `sizes.rs`'s 12 / 24 / 40 and not the reference's
    /// 24 / 36 / 48 / 60. That margin is the phone's, pinned to Android and Apple, and the table
    /// in the report of 2026-11-13 states what the difference costs at each of the nine.
    #[test]
    fn the_one_rule_gives_the_reference_s_own_tile() {
        let reference = source(WEB_REFERENCE);
        let gutter = one_number(
            &["The gutter is **", " px at every width measured"],
            &reference,
            "the reference's gutter",
        );
        assert_eq!(gutter, TILE_GAP, "the gutter this file computes with");
        let tolerance = one_number(
            &["within **", " px** of the number"],
            &reference,
            "the reference's tolerance",
        );
        for (heading, widths) in [
            (WIDE_TABLE, &MEASURED_WIDE[..]),
            (NARROW_TABLE, &MEASURED_NARROW[..]),
        ] {
            let table = section(&reference, heading);
            for wide in widths {
                let (margin, across, tile) = reference_row(&table, *wide);
                let drawn = (wide - margin * 2.0 - (across - 1.0) * gutter) / across;
                assert!(
                    (drawn - tile).abs() <= tolerance,
                    "at {wide} the rule gives {drawn} against the measured {tile}"
                );
            }
        }
    }

    /// Where the count steps: four of the five are a width the reference states, and the one it
    /// has not bracketed sits inside the bounds it does state and on a boundary this page already
    /// has.
    ///
    /// The point of the test is that no step is a number nothing measured. The first entry of the
    /// table is the narrowest window rather than a step, so it is not checked here.
    #[test]
    fn every_step_is_a_width_something_measured() {
        let reference = source(WEB_REFERENCE);
        let queries = numbers(
            &[
                "the stylesheet carries `min-width` queries at ",
                ", ",
                " and ",
                ",",
            ],
            &reference,
            "the reference's own breakpoints",
        );
        assert_eq!(
            queries,
            vec![ACROSS_STEPS[3].0, ACROSS_STEPS[4].0, ACROSS_STEPS[5].0],
            "the three steps above 1024 are the reference's own queries"
        );
        // The first step was bracketed to 1 px on the afternoon of 2026-11-13: 2 across at 599 and
        // 3 at 600, which is also where this page's own margin changes class.
        let first = one_number(
            &["### The first step is at exactly ", " Bracketed"],
            &reference,
            "the reference's first step",
        );
        assert_eq!(first, ACROSS_STEPS[1].0);
        // The last one is the step nothing has bracketed. The reference has 3 across at the widest
        // width it measured 3 at and 4 at the narrowest it measured 4 at, so the step may sit
        // anywhere in between; it sits where this page's margin changes class rather than on a
        // number nothing measured.
        let counts = reference_counts(&reference);
        let widest_at_three = counts
            .iter()
            .filter(|(_, across)| *across == 3.0)
            .fold(f64::MIN, |most, (wide, _)| most.max(*wide));
        let narrowest_at_four = counts
            .iter()
            .filter(|(_, across)| *across == 4.0)
            .fold(f64::MAX, |least, (wide, _)| least.min(*wide));
        let step = ACROSS_STEPS[2].0;
        assert!(
            widest_at_three < step && step <= narrowest_at_four,
            "a row steps to 4 across at {step}, outside the {widest_at_three} to \
             {narrowest_at_four} the reference leaves open"
        );
        for (at, count) in [ACROSS_STEPS[1], ACROSS_STEPS[2]] {
            assert_ne!(
                WidthClass::of(at - 1.0),
                WidthClass::of(at),
                "{at}, where a row steps to {count} across, is not a boundary this page has"
            );
        }
        // And the six rise, both in the width they start at and in what they count.
        for pair in ACROSS_STEPS.windows(2) {
            assert!(pair[0].0 < pair[1].0 && pair[0].1 < pair[1].1);
        }
    }

    /// The steps this file counts with are the steps the style sheet draws.
    ///
    /// Two copies of the same six, one in Rust for the count and one in CSS for the layout. A page
    /// whose Rust and CSS disagreed would draw a placeholder row of a different length from the
    /// row that replaces it, which is the one thing a placeholder must not do.
    #[test]
    fn the_steps_this_file_counts_with_are_the_ones_the_style_sheet_draws() {
        let css = source("apps/web/style.css");
        let drawn = numbers(
            &[
                ":root { --tile-across: ",
                "; } @media (min-width: ",
                "px) { :root { --tile-across: ",
                "; } } @media (min-width: ",
                "px) { :root { --tile-across: ",
                "; } } @media (min-width: ",
                "px) { :root { --tile-across: ",
                "; } } @media (min-width: ",
                "px) { :root { --tile-across: ",
                "; } } @media (min-width: ",
                "px) { :root { --tile-across: ",
                "; } }",
            ],
            &css,
            "how many tiles fill a row in the style sheet",
        );
        let mut wanted = vec![ACROSS_STEPS[0].1 as f64];
        for (at, count) in ACROSS_STEPS.into_iter().skip(1) {
            wanted.push(at);
            wanted.push(count as f64);
        }
        assert_eq!(drawn, wanted);
    }

    /// The tile this file computes with is the tile the style sheet draws.
    ///
    /// The style sheet's width is read from its own colon onwards, as the whole of the
    /// declaration, so the floor that was in front of this division until 2026-11-13 cannot come
    /// back on the CSS side alone: `max(var(--tile-least), calc(...))` stops matching the shape
    /// and this fails, while the same floor put back on the Rust side fails
    /// `no_window_draws_part_of_a_tile`.
    #[test]
    fn the_tile_this_file_computes_with_is_the_one_the_style_sheet_draws() {
        let css = source("apps/web/style.css");
        // The style sheet writes the gutter in brand units; this file writes it in pixels, and the
        // unit is brand.json's own `spacing.unit`.
        // No trailing comma in the shape: `spacing.unit` became the last key in its object when
        // the brand's orphaned gutter was removed on 2026-11-12, and a shape ending in an empty
        // piece means "and then anything", which is what `numbers` documents it for. Reading the
        // comma tied this test to where `unit` happened to sit among its neighbours.
        let unit = one_number(
            &["\"unit\": ", ""],
            &source("brand.json"),
            "the brand's unit",
        );
        let gap = one_number(
            &[".strip { display: flex; gap: calc(var(--unit) * ", ")"],
            &css,
            "the gap between tiles",
        );
        assert_eq!(gap * unit, TILE_GAP);
        // One gutter out of the row for each pair of tiles, written with the same `--unit * 2` the
        // gap above is written with, so the two cannot drift apart. Matched as the whole
        // declaration rather than read as a number, because `flat` eats a `*` that starts a piece.
        let rule = format!(
            "--tile-wide: calc((100% - var(--unit) * {gap} * (var(--tile-across) - 1)) / var(--tile-across));"
        );
        assert!(
            flat(&css).contains(&rule),
            "the style sheet's tile is not {rule}"
        );
    }

    #[test]
    fn the_only_words_this_screen_says_are_in_the_strings_file() {
        assert!(english().contains_key("say.loading"));
    }
}
