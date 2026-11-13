//! The one rule that produces a row, and the vertical rhythm around it.
//!
//! Every number here is read off `reference/netflix-on-the-web.md`, which was measured at 28
//! widths from 160 to 1920 in a signed-in browser on 2026-11-13. A desktop window is a wide
//! viewport, so that file is this surface's reference and not the television's: the tests below
//! read the reference's own two tables, row by row, and hold every function here to all 28 widths.
//! A number that drifts from the file fails on the spot, and a number the file does not carry
//! cannot be written here at all.
//!
//! What this file is not: it is not a copy of `apps/web/src/surfaces/sizes.rs`. That file answers
//! a different question - which of the phone's three width classes a window is in, which is where
//! the core's idle limit comes from - and it states no tile, no row and no rhythm. The two have
//! one number in common, the 600 where the first step happens, and the test below reads it out of
//! the web's file rather than writing it down a second time.

/// The gap between two tiles, at every width measured, from 160 to 1920.
pub const GUTTER: f64 = 8.0;

/// A tile's height as a share of its width: 0.5614 at every width measured. The image inside is
/// exactly 16:9 and overflows the box by about 0.2 px, which is the reference's own figure and not
/// a rounding here.
pub const TILE_HIGH: f64 = 0.5614;

/// The top bar, 80 px tall at every width measured.
pub const BAR_HIGH: f64 = 80.0;

/// Where the page margin and the count across take their first step. Bracketed to 1 px in the
/// reference: at 599 the margin is 24 and two tiles fit; at 600 the margin is 36 and three do.
const FIRST_STEP: f64 = 600.0;

/// The gap between a row's title and its tiles, and the margin under a whole row. Both are the
/// same at 1920 and at 390.
pub const TITLE_GAP: f64 = 10.0;
pub const ROW_GAP: f64 = 32.0;

/// The inset a row's scroller carries above and below its tiles, 4 px at both widths measured.
pub const SCROLLER_INSET: f64 = 4.0;

/// The gap between the hero and the first row title, 32 px at both widths measured.
pub const HERO_GAP: f64 = 32.0;

/// How far a row travels on one press of its forward control: the content box plus one gutter.
/// 1808 px at 1920 and 350 px at 390, both of which the reference measured directly.
#[must_use]
pub fn page_step(wide: f64) -> f64 {
    content(wide) + GUTTER
}

/// The window a screen draws into, in the window's own pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Window {
    pub wide: f64,
    pub high: f64,
}

impl Window {
    #[must_use]
    pub const fn new(wide: f64, high: f64) -> Self {
        Self { wide, high }
    }
}

/// The page margin at this width: 24, 36, 48 or 60.
///
/// The reference measured 24 from 160 to 599, 36 at 600, 768 and 1024, 48 at 1280 and 1536, and 60
/// at 1600, 1800 and 1920. Only the step at 600 is bracketed to 1 px; the three above it are
/// placed at the lowest width at which the larger margin was measured, which is the most this
/// reference can say. The stylesheet it was read from carries `min-width` queries at 1280, 1600
/// and 1920, and two of those three fall where this function steps.
#[must_use]
pub fn margin(wide: f64) -> f64 {
    if wide < FIRST_STEP {
        24.0
    } else if wide < 1280.0 {
        36.0
    } else if wide < 1600.0 {
        48.0
    } else {
        60.0
    }
}

/// How many tiles are across at this width: 2, 3, 4, 5, 6 or 7.
///
/// Measured 2 from 160 to 599, 3 at 768, 4 at 1024, 5 at 1280 and 1536, 6 at 1600 and 1800, 7 at
/// 1920. The layout never falls to one across, down to 160. As with the margin, every step but the
/// one at 600 sits at the lowest width at which the larger count was measured.
#[must_use]
pub fn across(wide: f64) -> f64 {
    if wide < FIRST_STEP {
        2.0
    } else if wide < 1024.0 {
        3.0
    } else if wide < 1280.0 {
        4.0
    } else if wide < 1600.0 {
        5.0
    } else if wide < 1920.0 {
        6.0
    } else {
        7.0
    }
}

/// The content box: the window less its two margins. The hero is exactly this wide at every width
/// measured, and so is a full page of tiles.
#[must_use]
pub fn content(wide: f64) -> f64 {
    (wide - 2.0 * margin(wide)).max(0.0)
}

/// **tile width = (viewport - 2 x page margin - (across - 1) x 8) / across.**
///
/// The one rule of the reference. It reproduced the measured tile to 0.00 px at all 19 of the
/// widths from 160 to 599 and to 0.05 px at the nine above, and the test below runs it against
/// every one of them.
#[must_use]
pub fn tile(wide: f64) -> f64 {
    let across = across(wide);
    ((content(wide) - (across - 1.0) * GUTTER) / across).max(0.0)
}

/// A tile plus its gutter, which is what a row advances by.
#[must_use]
pub fn pitch(wide: f64) -> f64 {
    tile(wide) + GUTTER
}

/// How tall the hero card is at this width.
///
/// Three bands, each a constant multiple of the hero's own width, all three reproduced from the
/// reference's own numbers: 1.4925 from 160 to 599 (779.10/522 at 570), 1.2048 from 600 to 768
/// (838.55/696 at 768), and 0.4587 from 1024 to 1920 (825.7/1800 at 1920, constant to four
/// decimals at seven widths). The width where the middle band ends was not bracketed - the
/// reference measured 768 and then 1024 - so it is placed at 1024, the lowest width at which the
/// wide card was measured.
#[must_use]
pub fn hero(wide: f64) -> (f64, f64) {
    let across = content(wide);
    let ratio = if wide < FIRST_STEP {
        1.4925
    } else if wide < 1024.0 {
        1.2048
    } else {
        0.4587
    };
    (across, across * ratio)
}

/// The gap between the top bar and the hero: 12 px at 1920, 4 px at 390, from the card's own
/// `margin` at each width.
#[must_use]
pub fn hero_top(wide: f64) -> f64 {
    if wide < FIRST_STEP { 4.0 } else { 12.0 }
}

/// How far inside the hero card its own content sits: 40 px at 1920, 16 px at 390.
#[must_use]
pub fn hero_inset(wide: f64) -> f64 {
    if wide < FIRST_STEP { 16.0 } else { 40.0 }
}

/// A row title's type size and the height of its box.
///
/// Two bands, each a pair the reference measured together: 16 px / 500 in a 20.5 box at 390, and
/// 24 px / 500 in a 30.5 box at 1920. **The reference also measured 18 px at exactly 600 and did
/// not measure that box**, so this surface does not reproduce the 18: a window from 600 to 1919
/// draws the 1920 pair. What would close it is one reading of the row title box between 600 and
/// 1536, and until there is one, writing a third pair here would be a number nobody measured.
#[must_use]
pub fn row_title(wide: f64) -> (f64, f64) {
    if wide < FIRST_STEP {
        (16.0, 20.5)
    } else {
        (24.0, 30.5)
    }
}

/// How tall a row's scroller is: 4 px of inset, the tile, 4 px of inset. 148.5 at 1920, 101.75 at
/// 390, both measured.
#[must_use]
pub fn scroller(wide: f64) -> f64 {
    tile(wide) * TILE_HIGH + 2.0 * SCROLLER_INSET
}

/// Row title to row title: the title's box, the 10 gap, the scroller and the next section's 32 px
/// margin. 221.0 at 1920 and 164.25 at 390, which are the two figures the reference states as
/// measured rather than derived.
#[must_use]
pub fn row_pitch(wide: f64) -> f64 {
    row_title(wide).1 + TITLE_GAP + scroller(wide) + ROW_GAP
}

/// The scrim a row's paging control sits in: as wide as the page margin, at both widths measured.
#[must_use]
pub fn scrim(wide: f64) -> f64 {
    margin(wide)
}

/// The widest column of running text the reference measured: the hero's synopsis at 1920, 691.2
/// px inside a 1800 px card.
///
/// The screens before a film have no measurement of their own in that file, which measures Home
/// alone. Rather than invent one, a door card is the hero's own reading column with the hero's own
/// inset on each side, which is the widest the reference ever lets a line of text be.
pub const READING: f64 = 691.2;

/// An action pill's height: 48 px at 1920, 40 px at 390, both measured on the hero's buttons. A
/// field a viewer types into is the same height as the button under it.
#[must_use]
pub fn pill(wide: f64) -> f64 {
    if wide < FIRST_STEP { 40.0 } else { 48.0 }
}

/// Running text: 24 px on a 30 px line at 1920, 14 px on a 24 px line at 390, measured on the
/// hero's synopsis at both.
#[must_use]
pub fn body_type(wide: f64) -> (f64, f64) {
    if wide < FIRST_STEP {
        (14.0, 24.0)
    } else {
        (24.0, 30.0)
    }
}

/// How wide a card of fields is: the reading column with the hero's inset on each side, never
/// wider than the content box.
#[must_use]
pub fn card(wide: f64) -> f64 {
    content(wide).min(READING + 2.0 * hero_inset(wide))
}

/// The seven navigation labels: 16 px / 400, with 6 px between their boxes, measured at 1920.
pub const NAV_TYPE: f64 = 16.0;
pub const NAV_GAP: f64 = 6.0;

/// The logo in the top bar, 76 x 20 at every width measured, on the page margin.
pub const LOGO: (f64, f64) = (76.0, 20.0);

/// The gap between the logo and the first navigation label: 160 - 136 at 1920, where the logo ends
/// at x 136 and the first label starts at x 160.
pub const NAV_FROM_LOGO: f64 = 24.0;

/// The progress bar under a tile in a Continue Watching row: 8 px, the whole of the difference
/// between a 189 px section and a 197 px one, and the same 8 px at 390.
pub const PROGRESS: f64 = 8.0;

/// A Top 10 tile is the same width and pitch as every other tile and 178.8 tall at 1920, split
/// into two halves of 125.1: the rank numeral in the left half and a portrait poster in the right.
/// The two readings give one constant, the poster's own shape, and the test holds both.
pub const RANKED_HIGH: f64 = 178.8 / 125.1;

#[cfg(test)]
mod tests {
    use super::{
        BAR_HIGH, GUTTER, HERO_GAP, LOGO, NAV_FROM_LOGO, NAV_GAP, NAV_TYPE, PROGRESS, RANKED_HIGH,
        READING, ROW_GAP, SCROLLER_INSET, TILE_HIGH, TITLE_GAP, across, body_type, card, content,
        hero, margin, page_step, pill, pitch, row_pitch, row_title, scrim, scroller, tile,
    };
    use crate::surfaces::reading::{flat, numbers, source};

    /// The reference this whole file is a reading of.
    const WEB: &str = "reference/netflix-on-the-web.md";

    /// One row of one of the reference's two tables: the width, the margin, the count across, the
    /// tile's width and height where it states one, the pitch, and the hero's width and height
    /// where it states one.
    #[derive(Debug)]
    struct Row {
        wide: f64,
        margin: f64,
        across: f64,
        tile: f64,
        tile_high: Option<f64>,
        pitch: f64,
        hero: f64,
        hero_high: Option<f64>,
    }

    /// A cell of a table: the numbers in it, in order. `250.3 x 140.5` is two, `259.5 x —` is one.
    fn cell(text: &str) -> Vec<f64> {
        text.split(|mark: char| !mark.is_ascii_digit() && mark != '.')
            .filter(|piece| !piece.is_empty())
            .filter_map(|piece| piece.parse().ok())
            .collect()
    }

    /// Every row of the reference's two layout tables. Both are read by one function on purpose:
    /// a column that moved in one of them and not the other shows up here as a row that no longer
    /// parses, and the count below is what notices.
    ///
    /// The file holds a third table with six columns, the hover preview's, whose second column is
    /// a tile width rather than a page margin. A row of it is told apart by that: a page margin is
    /// one of the four whole numbers the reference ever measured, and a tile width is not.
    fn rows() -> Vec<Row> {
        let text = source(WEB);
        let mut rows = Vec::new();
        for line in text.lines().map(str::trim) {
            let Some(body) = line.strip_prefix('|').and_then(|l| l.strip_suffix('|')) else {
                continue;
            };
            let cells: Vec<&str> = body.split('|').map(str::trim).collect();
            if cells.len() != 6 {
                continue;
            }
            let head = cell(cells[0]);
            if head.len() != 1 || cells[0].contains('.') {
                continue;
            }
            let (margin, across) = (cell(cells[1]), cell(cells[2]));
            let layout = margin.len() == 1
                && [24.0, 36.0, 48.0, 60.0].contains(&margin[0])
                && !cells[1].contains('.')
                && across.len() == 1
                && !cells[2].contains('.')
                && (2.0..=7.0).contains(&across[0]);
            if !layout {
                continue;
            }
            let (tile, hero) = (cell(cells[3]), cell(cells[5]));
            rows.push(Row {
                wide: head[0],
                margin: margin[0],
                across: across[0],
                tile: tile[0],
                tile_high: tile.get(1).copied(),
                pitch: cell(cells[4])[0],
                hero: hero[0],
                hero_high: hero.get(1).copied(),
            });
        }
        rows
    }

    /// The reference's own tolerance: "within 1 px of the number here", and a tile derived from
    /// the rule matched the measurement to 0.00 px at 19 widths and 0.05 at the other nine, so the
    /// rule is held tighter than the drawing is.
    const CLOSE: f64 = 1.0;
    const EXACT: f64 = 0.06;

    fn near(got: f64, want: f64, within: f64, what: &str) {
        assert!(
            (got - want).abs() <= within,
            "{what}: {got} is not within {within} of the reference's {want}"
        );
    }

    #[test]
    fn the_reference_still_holds_the_two_tables_this_file_is_read_from() {
        let rows = rows();
        // Nine widths in the first table and nineteen in the second. A reference that lost a
        // table, or a column, stops this file being a reading of anything.
        assert_eq!(rows.len(), 28, "the reference's two layout tables");
        let widest = rows.iter().map(|row| row.wide).fold(0.0_f64, f64::max);
        let narrowest = rows.iter().map(|row| row.wide).fold(f64::MAX, f64::min);
        assert_eq!((narrowest, widest), (160.0, 1920.0));
    }

    /// The whole point of this file: every function here, against every width the reference
    /// measured.
    #[test]
    fn every_width_the_reference_measured_comes_out_of_this_file_s_rule() {
        for row in rows() {
            let what = format!("at {}", row.wide);
            near(margin(row.wide), row.margin, 0.0, &format!("margin {what}"));
            near(across(row.wide), row.across, 0.0, &format!("across {what}"));
            near(tile(row.wide), row.tile, EXACT, &format!("tile {what}"));
            near(pitch(row.wide), row.pitch, EXACT, &format!("pitch {what}"));
            let (hero_wide, hero_high) = hero(row.wide);
            near(hero_wide, row.hero, 0.0, &format!("hero width {what}"));
            if let Some(want) = row.hero_high {
                near(hero_high, want, CLOSE, &format!("hero height {what}"));
            }
            if let Some(want) = row.tile_high {
                // Held at 0.05 and not at the reference's 1 px. The worst of the 23 heights the
                // two tables print is 0.049 away from this constant, so a tile's share of its own
                // width is pinned to two more decimals than the file's tolerance would pin it.
                // At 1 px, changing 0.5614 to 0.5625 passed this test: proved on 2026-11-13 by
                // making that change, whose worst deviation is 0.35.
                near(
                    tile(row.wide) * TILE_HIGH,
                    want,
                    0.05,
                    &format!("tile height {what}"),
                );
            }
        }
    }

    /// The rule written out, against the rule as the reference states it in words. A tile that
    /// came from anywhere but this division would pass the table above by accident at one width;
    /// it cannot pass this at all 28.
    #[test]
    fn a_tile_is_the_content_box_less_its_gutters_shared_out() {
        let text = source(WEB);
        assert!(
            text.contains(
                "**tile width = (viewport - 2 x page margin - (across - 1) x 8) / across**"
            ),
            "the reference no longer states the rule this file implements"
        );
        for row in rows() {
            let across = across(row.wide);
            let by_hand = (row.wide - 2.0 * margin(row.wide) - (across - 1.0) * GUTTER) / across;
            near(tile(row.wide), by_hand, 0.0, "the rule, written out");
            near(
                content(row.wide),
                across * tile(row.wide) + (across - 1.0) * GUTTER,
                EXACT,
                "the tiles fill the content box exactly",
            );
        }
    }

    /// A tile never falls below 226.7 px and never rises above 281.6 px across the nine widths of
    /// the first table: the count across steps before the tile is allowed to drift far.
    ///
    /// **At the widths the reference did not measure, it does not hold, and this test does not
    /// pretend it does.** The reference brackets one step, at 600, and places the rest at the
    /// lowest width each was seen at, so between two measured widths a tile grows until the next
    /// step - 311.7 px at 1023, and the reference's own reading at 600 is 170.66, well under the
    /// band. Asserting the band at every integer width would be asserting a bracketing nobody
    /// has done. What closes it is the four unbracketed steps, measured to 1 px as the one at 600
    /// was.
    #[test]
    fn a_tile_stays_in_its_measured_band_at_every_width_that_was_measured() {
        let text = source(WEB);
        let band = numbers(
            &[
                "A tile never falls below **",
                " px** and never rises above **",
                " px**",
            ],
            &text,
            "the band a tile stays inside",
        );
        let mut held = 0;
        for row in rows().into_iter().filter(|row| row.wide >= 570.0) {
            let tile = tile(row.wide);
            assert!(
                tile >= band[0] - CLOSE && tile <= band[1] + CLOSE,
                "a tile at {} is {tile}, outside the reference's {band:?}",
                row.wide
            );
            held += 1;
        }
        assert_eq!(
            held, 15,
            "the widths at or above 570 the reference measured"
        );
    }

    /// The two figures the reference states for a row's rhythm, which are the sum of four numbers
    /// each. Both come out of the same function.
    #[test]
    fn a_row_pitches_the_measured_distance_to_the_row_below_it() {
        let text = source(WEB);
        let wide = numbers(
            &["**Row title to row title: ", " px.** Hero bottom"],
            &text,
            "the row pitch at 1920",
        )[0];
        // The reference states 221.0 to one decimal and the sum of its four measured parts is
        // 221.01, because a tile at 1920 is 250.2857 and not the 250.3 the table prints.
        near(row_pitch(1920.0), wide, 0.02, "row pitch at 1920");
        let narrow = numbers(
            &[
                "**Row title to row title: ",
                " px.**\n\nA Continue Watching row",
            ],
            &text,
            "the row pitch at 390",
        )[0];
        near(row_pitch(390.0), narrow, 0.02, "row pitch at 390");
        // And the parts it is made of, each measured in its own right.
        near(scroller(1920.0), 148.5, 0.02, "the scroller at 1920");
        near(scroller(390.0), 101.75, 0.01, "the scroller at 390");
        assert_eq!(row_title(1920.0), (24.0, 30.5));
        assert_eq!(row_title(390.0), (16.0, 20.5));
        assert_eq!((TITLE_GAP, ROW_GAP, SCROLLER_INSET), (10.0, 32.0, 4.0));
        assert_eq!(HERO_GAP, 32.0);
    }

    /// One press of a row's forward control, at the two widths the reference measured it.
    #[test]
    fn a_row_pages_by_the_content_box_and_one_gutter() {
        near(page_step(1920.0), 1808.0, 0.02, "a page at 1920");
        near(page_step(390.0), 350.0, 0.01, "a page at 390");
        // And it is two pitches at 390, which the reference states as the same derivation.
        near(page_step(390.0), 2.0 * pitch(390.0), 0.01, "two pitches");
        // The scrim the control sits in is the page margin, at both widths measured.
        assert_eq!((scrim(1920.0), scrim(390.0)), (60.0, 24.0));
    }

    /// The bar is 80 tall at every width measured, 160 to 1920, and the reference says so twice.
    #[test]
    fn the_top_bar_is_the_same_height_at_every_width() {
        let text = source(WEB);
        assert_eq!(
            text.matches("**80 px tall**").count()
                + text
                    .matches("**80 px tall at every width measured**")
                    .count(),
            2,
            "the reference no longer states the bar height at both ends of its range"
        );
        assert_eq!(BAR_HIGH, 80.0);
    }

    /// The first step is the web surface's own breakpoint, read out of that file rather than
    /// written down twice. The reference brackets it to 1 px and the phone compares against the
    /// same number, so the three families step together or this fails.
    #[test]
    fn the_first_step_is_the_one_the_other_surfaces_already_use() {
        let web = source("apps/web/src/surfaces/sizes.rs");
        let breakpoint = numbers(
            &["const COMPACT_UNDER: f64 = ", ";"],
            &web,
            "the web surface's first breakpoint",
        )[0];
        assert_eq!(margin(breakpoint - 1.0), 24.0);
        assert_eq!(margin(breakpoint), 36.0);
        assert_eq!(across(breakpoint - 1.0), 2.0);
        assert_eq!(across(breakpoint), 3.0);
        assert_eq!(row_title(breakpoint - 1.0).0, 16.0);
        let text = source(WEB);
        assert!(
            text.contains("### The first step is at exactly 600"),
            "the reference no longer brackets the step these three families share"
        );
    }

    /// The numbers the screens before a film borrow from the one page the reference measured.
    /// Every one of them is read back out of the file here, so a door that drew a field 56 px tall
    /// because it looked right would fail on the spot.
    #[test]
    fn the_door_s_numbers_are_the_hero_s_own_measured_numbers() {
        let text = source(WEB);
        let reading = numbers(
            &["synopsis ", " wide, ", " px / ", " px line"],
            &text,
            "the hero's reading column at 1920",
        );
        assert_eq!(
            reading,
            vec![READING, body_type(1920.0).0, body_type(1920.0).1]
        );
        let narrow = numbers(
            &["Synopsis 310 wide, **", " px / ", " px line"],
            &text,
            "the hero's reading column at 390",
        );
        assert_eq!(narrow, vec![body_type(390.0).0, body_type(390.0).1]);
        let tall = numbers(
            &["a button row ", " px tall at y 829.7"],
            &text,
            "a pill at 1920",
        )[0];
        assert_eq!(tall, pill(1920.0));
        // Read as text rather than as a shape: the piece after the number starts with the
        // reference's own emphasis marks, and the reader that pulls numbers out of prose treats a
        // star at the start of a line as a comment marker.
        assert!(
            flat(&text).contains(&format!(
                "full content width, 310 x {}**, stacked with an {} px gap",
                pill(390.0),
                GUTTER
            )),
            "the reference no longer measures a pill at 390 as this file draws one"
        );
        // A card of fields is never wider than that column with the hero's own inset each side,
        // and on a narrow window it is the content box instead.
        near(card(1920.0), READING + 2.0 * 40.0, 0.01, "a card at 1920");
        near(card(390.0), content(390.0), 0.01, "a card at 390");
        assert!(
            card(1920.0) < content(1920.0),
            "a card is not the whole page"
        );
    }

    /// The top bar's own measured parts, which the browse screen lays out from.
    #[test]
    fn the_top_bar_draws_the_logo_and_the_labels_where_they_were_measured() {
        let text = source(WEB);
        let bar = numbers(
            &[
                "navigation labels, ",
                " px / 400, boxes 40.5 tall at y 19.8, the first at x ",
                ", ",
                " px between boxes",
            ],
            &text,
            "the navigation labels at 1920",
        );
        assert_eq!(bar[0], NAV_TYPE);
        assert_eq!(bar[2], NAV_GAP);
        let logo = numbers(&["logo ", " x ", " at x 60-136"], &text, "the logo at 1920");
        assert_eq!((logo[0], logo[1]), LOGO);
        // The first label starts 24 px after the logo ends, which is the only place that gap is
        // written down: the logo ends at x 136 and the first label starts at the x above.
        assert_eq!(bar[1] - 136.0, NAV_FROM_LOGO);
        // The logo sits in the middle of the bar, which is where y 30 to 50 in an 80 px bar puts
        // it, and so do the label boxes at y 19.8 in a box 40.5 tall.
        near((BAR_HIGH - LOGO.1) / 2.0, 30.0, 0.01, "the logo's own y");
        near((BAR_HIGH - 40.5) / 2.0, 19.8, 0.06, "a label box's own y");
    }

    /// The two rows that are not a plain row, by the numbers that make them different.
    #[test]
    fn a_continue_row_and_a_top_ten_row_differ_by_their_measured_parts() {
        let text = source(WEB);
        let progress = numbers(
            &["**", " px** of progress bar added under it"],
            &text,
            "the progress bar",
        )[0];
        assert_eq!(progress, PROGRESS);
        let tall = numbers(
            &["(its tile is ", " tall)"],
            &text,
            "a Top 10 tile's height at 1920",
        )[0];
        let half = numbers(
            &["split into two halves of ", ":"],
            &text,
            "a Top 10 tile's two halves at 1920",
        )[0];
        near(
            tall / half,
            RANKED_HIGH,
            0.001,
            "the poster in a Top 10 tile",
        );
        let ranked = [tall, half];
        // The same construction at 390, from the other reading of it: 119.28 over 83.5.
        near(119.28 / 83.5, RANKED_HIGH, 0.001, "the same poster at 390");
        // And its two halves are the tile the rest of the row draws.
        near(ranked[1] * 2.0, tile(1920.0), CLOSE, "two halves of a tile");
    }

    /// A window narrower than anything the reference measured, and one wider, still produce a
    /// layout rather than a negative tile or a division by zero. Nothing here is a measurement:
    /// it is the promise that a dragged window never draws nonsense.
    #[test]
    fn a_window_outside_the_measured_range_still_lays_out() {
        for wide in [0.0, 1.0, 80.0, 159.0, 2560.0, 3840.0] {
            assert!(tile(wide) >= 0.0, "a tile at {wide}");
            assert!(content(wide) >= 0.0, "a content box at {wide}");
            assert!(hero(wide).1 >= 0.0, "a hero at {wide}");
        }
        // Above 1920 the reference stops, and the widest step it measured carries on.
        assert_eq!((margin(2560.0), across(2560.0)), (60.0, 7.0));
    }
}
