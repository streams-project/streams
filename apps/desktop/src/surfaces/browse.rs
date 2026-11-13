//! The page of rows: the top bar, the hero and the rows under it.
//!
//! Every number this file lays out with comes from [`super::sizes`], which is a reading of
//! `reference/netflix-on-the-web.md`. Nothing is measured here and nothing is rounded here: a tile
//! is where the rule puts it at this window's width, at 160 px and at 1920.
//!
//! Two departures from the reference, both deliberate and both named here rather than found later:
//!
//! * **A press on a tile plays it.** The reference opens a title page over the rows. This family
//!   has no title page yet, and a press that visibly did nothing would be worse than a press that
//!   does the thing a viewer came for. The title page is what closes it.
//! * **The bar gives each navigation label an equal share of the space.** The reference measured
//!   seven labels in boxes of their own widths, 6 px apart, which needs the width of a line of
//!   text in a font - and this surface has no font: a painter has one. The 6 px gap and the
//!   16 px / 400 type are the reference's; the equal share is this file's, and it goes the day a
//!   painter can be asked how wide a word is.

use contract::address;
use contract::core_msg::Action;
use contract::screen::{ImageRole, Section, SectionKind, Tile};

use super::scene::{Corner, Ink, Mark, Part, Rect, Scene, Touch};
use super::{Ask, Surface, here, sizes};

/// The rows a page draws. A kind this build does not know is drawn as a plain row, which is what
/// `SectionKind::drawn_as` is for.
const ROWS: [SectionKind; 4] = [
    SectionKind::Row,
    SectionKind::Ranked,
    SectionKind::Continue,
    SectionKind::Grid,
];

/// The first image of the first role that has one.
fn image(tile: &Tile, roles: &[ImageRole]) -> Option<String> {
    roles.iter().find_map(|role| {
        tile.images
            .iter()
            .find(|image| image.role == *role)
            .map(|image| image.url.clone())
    })
}

/// The top bar: the lockup on the page margin and the menu beside it. 80 px tall at every width
/// the reference measured, and it stays where it is while the page moves under it.
fn bar(surface: &Surface, scene: &mut Scene) {
    let wide = scene.window.wide;
    let margin = sizes::margin(wide);
    scene.add(Part::plate(
        Rect::new(0.0, 0.0, wide, sizes::BAR_HIGH),
        Ink::Background,
        Corner::Square,
    ));
    let (logo_wide, logo_high) = sizes::LOGO;
    scene.add(Part {
        rect: Rect::new(
            margin,
            (sizes::BAR_HIGH - logo_high) / 2.0,
            logo_wide,
            logo_high,
        ),
        ink: Ink::Text,
        corner: Corner::Square,
        mark: Mark::Lockup,
        touch: Some(Touch::Does(Ask::Go(address::HOME.to_owned()))),
        says: surface.text("nav.home"),
    });
    let from = margin + logo_wide + sizes::NAV_FROM_LOGO;
    let room = (wide - margin - from).max(0.0);
    let items = surface.nav.len();
    if items == 0 || room <= 0.0 {
        return;
    }
    let count = f64::from(u32::try_from(items).unwrap_or(u32::MAX));
    let each = ((room - (count - 1.0) * sizes::NAV_GAP) / count).max(0.0);
    let (size, box_high) = (sizes::NAV_TYPE, 40.5);
    for (place, item) in surface.nav.iter().enumerate() {
        let at = f64::from(u32::try_from(place).unwrap_or(u32::MAX));
        let rect = Rect::new(
            from + at * (each + sizes::NAV_GAP),
            (sizes::BAR_HIGH - box_high) / 2.0,
            each,
            box_high,
        );
        // The item the viewer is on is drawn heavier, which is the one thing the reference says
        // the bar does with state: weight 500 on that one and 400 on the rest.
        let weight = if here(surface.route.as_ref(), item) {
            500
        } else {
            400
        };
        scene.add(
            Part::words(rect, Ink::Text, item.label.clone(), size, weight)
                .middle()
                .touched(Touch::Does(Ask::Go(address::of_screen(
                    item.screen,
                    item.genre.as_deref(),
                )))),
        );
    }
}

/// The hero card, from the page's own hero section. A page with none draws none and the rows move
/// up: nothing is invented to fill it.
fn hero(surface: &Surface, scene: &mut Scene, section: &Section, y: f64) -> f64 {
    let wide = scene.window.wide;
    let margin = sizes::margin(wide);
    let (card_wide, card_high) = sizes::hero(wide);
    let top = y + sizes::hero_top(wide);
    let rect = Rect::new(margin, top, card_wide, card_high);
    scene.add(Part::plate(rect, Ink::Surface, Corner::Panel));
    let Some(tile) = section.tiles.first() else {
        return sizes::hero_top(wide) + card_high + sizes::HERO_GAP;
    };
    if let Some(url) = image(
        tile,
        &[ImageRole::Backdrop, ImageRole::Still, ImageRole::Art],
    ) {
        scene.add(Part::picture(rect, Corner::Panel, url).saying(tile.name.clone()));
    }
    let inset = sizes::hero_inset(wide);
    let (name_size, name_high) = sizes::row_title(wide);
    let pill = sizes::pill(wide);
    let inside = (rect.wide - 2.0 * inset).max(0.0);
    scene.add(
        Part::words(
            Rect::new(
                rect.x + inset,
                rect.bottom() - inset - pill - sizes::GUTTER - name_high,
                inside,
                name_high,
            ),
            Ink::Text,
            tile.name.clone(),
            name_size,
            500,
        )
        .displayed(),
    );
    // One pill, and it is the one a viewer came for. The reference draws a second, "More Info",
    // which opens the title page this family has not got.
    let play = Rect::new(
        rect.x + inset,
        rect.bottom() - inset - pill,
        inside.min(sizes::READING),
        pill,
    );
    super::door::button(
        scene,
        play,
        Ink::Accent,
        Ink::AccentText,
        &surface.text("common.play"),
        Ask::Act(Action::Play {
            title: tile.title,
            episode: tile.episode.clone(),
        }),
    );
    sizes::hero_top(wide) + card_high + sizes::HERO_GAP
}

/// One tile of a row, with what it is marked with.
fn tile_at(surface: &Surface, scene: &mut Scene, rect: Rect, tile: &Tile, kind: SectionKind) {
    let ask = Ask::Act(Action::Play {
        title: tile.title,
        episode: tile.episode.clone(),
    });
    if kind == SectionKind::Ranked {
        // The rank in the left half and a portrait poster in the right, which is the one place a
        // portrait poster appears on a page of rows.
        let half = rect.wide / 2.0;
        let poster = Rect::new(rect.x + half, rect.y, half, half * sizes::RANKED_HIGH);
        scene.add(
            Part::plate(poster, Ink::Raised, Corner::Card)
                .touched(Touch::Does(ask.clone()))
                .saying(tile.name.clone()),
        );
        if let Some(url) = image(tile, &[ImageRole::Poster, ImageRole::Art]) {
            scene.add(Part::picture(poster, Corner::Card, url));
        }
        if let Some(rank) = tile.badges.first().and_then(|badge| badge.rank) {
            let words = surface
                .filled("badge.ranked", "rank", &rank.to_string())
                .unwrap_or_default();
            scene.add(
                Part::words(
                    Rect::new(rect.x, rect.y, half, poster.high),
                    Ink::Text,
                    words,
                    poster.high / 2.0,
                    500,
                )
                .middle()
                .displayed(),
            );
        }
        return;
    }
    scene.add(
        Part::plate(rect, Ink::Raised, Corner::Card)
            .touched(Touch::Does(ask))
            .saying(tile.name.clone()),
    );
    if let Some(url) = image(
        tile,
        &[ImageRole::Still, ImageRole::Backdrop, ImageRole::Art],
    ) {
        scene.add(Part::picture(rect, Corner::Card, url));
    }
    // A Continue Watching row carries 8 px of progress under the tile, which is the whole of the
    // difference between that row and a plain one.
    if kind == SectionKind::Continue {
        let bar = Rect::new(rect.x, rect.bottom(), rect.wide, sizes::PROGRESS);
        scene.add(Part::plate(bar, Ink::Raised, Corner::Square));
        let percent = f64::from(tile.percent.unwrap_or(0).min(100)) / 100.0;
        if percent > 0.0 {
            scene.add(
                Part::plate(
                    Rect::new(bar.x, bar.y, bar.wide * percent, bar.high),
                    Ink::Join,
                    Corner::Square,
                )
                .saying(
                    surface
                        .filled(
                            "say.watched_percent",
                            "percent",
                            &tile.percent.unwrap_or(0).to_string(),
                        )
                        .unwrap_or_default(),
                ),
            );
        }
    }
}

/// One row: its title, the page of tiles the viewer is on, and the controls that move it.
fn row(surface: &Surface, scene: &mut Scene, section: &Section, y: f64) -> f64 {
    let wide = scene.window.wide;
    let margin = sizes::margin(wide);
    let (size, box_high) = sizes::row_title(wide);
    if let Some(title) = &section.title {
        scene.add(
            Part::words(
                Rect::new(margin, y, sizes::content(wide), box_high),
                Ink::Text,
                title.clone(),
                size,
                500,
            )
            .displayed(),
        );
    }
    let kind = section.kind.drawn_as();
    let top = y + box_high + sizes::TITLE_GAP + sizes::SCROLLER_INSET;
    let across = sizes::across(wide);
    let count = usize::try_from(across.max(0.0) as u64).unwrap_or(0);
    let page = usize::try_from(*surface.paged.get(&section.id).unwrap_or(&0)).unwrap_or(0);
    let from = page * count;
    for (place, tile) in section.tiles.iter().skip(from).take(count).enumerate() {
        let at = f64::from(u32::try_from(place).unwrap_or(u32::MAX));
        let rect = Rect::new(
            margin + at * sizes::pitch(wide),
            top,
            sizes::tile(wide),
            sizes::tile(wide) * sizes::TILE_HIGH,
        );
        tile_at(surface, scene, rect, tile, kind);
    }
    // The controls that page the row, each in a scrim as wide as the page margin. The one that
    // goes back has no width on the first page, exactly as the reference measures it.
    let scrim = sizes::scrim(wide);
    let scroller = sizes::scroller(wide);
    if from + count < section.tiles.len() {
        scene.add(
            Part::plate(
                Rect::new(wide - scrim, top - sizes::SCROLLER_INSET, scrim, scroller),
                Ink::Surface,
                Corner::Square,
            )
            .touched(Touch::Does(Ask::Page {
                section: section.id.clone(),
                on: true,
            }))
            .saying(surface.text("common.next")),
        );
    }
    if page > 0 {
        scene.add(
            Part::plate(
                Rect::new(0.0, top - sizes::SCROLLER_INSET, scrim, scroller),
                Ink::Surface,
                Corner::Square,
            )
            .touched(Touch::Does(Ask::Page {
                section: section.id.clone(),
                on: false,
            }))
            .saying(surface.text("common.back")),
        );
    }
    sizes::row_pitch(wide)
}

/// Which sections a page draws, in the order it draws them.
fn drawn(surface: &Surface) -> Vec<&Section> {
    surface.screen.as_ref().map_or_else(Vec::new, |screen| {
        screen
            .sections
            .iter()
            .filter(|section| !section.tiles.is_empty())
            .collect()
    })
}

/// How tall the whole page is, which is what a window may be scrolled over.
pub(super) fn page_high(surface: &Surface, window: super::Window) -> f64 {
    let wide = window.wide;
    let mut high = sizes::BAR_HIGH;
    for section in drawn(surface) {
        if section.kind.drawn_as() == SectionKind::Hero {
            high += sizes::hero_top(wide) + sizes::hero(wide).1 + sizes::HERO_GAP;
        } else if ROWS.contains(&section.kind.drawn_as()) {
            high += sizes::row_pitch(wide);
        }
    }
    high
}

/// The page, every section of it.
///
/// Nothing is culled: a scene holds the whole page and a painter draws the part of it that is on
/// screen. At 1920 the reference's own hero is 18.7 px taller than the window, so a page that
/// drew only what fits would hold one card and nothing else, and the first wheel would have
/// nothing to show. What this costs is parts for rows nobody has scrolled to, which is counted in
/// floats and not in pixels.
pub(super) fn draw(surface: &Surface, scene: &mut Scene) {
    let mut y = sizes::BAR_HIGH - surface.down;
    for section in drawn(surface) {
        let kind = section.kind.drawn_as();
        if kind == SectionKind::Hero {
            y += hero(surface, scene, section, y);
        } else if ROWS.contains(&kind) {
            y += row(surface, scene, section, y);
        }
    }
    if surface.screen.is_none() {
        waiting(surface, scene);
    }
    bar(surface, scene);
}

/// What a page draws while it is waiting: the shape of the page that is coming, in plates with
/// nothing in them, on the same grid the real rows arrive on.
pub(super) fn waiting(_surface: &Surface, scene: &mut Scene) {
    let wide = scene.window.wide;
    let margin = sizes::margin(wide);
    let (card_wide, card_high) = sizes::hero(wide);
    let top = sizes::BAR_HIGH + sizes::hero_top(wide);
    scene.add(Part::plate(
        Rect::new(margin, top, card_wide, card_high),
        Ink::Surface,
        Corner::Panel,
    ));
    let mut y = top + card_high + sizes::HERO_GAP;
    let (_, box_high) = sizes::row_title(wide);
    let across = usize::try_from(sizes::across(wide).max(0.0) as u64).unwrap_or(0);
    while y < scene.window.high {
        scene.add(Part::plate(
            Rect::new(margin, y, sizes::tile(wide) * 2.0, box_high),
            Ink::Surface,
            Corner::Chip,
        ));
        for place in 0..across {
            let at = f64::from(u32::try_from(place).unwrap_or(u32::MAX));
            scene.add(Part::plate(
                Rect::new(
                    margin + at * sizes::pitch(wide),
                    y + box_high + sizes::TITLE_GAP + sizes::SCROLLER_INSET,
                    sizes::tile(wide),
                    sizes::tile(wide) * sizes::TILE_HIGH,
                ),
                Ink::Raised,
                Corner::Card,
            ));
        }
        y += sizes::row_pitch(wide);
    }
}

#[cfg(test)]
mod tests {
    use super::super::reading::english;
    use super::super::scene::Mark;
    use super::super::tests::{opened, route};
    use super::super::{Ask, Face, Surface, Window, sizes};
    use crate::host::Turn;
    use contract::core_msg::{Action, Target};

    const WINDOWS: [(f64, f64); 5] = [
        (390.0, 844.0),
        (768.0, 899.0),
        (1280.0, 899.0),
        (1440.0, 900.0),
        (1920.0, 899.0),
    ];

    /// A page of rows as the server really sends one: a hero, a plain row of twenty, a Top 10 and
    /// a Continue Watching row. The names are distinct per row so a test can say which row it is
    /// looking at.
    pub fn a_page() -> String {
        let row = |word: &str, from: u32, many: u32, role: &str, extra: &str| {
            (0..many)
                .map(|at| {
                    format!(
                        r#"{{"title":"movie:{}","name":"{word} {}","images":[{{"role":"{role}","url":"https://pictures.example/{}.jpg"}}]{}}}"#,
                        from + at,
                        at + 1,
                        from + at,
                        extra.replace("{rank}", &(at + 1).to_string())
                    )
                })
                .collect::<Vec<_>>()
                .join(",")
        };
        format!(
            r#"{{"name":"home","ttl_s":60,"sections":[
              {{"id":"hero","kind":"hero","tiles":[{}]}},
              {{"id":"new","kind":"row","title":"New Releases","tiles":[{}]}},
              {{"id":"top","kind":"ranked","title":"Top 10","tiles":[{}]}},
              {{"id":"keep","kind":"continue","title":"Continue Watching","tiles":[{}]}}
            ]}}"#,
            row("Opening", 1, 1, "backdrop", ""),
            row("New", 10, 20, "still", ""),
            row(
                "Top",
                100,
                10,
                "poster",
                r#","badges":[{"kind":"ranked","rank":{rank}}]"#
            ),
            row("Keep", 200, 3, "still", r#","percent":40"#),
        )
    }

    fn browsing() -> Surface {
        let mut surface = opened();
        route(&mut surface, Target::Home);
        let mut turn = Turn::default();
        turn.pages.push(contract::core_msg::ScreenBody {
            request: "page".to_owned(),
            body: a_page(),
            stale: false,
        });
        turn.pages.push(contract::core_msg::ScreenBody {
            request: "nav".to_owned(),
            body: r#"{"ttl_s":60,"items":[
                {"id":"home","label":"Home","screen":"home"},
                {"id":"films","label":"Films","screen":"films"}
            ]}"#
            .to_owned(),
            stale: false,
        });
        surface.apply(&turn);
        assert_eq!(surface.face(), Face::Browse);
        assert!(
            surface.screen.is_some(),
            "the page the server sent was not read"
        );
        assert_eq!(
            surface.nav.len(),
            2,
            "the menu the server sent was not read"
        );
        surface
    }

    /// The rows are drawn, each tile where the reference's one rule puts it, at every width.
    #[test]
    fn a_row_puts_its_tiles_where_the_reference_s_rule_puts_them() {
        let surface = browsing();
        for (wide, high) in WINDOWS {
            let scene = surface.draw(Window::new(wide, high));
            let tiles: Vec<_> = scene
                .parts
                .iter()
                .filter(|part| part.says.starts_with("New ") && part.touch.is_some())
                .collect();
            assert_eq!(
                tiles.len(),
                sizes::across(wide) as usize,
                "a row draws one page of tiles, and drew {} at {wide}",
                tiles.len()
            );
            for (place, part) in tiles.iter().enumerate() {
                let at = place as f64;
                let want_x = sizes::margin(wide) + at * sizes::pitch(wide);
                assert!(
                    (part.rect.x - want_x).abs() < 0.01,
                    "tile {place} is at {} and the rule says {want_x} at {wide}",
                    part.rect.x
                );
                assert!(
                    (part.rect.wide - sizes::tile(wide)).abs() < 0.01,
                    "tile {place} is {} wide and the rule says {} at {wide}",
                    part.rect.wide,
                    sizes::tile(wide)
                );
            }
            // Every tile is inside the page's margins.
            let last = tiles.last().expect("a tile");
            assert!(
                last.rect.right() <= wide - sizes::margin(wide) + 0.01,
                "a tile crosses the page margin at {wide}"
            );
        }
    }

    /// The row's title and the bar are drawn, and the title the server sent is the words used.
    #[test]
    fn a_page_draws_the_bar_the_hero_and_the_server_s_own_row_titles() {
        let surface = browsing();
        for (wide, high) in WINDOWS {
            let scene = surface.draw(Window::new(wide, high));
            assert!(
                scene.says("New Releases"),
                "a row title is missing at {wide}"
            );
            assert!(scene.says("Top 10"), "at {wide}");
            // The menu, by the labels the server sent, and not by the lockup beside it: the
            // lockup says `nav.home` to a reader, so asserting those words alone passed with the
            // whole menu deleted. Proved on 2026-11-13 by deleting it.
            let labels: Vec<&str> = scene.words().into_iter().collect();
            for label in ["Home", "Films"] {
                assert!(
                    labels.contains(&label),
                    "the menu item {label} is not drawn at {wide}: {labels:?}"
                );
            }
            // The item the viewer is on is the heavier one. The route is Home here.
            let weight = |words: &str| {
                scene
                    .parts
                    .iter()
                    .find_map(|part| match &part.mark {
                        Mark::Words { text, weight, .. } if text == words => Some(*weight),
                        _ => None,
                    })
                    .unwrap_or_else(|| panic!("no {words} at {wide}"))
            };
            assert_eq!(weight("Home"), 500, "at {wide}");
            assert_eq!(weight("Films"), 400, "at {wide}");
            assert!(
                scene.says(&english()["common.play"]),
                "the hero has no way to play at {wide}"
            );
            // The bar is the measured height and it is drawn over the rows, so it is last.
            let bar = scene
                .parts
                .iter()
                .rev()
                .find(|part| part.rect.y == 0.0 && part.rect.wide == wide)
                .expect("a bar");
            assert!((bar.rect.high - sizes::BAR_HIGH).abs() < 0.01, "at {wide}");
        }
    }

    /// The reference's own figures at 1920: the hero fills the content box and 0 rows are visible
    /// before the page is scrolled.
    #[test]
    fn at_1920_the_hero_is_taller_than_the_window_and_no_row_is_visible() {
        let surface = browsing();
        let window = Window::new(1920.0, 899.0);
        let scene = surface.draw(window);
        let hero = scene
            .parts
            .iter()
            .find(|part| (part.rect.wide - 1800.0).abs() < 0.01)
            .expect("the hero card");
        assert!((hero.rect.high - 825.7).abs() < 1.0, "{:?}", hero.rect);
        assert!(hero.rect.x == 60.0, "the hero starts on the page margin");
        assert!(
            hero.rect.bottom() > window.high,
            "the reference sees no row at 1920 before scrolling, and this window does"
        );
    }

    /// A wheel moves the page under the bar, and never past the ends of it.
    #[test]
    fn the_page_scrolls_under_a_bar_that_stays() {
        let mut surface = browsing();
        let window = Window::new(1920.0, 899.0);
        let before = surface.draw(window);
        surface.wheel(window, 600.0);
        let after = surface.draw(window);
        let title_at = |scene: &super::super::scene::Scene| {
            scene
                .parts
                .iter()
                .find(|part| part.text() == Some("New Releases"))
                .expect("the row title")
                .rect
                .y
        };
        assert!(
            (title_at(&before) - title_at(&after) - 600.0).abs() < 0.01,
            "the page did not move by what the wheel said"
        );
        let bar_before = before.parts.iter().rev().find(|p| p.rect.y == 0.0);
        let bar_after = after.parts.iter().rev().find(|p| p.rect.y == 0.0);
        assert_eq!(
            bar_before.map(|part| part.rect),
            bar_after.map(|part| part.rect),
            "the bar moved with the page"
        );
        // And it stops at the ends.
        surface.wheel(window, -10_000.0);
        assert!((title_at(&surface.draw(window)) - title_at(&before)).abs() < 0.01);
        surface.wheel(window, 1_000_000.0);
        let far = surface.draw(window);
        assert!(title_at(&far) > -super::page_high(&surface, window));
    }

    /// A row pages by a full page of tiles, which is the content box and one gutter.
    #[test]
    fn a_row_pages_by_a_full_page_of_tiles() {
        let mut surface = browsing();
        let window = Window::new(1920.0, 899.0);
        let page = |surface: &Surface| {
            surface
                .draw(window)
                .parts
                .iter()
                .filter(|part| part.says.starts_with("New ") && part.touch.is_some())
                .map(|part| part.says.clone())
                .collect::<Vec<_>>()
        };
        let first = page(&surface);
        // The control on this row and not on another: every row has one, so it is found by the
        // row it sits beside rather than by its words.
        let tile = surface
            .draw(window)
            .parts
            .iter()
            .find(|part| part.says == "New 1")
            .expect("the first tile of the row")
            .rect;
        let on = surface
            .draw(window)
            .parts
            .iter()
            .find(|part| {
                part.says == english()["common.next"] && (part.rect.y - tile.y).abs() < 20.0
            })
            .expect("the control that pages this row")
            .rect;
        let asks = surface.click(window, on.x + 1.0, on.y + 1.0);
        assert!(asks.is_empty(), "paging a row asks the shell for nothing");
        let second = page(&surface);
        assert_eq!(first.len(), 7, "seven across at 1920: {first:?}");
        assert_eq!(first.first().map(String::as_str), Some("New 1"));
        assert_eq!(
            second.first().map(String::as_str),
            Some("New 8"),
            "a page of seven tiles did not start at the eighth: {second:?}"
        );
    }

    /// A press on a tile plays it, with the title the server named.
    #[test]
    fn a_press_on_a_tile_plays_that_title() {
        let mut surface = browsing();
        let window = Window::new(1440.0, 900.0);
        let scene = surface.draw(window);
        let tile = scene
            .parts
            .iter()
            .find(|part| part.says == "New 2" && part.touch.is_some())
            .expect("a tile")
            .rect;
        let asks = surface.click(window, tile.x + 1.0, tile.y + 1.0);
        assert_eq!(
            asks,
            vec![Ask::Act(Action::Play {
                title: contract::TitleKey::parse("movie:11").expect("a title key"),
                episode: None,
            })]
        );
    }

    /// A page with nothing in it yet draws the shape of the page that is coming, and nothing of it
    /// is outside the window.
    #[test]
    fn a_page_that_has_not_arrived_draws_the_shape_of_the_one_that_will() {
        let mut surface = opened();
        route(&mut surface, Target::Wait);
        for (wide, high) in WINDOWS {
            let scene = surface.draw(Window::new(wide, high));
            assert_eq!(scene.face, Face::Waiting, "at {wide}");
            // The hero's own plate at least, and a row of tiles wherever the window has room
            // under it. At 768 the hero is 838.5 tall under an 80 px bar, so there is none.
            assert!(
                scene.parts.len() >= 2,
                "the waiting page drew nothing at {wide}"
            );
            assert!(scene.words().is_empty(), "it invented words at {wide}");
            assert!(scene.outside().is_empty(), "a part is outside at {wide}");
        }
    }

    #[test]
    fn every_label_this_page_names_is_in_the_strings_file() {
        let book = english();
        for label in [
            "badge.ranked",
            "common.back",
            "common.next",
            "common.play",
            "nav.home",
            "say.watched_percent",
        ] {
            assert!(book.contains_key(label), "strings/en.json has no {label}");
        }
    }
}
