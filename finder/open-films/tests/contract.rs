//! The demo finder against the published contract. Requests stay in process.

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use contract::finder::{FindAnswer, FindOutcome, FinderHealth};
use contract::hello::ServiceState;
use finder_demo::{ListError, Settings, parse_films, router};
use http_body_util::BodyExt;
use tower::ServiceExt;

const FILMS: &str = r#"[{"title":"movie:7","url":"https://media.example.invalid/open/master.m3u8",
    "kind":"hls","licence":"CC-BY-3.0","height":720,"audio":["en"]}]"#;

fn settings() -> Settings {
    Settings {
        build: "test".to_owned(),
        tokens: vec!["old".to_owned(), "new".to_owned()],
        films: parse_films(FILMS).unwrap(),
        off: Vec::new(),
    }
}

fn find(body: &str, token: Option<&str>) -> Request<Body> {
    let mut request = Request::post("/v1/find").header(header::CONTENT_TYPE, "application/json");
    if let Some(token) = token {
        request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    request.body(Body::from(body.to_owned())).unwrap()
}

async fn answer(settings: Settings, body: &str) -> FindAnswer {
    let response = router(settings)
        .oneshot(find(body, Some("new")))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

const LISTED: &str = r#"{"title":"movie:7","name":"Open Film","budget_ms":1000}"#;

#[tokio::test]
async fn s6_listed_film_is_found_verified_with_no_order_field() {
    let found = answer(settings(), LISTED).await;
    assert_eq!(found.outcome, FindOutcome::Found);
    assert_eq!(found.sources.len(), 1);
    assert!(found.sources[0].verified);
    assert_eq!(found.sources[0].height, Some(720));
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
    let response = router(settings())
        .oneshot(find(bare, Some("new")))
        .await
        .unwrap();
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
        let response = router(settings())
            .oneshot(find(LISTED, token))
            .await
            .unwrap();
        assert_eq!(response.status(), status);
    }
}

#[tokio::test]
async fn h127_no_token_turns_finding_off_and_says_so() {
    let mut off = settings();
    off.tokens.clear();
    let response = router(off)
        .oneshot(find(LISTED, Some("new")))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert!(String::from_utf8_lossy(&bytes).contains("off"));
}

#[tokio::test]
async fn health_is_degraded_with_no_films_and_names_no_source() {
    let mut empty = settings();
    empty.films.clear();
    let response = router(empty)
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
