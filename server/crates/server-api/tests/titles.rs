//! Title routes through the real router, on the sample set read through its manifest.
#![allow(clippy::disallowed_types)] // a test reads answers as a client does

mod common;

use axum::http::{Method, StatusCode};
use common::{call, get, harness, harness_no_data, member};
use contract::screen::TitleDetail;
use serde_json::json;
use server_catalog::sample::special;

/// A token whose session has entered the named kind of profile.
async fn watching(server: &common::Harness, kids: bool) -> String {
    let (_, token) = member(&server.door).await;
    let profiles = get("/v1/profiles").token(&token).on(&server.state).await;
    let list = profiles.body["profiles"].as_array().unwrap();
    let chosen = list.iter().find(|p| p["kids"] == kids).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let path = format!("/v1/profiles/{chosen}/enter");
    let entered = call(Method::POST, &path)
        .token(&token)
        .json(json!({}))
        .on(&server.state)
        .await;
    assert_eq!(entered.status, StatusCode::OK);
    assert_eq!(entered.body["home"], if kids { "kids" } else { "home" });
    token
}

#[tokio::test]
async fn h30_tmk1_a_session_with_no_profile_reads_no_title() {
    let server = harness(&[]);
    let (_, token) = member(&server.door).await;
    let answer = get("/v1/titles/movie/100")
        .token(&token)
        .on(&server.state)
        .await;
    assert_eq!(answer.status, StatusCode::FORBIDDEN);
    assert_eq!(answer.code(), "profile_required");
    let none = get("/v1/titles/movie/100").on(&server.state).await;
    assert_eq!(none.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn s108_h131_a_title_reads_as_the_contract_type_with_facts_from_the_catalogue() {
    let server = harness(&[("IMAGE_BASE", "https://images.host.example/t/")]);
    let token = watching(&server, false).await;
    let answer = get("/v1/titles/movie/100")
        .token(&token)
        .on(&server.state)
        .await;
    assert_eq!(answer.status, StatusCode::OK);
    let page: TitleDetail = serde_json::from_value(answer.body).expect("TitleDetail");
    assert_eq!(page.title.to_string(), "movie:100");
    assert_eq!(page.images.len(), 2);
    assert!(
        page.images[0]
            .url
            .starts_with("https://images.host.example/t/")
    );
    assert!(!page.more_like_this.is_empty(), "S104");
    assert!(
        page.more_like_this
            .iter()
            .all(|tile| tile.title != page.title)
    );
}

#[tokio::test]
async fn h57_a_film_and_a_series_with_one_id_are_two_titles() {
    let server = harness(&[]);
    let token = watching(&server, false).await;
    let id = special::SHARED_ID;
    let film = get(&format!("/v1/titles/movie/{id}"))
        .token(&token)
        .on(&server.state)
        .await;
    let series = get(&format!("/v1/titles/tv/{id}"))
        .token(&token)
        .on(&server.state)
        .await;
    assert_eq!(film.body["name"], "Twin Film");
    assert_eq!(series.body["name"], "Twin Series");
}

#[tokio::test]
async fn h35_tmk10_a_kids_session_is_refused_a_title_its_tier_cannot_see() {
    let server = harness(&[]);
    let token = watching(&server, true).await;
    let path = format!("/v1/titles/movie/{}", special::ADULT_FLAG);
    let refused = get(&path).token(&token).on(&server.state).await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
    assert_eq!(refused.code(), "refused");
    assert!(
        refused.body.get("name").is_none(),
        "H34: a refusal carries no title fact"
    );
    let adult = watching(&server, false).await;
    assert_eq!(
        get(&path).token(&adult).on(&server.state).await.status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn h60_changing_profile_changes_the_answer_on_the_same_token() {
    let server = harness(&[]);
    let token = watching(&server, false).await;
    let path = format!("/v1/titles/movie/{}", special::ADULT_FLAG);
    assert_eq!(
        get(&path).token(&token).on(&server.state).await.status,
        StatusCode::OK
    );
    let profiles = get("/v1/profiles").token(&token).on(&server.state).await;
    let kids = profiles.body["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["kids"] == true)
        .unwrap();
    let enter = format!("/v1/profiles/{}/enter", kids["id"].as_str().unwrap());
    call(Method::POST, &enter)
        .token(&token)
        .json(json!({}))
        .on(&server.state)
        .await;
    assert_eq!(
        get(&path).token(&token).on(&server.state).await.status,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn h28_a_client_header_cannot_name_a_profile() {
    let server = harness(&[]);
    let token = watching(&server, true).await;
    let path = format!("/v1/titles/movie/{}", special::ADULT_FLAG);
    let answer = get(&path)
        .token(&token)
        .header("x-profile-id", "00000000-0000-0000-0000-000000000000")
        .header("x-maturity", "adult")
        .on(&server.state)
        .await;
    assert_eq!(answer.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn h57_a_bare_id_or_an_unknown_kind_has_no_route() {
    let server = harness(&[]);
    let token = watching(&server, false).await;
    for path in [
        "/v1/titles/100",
        "/v1/titles/film/100",
        "/v1/titles/movie/0",
        "/v1/titles/movie/abc",
    ] {
        let answer = get(path).token(&token).on(&server.state).await;
        assert_eq!(answer.status, StatusCode::NOT_FOUND, "{path}");
    }
}

#[tokio::test]
async fn a_title_outside_the_catalogue_answers_no_longer_available() {
    let server = harness(&[]);
    let token = watching(&server, false).await;
    let answer = get("/v1/titles/movie/999999")
        .token(&token)
        .on(&server.state)
        .await;
    assert_eq!(answer.status, StatusCode::NOT_FOUND);
    assert_eq!(answer.code(), "no_longer_available");
}

#[tokio::test]
async fn tmt18_a_title_is_private_and_revalidated() {
    let server = harness(&[]);
    let token = watching(&server, false).await;
    let first = get("/v1/titles/movie/100")
        .token(&token)
        .on(&server.state)
        .await;
    assert_eq!(first.headers["cache-control"], "private, no-cache");
    let tag = first.headers["etag"].to_str().unwrap().to_owned();
    let again = get("/v1/titles/movie/100")
        .token(&token)
        .header("if-none-match", &tag)
        .on(&server.state)
        .await;
    assert_eq!(again.status, StatusCode::NOT_MODIFIED);
}

#[tokio::test]
async fn tmk1_the_public_title_is_off_without_a_stated_tier_and_filtered_with_one() {
    let closed = harness(&[]);
    let off = get("/v1/public/titles/movie/100").on(&closed.state).await;
    assert_eq!(off.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(off.code(), "unsupported");

    let open = harness(&[("VISITOR_TIER", "PG")]);
    let family = get("/v1/public/titles/movie/106").on(&open.state).await;
    assert_eq!(
        family.status,
        StatusCode::OK,
        "a U title under a PG tier: {}",
        family.body
    );
    let adult = format!("/v1/public/titles/movie/{}", special::ADULT_FLAG);
    assert_eq!(
        get(&adult).on(&open.state).await.status,
        StatusCode::FORBIDDEN
    );
    assert!(
        family.headers["cache-control"]
            .to_str()
            .unwrap()
            .starts_with("public")
    );
}

#[tokio::test]
async fn s213_with_no_data_set_titles_say_they_are_off() {
    let server = harness_no_data(&[]);
    let token = watching(&server, false).await;
    let answer = get("/v1/titles/movie/100")
        .token(&token)
        .on(&server.state)
        .await;
    assert_eq!(answer.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(answer.code(), "unsupported");
    assert!(
        answer.body["notice"]["text"]
            .as_str()
            .unwrap()
            .contains("titles")
    );
}

#[tokio::test]
async fn a_series_names_its_seasons_and_a_season_lists_its_episodes() {
    let server = harness(&[]);
    let token = watching(&server, false).await;
    let id = special::SHARED_ID;
    let series = get(&format!("/v1/titles/tv/{id}"))
        .token(&token)
        .on(&server.state)
        .await;
    let seasons = series.body["seasons"].as_array().unwrap();
    assert_eq!(
        seasons.len(),
        1,
        "specials are not on the menu: {seasons:?}"
    );
    assert_eq!(seasons[0]["number"], 1);
    assert_eq!(seasons[0]["episode_count"], 2);

    let season = get(&format!("/v1/titles/tv/{id}/seasons/1"))
        .token(&token)
        .on(&server.state)
        .await;
    assert_eq!(season.status, StatusCode::OK, "{}", season.body);
    let episodes = season.body["episodes"].as_array().unwrap();
    assert_eq!(episodes.len(), 2);
    assert_eq!(episodes[0]["name"], "First Episode");
    assert_eq!(episodes[1]["runtime_min"], 44);

    let film = get(&format!("/v1/titles/movie/{id}/seasons/1"))
        .token(&token)
        .on(&server.state)
        .await;
    assert_eq!(film.status, StatusCode::NOT_FOUND, "a film has no seasons");
}

/// Saves how far this profile got in an episode.
async fn watched(server: &common::Harness, token: &str, op: &str, episode: u32, position_s: u32) {
    let title = format!("tv:{}", special::SHARED_ID);
    let ops = json!({ "ops": [{
        "op_id": op,
        "command": "position",
        "title": title,
        "episode": { "season": 1, "episode": episode },
        "position_s": position_s,
        "duration_s": 3_000,
    }] });
    let sent = call(Method::POST, "/v1/commands")
        .token(token)
        .json(ops)
        .on(&server.state)
        .await;
    assert_eq!(sent.status, StatusCode::OK, "{}", sent.body);
}

#[tokio::test]
async fn a_title_says_where_this_profile_picks_up() {
    let server = harness(&[]);
    let token = watching(&server, false).await;
    let path = format!("/v1/titles/tv/{}", special::SHARED_ID);
    let read = || get(&path).token(&token);
    let fresh = read().on(&server.state).await;
    assert!(fresh.body.get("resume").is_none(), "{}", fresh.body);

    watched(&server, &token, "a", 1, 600).await;
    let halfway = read().on(&server.state).await;
    assert_eq!(
        halfway.body["resume"],
        json!({ "episode": { "season": 1, "episode": 1 }, "position_s": 600 })
    );

    watched(&server, &token, "b", 1, 2_990).await;
    let next = read().on(&server.state).await;
    assert_eq!(
        next.body["resume"],
        json!({ "episode": { "season": 1, "episode": 2 }, "position_s": 0 }),
        "a finished episode leads to the next one"
    );

    watched(&server, &token, "c", 2, 2_990).await;
    let done = read().on(&server.state).await;
    assert!(
        done.body.get("resume").is_none(),
        "nothing follows the last episode: {}",
        done.body
    );
}
