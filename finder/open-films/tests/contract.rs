//! The open film finder against the published contract. Requests stay in process.

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use contract::finder::{FindAnswer, FindOutcome, FinderHealth};
use contract::hello::ServiceState;
use finder_open_films::{ListError, OpenFilms, parse_films};
use finder_service::{Finder, Provider, Settings, router};
use http_body_util::BodyExt;
use tower::ServiceExt;

const FILMS: &str = r#"[{"title":"movie:7","url":"https://media.example.invalid/open/master.m3u8",
    "kind":"hls","licence":"CC-BY-3.0","height":720,"audio":["en"]}]"#;

fn app(tokens: &[&str], films: Vec<Arc<dyn Provider>>) -> Router {
    let settings = Settings {
        build: "test".to_owned(),
        tokens: tokens.iter().map(|token| (*token).to_owned()).collect(),
        ..Settings::default()
    };
    router(Arc::new(Finder::new(settings, films)))
}

fn listed() -> Vec<Arc<dyn Provider>> {
    vec![Arc::new(OpenFilms(parse_films(FILMS).unwrap()))]
}

fn settings() -> Router {
    app(&["old", "new"], listed())
}

fn find(body: &str, token: Option<&str>) -> Request<Body> {
    let mut request = Request::post("/v1/find").header(header::CONTENT_TYPE, "application/json");
    if let Some(token) = token {
        request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    request.body(Body::from(body.to_owned())).unwrap()
}

async fn answer(app: Router, body: &str) -> FindAnswer {
    let response = app.oneshot(find(body, Some("new"))).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

const LISTED: &str = r#"{"title":"movie:7","name":"Open Film","budget_ms":1000}"#;

#[tokio::test]
async fn s6_listed_film_is_found_unverified_with_no_order_field() {
    let found = answer(settings(), LISTED).await;
    assert_eq!(found.outcome, FindOutcome::Found);
    assert_eq!(found.sources.len(), 1);
    assert_eq!(found.sources[0].height, Some(720));
    // The flag says the finder fetched and read the address. This one reads a file the operator
    // wrote and fetches nothing, so it may not claim it, whatever that costs a kids tier.
    assert!(!found.sources[0].verified);
}

#[tokio::test]
async fn h71_s171_unlisted_title_is_a_real_not_found() {
    let body = r#"{"title":"movie:8","name":"Other","budget_ms":1000}"#;
    let missing = answer(settings(), body).await;
    assert_eq!(missing.outcome, FindOutcome::NotFound);
    assert!(missing.sources.is_empty());
}

#[tokio::test]
async fn h57_kinds_stay_apart_and_a_bare_id_is_refused() {
    let tv = r#"{"title":"tv:7","name":"Open Film","budget_ms":1000}"#;
    assert_eq!(answer(settings(), tv).await.outcome, FindOutcome::NotFound);
    let bare = r#"{"title":"7","name":"Open Film","budget_ms":1000}"#;
    let response = settings().oneshot(find(bare, Some("new"))).await.unwrap();
    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn h78_a_wrong_name_does_not_change_the_answer() {
    let body = r#"{"title":"movie:7","name":"Anything","budget_ms":1000}"#;
    assert_eq!(answer(settings(), body).await.outcome, FindOutcome::Found);
}

#[tokio::test]
async fn episode_request_on_a_film_finds_nothing() {
    let body = r#"{"title":"movie:7","episode":{"season":1,"episode":1},"name":"x","budget_ms":1}"#;
    assert_eq!(
        answer(settings(), body).await.outcome,
        FindOutcome::NotFound
    );
}

#[tokio::test]
async fn both_rotation_tokens_work_and_others_do_not() {
    for (token, status) in [
        (Some("old"), StatusCode::OK),
        (Some("new"), StatusCode::OK),
        (Some("wrong"), StatusCode::UNAUTHORIZED),
        (None, StatusCode::UNAUTHORIZED),
    ] {
        let response = settings().oneshot(find(LISTED, token)).await.unwrap();
        assert_eq!(response.status(), status);
    }
}

#[tokio::test]
async fn h127_no_token_turns_finding_off_and_says_so() {
    let response = app(&[], listed())
        .oneshot(find(LISTED, Some("new")))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert!(String::from_utf8_lossy(&bytes).contains("off"));
}

#[tokio::test]
async fn health_is_degraded_with_no_films_and_names_no_source() {
    let response = app(&["new"], Vec::new())
        .oneshot(Request::get("/v1/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let health: FinderHealth = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(health.state, ServiceState::Degraded);
}

#[test]
fn list_refuses_no_licence_bad_scheme_and_embeds() {
    let entry = |url: &str, kind: &str, licence: &str| {
        format!(r#"[{{"title":"movie:7","url":"{url}","kind":"{kind}","licence":"{licence}"}}]"#)
    };
    let good = "https://media.example.invalid/a.mp4";
    assert!(matches!(
        parse_films(&entry(good, "file", " ")),
        Err(ListError::NoLicence(_))
    ));
    assert!(matches!(
        parse_films(&entry("file:///etc/passwd", "file", "CC0-1.0")),
        Err(ListError::BadUrl(_))
    ));
    assert!(matches!(
        parse_films(&entry(good, "embed", "CC0-1.0")),
        Err(ListError::EmbedRefused(_))
    ));
    assert!(parse_films(&entry(good, "file", "CC0-1.0")).is_ok());
}

#[tokio::test]
async fn a_film_that_stands_in_for_a_series_answers_an_episode() {
    let list = r#"[{"title":"tv:7","url":"https://media.example.invalid/open/a.m3u8",
        "kind":"hls","licence":"CC-BY-3.0","every_episode":true}]"#;
    let films = OpenFilms(parse_films(list).unwrap());
    let request = serde_json::from_str(
        r#"{"title":"tv:7","episode":{"season":1,"episode":1},"name":"x","budget_ms":1000}"#,
    )
    .unwrap();
    assert_eq!(films.find(&request).await.unwrap().len(), 1);
}

/// The published found answer. Its one source carries exactly the facts an operator writes in a
/// film list, so the list below is built from those bytes rather than from a shape invented here.
const FOUND: &str =
    include_str!("../../../platform/contract/fixtures/2026-11-05/valid/FindAnswer.found.json");

/// Two addresses for one film. A second address is all a deployer can do about the first going
/// down, so the finder has to answer with both and let the server rank them.
const MIRRORED: &str = r#"[
    {"title":"movie:7","url":"https://one.example.invalid/open/master.m3u8",
     "kind":"hls","licence":"CC-BY-3.0","height":1080},
    {"title":"movie:7","url":"https://two.example.invalid/open/b.mp4",
     "kind":"file","licence":"CC-BY-3.0","height":720}
]"#;

#[tokio::test]
async fn a_film_with_a_second_address_answers_with_both_of_them() {
    let films = vec![Arc::new(OpenFilms(parse_films(MIRRORED).unwrap())) as Arc<dyn Provider>];
    let found = answer(app(&["new"], films), LISTED).await;
    assert_eq!(found.outcome, FindOutcome::Found);
    assert_eq!(
        found.sources.len(),
        2,
        "both addresses, so one being down is not the end"
    );
    let urls: Vec<&str> = found.sources.iter().map(|s| s.url.as_str()).collect();
    assert!(urls.contains(&"https://one.example.invalid/open/master.m3u8"));
    assert!(urls.contains(&"https://two.example.invalid/open/b.mp4"));
}

#[tokio::test]
async fn the_facts_of_a_listed_film_reach_the_answer_unchanged() {
    let published: FindAnswer = serde_json::from_str(FOUND).unwrap();
    let source = &published.sources[0];
    let list = format!(
        r#"[{{"title":"movie:7","url":"{}","kind":"hls","licence":"CC-BY-3.0",
            "height":{},"audio":{}}}]"#,
        source.url,
        source.height.unwrap(),
        serde_json::to_string(&source.audio).unwrap()
    );
    let films = vec![Arc::new(OpenFilms(parse_films(&list).unwrap())) as Arc<dyn Provider>];
    let found = answer(app(&["new"], films), LISTED).await;
    assert_eq!(found.sources.len(), 1);
    assert_eq!(found.sources[0].url, source.url);
    assert_eq!(found.sources[0].height, source.height);
    assert_eq!(found.sources[0].audio, source.audio);
    // The published source is verified; this one may not claim that, whatever else it copies.
    assert!(source.verified && !found.sources[0].verified);
}

#[test]
fn an_http_address_is_refused_because_the_server_would_drop_it() {
    // A device is sent to this address, and the server keeps nothing that is not https, so a list
    // that took http passed its own check and then played nothing with no word to the deployer.
    let plain = r#"[{"title":"movie:7","url":"http://media.example.invalid/a.mp4",
        "kind":"file","licence":"CC0-1.0"}]"#;
    assert!(matches!(parse_films(plain), Err(ListError::BadUrl(_))));
}

#[test]
fn the_same_address_listed_twice_for_one_film_is_refused() {
    let twice = r#"[
        {"title":"movie:7","url":"https://one.example.invalid/a.m3u8","kind":"hls","licence":"CC0-1.0"},
        {"title":"movie:7","url":"https://one.example.invalid/a.m3u8","kind":"hls","licence":"CC0-1.0"}
    ]"#;
    assert!(matches!(parse_films(twice), Err(ListError::TwiceListed(_))));
    // The same address under a different film is a different statement and stands.
    let shared = twice.replacen("movie:7", "movie:8", 1);
    assert!(parse_films(&shared).is_ok());
}

/// A finder with no film list used to answer not found for every title in the catalogue, which
/// the server reads as a four hundred and four and a television draws as a dead end. It cannot
/// look at all, and that is a different sentence.
#[tokio::test]
async fn with_no_film_list_a_search_is_not_a_not_found() {
    let response = app(&["new"], Vec::new())
        .oneshot(find(LISTED, Some("new")))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert!(String::from_utf8_lossy(&bytes).contains("no providers"));
}
