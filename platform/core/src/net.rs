//! Builds requests by path, attaches tokens only for the server's own routes, sorts problems by
//! what the screen does next and marks each effect command, cacheable or plain.

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use contract::api::{self, Auth};
use contract::core_msg::{BodyTarget, EffectBody, HttpClass, HttpMethod, NoAnswerReason};
use contract::version::{API_VERSION_HEADER, WIRE_VERSION};

use crate::config::{self, Config};

/// How long the host may spend on one request.
pub const REQUEST_DEADLINE_MS: u32 = 60_000;

const BACKOFF_FIRST_MS: u64 = 2_000;
const BACKOFF_MAX_MS: u64 = 60_000;
const BUDGET_DEFAULT_S: u32 = 300;
const BUDGET_MIN_S: u32 = 30;
const BUDGET_MAX_S: u32 = 600;

/// Wait before each try: 2 s, doubling, at most 60 s, back to 2 s after a success.
#[derive(Clone, Copy, Debug, Default)]
pub struct Backoff {
    wait_ms: u64,
}

impl Backoff {
    /// The wait before the next try.
    pub fn next_wait(&mut self) -> u64 {
        self.wait_ms = match self.wait_ms {
            0 => BACKOFF_FIRST_MS,
            wait => (wait * 2).min(BACKOFF_MAX_MS),
        };
        self.wait_ms
    }

    pub const fn reset(&mut self) {
        self.wait_ms = 0;
    }
}

/// A budget wait is the server's value clamped to 30 to 600 s, 300 s when it gave none.
#[must_use]
pub fn budget_wait_ms(retry_after_s: Option<u32>) -> u64 {
    let seconds = retry_after_s
        .unwrap_or(BUDGET_DEFAULT_S)
        .clamp(BUDGET_MIN_S, BUDGET_MAX_S);
    u64::from(seconds) * 1_000
}

/// What went wrong.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Problem {
    /// No answer arrived.
    Offline,
    /// The server was slow or broke: a timeout, or a status of 500 or more.
    Slow,
    /// The server asked for a wait.
    Budget { wait_ms: u64 },
    /// The server refused this request. Asking again gives the same answer.
    Failed,
    /// The token was refused.
    SignedOut,
}

/// What the screen does next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Next {
    Retry,
    Wait { after_ms: u64 },
    Stop,
    SignIn,
}

impl Problem {
    #[must_use]
    pub const fn next(self) -> Next {
        match self {
            Self::Offline | Self::Slow => Next::Retry,
            Self::Budget { wait_ms } => Next::Wait { after_ms: wait_ms },
            Self::Failed => Next::Stop,
            Self::SignedOut => Next::SignIn,
        }
    }

    /// True when the same request may work later without anyone doing anything.
    #[must_use]
    pub const fn is_transient(self) -> bool {
        matches!(self, Self::Offline | Self::Slow | Self::Budget { .. })
    }

    /// Label key a screen draws.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Offline => "problem-offline",
            Self::Slow => "problem-slow",
            Self::Budget { .. } => "problem-busy",
            Self::Failed => "problem-failed",
            Self::SignedOut => "problem-signed-out",
        }
    }
}

/// Every status is data: this only says what a status means.
#[must_use]
pub fn problem_of_status(status: u32, retry_after_s: Option<u32>) -> Option<Problem> {
    match status {
        200..=399 => None,
        401 => Some(Problem::SignedOut),
        429 => Some(Problem::Budget {
            wait_ms: budget_wait_ms(retry_after_s),
        }),
        500.. => Some(Problem::Slow),
        _ => Some(Problem::Failed),
    }
}

/// An error from the host means no answer arrived.
#[must_use]
pub const fn problem_of_failure(reason: NoAnswerReason) -> Problem {
    match reason {
        NoAnswerReason::Timeout => Problem::Slow,
        _ => Problem::Offline,
    }
}

const fn method_word(method: HttpMethod) -> &'static str {
    match method {
        HttpMethod::Get => "GET",
        HttpMethod::Post => "POST",
        HttpMethod::Put => "PUT",
        HttpMethod::Patch => "PATCH",
        HttpMethod::Delete => "DELETE",
        _ => "",
    }
}

fn route_of(method: HttpMethod, path: &str) -> Option<&'static api::Route> {
    let path = path.split('?').next().unwrap_or(path);
    api::find(method_word(method), path)
}

/// Marks a request. The contract's route table is the one home of what may be kept.
#[must_use]
pub fn class_of(method: HttpMethod, path: &str) -> HttpClass {
    match route_of(method, path) {
        Some(route) if route.name == "sendCommands" => HttpClass::Command,
        Some(route) if route.cacheable => HttpClass::Cacheable,
        _ => HttpClass::Plain,
    }
}

/// What an answer is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Purpose {
    Hello,
    Refresh,
    SignIn,
    CodeSent,
    SignOut,
    Card,
    Profiles,
    Enter,
    Page,
    Play(u32),
    More(u32),
    Marks(u32),
    Command,
    /// Prepared by whoever sent `Action::Request`. The answer is theirs.
    Passed,
}

/// One request.
#[derive(Clone, Debug)]
pub(crate) struct Request {
    pub method: HttpMethod,
    pub path: String,
    pub body: Option<String>,
    pub to: BodyTarget,
    pub purpose: Purpose,
}

impl Request {
    pub(crate) fn new(method: HttpMethod, path: impl Into<String>, purpose: Purpose) -> Self {
        Self {
            method,
            path: path.into(),
            body: None,
            to: BodyTarget::Core,
            purpose,
        }
    }

    pub(crate) fn with_body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }

    pub(crate) const fn for_screen(mut self) -> Self {
        self.to = BodyTarget::Screen;
        self
    }

    /// True when the route takes a token. Only the contract's routes ever get one.
    pub(crate) fn takes_token(&self) -> bool {
        route_of(self.method, &self.path).is_some_and(|route| !matches!(route.auth, Auth::Open))
    }

    fn class(&self) -> HttpClass {
        if self.purpose == Purpose::Passed {
            HttpClass::Plain
        } else {
            class_of(self.method, &self.path)
        }
    }
}

/// A request on its way.
pub(crate) struct Pending {
    pub request: Request,
    /// Which token it carried.
    pub serial: u32,
    /// True after a 401 sent it again.
    pub retried: bool,
}

/// A request waiting for a fresh token.
pub(crate) struct Held {
    pub id: String,
    pub request: Request,
    pub retried: bool,
}

/// Ids, requests on their way and the headers every request carries.
pub(crate) struct Net {
    headers: BTreeMap<String, String>,
    next_id: u64,
    pending: BTreeMap<String, Pending>,
    held: Vec<Held>,
}

impl Net {
    pub(crate) fn new(config: &Config) -> Self {
        let mut headers = BTreeMap::new();
        headers.insert(API_VERSION_HEADER.to_string(), WIRE_VERSION.to_string());
        for (name, value) in config::client_headers(config) {
            headers.insert(name.to_string(), value);
        }
        Self {
            headers,
            next_id: 0,
            pending: BTreeMap::new(),
            held: Vec::new(),
        }
    }

    pub(crate) fn new_id(&mut self) -> String {
        self.next_id += 1;
        format!("e{}", self.next_id)
    }

    /// The effect for a request. `token` is `Some` only when the route takes one.
    pub(crate) fn effect(
        &self,
        request: &Request,
        token: Option<&str>,
        scope: Option<&str>,
    ) -> EffectBody {
        let class = request.class();
        let mut headers = self.headers.clone();
        if request.body.is_some() {
            headers.insert("Content-Type".to_string(), "application/json".to_string());
        }
        if let Some(token) = token.filter(|token| !token.is_empty()) {
            headers.insert("Authorization".to_string(), format!("Bearer {token}"));
        }
        EffectBody::Http {
            method: request.method,
            path: request.path.clone(),
            headers,
            body: request.body.clone(),
            deadline_ms: REQUEST_DEADLINE_MS,
            class,
            to: request.to,
            scope: if matches!(class, HttpClass::Plain) {
                None
            } else {
                scope.map(ToString::to_string)
            },
        }
    }

    pub(crate) fn track(&mut self, id: String, pending: Pending) {
        self.pending.insert(id, pending);
    }

    pub(crate) fn take(&mut self, id: &str) -> Option<Pending> {
        self.pending.remove(id)
    }

    pub(crate) fn hold(&mut self, held: Held) {
        self.held.push(held);
    }

    pub(crate) fn take_held(&mut self) -> Vec<Held> {
        core::mem::take(&mut self.held)
    }

    /// Forget everything that belongs to the session that ended.
    pub(crate) fn forget_session(&mut self) {
        self.held.clear();
        self.pending
            .retain(|_, pending| !pending.request.takes_token());
    }
}
