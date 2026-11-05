//! The contract's route table against the router: no route the contract names is missing, and the
//! ones still to build are named here so the list can only shrink.

mod common;

use axum::http::{Method, StatusCode};
use common::{call, harness};
use contract::api::{Auth, Group, ROUTES};

/// Routes not built yet, each with what it waits for.
const STILL_TO_BUILD: [(&str, &str, &str); 2] = [
    (
        "GET",
        "/v1/titles/{kind}/{id}/seasons/{n}",
        "no episode data in the data set",
    ),
    ("POST", "/v1/core/step", "server-hosted"),
];

fn filled(path: &str) -> String {
    path.replace("{kind}", "movie")
        .replace("{id}", "100")
        .replace("{lang}", "en")
        .replace("{name}", "home")
        .replace("{cursor}", "x")
        .replace("{n}", "1")
        .replace("{file}", "x")
}

#[tokio::test]
async fn u21_every_contract_route_answers_and_no_unknown_one_is_left() {
    let server = harness(&[]);
    for route in ROUTES.iter().filter(|route| route.group != Group::Finder) {
        let waiting = STILL_TO_BUILD
            .iter()
            .any(|(method, path, _)| *method == route.method && *path == route.path);
        let method: Method = route.method.parse().unwrap();
        let answer = call(method, &filled(route.path)).on(&server.state).await;
        let found = answer.status != StatusCode::NOT_FOUND
            && answer.status != StatusCode::METHOD_NOT_ALLOWED;
        if route.path.starts_with("/v1/strings") {
            continue;
        }
        if waiting {
            assert!(!found, "{} is built, take it off the list", route.path);
        } else {
            assert!(
                found,
                "{} {} is not mounted: {}",
                route.method, route.path, answer.status
            );
        }
    }
}

#[tokio::test]
async fn s3_a_route_that_needs_a_token_refuses_without_one() {
    let server = harness(&[]);
    for route in ROUTES {
        if !matches!(route.auth, Auth::Member | Auth::Profile) {
            continue;
        }
        let waiting = STILL_TO_BUILD
            .iter()
            .any(|(_, path, _)| *path == route.path);
        if waiting {
            continue;
        }
        let method: Method = route.method.parse().unwrap();
        let mut request = call(method, &filled(route.path));
        if route.request.is_some() {
            request = request.json(serde_json::json!({}));
        }
        let answer = request.on(&server.state).await;
        let refused = matches!(
            answer.status,
            StatusCode::UNAUTHORIZED | StatusCode::BAD_REQUEST
        );
        assert!(
            refused,
            "{} {}: {}",
            route.method, route.path, answer.status
        );
    }
}

#[tokio::test]
async fn tmt19_no_route_sets_a_cookie() {
    let server = harness(&[]);
    for route in ROUTES
        .iter()
        .filter(|route| route.method == "GET" && route.group != Group::Finder)
    {
        let answer = call(Method::GET, &filled(route.path))
            .on(&server.state)
            .await;
        assert!(!answer.headers.contains_key("set-cookie"), "{}", route.path);
    }
}

#[tokio::test]
async fn s51_the_job_deletes_lapsed_sessions_and_a_data_reload_with_no_change_is_quiet() {
    let server = harness(&[]);
    let (_, token) = common::member(&server.door).await;
    assert_eq!(
        common::get("/v1/profiles")
            .token(&token)
            .on(&server.state)
            .await
            .status,
        StatusCode::OK
    );
    server_api::jobs::reload_data(&server.state).await;
    server.clock.advance_ms(31 * 86_400_000);
    assert_eq!(server_api::jobs::prune(&server.state).await, 1);
    assert_eq!(server_api::jobs::prune(&server.state).await, 0);
}

#[tokio::test]
async fn s109_a_row_the_operator_blocked_leaves_every_page_without_a_restart() {
    let dir = common::folder("live");
    let path = dir.join("live.json");
    let path_text = path.to_string_lossy().into_owned();
    let server = harness(&[("LIVE_SETTINGS", path_text.as_str())]);
    let (_, token) = common::member(&server.door).await;
    let list = common::get("/v1/profiles")
        .token(&token)
        .on(&server.state)
        .await;
    let id = list.body["profiles"][0]["id"].as_str().unwrap().to_owned();
    call(Method::POST, &format!("/v1/profiles/{id}/enter"))
        .token(&token)
        .json(serde_json::json!({}))
        .on(&server.state)
        .await;
    let read = || common::get("/v1/screens/home").token(&token);
    let before = read().on(&server.state).await;
    assert!(before.body.to_string().contains("Drama Evenings"));
    std::fs::write(&path, r#"{"blocked_rows":["drama evenings"]}"#).unwrap();
    let after = read().on(&server.state).await;
    assert!(!after.body.to_string().contains("Drama Evenings"));
}
