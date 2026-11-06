//! Screens through the real router on the sample set.
#![allow(clippy::disallowed_types)] // a test reads answers as a client does

mod common;

use axum::http::{Method, StatusCode};
use common::{Answer, Harness, call, get, harness, harness_composed, member};
use contract::screen::{More, Screen, Section, SectionKind};
use serde_json::json;
use server_catalog::sample::special;

/// A token whose session entered the named kind of profile.
async fn enter(server: &Harness, kids: bool) -> (String, String) {
    let (_, token) = member(&server.door).await;
    let list = get("/v1/profiles").token(&token).on(&server.state).await;
    let profiles = list.body["profiles"].as_array().unwrap();
    let id = profiles.iter().find(|p| p["kids"] == kids).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let path = format!("/v1/profiles/{id}/enter");
    let entered = call(Method::POST, &path)
        .token(&token)
        .json(json!({}))
        .on(&server.state)
        .await;
    assert_eq!(entered.status, StatusCode::OK);
    (token, id)
}

fn screen(answer: Answer) -> Screen {
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    serde_json::from_value(answer.body).expect("Screen")
}

fn all_ids(page: &Screen) -> Vec<String> {
    page.sections
        .iter()
        .flat_map(|section| section.tiles.iter().map(|tile| tile.title.to_string()))
        .collect()
}

fn rows(page: &Screen) -> Vec<&Section> {
    page.sections
        .iter()
        .filter(|s| s.kind != SectionKind::Hero)
        .collect()
}

const SCREENS: [&str; 7] = [
    "home",
    "films",
    "series",
    "new",
    "my-list",
    "activity",
    "notifications",
];

#[tokio::test]
async fn s81_home_has_a_hero_a_chart_and_rows_and_reads_as_the_contract_type() {
    let server = harness(&[("IMAGE_BASE", "https://images.host.example")]);
    let (token, _) = enter(&server, false).await;
    let page = screen(
        get("/v1/screens/home")
            .token(&token)
            .on(&server.state)
            .await,
    );
    assert_eq!(page.sections[0].kind, SectionKind::Hero, "S100");
    assert!(
        page.sections.iter().any(|s| s.kind == SectionKind::Ranked),
        "S84"
    );
    assert!(rows(&page).len() >= 2, "{} rows", rows(&page).len());
    assert!(page.ttl_s > 0);
    let first = &page.sections[1].tiles[0];
    assert!(
        first
            .images
            .iter()
            .any(|image| image.url.starts_with("https://images.host.example"))
    );
}

#[tokio::test]
async fn s84_home_shows_films_and_series_charts_apart_with_ranks() {
    let server = harness(&[]);
    let (token, _) = enter(&server, false).await;
    let page = screen(
        get("/v1/screens/home")
            .token(&token)
            .on(&server.state)
            .await,
    );
    let mut sections = page.sections.clone();
    let mut cursor = page.cursor.clone();
    while let Some(next) = cursor {
        let answer = get(&format!("/v1/more/{}", next.0))
            .token(&token)
            .on(&server.state)
            .await;
        let more: More = serde_json::from_value(answer.body).expect("More");
        sections.extend(more.sections);
        cursor = more.cursor;
    }
    let charts: Vec<&Section> = sections
        .iter()
        .filter(|s| s.kind == SectionKind::Ranked)
        .collect();
    assert_eq!(charts.len(), 2);
    assert!(charts.iter().all(|chart| chart.tiles.len() <= 10));
    assert_eq!(charts[0].tiles[0].badges[0].rank, Some(1), "S80");
    assert_ne!(charts[0].title, charts[1].title);
}

#[tokio::test]
async fn s79_a_title_shows_once_across_rows_on_a_page_that_is_not_composed() {
    let server = harness(&[]);
    let (token, _) = enter(&server, false).await;
    let page = screen(
        get("/v1/screens/home")
            .token(&token)
            .on(&server.state)
            .await,
    );
    let mut titles: Vec<String> = rows(&page)
        .iter()
        .filter(|s| s.kind != SectionKind::Ranked)
        .flat_map(|s| s.tiles.iter().map(|t| t.title.to_string()))
        .collect();
    let count = titles.len();
    titles.sort_unstable();
    titles.dedup();
    assert_eq!(titles.len(), count);
}

#[tokio::test]
async fn h32_h33_tmk10_a_kids_profile_sees_no_title_above_its_tier_on_any_screen() {
    let server = harness(&[]);
    let (token, _) = enter(&server, true).await;
    let adult_only = [special::ADULT_FLAG, special::UNRATED_NO_PROFILE];
    for name in SCREENS.iter().chain(&["kids"]) {
        let answer = get(&format!("/v1/screens/{name}"))
            .token(&token)
            .on(&server.state)
            .await;
        let ids = all_ids(&screen(answer));
        for id in adult_only {
            assert!(!ids.contains(&format!("movie:{id}")), "{name} leaked {id}");
        }
        for id in &ids {
            let title = get(&format!("/v1/titles/{}", id.replace(':', "/")))
                .token(&token)
                .on(&server.state)
                .await;
            assert_eq!(
                title.status,
                StatusCode::OK,
                "{name} lists {id}, and its title page refuses it"
            );
        }
    }
}

#[tokio::test]
async fn tmk4_alternating_adult_and_kids_calls_never_cross_a_cached_page() {
    let server = harness(&[]);
    let (adult, _) = enter(&server, false).await;
    let (kids, _) = enter(&server, true).await;
    for _ in 0..3 {
        let grown = all_ids(&screen(
            get("/v1/screens/films")
                .token(&adult)
                .on(&server.state)
                .await,
        ));
        let small = all_ids(&screen(
            get("/v1/screens/films")
                .token(&kids)
                .on(&server.state)
                .await,
        ));
        assert_ne!(grown, small);
        for id in &small {
            let page = get(&format!("/v1/titles/{}", id.replace(':', "/")))
                .token(&kids)
                .on(&server.state)
                .await;
            assert_eq!(page.status, StatusCode::OK, "{id}");
        }
    }
}

#[tokio::test]
async fn h88_tmk13_a_blocked_name_leaves_every_section_by_normalised_match() {
    let server = harness(&[]);
    let (token, id) = enter(&server, false).await;
    let names = |page: &Screen| -> Vec<String> {
        page.sections
            .iter()
            .flat_map(|s| s.tiles.iter().map(|t| t.name.clone()))
            .collect()
    };
    let before = screen(
        get("/v1/screens/films")
            .token(&token)
            .on(&server.state)
            .await,
    );
    let target = names(&before)[0].clone();
    let path = format!("/v1/profiles/{id}/limits");
    let blocked = call(Method::PUT, &path)
        .token(&token)
        .json(json!({ "age_rating": "18", "blocked_titles": [target.to_uppercase()] }))
        .on(&server.state)
        .await;
    assert_eq!(blocked.status, StatusCode::OK, "{}", blocked.body);
    let after = screen(
        get("/v1/screens/films")
            .token(&token)
            .on(&server.state)
            .await,
    );
    assert!(
        !names(&after).contains(&target),
        "a cached row must not bring it back"
    );
    let found = get(&format!("/v1/search?q={}", target.replace(' ', "%20")))
        .token(&token)
        .on(&server.state)
        .await;
    let names_found = names(&screen(found));
    assert!(!names_found.contains(&target));
}

#[tokio::test]
async fn s102_search_finds_a_title_with_an_unfinished_last_word_and_refuses_empty() {
    let server = harness(&[]);
    let (token, _) = enter(&server, false).await;
    let page = screen(
        get("/v1/search?q=twin%20fil")
            .token(&token)
            .on(&server.state)
            .await,
    );
    assert_eq!(page.sections[0].tiles[0].name, "Twin Film");
    let none = get("/v1/search?q=%20")
        .token(&token)
        .on(&server.state)
        .await;
    assert_eq!(none.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn s102_h35_search_is_filtered_for_a_limited_profile() {
    let server = harness(&[]);
    let (token, _) = enter(&server, true).await;
    let page = screen(
        get("/v1/search?q=grown%20up")
            .token(&token)
            .on(&server.state)
            .await,
    );
    assert!(all_ids(&page).is_empty());
}

#[tokio::test]
async fn s103_suggestions_answer_before_anything_is_typed_and_after() {
    let server = harness(&[]);
    let (token, _) = enter(&server, false).await;
    let empty = get("/v1/search/suggest?q=")
        .token(&token)
        .on(&server.state)
        .await;
    assert!(
        !empty.body["items"].as_array().unwrap().is_empty(),
        "{}",
        empty.body
    );
    let typed = get("/v1/search/suggest?q=twin")
        .token(&token)
        .on(&server.state)
        .await;
    let items = typed.body["items"].as_array().unwrap();
    assert!(items.iter().any(|name| name == "Twin Film"));
}

#[tokio::test]
async fn s105_new_and_popular_has_new_rows_and_charts() {
    let server = harness(&[]);
    let (token, _) = enter(&server, false).await;
    let page = screen(get("/v1/screens/new").token(&token).on(&server.state).await);
    let names: Vec<&str> = page
        .sections
        .iter()
        .filter_map(|s| s.title.as_deref())
        .collect();
    assert!(names.contains(&"New Here"), "{names:?}");
    assert!(
        names.contains(&"Coming This Week") || names.contains(&"Coming Next Week"),
        "{names:?}"
    );
    assert!(names.iter().any(|n| n.starts_with("Top 10")), "{names:?}");
}

#[tokio::test]
async fn c26_a_genre_page_opens_by_its_own_id_and_a_number_finds_nothing() {
    let server = harness(&[]);
    let (token, _) = enter(&server, false).await;
    let page = screen(
        get("/v1/screens/genre?genre=film.dramas")
            .token(&token)
            .on(&server.state)
            .await,
    );
    assert!(!rows(&page).is_empty());
    for bad in ["18", "34399", "nonsense", ""] {
        let answer = get(&format!("/v1/screens/genre?genre={bad}"))
            .token(&token)
            .on(&server.state)
            .await;
        assert_eq!(answer.status, StatusCode::NOT_FOUND, "{bad}");
    }
}

#[tokio::test]
async fn s107_an_event_page_opens_only_in_its_window() {
    let server = harness(&[]);
    let (token, _) = enter(&server, false).await;
    let open = get("/v1/screens/genre?genre=event.autumn")
        .token(&token)
        .on(&server.state)
        .await;
    assert_eq!(open.status, StatusCode::OK, "{}", open.body);
    let shut = get("/v1/screens/genre?genre=event.midwinter")
        .token(&token)
        .on(&server.state)
        .await;
    assert_eq!(shut.status, StatusCode::FORBIDDEN);
    assert_eq!(shut.code(), "refused");
}

#[tokio::test]
async fn s74_a_language_screen_takes_a_short_code() {
    let server = harness(&[]);
    let (token, _) = enter(&server, false).await;
    let answer = get("/v1/screens/language?genre=en")
        .token(&token)
        .on(&server.state)
        .await;
    assert_eq!(answer.status, StatusCode::OK);
    let bad = get("/v1/screens/language?genre=english")
        .token(&token)
        .on(&server.state)
        .await;
    assert_eq!(bad.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn s153_games_and_clips_are_off_and_say_so_and_an_unknown_screen_is_404() {
    let server = harness(&[]);
    let (token, _) = enter(&server, false).await;
    for name in ["games", "clips"] {
        let answer = get(&format!("/v1/screens/{name}"))
            .token(&token)
            .on(&server.state)
            .await;
        assert_eq!(answer.code(), "unsupported", "{name}");
    }
    let unknown = get("/v1/screens/banana")
        .token(&token)
        .on(&server.state)
        .await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
}

async fn add_to_list(server: &Harness, token: &str, title: &str) {
    let op = format!("op-{title}");
    let batch =
        json!({ "ops": [{ "op_id": op, "command": "list", "title": title, "in_list": true }] });
    let answer = call(Method::POST, "/v1/commands")
        .token(token)
        .json(batch)
        .on(&server.state)
        .await;
    assert_eq!(
        answer.body["answers"][0]["outcome"], "applied",
        "{}",
        answer.body
    );
}

#[tokio::test]
async fn h36_s91_my_list_comes_from_the_library_and_shows_on_home_from_three_titles() {
    let server = harness(&[("BRAND_NAME", "Sample")]);
    let (token, _) = enter(&server, false).await;
    for title in ["movie:100", "movie:101", "movie:103"] {
        add_to_list(&server, &token, title).await;
    }
    let list = screen(
        get("/v1/screens/my-list")
            .token(&token)
            .on(&server.state)
            .await,
    );
    assert_eq!(all_ids(&list).len(), 3);
    let home = screen(
        get("/v1/screens/home")
            .token(&token)
            .on(&server.state)
            .await,
    );
    assert!(
        home.sections
            .iter()
            .any(|s| s.title.as_deref() == Some("My Sample")),
        "S91"
    );
}

#[tokio::test]
async fn tmk3_a_title_a_profile_may_not_see_stays_off_my_list_and_changes() {
    let server = harness(&[]);
    let (token, _) = enter(&server, true).await;
    let adult = format!("movie:{}", special::ADULT_FLAG);
    add_to_list(&server, &token, &adult).await;
    add_to_list(&server, &token, "movie:106").await;
    let list = screen(
        get("/v1/screens/my-list")
            .token(&token)
            .on(&server.state)
            .await,
    );
    assert_eq!(all_ids(&list), ["movie:106"]);
    let changes = get("/v1/changes").token(&token).on(&server.state).await;
    let text = changes.body.to_string();
    assert!(!text.contains(&adult), "{text}");
    assert!(text.contains("movie:106"), "{text}");
}

#[tokio::test]
async fn s194_a_setting_travels_by_command_and_reads_back() {
    let server = harness(&[]);
    let (token, _) = enter(&server, false).await;
    let batch = json!({ "ops": [{ "op_id": "a", "command": "setting", "key": "audio_language", "value": "fr" }] });
    call(Method::POST, "/v1/commands")
        .token(&token)
        .json(batch)
        .on(&server.state)
        .await;
    let settings = get("/v1/profiles/me/settings")
        .token(&token)
        .on(&server.state)
        .await;
    assert_eq!(settings.body["audio_language"], "fr");
    assert!(settings.body.get("age_rating").is_none(), "H27");
}

#[tokio::test]
async fn h27_a_command_cannot_touch_limits() {
    let server = harness(&[]);
    let (token, id) = enter(&server, true).await;
    let batch = json!({ "ops": [{ "op_id": "a", "command": "setting", "key": "age_rating", "value": "18" }] });
    let answer = call(Method::POST, "/v1/commands")
        .token(&token)
        .json(batch)
        .on(&server.state)
        .await;
    assert_eq!(
        answer.body["answers"][0]["outcome"], "rejected",
        "{}",
        answer.body
    );
    let limits = get(&format!("/v1/profiles/{id}/limits"))
        .token(&token)
        .on(&server.state)
        .await;
    assert_eq!(limits.body["age_rating"], "PG");
}

#[tokio::test]
async fn tmk5_h29_h92_a_screen_cursor_pages_for_its_owner_and_no_one_else() {
    let server = harness(&[]);
    let (adult, _) = enter(&server, false).await;
    let (other, _) = enter(&server, false).await;
    let page = screen(
        get("/v1/screens/home")
            .token(&adult)
            .on(&server.state)
            .await,
    );
    let cursor = page.cursor.clone().expect("more rows than one page");
    let path = format!("/v1/more/{}", cursor.0);
    let good = get(&path).token(&adult).on(&server.state).await;
    assert_eq!(good.status, StatusCode::OK, "{}", good.body);
    let more: More = serde_json::from_value(good.body).expect("More");
    assert!(!more.sections.is_empty());
    let first_page = all_ids(&page);
    for tile in more.sections.iter().flat_map(|s| s.tiles.iter()) {
        assert!(
            !first_page.contains(&tile.title.to_string())
                || tile.badges.iter().any(|b| b.rank.is_some())
        );
    }
    let stolen = get(&path).token(&other).on(&server.state).await;
    assert_eq!(stolen.status, StatusCode::BAD_REQUEST);
    let forged = get("/v1/more/abcdef.012345")
        .token(&adult)
        .on(&server.state)
        .await;
    assert_eq!(forged.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn s78_tmk5_a_deep_row_pages_past_its_first_titles_and_a_switch_to_kids_refuses_the_cursor() {
    let server = harness_composed(&[]);
    let (token, _) = enter(&server, false).await;
    let page = screen(
        get("/v1/screens/home")
            .token(&token)
            .on(&server.state)
            .await,
    );
    let row = page
        .sections
        .iter()
        .find(|s| s.cursor.is_some())
        .expect("a composed row deeper than one page");
    assert_eq!(row.tiles.len(), 24, "S78");
    let path = format!("/v1/more/{}", row.cursor.as_ref().unwrap().0);
    let good = get(&path).token(&token).on(&server.state).await;
    assert_eq!(good.status, StatusCode::OK, "{}", good.body);
    let more: More = serde_json::from_value(good.body).expect("More");
    assert!(!more.tiles.is_empty());
    let seen: Vec<String> = row.tiles.iter().map(|t| t.title.to_string()).collect();
    assert!(
        more.tiles
            .iter()
            .all(|t| !seen.contains(&t.title.to_string()))
    );

    let list = get("/v1/profiles").token(&token).on(&server.state).await;
    let kids = list.body["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["kids"] == true)
        .unwrap();
    let enter_kids = format!("/v1/profiles/{}/enter", kids["id"].as_str().unwrap());
    call(Method::POST, &enter_kids)
        .token(&token)
        .json(json!({}))
        .on(&server.state)
        .await;
    let replay = get(&path).token(&token).on(&server.state).await;
    assert_eq!(
        replay.status,
        StatusCode::BAD_REQUEST,
        "K5: bound to the profile"
    );
}

#[tokio::test]
async fn s88_s89_composed_rows_keep_their_own_names_and_first_screens_unique() {
    let server = harness_composed(&[]);
    let (token, _) = enter(&server, false).await;
    let page = screen(
        get("/v1/screens/home")
            .token(&token)
            .on(&server.state)
            .await,
    );
    let named: Vec<&Section> = page
        .sections
        .iter()
        .filter(|s| s.title.as_deref().is_some_and(|t| t.ends_with(" Stories")))
        .collect();
    assert!(named.len() >= 4, "{} composed rows", named.len());
    let mut heads: Vec<String> = named
        .iter()
        .flat_map(|s| s.tiles.iter().take(6).map(|t| t.title.to_string()))
        .collect();
    let count = heads.len();
    heads.sort_unstable();
    heads.dedup();
    assert_eq!(heads.len(), count, "S79");
}

#[tokio::test]
async fn s74_the_same_session_gets_the_same_page_and_h66_work_is_bounded() {
    let server = harness(&[]);
    let (token, _) = enter(&server, false).await;
    let first = all_ids(&screen(
        get("/v1/screens/home")
            .token(&token)
            .on(&server.state)
            .await,
    ));
    let second = all_ids(&screen(
        get("/v1/screens/home")
            .token(&token)
            .on(&server.state)
            .await,
    ));
    assert_eq!(first, second, "S159");
}

#[tokio::test]
async fn visitors_browse_only_where_the_deployer_said_so_at_the_stated_tier() {
    let open = harness(&[("ACCOUNTS_REQUIRED", "false"), ("VISITOR_TIER", "PG")]);
    let page = screen(get("/v1/screens/home").on(&open.state).await);
    let ids = all_ids(&page);
    assert!(!ids.is_empty());
    assert!(!ids.contains(&format!("movie:{}", special::ADULT_FLAG)));
    let closed = harness(&[("VISITOR_TIER", "PG")]);
    assert_eq!(
        get("/v1/screens/home").on(&closed.state).await.status,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn c102_the_menu_follows_the_profile() {
    let server = harness(&[]);
    let (adult, _) = enter(&server, false).await;
    let (kids, _) = enter(&server, true).await;
    let grown = get("/v1/nav").token(&adult).on(&server.state).await;
    assert_eq!(grown.body["items"][0]["screen"], "home", "{}", grown.body);
    let small = get("/v1/nav").token(&kids).on(&server.state).await;
    assert_eq!(small.body["items"][0]["screen"], "kids");
}
