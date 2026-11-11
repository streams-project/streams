//! A server on the invented data set and an in memory store. No socket is opened.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, Response, StatusCode};
use contract::version::{API_VERSION_HEADER, WIRE_VERSION};
use server_accounts::limits::DEFAULT_LIMITS;
use server_accounts::{
    AccountId, AccountStore, AuthService, Door, ManualClock, MemoryAuth, MemoryStore, NewAccount,
    StepUpMethod,
};
use server_api::{AppState, Config, Shared};
use server_catalog::DataSlot;
use server_catalog::sample::{sample_trust, write_composed_sample, write_sample};
use server_library::{Library, LibraryStore, MemoryLibrary};
use tower::ServiceExt;

static NEXT: AtomicU32 = AtomicU32::new(0);

/// A fresh folder under the test target.
#[must_use]
pub fn folder(name: &str) -> PathBuf {
    let n = NEXT.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("server_api_{}_{name}_{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("test folder");
    dir
}

/// One server for one test.
pub struct Harness {
    pub state: Shared,
    pub door: Door,
    pub clock: Arc<ManualClock>,
    pub dir: PathBuf,
    /// The rows behind the door, for a test that reads what a route wrote.
    pub store: Arc<MemoryStore>,
}

/// Time the tests start at. After the sample's build time.
pub const START_MS: u64 = 1_700_000_100_000;

/// A server with the settings given, the sample data and an empty store.
#[must_use]
pub fn harness(pairs: &[(&str, &str)]) -> Harness {
    let dir = folder("data");
    let manifest = write_sample(&dir);
    harness_with(pairs, Some(&manifest), dir)
}

/// A server with an auth service in memory.
#[must_use]
pub fn harness_with_auth(pairs: &[(&str, &str)]) -> Harness {
    harness_on(pairs, Arc::new(MemoryAuth::default()))
}

/// A server with this auth service.
#[must_use]
pub fn harness_on(pairs: &[(&str, &str)], auth: Arc<dyn AuthService>) -> Harness {
    let dir = folder("auth");
    let manifest = write_sample(&dir);
    make(pairs, Some(&manifest), dir, Some(auth))
}

/// A server whose finder a test scripts.
#[must_use]
pub fn harness_play(finder: Arc<dyn server_play::Finder>) -> Harness {
    let dir = folder("play");
    let manifest = write_sample(&dir);
    let settings = [
        ("FINDER_URLS", "http://unused.invalid"),
        ("FINDER_TOKEN", "unused"),
    ];
    let mut server = make(&settings, Some(&manifest), dir, None);
    let state = Arc::get_mut(&mut server.state).expect("one owner");
    state.play = Some(server_play::Play::new(finder));
    server
}

/// A server on the sample with composed rows.
#[must_use]
pub fn harness_composed(pairs: &[(&str, &str)]) -> Harness {
    let dir = folder("composed");
    let manifest = write_composed_sample(&dir);
    harness_with(pairs, Some(&manifest), dir)
}

/// A server with no data set.
#[must_use]
pub fn harness_no_data(pairs: &[(&str, &str)]) -> Harness {
    harness_with(pairs, None, folder("none"))
}

fn harness_with(pairs: &[(&str, &str)], manifest: Option<&Path>, dir: PathBuf) -> Harness {
    make(pairs, manifest, dir, None)
}

fn make(
    pairs: &[(&str, &str)],
    manifest: Option<&Path>,
    dir: PathBuf,
    auth: Option<Arc<dyn AuthService>>,
) -> Harness {
    let clock = Arc::new(ManualClock::new(START_MS));
    let rows = Arc::new(MemoryStore::default());
    let store: Arc<dyn AccountStore> = rows.clone();
    let mut door = Door::new(store, clock.clone(), DEFAULT_LIMITS);
    if let Some(auth) = auth {
        door = door.with_auth(auth);
    }
    let data =
        manifest.map(|path| Arc::new(DataSlot::open(path, sample_trust()).expect("sample set")));
    let mut settings: Vec<(&str, &str)> = vec![("BUILD_ID", "test-build")];
    settings.extend_from_slice(pairs);
    let books: Arc<dyn LibraryStore> = Arc::new(Arc::new(MemoryLibrary::default()));
    let library = Library::new(books, clock.clone());
    let mut state = AppState::new(
        Config::from_pairs(&settings),
        Some(door.clone()),
        data,
        Some(library),
    );
    state.clock = clock.clone();
    Harness {
        state: Arc::new(state),
        door,
        clock,
        dir,
        store: rows,
    }
}

/// An answer, read whole.
pub struct Answer {
    pub status: StatusCode,
    pub headers: axum::http::HeaderMap,
    pub body: serde_json::Value,
}

impl Answer {
    async fn read(response: Response<Body>) -> Self {
        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, usize::MAX).await.expect("body");
        Self {
            status: parts.status,
            headers: parts.headers,
            body: serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null),
        }
    }

    /// The error code of an error body.
    #[must_use]
    pub fn code(&self) -> &str {
        self.body["code"].as_str().unwrap_or("")
    }
}

/// One request.
pub struct Call {
    method: Method,
    path: String,
    token: Option<String>,
    body: Option<serde_json::Value>,
    raw: Option<String>,
    headers: Vec<(String, String)>,
    version: bool,
}

/// A request with the wire date set.
#[must_use]
pub fn call(method: Method, path: &str) -> Call {
    Call {
        method,
        path: path.to_owned(),
        token: None,
        body: None,
        raw: None,
        headers: Vec::new(),
        version: true,
    }
}

/// A GET with the wire date set.
#[must_use]
pub fn get(path: &str) -> Call {
    call(Method::GET, path)
}

impl Call {
    #[must_use]
    pub fn token(mut self, token: &str) -> Self {
        self.token = Some(token.to_owned());
        self
    }

    #[must_use]
    pub fn json(mut self, body: serde_json::Value) -> Self {
        self.body = Some(body);
        self
    }

    #[must_use]
    pub fn raw(mut self, body: &str) -> Self {
        self.raw = Some(body.to_owned());
        self
    }

    #[must_use]
    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_owned(), value.to_owned()));
        self
    }

    #[must_use]
    pub fn no_version(mut self) -> Self {
        self.version = false;
        self
    }

    pub async fn on(self, state: &Shared) -> Answer {
        let mut request = Request::builder().method(self.method).uri(self.path);
        if self.version {
            request = request.header(API_VERSION_HEADER, WIRE_VERSION);
        }
        if let Some(token) = self.token {
            request = request.header("authorization", format!("Bearer {token}"));
        }
        for (name, value) in self.headers {
            request = request.header(name, value);
        }
        let body = match self.body {
            Some(value) => {
                request = request.header("content-type", "application/json");
                Body::from(serde_json::to_vec(&value).expect("json"))
            }
            None => self.raw.map_or_else(Body::empty, Body::from),
        };
        let response = server_api::router(state.clone())
            .oneshot(request.body(body).expect("request"))
            .await
            .expect("answer");
        Answer::read(response).await
    }
}

/// A signed in account on this door, with the access token.
pub async fn member(door: &Door) -> (AccountId, String) {
    let account = NewAccount {
        id: AccountId::new(),
        email: format!("{}@example.test", AccountId::new()),
        first_name: "Test".into(),
        last_name: "Person".into(),
    };
    door.create_account(&account).await.expect("account");
    let (_, tokens) = door
        .open_session(account.id, "test device", StepUpMethod::Password)
        .await
        .expect("session");
    (account.id, tokens.access)
}
