//! The titles over the profile chooser: a member route, at most three titles, each with a logo, a
//! poster and a backdrop and its line as facts, never above the account's most open tier.

mod common;

use axum::http::{Method, StatusCode};
use common::{Harness, call, get, harness, member};
use contract::TitleKind;
use contract::screen::{GateLineKind, ImageRole, ProfileGate};
use serde_json::json;
use server_accounts::{AccountId, StepUpMethod};
use server_catalog::sample::{SEASON_COMING_BACK, sample_titles};

const IMAGES: (&str, &str) = ("IMAGE_BASE", "https://images.host.example/t/");
const DAY_MS: u64 = 86_400_000;

async fn gate(server: &Harness, token: &str) -> ProfileGate {
    let answer = get("/v1/profiles/gate")
        .token(token)
        .on(&server.state)
        .await;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    assert_eq!(answer.headers["cache-control"], "no-store");
    serde_json::from_value(answer.body).expect("ProfileGate")
}

/// A new session for the account, after the clock moved past the last one's life.
async fn again(server: &Harness, account: AccountId) -> String {
    let (_, tokens) = server
        .door
        .open_session(account, "test device", StepUpMethod::Password)
        .await
        .expect("session");
    tokens.access
}

fn has(gate: &ProfileGate, role: ImageRole) -> bool {
    gate.titles
        .iter()
        .all(|title| title.tile.images.iter().any(|image| image.role == role))
}

/// The certificate the sample gives a title, by its key.
fn cert_of(key: &str) -> String {
    let (kind, id) = key.split_once(':').unwrap();
    let kind = if kind == "tv" { "tv" } else { "movie" };
    let title = sample_titles()
        .into_iter()
        .find(|title| title.kind == kind && title.id.to_string() == id)
        .unwrap();
    let certs: serde_json::Value = serde_json::from_str(title.certs.as_deref().unwrap()).unwrap();
    certs["GB"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn n12_the_gate_needs_a_signed_in_member() {
    let server = harness(&[IMAGES]);
    let answer = get("/v1/profiles/gate").on(&server.state).await;
    assert_eq!(answer.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn n12_three_titles_coming_within_thirty_days_each_with_logo_and_art() {
    let server = harness(&[IMAGES]);
    let (_, token) = member(&server.door).await;
    let answer = gate(&server, &token).await;
    assert_eq!(answer.titles.len(), 3, "{answer:?}");
    assert!(has(&answer, ImageRole::Logo));
    assert!(has(&answer, ImageRole::Poster));
    assert!(has(&answer, ImageRole::Backdrop));
    let mut keys: Vec<String> = answer
        .titles
        .iter()
        .map(|t| t.tile.title.to_string())
        .collect();
    keys.dedup();
    assert_eq!(keys.len(), 3);
    for title in &answer.titles {
        assert_eq!(title.line.kind, GateLineKind::Coming, "{title:?}");
        let day = title
            .line
            .date
            .as_ref()
            .expect("a coming line has a day")
            .as_str();
        // The sample's own time is 2023-11-14, and its releases run to 2023-11-26.
        assert!(("2023-11-15".."2023-12-15").contains(&day), "{day}");
        assert!(title.line.rank.is_none() && title.line.media.is_none());
    }
    assert_eq!(answer.ttl_s, 3600);
}

#[tokio::test]
async fn n12_after_the_premieres_a_returning_series_then_todays_top_ten() {
    let server = harness(&[IMAGES]);
    let (account, _) = member(&server.door).await;
    server.clock.advance_ms(13 * DAY_MS);
    let answer = gate(&server, &again(&server, account).await).await;
    assert_eq!(answer.titles.len(), 3, "{answer:?}");
    let first = &answer.titles[0];
    assert_eq!(first.line.kind, GateLineKind::SeasonsComing);
    assert_eq!(
        first.tile.title.to_string(),
        format!("tv:{SEASON_COMING_BACK}")
    );
    assert_eq!(
        first.line.date.as_ref().map(|day| day.as_str()),
        Some("2023-11-30")
    );
    for title in &answer.titles[1..] {
        assert_eq!(title.line.kind, GateLineKind::Ranked, "{title:?}");
        let rank = title.line.rank.expect("a place");
        assert!((1..=10).contains(&rank));
        let kind = title.tile.title.kind();
        let media = title.line.media.expect("a kind");
        assert_eq!(media, kind);
        assert!(matches!(media, TitleKind::Movie | TitleKind::Tv));
    }
}

#[tokio::test]
async fn n12_the_choice_turns_with_the_day_and_holds_within_it() {
    let server = harness(&[IMAGES]);
    let (account, token) = member(&server.door).await;
    let morning = gate(&server, &token).await;
    // The sample starts at 22:15 UTC, so an hour stays within the same day.
    server.clock.advance_ms(DAY_MS / 24);
    let later = gate(&server, &again(&server, account).await).await;
    assert_eq!(morning, later);
    let mut seen = std::collections::BTreeSet::new();
    for _ in 0..6 {
        server.clock.advance_ms(DAY_MS);
        let answer = gate(&server, &again(&server, account).await).await;
        seen.insert(
            answer
                .titles
                .iter()
                .map(|t| t.tile.title.to_string())
                .collect::<Vec<_>>(),
        );
    }
    assert!(seen.len() > 1, "the same titles every day: {seen:?}");
}

#[tokio::test]
async fn n12_an_account_kept_to_pg_never_sees_a_title_above_it() {
    let server = harness(&[IMAGES]);
    let (account, token) = member(&server.door).await;
    let profiles = get("/v1/profiles").token(&token).on(&server.state).await;
    let grown_up = profiles.body["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["kids"] == false)
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let path = format!("/v1/profiles/{grown_up}/limits");
    let limited = call(Method::PUT, &path)
        .token(&token)
        .json(json!({ "age_rating": "PG", "blocked_titles": [] }))
        .on(&server.state)
        .await;
    assert_eq!(limited.status, StatusCode::OK, "{}", limited.body);
    for days in [0, 13] {
        server.clock.advance_ms(days * DAY_MS);
        let answer = gate(&server, &again(&server, account).await).await;
        assert!(
            !answer.titles.is_empty(),
            "the sample has titles for children"
        );
        for title in &answer.titles {
            let cert = cert_of(&title.tile.title.to_string());
            assert!(
                cert == "U" || cert == "PG",
                "{} is {cert}",
                title.tile.title
            );
            // A child's chart is no place in Today's Top 10.
            assert_ne!(title.line.kind, GateLineKind::Ranked);
        }
    }
}
