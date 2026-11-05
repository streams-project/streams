//! Page rules on the invented sample, through the builder.

use std::sync::Arc;

use contract::client::DeviceClass;
use contract::screen::{ScreenName, SectionKind};
use server_catalog::catalog::{CatalogItem, FLAG_NONSCRIPTED};
use server_catalog::manifest::role;
use server_catalog::sample::{sample_trust, special, write_composed_sample, write_sample};
use server_catalog::{DataSet, item_key};
use server_pages::genre_spec::{GenreSpec, SpecFacts, resolve};
use server_pages::hero::{HeroAsk, hero_keys};
use server_pages::more_like::more_like_keys;
use server_pages::page::{
    FIRST_SCREEN, MAX_ROW_DEPTH, PageAsk, PageContext, RowItem, THIN_PAGE_FLOOR, meets_vote_floor,
    member_quality_for,
};
use server_pages::plan::{
    INITIAL_ROWS, INITIAL_ROWS_GENRE, MIN_ROWS_FLOOR, PagePlan, ROWS_CAP, page_cap,
};
use server_pages::requests::RequestStore;
use server_pages::signals::Signals;
use server_pages::{Pages, Personal, ScreenAsk};
use server_rules::answer::{Cleared, Images};
use server_rules::{Tier, Viewer};

/// Time of the sample: 2023-11-14.
const NOW: u64 = 1_700_000_100_000;
/// Titles a row holds.
const ROW_TITLES: usize = 15;
/// Titles a row made from a request needs.
const REQUEST_MIN_ITEMS: usize = 12;
/// Tiles a row sends before its cursor.
const ROW_PAGE: usize = 24;
/// A title is New for this many days after its release.
const NEW_DAYS: i64 = 45;
/// Notification tiles.
const NOTIFICATIONS_MAX: usize = 12;

/// One folder per call, so two tests never write one sample at the same time.
static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

fn set(name: &str, composed: bool) -> Arc<DataSet> {
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("pages_{name}_{}_{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let manifest = if composed {
        write_composed_sample(&dir)
    } else {
        write_sample(&dir)
    };
    Arc::new(DataSet::load(&manifest, &sample_trust()).expect("sample"))
}

fn ask(screen: ScreenName, genre: Option<&str>, seed: &str) -> ScreenAsk {
    ScreenAsk {
        screen,
        genre: genre.map(str::to_owned),
        class: DeviceClass::Web,
        now_ms: NOW,
        owner: String::from("owner"),
        seed: seed.to_owned(),
        personal: Personal::default(),
        blocked_rows: Vec::new(),
    }
}

fn wire<T: Cleared>(value: &T) -> serde_json::Value {
    serde_json::to_value(value).expect("json")
}

fn sections(page: &serde_json::Value) -> Vec<serde_json::Value> {
    page["sections"].as_array().cloned().unwrap_or_default()
}

fn ids(page: &serde_json::Value) -> Vec<String> {
    sections(page)
        .iter()
        .flat_map(|section| {
            let tiles = section["tiles"].as_array().cloned().unwrap_or_default();
            tiles
                .into_iter()
                .map(|tile| tile["title"].as_str().unwrap_or("").to_owned())
                .collect::<Vec<_>>()
        })
        .collect()
}

fn item() -> CatalogItem {
    let data = set("gate", false);
    data.catalog[&item_key("movie", 100)].clone()
}

fn tiles(section: &serde_json::Value) -> Vec<serde_json::Value> {
    section["tiles"].as_array().cloned().unwrap_or_default()
}

/// The kind word and source id of the title a tile stands for.
fn parts(tile: &serde_json::Value) -> (&'static str, u32) {
    let text = tile["title"].as_str().expect("a title key");
    let key = contract::TitleKey::parse(text).expect("a title key");
    let word = if key.kind() == contract::TitleKind::Tv {
        "tv"
    } else {
        "movie"
    };
    (word, key.id())
}

/// The catalogue row behind a tile.
fn item_of<'a>(data: &'a DataSet, tile: &serde_json::Value) -> &'a CatalogItem {
    let (word, id) = parts(tile);
    &data.catalog[&item_key(word, id)]
}

/// Days between the sample's today and a title's release date, or None when it has none.
fn days_out(data: &DataSet, tile: &serde_json::Value) -> Option<i64> {
    let (word, id) = parts(tile);
    let date = &data.fresh.get(word, id)?.release_date;
    let day = server_catalog::fresh::parse_days(date)?;
    Some(server_catalog::fresh::days_of_ms(NOW) - day)
}

/// The rows the page builder makes for one page of the composed sample, with every title in them.
fn built_rows(
    data: &DataSet,
    requests: &RequestStore,
    signals: &Signals,
) -> Vec<server_pages::page::Row> {
    let context = PageContext::new(
        data,
        requests,
        PageAsk {
            surface: "both",
            genre: None,
            genre_spec: None,
            language: None,
            tier: Tier::Adult,
            late_night: false,
            taste: vec![0.0; data.vectors.dims],
            target_rows: INITIAL_ROWS,
            session: "session-a",
            signals,
            now_ms: NOW,
        },
    );
    context.build_page()
}

/// A page context whose own filters a title is checked against (no genre, or one page's rule).
fn pool_check(
    data: &DataSet,
    requests: &RequestStore,
    signals: &Signals,
    spec: Option<&GenreSpec>,
    item: &CatalogItem,
) -> bool {
    let context = PageContext::new(
        data,
        requests,
        PageAsk {
            surface: "both",
            genre: None,
            genre_spec: spec,
            language: None,
            tier: Tier::Adult,
            late_night: false,
            taste: vec![0.0; data.vectors.dims],
            target_rows: INITIAL_ROWS,
            session: "session-a",
            signals,
            now_ms: NOW,
        },
    );
    context.page_pool_member(item)
}

#[test]
fn s72_quality_gates_for_rows_and_for_judged_pages() {
    let data = set("quality", false);
    let facts = SpecFacts {
        events: &data.events,
        judged: false,
    };
    let good = item();
    assert!(member_quality_for(None, &good, &facts));
    let mut weak = good.clone();
    weak.vote_average = 5.9;
    assert!(!member_quality_for(None, &weak, &facts));
    let reality = resolve("Reality", "tv", &data.events);
    assert!(
        member_quality_for(Some(&reality), &weak, &facts),
        "a lower gate for reality"
    );
    weak.vote_average = 5.4;
    assert!(!member_quality_for(Some(&reality), &weak, &facts));
    // a judged page takes the same lower gate, and only while the judged file is loaded
    let judged = SpecFacts {
        events: &data.events,
        judged: true,
    };
    let stand_up = GenreSpec::StandUp;
    assert!(
        stand_up.is_judged(&judged) && !stand_up.is_judged(&facts),
        "a page is judged only while the judged file is loaded"
    );
    let mut niche = good.clone();
    niche.vote_average = 5.9;
    niche.popularity = 1.9;
    assert!(
        member_quality_for(Some(&stand_up), &niche, &judged),
        "a judged page keeps a less known title"
    );
    assert!(
        !member_quality_for(Some(&stand_up), &niche, &facts),
        "with no judged file the same page falls back to the default gate"
    );
    niche.vote_average = 5.4;
    assert!(!member_quality_for(Some(&stand_up), &niche, &judged));
    // the rest of the gate: an overview of some length, both pictures, and popularity
    for (why, flag) in [
        ("overview", server_catalog::catalog::FLAG_OVERVIEW),
        ("poster", server_catalog::catalog::FLAG_POSTER),
        ("backdrop", server_catalog::catalog::FLAG_BACKDROP),
    ] {
        let mut broken = good.clone();
        broken.flags &= !flag;
        assert!(!member_quality_for(None, &broken, &facts), "{why}");
        assert!(
            !member_quality_for(Some(&reality), &broken, &facts),
            "{why} on reality"
        );
    }
    let mut thin = good.clone();
    thin.popularity = 1.9;
    assert!(!member_quality_for(None, &thin, &facts), "popularity 2");
    assert!(
        member_quality_for(Some(&reality), &thin, &facts),
        "reality takes 0.5"
    );
    thin.popularity = 0.4;
    assert!(!member_quality_for(Some(&reality), &thin, &facts));
    let mut few = good;
    few.vote_count = 9;
    assert!(!meets_vote_floor(&few, false));
    few.vote_count = 5;
    assert!(meets_vote_floor(&few, true));
    few.vote_count = 0;
    assert!(meets_vote_floor(&few, false), "an unknown count passes");
}

#[test]
fn s73_the_safety_nets_keep_talk_suggestive_and_out_of_season_titles_out_of_the_pool() {
    use server_catalog::catalog::{FLAG_CHRISTMAS, FLAG_SUGGESTIVE, compute_flags};

    let data = set("nets", false);
    let requests = RequestStore::default();
    let signals = Signals::default();
    let chat = GenreSpec::ChatShows;
    assert!(chat.includes_nonscripted());
    let dramas = resolve("Dramas", "both", &data.events);
    assert!(!dramas.includes_nonscripted());
    // the plain title the three below are made from is in the pool, so the nets are what refuses them
    let plain = item();
    assert!(pool_check(&data, &requests, &signals, None, &plain));

    // a talk show: off every page except one that asks for nonscripted titles
    let mut talk = plain.clone();
    talk.media_type = String::from("tv");
    talk.tv_type = Some(String::from("Talk Show"));
    talk.flags |= compute_flags("Late Talk", &[], Some("Talk Show"), "", None, None);
    assert_eq!(talk.flags & FLAG_NONSCRIPTED, FLAG_NONSCRIPTED);
    assert!(!pool_check(&data, &requests, &signals, None, &talk));
    assert!(!pool_check(
        &data,
        &requests,
        &signals,
        Some(&dramas),
        &talk
    ));
    assert!(pool_check(&data, &requests, &signals, Some(&chat), &talk));

    // suggestive: off every page, the one that would ask for it included
    let mut suggestive = plain.clone();
    suggestive.keywords = vec![String::from("erotic")];
    suggestive.flags |= compute_flags("A Film", &[String::from("erotic")], None, "", None, None);
    assert_eq!(suggestive.flags & FLAG_SUGGESTIVE, FLAG_SUGGESTIVE);
    assert!(!pool_check(&data, &requests, &signals, None, &suggestive));
    assert!(!pool_check(
        &data,
        &requests,
        &signals,
        Some(&chat),
        &suggestive
    ));

    // the sample's today is 2023-11-14, before the Christmas window opens on 11-20
    let mut yule = plain.clone();
    yule.flags |= compute_flags("A Christmas Film", &[], None, "", None, None);
    assert_eq!(yule.flags & FLAG_CHRISTMAS, FLAG_CHRISTMAS);
    assert!(!pool_check(&data, &requests, &signals, None, &yule));
}

#[test]
fn s85_s159_a_plain_page_has_a_row_per_seed_and_one_session_stays_stable_while_two_differ() {
    let data = set("clusters", false);
    let pages = Pages::new(Images::new(None));
    let viewer = Viewer::visitor(Tier::Adult);
    let page = |seed: &str| {
        wire(
            &pages
                .screen(&data, &viewer, &ask(ScreenName::Home, None, seed))
                .unwrap(),
        )
    };
    let first = page("session-a");
    assert_eq!(ids(&first), ids(&page("session-a")), "S159");
    assert!(sections(&first).len() >= 8);
    // S85: one row per cluster seed, named for that seed, and no seed used twice
    let seeds: Vec<&str> = data
        .seeds
        .seeds
        .iter()
        .map(|seed| seed.name.as_str())
        .collect();
    assert_eq!(seeds.len(), 12, "the sample has a seed per genre");
    let mut used: Vec<String> = Vec::new();
    for section in sections(&first) {
        let id = section["id"].as_str().unwrap_or_default().to_owned();
        if !id.starts_with("thematic-") {
            continue;
        }
        let title = section["title"].as_str().unwrap_or_default().to_owned();
        assert!(seeds.contains(&title.as_str()), "{title} is no seed name");
        assert!(!used.contains(&title), "{title} made two rows");
        assert!(
            tiles(&section).len() <= ROW_TITLES,
            "{title} holds {} titles",
            tiles(&section).len()
        );
        used.push(title);
    }
    assert!(used.len() >= 6, "rows from seeds: {used:?}");
    let other = Pages::new(Images::new(None));
    let elsewhere = wire(
        &other
            .screen(&data, &viewer, &ask(ScreenName::Home, None, "session-b"))
            .unwrap(),
    );
    assert_ne!(
        ids(&first),
        ids(&elsewhere),
        "two sessions see different pages"
    );
}

/// The session ranked Top Picks row takes up to 150 films and 150 series, so a visitor with no
/// taste still sees both kinds in it. S86's blending of the other kind into a thematic row cannot
/// be reached on this sample: every sample genre holds one kind only, so a row's other kind
/// candidates all fall under the closeness floor.
#[test]
fn s92_the_session_ranked_top_picks_row_draws_on_films_and_series() {
    let data = set("blend", false);
    let pages = Pages::new(Images::new(None));
    let page = wire(
        &pages
            .screen(
                &data,
                &Viewer::visitor(Tier::Adult),
                &ask(ScreenName::Home, None, "session-a"),
            )
            .unwrap(),
    );
    let row = sections(&page)
        .into_iter()
        .find(|section| {
            section["id"]
                .as_str()
                .is_some_and(|id| id.starts_with("top_picks-"))
        })
        .expect("a top picks row");
    let kinds: Vec<&'static str> = tiles(&row).iter().map(|tile| parts(tile).0).collect();
    let films = kinds.iter().filter(|word| **word == "movie").count();
    let series = kinds.len() - films;
    assert!(
        films >= 3 && series >= 3,
        "films {films}, series {series} in {} tiles",
        kinds.len()
    );
}

#[test]
fn s87_a_genre_page_keeps_its_kind_and_leads_with_its_genre() {
    let data = set("genre", false);
    let pages = Pages::new(Images::new(None));
    let viewer = Viewer::visitor(Tier::Adult);
    let page = wire(
        &pages
            .screen(
                &data,
                &viewer,
                &ask(ScreenName::Genre, Some("film.dramas"), "s"),
            )
            .unwrap(),
    );
    assert!(
        ids(&page).iter().all(|id| id.starts_with("movie:")),
        "the films page holds films"
    );
    let all = sections(&page);
    let first_row = all.iter().find(|s| s["kind"] == "row").expect("a row");
    for tile in tiles(first_row) {
        let item = item_of(&data, &tile);
        assert!(item.genres.iter().any(|g| g == "Drama"), "{tile}");
    }
    for section in all.iter().filter(|s| s["kind"] != "hero") {
        let held = tiles(section).len();
        assert!(
            held <= ROW_TITLES,
            "{} holds {held} titles",
            section["title"]
        );
    }
}

#[test]
fn s89_a_composed_row_holds_its_minimum_runs_deep_and_keeps_animation_to_a_share() {
    let data = set("composed", true);
    let requests = RequestStore::load(
        data.files
            .get(role::REQUESTS)
            .expect("the composed sample carries requests"),
        &data.catalog,
        &data.vectors,
        &data.mean,
        &data.events,
    );
    let signals = Signals::default();
    let rows = built_rows(&data, &requests, &signals);
    let made: Vec<&server_pages::page::Row> = rows
        .iter()
        .filter(|row| row.archetype == "thematic")
        .collect();
    assert!(made.len() >= 8, "a composed page of rows: {}", rows.len());
    let animated = |item: &RowItem| item.genres.iter().any(|g| g == "Animation");
    let mut about_animation = 0;
    for row in made {
        let held = row.items.len();
        assert!(
            held >= REQUEST_MIN_ITEMS,
            "{} holds {held} titles, under the row minimum",
            row.title
        );
        assert!(held <= MAX_ROW_DEPTH, "{} runs {held} deep", row.title);
        assert!(
            held > FIRST_SCREEN,
            "{} does not reach past its first screen",
            row.title
        );
        if row.title.to_lowercase().contains("anim") {
            about_animation += 1;
            assert!(
                row.items.iter().all(animated),
                "{} is about animation and holds live action",
                row.title
            );
            continue;
        }
        let share = row.items.iter().filter(|item| animated(item)).count();
        assert!(
            share * 4 <= held,
            "{} holds {share} animated of {held}",
            row.title
        );
        assert!(
            row.items.iter().take(ROW_PAGE).all(|item| !animated(item)),
            "{} opens with an animated title",
            row.title
        );
    }
    assert_eq!(
        about_animation, 1,
        "the sample has one row about animation, so the share rule is not passing on an empty set"
    );
    // the depth reaches the client as a cursor on the row
    let pages = Pages::new(Images::new(None));
    let page = wire(
        &pages
            .screen(
                &data,
                &Viewer::visitor(Tier::Adult),
                &ask(ScreenName::Home, None, "session-a"),
            )
            .unwrap(),
    );
    let deep = sections(&page)
        .iter()
        .filter(|section| section.get("cursor").is_some())
        .count();
    assert!(deep > 0, "a row deeper than one page carries a cursor");
}

#[test]
fn s93_the_new_releases_row_holds_real_recent_dates_best_known_first() {
    let data = set("new", false);
    let pages = Pages::new(Images::new(None));
    let page = wire(
        &pages
            .screen(
                &data,
                &Viewer::visitor(Tier::Adult),
                &ask(ScreenName::Home, None, "session-a"),
            )
            .unwrap(),
    );
    let row = sections(&page)
        .into_iter()
        .find(|section| section["title"] == "New Releases")
        .expect("the sample's release dates make a New Releases row");
    let held = tiles(&row);
    assert!(
        (3..=ROW_TITLES).contains(&held.len()),
        "{} titles",
        held.len()
    );
    let mut known = f32::MAX;
    for tile in &held {
        let item = item_of(&data, tile);
        let out = days_out(&data, tile).expect("a release date");
        assert!(
            (0..=NEW_DAYS).contains(&out),
            "{} came out {out} days ago",
            item.title
        );
        assert!(item.has_poster() && item.has_overview(), "{}", item.title);
        assert!(
            item.popularity <= known,
            "{} breaks the order by popularity",
            item.title
        );
        known = item.popularity;
        // S80: a title this new is badged new
        assert!(
            tile["badges"]
                .as_array()
                .is_some_and(|badges| badges.iter().any(|badge| badge["kind"] == "new")),
            "{} carries no new badge",
            item.title
        );
    }
}

#[test]
fn s100_the_hero_has_the_limit_alternates_kinds_and_keeps_to_the_tier() {
    let data = set("hero", false);
    let ask = HeroAsk {
        tier: Tier::Adult,
        surface: "both",
        genre: None,
        session: "s",
        limit: 8,
        seen: &[],
    };
    let keys = hero_keys(&data, &ask);
    assert_eq!(keys.len(), 8);
    let kinds: Vec<&str> = keys
        .iter()
        .map(|key| data.catalog[key].media_type.as_str())
        .collect();
    assert_ne!(kinds[0], kinds[1], "films and series alternate");
    assert_eq!(keys, hero_keys(&data, &ask), "a session keeps its hero");
    let small = HeroAsk {
        tier: Tier::LittleKids,
        ..ask
    };
    for key in hero_keys(&data, &small) {
        assert!(server_rules::eligible(
            Tier::LittleKids,
            &data.catalog[&key],
            &data.profiles
        ));
    }
    let film = HeroAsk {
        surface: "movie",
        ..ask
    };
    assert!(
        hero_keys(&data, &film)
            .iter()
            .all(|key| data.catalog[key].media_type == "movie")
    );
    let seen = [keys[0]];
    let without = HeroAsk { seen: &seen, ..ask };
    assert!(
        !hero_keys(&data, &without).contains(&keys[0]),
        "a seen title leaves the hero"
    );
}

#[test]
fn s104_more_like_this_leaves_out_the_anchor_and_keeps_to_the_tier_and_limit() {
    let data = set("more_like", false);
    let anchor = item_key("movie", 100);
    let keys = more_like_keys(&data, Tier::Adult, anchor, 5);
    assert_eq!(keys.len(), 5);
    assert!(!keys.contains(&anchor));
    let kept = more_like_keys(&data, Tier::OlderKids, item_key("movie", 106), 5);
    assert!(kept.iter().all(|key| server_rules::eligible(
        Tier::OlderKids,
        &data.catalog[key],
        &data.profiles
    )));
    assert!(
        more_like_keys(&data, Tier::Adult, 1, 5).is_empty(),
        "a title the catalogue does not hold has no neighbours"
    );
}

#[test]
fn s96_the_kids_screen_is_built_at_the_child_tier_even_for_an_adult() {
    let data = set("kids", false);
    let pages = Pages::new(Images::new(None));
    let adult = wire(
        &pages
            .screen(
                &data,
                &Viewer::visitor(Tier::Adult),
                &ask(ScreenName::Kids, None, ""),
            )
            .unwrap(),
    );
    let found = ids(&adult);
    assert!(!found.is_empty(), "the Kids screen has titles");
    for id in &found {
        let key = contract::TitleKey::parse(id).unwrap();
        let word = if key.kind() == contract::TitleKind::Tv {
            "tv"
        } else {
            "movie"
        };
        let item = &data.catalog[&item_key(word, key.id())];
        assert!(
            server_rules::eligible(Tier::OlderKids, item, &data.profiles),
            "{id}"
        );
    }
    // a child's own ask of the same screen reaches the same titles
    let child = wire(
        &pages
            .screen(
                &data,
                &Viewer::visitor(Tier::OlderKids),
                &ask(ScreenName::Kids, None, ""),
            )
            .unwrap(),
    );
    assert_eq!(found, ids(&child), "the adult sees the child's page");
}

#[test]
fn s200_notifications_hold_dated_titles_with_art_newest_first_then_due_soonest() {
    let data = set("notify", false);
    let pages = Pages::new(Images::new(None));
    let page = wire(
        &pages
            .screen(
                &data,
                &Viewer::visitor(Tier::Adult),
                &ask(ScreenName::Notifications, None, ""),
            )
            .unwrap(),
    );
    let section = sections(&page).into_iter().next().expect("a grid");
    let held = tiles(&section);
    assert!(
        !held.is_empty() && held.len() <= NOTIFICATIONS_MAX,
        "{} tiles",
        held.len()
    );
    let mut out = Vec::new();
    for tile in &held {
        let item = item_of(&data, tile);
        assert!(item.has_poster(), "{} has no art", item.title);
        let days = days_out(&data, tile).expect("a release date");
        assert!(days.abs() <= 14, "{} is {days} days from today", item.title);
        out.push(days);
    }
    // the released ones come first, newest first; the due ones follow, soonest first
    let split = out.iter().position(|days| *days < 0).unwrap_or(out.len());
    let (released, due) = out.split_at(split);
    assert!(released.iter().all(|days| *days >= 0));
    assert!(
        released.windows(2).all(|pair| pair[0] <= pair[1]),
        "released newest first: {released:?}"
    );
    assert!(due.iter().all(|days| *days < 0));
    assert!(
        due.windows(2).all(|pair| pair[0] >= pair[1]),
        "due soonest first: {due:?}"
    );
}

#[test]
fn s80_a_tile_shows_its_tone_label_before_its_genres() {
    let data = set("tone", false);
    let pages = Pages::new(Images::new(None));
    let page = wire(
        &pages
            .screen(
                &data,
                &Viewer::visitor(Tier::Adult),
                &ask(ScreenName::Kids, None, "session-a"),
            )
            .unwrap(),
    );
    // the one sample title with a model profile: tone "gentle", genre "Family"
    let key = format!("movie:{}", special::UNRATED_KIDS_PROFILE);
    let tile = sections(&page)
        .iter()
        .flat_map(tiles)
        .find(|tile| tile["title"] == key)
        .expect("the title with a model profile is on the Kids screen");
    assert_eq!(
        tile["tags"],
        serde_json::json!(["Gentle", "Family"]),
        "the tone label comes before the genre"
    );
    assert!(
        tiles(&sections(&page)[1]).iter().all(|tile| {
            let tags = tile["tags"].as_array().cloned().unwrap_or_default();
            tags.len() <= 3
        }),
        "at most three tags"
    );
}

#[test]
fn s75_a_page_shows_its_initial_rows_and_pages_the_rest_up_to_its_cap() {
    let data = set("plan", false);
    let pages = Pages::new(Images::new(None));
    let viewer = Viewer::visitor(Tier::Adult);
    let rows_of = |page: &serde_json::Value| {
        sections(page)
            .into_iter()
            .filter(|section| section["kind"] != "hero")
            .count()
    };
    // no genre: nothing but the page cap limits the rows, and the first page shows the initial rows
    assert_eq!(
        page_cap(&data, "both", None, None, Tier::Adult, false),
        usize::MAX
    );
    let plain = PagePlan::new(usize::MAX, false, false, INITIAL_ROWS);
    assert_eq!((plain.cap, plain.target_rows), (ROWS_CAP, INITIAL_ROWS));
    let home = wire(
        &pages
            .screen(&data, &viewer, &ask(ScreenName::Home, None, "session-a"))
            .unwrap(),
    );
    assert_eq!(rows_of(&home), INITIAL_ROWS);
    let mut all = rows_of(&home);
    let mut cursor = home["cursor"].as_str().map(str::to_owned);
    assert!(cursor.is_some(), "the rest of the rows are paged");
    for _ in 0..ROWS_CAP {
        let Some(text) = cursor.clone() else { break };
        let more = wire(
            &pages
                .more(
                    &data,
                    &viewer,
                    &text,
                    &ask(ScreenName::Home, None, "session-a"),
                )
                .unwrap(),
        );
        let got = more["sections"].as_array().map_or(0, Vec::len);
        assert!(got > 0 && got <= MIN_ROWS_FLOOR, "{got} rows in one page");
        all += got;
        cursor = more["cursor"].as_str().map(str::to_owned);
    }
    assert!(cursor.is_none(), "the paging ends");
    assert!(
        all > INITIAL_ROWS && all <= ROWS_CAP,
        "{all} rows in all, cap {ROWS_CAP}"
    );
    // a genre page asks for fewer rows, and its own cap comes from the titles it can fill
    let spec = resolve("Dramas", "movie", &data.events);
    let own = page_cap(&data, "movie", Some(&spec), None, Tier::Adult, false);
    let plan = PagePlan::new(own, true, false, INITIAL_ROWS_GENRE);
    assert_eq!(own, 1, "the sample's film dramas fill one row of their own");
    assert_eq!(plan.cap, THIN_PAGE_FLOOR, "a thin page is topped up");
    assert!(!plan.supported(), "and is not offered in a menu");
    assert_eq!(plan.target_rows, INITIAL_ROWS_GENRE);
    let genre = wire(
        &pages
            .screen(
                &data,
                &viewer,
                &ask(ScreenName::Genre, Some("film.dramas"), "session-a"),
            )
            .unwrap(),
    );
    assert_eq!(rows_of(&genre), INITIAL_ROWS_GENRE, "the plan is honoured");
}

#[test]
fn s79_the_hero_leads_and_every_row_names_its_archetype_and_place() {
    let data = set("kinds", false);
    let pages = Pages::new(Images::new(None));
    let page = pages
        .screen(
            &data,
            &Viewer::visitor(Tier::Adult),
            &ask(ScreenName::Home, None, "session-a"),
        )
        .unwrap();
    let all = page.wire().sections.clone();
    assert_eq!(
        all.first().map(|section| section.kind),
        Some(SectionKind::Hero)
    );
    assert!(
        all.iter()
            .skip(1)
            .all(|section| section.kind != SectionKind::Hero),
        "one hero"
    );
    for (place, section) in all.iter().skip(1).enumerate() {
        let (archetype, at) = section
            .id
            .rsplit_once('-')
            .expect("a row id is its archetype and its place");
        assert_eq!(
            at.parse::<usize>().expect("a place"),
            place,
            "{} is out of page order",
            section.id
        );
        let kind = match archetype {
            "trending" => SectionKind::Ranked,
            "continue" => SectionKind::Continue,
            _ => SectionKind::Row,
        };
        assert_eq!(section.kind, kind, "{}", section.id);
    }
    assert!(
        all.iter()
            .any(|section| section.kind == SectionKind::Ranked),
        "the page has a chart row, so the ranked kind is tested"
    );
}
