//! The finder's promises, with scripted providers and real waiting.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use contract::finder::{FindAnswer, FindOutcome, FindRequest, FinderHealth, FoundSource};
use contract::hello::ServiceState;
use contract::play::AttemptKind;
use finder_service::{Failed, Finder, Provider, Settings, router};
use http_body_util::BodyExt;
use tower::ServiceExt;

struct Scripted {
    name: &'static str,
    after: Duration,
    url: Option<&'static str>,
    fails: bool,
    calls: AtomicU32,
}

impl Scripted {
    fn new(name: &'static str, after_ms: u64, url: Option<&'static str>) -> Arc<Self> {
        Arc::new(Self {
            name,
            after: Duration::from_millis(after_ms),
            url,
            fails: false,
            calls: AtomicU32::new(0),
        })
    }

    fn failing(name: &'static str) -> Arc<Self> {
        Arc::new(Self {
            name,
            after: Duration::ZERO,
            url: None,
            fails: true,
            calls: AtomicU32::new(0),
        })
    }
}

#[async_trait]
impl Provider for Scripted {
    fn name(&self) -> &str {
        self.name
    }

    async fn find(&self, _: &FindRequest) -> Result<Vec<FoundSource>, Failed> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        tokio::time::sleep(self.after).await;
        if self.fails {
            return Err(Failed(String::from("scripted")));
        }
        Ok(self.url.into_iter().map(source).collect())
    }
}

fn source(url: &str) -> FoundSource {
    FoundSource {
        url: url.to_owned(),
        kind: AttemptKind::Hls,
        group: String::from("g"),
        headers: BTreeMap::new(),
        verified: true,
        container: None,
        height: None,
        audio: Vec::new(),
    }
}

fn ask(budget_ms: u32) -> FindRequest {
    serde_json::from_value(serde_json::json!({
        "title": "movie:7", "name": "A Film", "budget_ms": budget_ms,
    }))
    .expect("a request")
}

fn settings() -> Settings {
    Settings {
        tokens: vec![String::from("right")],
        provider_deadline: Duration::from_millis(400),
        grace: Duration::from_millis(60),
        job_keep: Duration::from_secs(5),
        ..Settings::default()
    }
}

fn finder(providers: Vec<Arc<dyn Provider>>) -> Finder {
    Finder::new(settings(), providers)
}

#[tokio::test]
async fn all_providers_are_heard_when_they_are_quick() {
    let finder = finder(vec![
        Scripted::new("a", 0, Some("https://a.test/1")),
        Scripted::new("b", 10, Some("https://b.test/1")),
    ]);
    let answer = finder.find(&ask(300)).await;
    assert_eq!(answer.outcome, FindOutcome::Found);
    assert_eq!(answer.sources.len(), 2);
    assert_eq!(answer.job, None);
}

#[tokio::test]
async fn the_same_link_from_two_providers_counts_once() {
    let finder = finder(vec![
        Scripted::new("a", 0, Some("https://same.test/1")),
        Scripted::new("b", 0, Some("https://same.test/1")),
    ]);
    assert_eq!(finder.find(&ask(300)).await.sources.len(), 1);
}

#[tokio::test]
async fn a_slow_provider_does_not_hold_a_found_title_back() {
    let finder = finder(vec![
        Scripted::new("quick", 0, Some("https://a.test/1")),
        Scripted::new("slow", 300, Some("https://b.test/1")),
    ]);
    let began = Instant::now();
    let answer = finder.find(&ask(5000)).await;
    assert!(
        began.elapsed() < Duration::from_millis(250),
        "{:?}",
        began.elapsed()
    );
    assert_eq!(answer.outcome, FindOutcome::Partial);
    assert_eq!(answer.sources.len(), 1);
    assert!(answer.job.is_some());
}

#[tokio::test]
async fn the_budget_ends_a_search_with_nothing_yet() {
    let finder = finder(vec![Scripted::new("slow", 300, Some("https://a.test/1"))]);
    let answer = finder.find(&ask(30)).await;
    assert_eq!(answer.outcome, FindOutcome::Pending);
    assert!(answer.job.is_some());
}

#[tokio::test]
async fn a_hung_provider_is_cut_at_its_deadline() {
    let finder = finder(vec![Scripted::new(
        "hung",
        60_000,
        Some("https://a.test/1"),
    )]);
    let began = Instant::now();
    let answer = finder.find(&ask(60_000)).await;
    assert!(
        began.elapsed() < Duration::from_millis(800),
        "{:?}",
        began.elapsed()
    );
    assert_eq!(answer.outcome, FindOutcome::NotFound);
}

struct Panics;

#[async_trait]
impl Provider for Panics {
    fn name(&self) -> &str {
        "panics"
    }

    async fn find(&self, _: &FindRequest) -> Result<Vec<FoundSource>, Failed> {
        panic!("a page it could not read");
    }
}

#[tokio::test]
async fn a_provider_that_panics_is_one_failure_and_the_search_ends() {
    let finder = finder(vec![
        Arc::new(Panics),
        Scripted::new("good", 0, Some("https://a.test/1")),
    ]);
    let answer = finder.find(&ask(5000)).await;
    assert_eq!(answer.outcome, FindOutcome::Found);
    assert_eq!(answer.job, None);
}

#[tokio::test]
async fn two_viewers_share_one_search() {
    let slow = Scripted::new("slow", 100, Some("https://a.test/1"));
    let finder = finder(vec![slow.clone()]);
    let request = ask(500);
    let (first, second) = tokio::join!(finder.find(&request), finder.find(&request));
    assert_eq!(first.sources.len(), 1);
    assert_eq!(second.sources.len(), 1);
    assert_eq!(slow.calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn a_finished_search_is_not_reused() {
    let quick = Scripted::new("quick", 0, Some("https://a.test/1"));
    let finder = finder(vec![quick.clone()]);
    finder.find(&ask(300)).await;
    finder.find(&ask(300)).await;
    assert_eq!(quick.calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn a_failing_provider_rests_and_the_others_carry_on() {
    let bad = Scripted::failing("bad");
    let good = Scripted::new("good", 0, Some("https://a.test/1"));
    let finder = finder(vec![bad.clone(), good.clone()]);
    for _ in 0..7 {
        assert_eq!(finder.find(&ask(300)).await.sources.len(), 1);
    }
    assert_eq!(bad.calls.load(Ordering::SeqCst), 5);
    assert_eq!(good.calls.load(Ordering::SeqCst), 7);
}

fn asking(body: String, token: Option<&str>) -> Request<Body> {
    let mut request = Request::post("/v1/find").header(header::CONTENT_TYPE, "application/json");
    if let Some(token) = token {
        request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    request.body(Body::from(body)).expect("a request")
}

async fn body<T: serde::de::DeserializeOwned>(answer: axum::response::Response) -> T {
    let bytes = answer
        .into_body()
        .collect()
        .await
        .expect("a body")
        .to_bytes();
    serde_json::from_slice(&bytes).expect("json")
}

#[tokio::test]
async fn the_routes_check_the_token_and_read_the_rest_by_job() {
    let quick = Scripted::new("quick", 0, Some("https://a.test/1"));
    let slow = Scripted::new("slow", 150, Some("https://b.test/1"));
    let app = router(Arc::new(finder(vec![quick, slow])));
    let text = serde_json::to_string(&ask(5000)).expect("json");

    let wrong = app
        .clone()
        .oneshot(asking(text.clone(), Some("wrong")))
        .await;
    assert_eq!(wrong.expect("answers").status(), StatusCode::UNAUTHORIZED);

    let first = app.clone().oneshot(asking(text, Some("right"))).await;
    let first: FindAnswer = body(first.expect("answers")).await;
    let job = first.job.expect("a job");

    tokio::time::sleep(Duration::from_millis(200)).await;
    let rest = Request::get(format!("/v1/find/{job}"))
        .header(header::AUTHORIZATION, "Bearer right")
        .body(Body::empty())
        .expect("a request");
    let rest: FindAnswer = body(app.clone().oneshot(rest).await.expect("answers")).await;
    assert_eq!(rest.outcome, FindOutcome::Found);
    assert_eq!(rest.sources.len(), 2);

    let health = Request::get("/v1/health")
        .body(Body::empty())
        .expect("a request");
    let health: FinderHealth = body(app.oneshot(health).await.expect("answers")).await;
    assert_eq!(health.state, ServiceState::Ready);
}

#[tokio::test]
async fn with_no_token_set_finding_is_off_and_health_says_so() {
    let mut off = settings();
    off.tokens.clear();
    let app = router(Arc::new(Finder::new(
        off,
        vec![Scripted::new("a", 0, Some("https://a.test/1"))],
    )));
    let text = serde_json::to_string(&ask(100)).expect("json");
    let answer = app.clone().oneshot(asking(text, Some("any"))).await;
    assert_eq!(
        answer.expect("answers").status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    let health = Request::get("/v1/health")
        .body(Body::empty())
        .expect("a request");
    let health: FinderHealth = body(app.oneshot(health).await.expect("answers")).await;
    assert_eq!(health.state, ServiceState::Degraded);
}
