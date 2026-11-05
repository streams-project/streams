//! `/v1/hello`, `/v1/ready`, version negotiation and the guard.

mod common;

use axum::http::{Method, StatusCode};
use common::{call, get, harness, harness_no_data};
use contract::hello::{Hello, Ready, ServiceState};

#[tokio::test]
async fn s2_h4_hello_publishes_the_range_and_reads_as_the_contract_type() {
    let server = harness(&[
        ("BRAND_NAME", "Sample"),
        ("SUPPORT_EMAIL", "help@host.example"),
    ]);
    let answer = get("/v1/hello").no_version().on(&server.state).await;
    assert_eq!(answer.status, StatusCode::OK);
    let hello: Hello = serde_json::from_value(answer.body).expect("Hello");
    assert_eq!(hello.contract.max.as_str(), contract::version::WIRE_VERSION);
    assert_eq!(hello.brand.name, "Sample");
    assert!(hello.features.accounts_required);
    assert!(
        !hello.playback,
        "no finder is set, so playback says it is off"
    );
}

#[tokio::test]
async fn s153_s199_features_stay_closed_by_default() {
    let server = harness(&[]);
    let answer = get("/v1/hello").on(&server.state).await;
    let features = &answer.body["features"];
    assert_eq!(features["accounts_required"], true);
    assert_eq!(features["games"], false);
    assert!(features.get("visitor_tier").is_none());
}

#[tokio::test]
async fn hello_answers_304_to_its_own_etag() {
    let server = harness(&[]);
    let first = get("/v1/hello").on(&server.state).await;
    let tag = first.headers["etag"].to_str().unwrap().to_owned();
    let second = get("/v1/hello")
        .header("if-none-match", &tag)
        .on(&server.state)
        .await;
    assert_eq!(second.status, StatusCode::NOT_MODIFIED);
}

#[tokio::test]
async fn s111_s216_ready_tells_warming_from_broken() {
    let warming = harness(&[]);
    let answer = get("/v1/ready").no_version().on(&warming.state).await;
    assert_eq!(answer.status, StatusCode::OK);
    let ready: Ready = serde_json::from_value(answer.body).expect("Ready");
    assert_eq!(ready.build, "test-build");
    assert_eq!(ready.index, ServiceState::Degraded, "the exact scan serves");
    assert_eq!(ready.finder, ServiceState::Down);
    assert!(!ready.data_hash.is_empty(), "H68: ready names the data set");
    assert_eq!(
        ready.data_age_s, 100,
        "H62: ready shows the age of the data"
    );

    let none = harness_no_data(&[]);
    let answer = get("/v1/ready").on(&none.state).await;
    assert_eq!(answer.body["index"], "down");
    assert_eq!(answer.body["data_hash"], "");
}

#[tokio::test]
async fn fmg10_ready_carries_no_meter_or_service_name() {
    let server = harness(&[]);
    let answer = get("/v1/ready").on(&server.state).await;
    let mut names: Vec<&str> = answer
        .body
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    names.sort_unstable();
    assert_eq!(
        names,
        ["build", "data_age_s", "data_hash", "finder", "index"]
    );
}

#[tokio::test]
async fn s2_h98_an_old_wire_date_is_told_to_upgrade_and_hello_still_answers() {
    let server = harness(&[]);
    let refused = get("/v1/profiles")
        .no_version()
        .header("api-version", "2000-01-01")
        .on(&server.state)
        .await;
    assert_eq!(refused.status, StatusCode::CONFLICT);
    assert_eq!(refused.code(), "upgrade_required");
    assert_eq!(
        refused.body["build"], "test-build",
        "H98: every error names the build"
    );
    assert_eq!(refused.body["label"], "error.upgrade");
    assert!(refused.body["notice"]["text"].is_string());
    let hello = get("/v1/hello")
        .no_version()
        .header("api-version", "2000-01-01")
        .on(&server.state)
        .await;
    assert_eq!(
        hello.status,
        StatusCode::OK,
        "an old client must be able to learn the range"
    );
}

#[tokio::test]
async fn s2_a_newer_date_is_served_and_a_route_needs_a_date() {
    let server = harness(&[]);
    let newer = get("/v1/profiles")
        .no_version()
        .header("api-version", "2999-01-01")
        .on(&server.state)
        .await;
    assert_eq!(
        newer.status,
        StatusCode::UNAUTHORIZED,
        "past the version check"
    );
    let none = get("/v1/profiles").no_version().on(&server.state).await;
    assert_eq!(none.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn s3_an_unknown_address_is_404_in_the_one_error_shape() {
    let server = harness(&[]);
    let answer = get("/v1/nothing/here").on(&server.state).await;
    assert_eq!(answer.status, StatusCode::NOT_FOUND);
    assert_eq!(answer.code(), "not_found");
    assert_eq!(answer.body["build"], "test-build");
}

#[tokio::test]
async fn s186_preflight_is_answered_before_routing_and_any_origin_may_call() {
    let server = harness(&[]);
    let answer = call(Method::OPTIONS, "/v1/profiles")
        .no_version()
        .header("origin", "null")
        .header("access-control-request-method", "GET")
        .header(
            "access-control-request-headers",
            "authorization, api-version",
        )
        .on(&server.state)
        .await;
    assert_eq!(answer.status, StatusCode::OK);
    assert_eq!(answer.headers["access-control-allow-origin"], "*");
}

#[tokio::test]
async fn s110_h76_the_meter_answers_429_with_retry_after_and_hello_is_uncounted() {
    let server = harness(&[("CLIENT_ADDRESS_HEADER", "x-real-address")]);
    for _ in 0..600 {
        let answer = get("/v1/profiles")
            .header("x-real-address", "198.51.100.7")
            .on(&server.state)
            .await;
        assert_eq!(answer.status, StatusCode::UNAUTHORIZED);
    }
    let over = get("/v1/profiles")
        .header("x-real-address", "198.51.100.7")
        .on(&server.state)
        .await;
    assert_eq!(
        over.status,
        StatusCode::TOO_MANY_REQUESTS,
        "429 stays apart from 401"
    );
    assert!(over.headers.contains_key("retry-after"));
    let other = get("/v1/profiles")
        .header("x-real-address", "198.51.100.8")
        .on(&server.state)
        .await;
    assert_eq!(other.status, StatusCode::UNAUTHORIZED);
    let hello = get("/v1/hello")
        .header("x-real-address", "198.51.100.7")
        .on(&server.state)
        .await;
    assert_eq!(hello.status, StatusCode::OK);
}

#[tokio::test]
async fn h66_load_is_shed_with_retry_after_when_no_slot_is_free() {
    let server = harness(&[]);
    let held = server.state.in_flight.clone();
    let _all = held.try_acquire_many(256).expect("slots");
    let answer = get("/v1/hello").on(&server.state).await;
    assert_eq!(answer.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(answer.code(), "overloaded");
    assert!(answer.headers.contains_key("retry-after"));
}
