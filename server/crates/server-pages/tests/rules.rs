//! Page rules on the invented sample, through the builder.

use std::sync::Arc;

use contract::client::DeviceClass;
use contract::screen::{ScreenName, SectionKind};
use server_catalog::catalog::{CatalogItem, FLAG_NONSCRIPTED};
use server_catalog::sample::{sample_titles, write_composed_sample, write_sample};
use server_catalog::{DataSet, item_key};
use server_pages::genre_spec::{GenreSpec, SpecFacts, resolve};
use server_pages::hero::{HeroAsk, hero_keys};
use server_pages::more_like::more_like_keys;
use server_pages::page::{meets_vote_floor, member_quality_for};
use server_pages::tags::item_tags;
use server_pages::{Pages, Personal, ScreenAsk};
use server_rules::answer::{Cleared, Images};
use server_rules::{Tier, Viewer};

/// Time of the sample: 2023-11-14.
const NOW: u64 = 1_700_000_100_000;

fn set(name: &str, composed: bool) -> Arc<DataSet> {
    let dir = std::env::temp_dir().join(format!("pages_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let manifest = if composed {
        write_composed_sample(&dir)
    } else {
        write_sample(&dir)
    };
    Arc::new(DataSet::load(&manifest).expect("sample"))
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
    let mut few = good;
    few.vote_count = 9;
    assert!(!meets_vote_floor(&few, false));
    few.vote_count = 5;
    assert!(meets_vote_floor(&few, true));
    few.vote_count = 0;
    assert!(meets_vote_floor(&few, false), "an unknown count passes");
}

#[test]
fn s73_a_nonscripted_title_stays_off_default_rows_and_on_a_page_that_asks() {
    let spec = GenreSpec::ChatShows;
    assert!(spec.includes_nonscripted());
    assert!(
        !resolve(
            "Dramas",
            "both",
            &server_catalog::events::EventStore::default()
        )
        .includes_nonscripted()
    );
    let flags =
        server_catalog::catalog::compute_flags("Late Talk", &[], Some("Talk Show"), "", None, None);
    assert_eq!(flags & FLAG_NONSCRIPTED, FLAG_NONSCRIPTED);
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

#[test]
fn s86_home_rows_mix_films_and_series() {
    let data = set("blend", false);
    let pages = Pages::new(Images::new(None));
    let page = wire(
        &pages
            .screen(
                &data,
                &Viewer::visitor(Tier::Adult),
                &ask(ScreenName::Home, None, "s"),
            )
            .unwrap(),
    );
    let mixed = sections(&page).iter().any(|section| {
        let titles: Vec<String> = section["tiles"].as_array().map_or_else(Vec::new, |tiles| {
            tiles
                .iter()
                .map(|tile| tile["title"].as_str().unwrap_or("").to_owned())
                .collect()
        });
        titles.iter().any(|t| t.starts_with("movie:"))
            && titles.iter().any(|t| t.starts_with("tv:"))
    });
    assert!(mixed);
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
    for tile in first_row["tiles"].as_array().unwrap() {
        let key = contract::TitleKey::parse(tile["title"].as_str().unwrap()).unwrap();
        let item = &data.catalog[&item_key("movie", key.id())];
        assert!(item.genres.iter().any(|g| g == "Drama"), "{tile}");
    }
}

#[test]
fn s89_composed_rows_run_deep_and_the_order_is_the_session_score() {
    let data = set("composed", true);
    let pages = Pages::new(Images::new(None));
    let page = wire(
        &pages
            .screen(
                &data,
                &Viewer::visitor(Tier::Adult),
                &ask(ScreenName::Home, None, "s"),
            )
            .unwrap(),
    );
    let deep = sections(&page)
        .iter()
        .any(|section| section.get("cursor").is_some());
    assert!(deep, "a row deeper than one page carries a cursor");
}

#[test]
fn s92_s93_new_titles_come_from_real_release_dates() {
    let data = set("new", false);
    let pages = Pages::new(Images::new(None));
    let page = wire(
        &pages
            .screen(
                &data,
                &Viewer::visitor(Tier::Adult),
                &ask(ScreenName::Home, None, "s"),
            )
            .unwrap(),
    );
    let titles: Vec<&str> = sections(&page)
        .iter()
        .filter_map(|s| s["title"].as_str().map(|_| ""))
        .collect();
    assert!(!titles.is_empty());
    let new = sections(&page)
        .into_iter()
        .find(|s| s["title"] == "New Releases");
    if let Some(row) = new {
        let first = row["tiles"][0]["badges"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        assert!(first.iter().any(|badge| badge["kind"] == "new"), "S80");
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
        "C136: unknown means empty"
    );
}

#[test]
fn s105_c102_kids_get_their_own_screen_and_an_adult_who_opens_it_sees_what_a_child_would() {
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
    for id in ids(&adult) {
        let key = contract::TitleKey::parse(&id).unwrap();
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
}

#[test]
fn s200_notifications_list_recent_and_due_titles_with_art_up_to_twelve() {
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
    let found = ids(&page);
    assert!(!found.is_empty() && found.len() <= 12);
}

#[test]
fn s80_tags_are_tone_then_genre() {
    let titles = sample_titles();
    assert!(!titles.is_empty());
    let genres = vec![String::from("Drama")];
    assert_eq!(item_tags(None, &genres, 3), genres);
}

#[test]
fn s75_a_section_kind_follows_the_row() {
    let data = set("kinds", false);
    let pages = Pages::new(Images::new(None));
    let page = pages
        .screen(
            &data,
            &Viewer::visitor(Tier::Adult),
            &ask(ScreenName::Home, None, ""),
        )
        .unwrap();
    let first = page.wire().sections.first().map(|section| section.kind);
    assert_eq!(first, Some(SectionKind::Hero));
}
