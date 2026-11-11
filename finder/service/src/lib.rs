//! The finder without its sources. A pack implements [`Provider`] and hands its list to [`run`];
//! everything else, from the token check to partial answers, lives here once.

mod search;
mod slot;

use std::collections::HashMap;
use std::process::ExitCode;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use async_trait::async_trait;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use contract::finder::{FindAnswer, FindRequest, FinderHealth, FoundSource};
use contract::hello::ServiceState;

use crate::search::{Run, RunKey, wait};
use crate::slot::Slot;

/// The way out of this process. A provider fetches pages from the outside, and every one of those
/// addresses comes from a page or a list rather than from this install, so they all go through the
/// guard: one deny table, the address pinned between resolving and connecting, each redirect hop
/// checked again, and bytes capped while they are read.
pub use net_guard::{Ask, Fetched, Guard, Limits, Refusal};

/// Why a provider gave no answer. A title it simply does not have is an empty list instead.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failed(pub String);

/// One source of video.
#[async_trait]
pub trait Provider: Send + Sync {
    /// A short stable name for health reports.
    fn name(&self) -> &str;
    /// Look for this title. Dropped at the provider deadline.
    async fn find(&self, request: &FindRequest) -> Result<Vec<FoundSource>, Failed>;
}

/// Settings read at start. A missing one turns its feature off and says so.
#[derive(Clone, Debug)]
pub struct Settings {
    pub build: String,
    /// Accepted service tokens. Two during a rotation, none turns finding off.
    pub tokens: Vec<String>,
    /// How long a provider may take, however large the budget.
    pub provider_deadline: Duration,
    /// How long to keep waiting for others once one source has arrived.
    pub grace: Duration,
    /// How long a finished search stays readable.
    pub job_keep: Duration,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            build: option_env!("FINDER_BUILD").unwrap_or("dev").to_owned(),
            tokens: Vec::new(),
            provider_deadline: Duration::from_secs(12),
            grace: Duration::from_millis(1500),
            job_keep: Duration::from_secs(120),
        }
    }
}

/// A finder with its providers.
pub struct Finder {
    settings: Settings,
    slots: Vec<Arc<Slot>>,
    runs: Mutex<Runs>,
}

#[derive(Default)]
struct Runs {
    by_job: HashMap<String, Arc<Run>>,
    by_title: HashMap<RunKey, Arc<Run>>,
}

impl Finder {
    /// A finder over these providers.
    #[must_use]
    pub fn new(settings: Settings, providers: Vec<Arc<dyn Provider>>) -> Self {
        Self {
            settings,
            slots: providers.into_iter().map(Slot::new).map(Arc::new).collect(),
            runs: Mutex::new(Runs::default()),
        }
    }

    /// The search running for this title, or a new one. Two viewers pressing play on the same
    /// title share one search.
    fn run_for(&self, request: &FindRequest) -> Arc<Run> {
        let key = RunKey::of(request);
        let mut runs = self.runs.lock().unwrap_or_else(PoisonError::into_inner);
        let keep = self.settings.job_keep;
        runs.by_job.retain(|_, run| !run.expired(keep));
        runs.by_title.retain(|_, run| !run.finished());
        if let Some(run) = runs.by_title.get(&key) {
            return run.clone();
        }
        let run = Run::start(request, &self.slots, self.settings.provider_deadline);
        runs.by_title.insert(key, run.clone());
        runs.by_job.insert(run.job.clone(), run.clone());
        run
    }

    fn job(&self, job: &str) -> Option<Arc<Run>> {
        let runs = self.runs.lock().unwrap_or_else(PoisonError::into_inner);
        runs.by_job.get(job).cloned()
    }

    /// Answer one request, within its budget.
    pub async fn find(&self, request: &FindRequest) -> FindAnswer {
        let run = self.run_for(request);
        let budget = Duration::from_millis(u64::from(request.budget_ms));
        wait(&run, budget, self.settings.grace).await;
        run.answer()
    }

    fn state(&self) -> ServiceState {
        let working = self.slots.iter().any(|slot| slot.working());
        if self.settings.tokens.is_empty() || !working {
            ServiceState::Degraded
        } else {
            ServiceState::Ready
        }
    }
}

fn same(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |seen, (x, y)| seen | (x ^ y)) == 0
}

fn allowed(tokens: &[String], headers: &HeaderMap) -> bool {
    let Some(given) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    else {
        return false;
    };
    // Every token is compared, so the time taken does not show which one matched.
    tokens
        .iter()
        .fold(false, |hit, t| same(t.as_bytes(), given.as_bytes()) | hit)
}

fn refused(finder: &Finder, headers: &HeaderMap) -> Option<Response> {
    if finder.settings.tokens.is_empty() {
        let why = "finding is off: no service token is set";
        return Some((StatusCode::SERVICE_UNAVAILABLE, why).into_response());
    }
    if !allowed(&finder.settings.tokens, headers) {
        return Some(StatusCode::UNAUTHORIZED.into_response());
    }
    None
}

async fn find(
    State(finder): State<Arc<Finder>>,
    headers: HeaderMap,
    Json(request): Json<FindRequest>,
) -> Response {
    if let Some(refusal) = refused(&finder, &headers) {
        return refusal;
    }
    Json(finder.find(&request).await).into_response()
}

async fn rest(
    State(finder): State<Arc<Finder>>,
    Path(job): Path<String>,
    headers: HeaderMap,
) -> Response {
    if let Some(refusal) = refused(&finder, &headers) {
        return refusal;
    }
    match finder.job(&job) {
        Some(run) => Json(run.answer()).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn health(State(finder): State<Arc<Finder>>) -> Json<FinderHealth> {
    Json(FinderHealth {
        build: finder.settings.build.clone(),
        state: finder.state(),
    })
}

/// The published routes.
pub fn router(finder: Arc<Finder>) -> Router {
    Router::new()
        .route("/v1/find", post(find))
        .route("/v1/find/{job}", get(rest))
        .route("/v1/health", get(health))
        .with_state(finder)
}

fn tokens_from_env() -> Vec<String> {
    ["FINDER_SERVICE_TOKEN", "FINDER_SERVICE_TOKEN_NEXT"]
        .into_iter()
        .filter_map(|name| std::env::var(name).ok())
        .filter(|token| !token.is_empty())
        .collect()
}

/// Run a finder from the environment until it stops. A pack's whole `main` is this one call.
pub async fn run(providers: Vec<Arc<dyn Provider>>) -> ExitCode {
    let Ok(bind) = std::env::var("FINDER_BIND") else {
        eprintln!("not started: FINDER_BIND is not set");
        return ExitCode::from(2);
    };
    let settings = Settings {
        tokens: tokens_from_env(),
        ..Settings::default()
    };
    if settings.tokens.is_empty() {
        eprintln!("finding is off: FINDER_SERVICE_TOKEN is not set");
    }
    if providers.is_empty() {
        eprintln!("no providers, so every search finds nothing");
    }
    let listener = match tokio::net::TcpListener::bind(&bind).await {
        Ok(listener) => listener,
        Err(why) => {
            eprintln!("not started: {why}");
            return ExitCode::from(2);
        }
    };
    let finder = Arc::new(Finder::new(settings, providers));
    if let Err(why) = axum::serve(listener, router(finder)).await {
        eprintln!("stopped: {why}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
