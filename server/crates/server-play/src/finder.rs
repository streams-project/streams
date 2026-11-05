//! The finder, reached over a private route with a service token. Tests use a scripted one.

use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use contract::finder::{FindAnswer, FindOutcome, FindRequest};
use thiserror::Error;

/// Why a finder gave no answer.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum FinderError {
    #[error("no finder answered")]
    Unavailable,
}

/// What play needs from a finder.
#[async_trait]
pub trait Finder: Send + Sync {
    /// Look for sources within the budget in the request.
    async fn find(&self, request: &FindRequest) -> Result<FindAnswer, FinderError>;
    /// The rest of a partial answer.
    async fn rest(&self, job: &str) -> Result<FindAnswer, FinderError>;
}

/// A finder with answers set by a test.
#[derive(Debug, Default)]
pub struct MemoryFinder {
    answers: Mutex<Vec<FindAnswer>>,
    rest: Mutex<Vec<FindAnswer>>,
    /// Requests seen, so a test can prove none was made.
    pub seen: Mutex<Vec<FindRequest>>,
    /// True makes every call fail.
    pub down: std::sync::atomic::AtomicBool,
}

impl MemoryFinder {
    /// A finder that gives this answer, then a not found.
    #[must_use]
    pub fn answering(answer: FindAnswer) -> Self {
        let finder = Self::default();
        finder.push(answer);
        finder
    }

    /// Queue an answer for `find`.
    pub fn push(&self, answer: FindAnswer) {
        self.answers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(answer);
    }

    /// Queue an answer for `rest`.
    pub fn push_rest(&self, answer: FindAnswer) {
        self.rest
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(answer);
    }

    fn up(&self) -> Result<(), FinderError> {
        if self.down.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(FinderError::Unavailable);
        }
        Ok(())
    }
}

fn not_found() -> FindAnswer {
    FindAnswer {
        outcome: FindOutcome::NotFound,
        sources: Vec::new(),
        job: None,
    }
}

#[async_trait]
impl Finder for MemoryFinder {
    async fn find(&self, request: &FindRequest) -> Result<FindAnswer, FinderError> {
        self.up()?;
        self.seen
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(request.clone());
        let mut answers = self
            .answers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(if answers.is_empty() {
            not_found()
        } else {
            answers.remove(0)
        })
    }

    async fn rest(&self, _: &str) -> Result<FindAnswer, FinderError> {
        self.up()?;
        let mut answers = self
            .rest
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(if answers.is_empty() {
            not_found()
        } else {
            answers.remove(0)
        })
    }
}

/// Finders over HTTP, tried in turn inside one budget.
#[derive(Clone, Debug)]
pub struct HttpFinder {
    addresses: Vec<String>,
    token: String,
    client: reqwest::Client,
}

impl HttpFinder {
    /// A client for these finders.
    ///
    /// # Errors
    /// The HTTP client could not be built.
    pub fn new(addresses: &[String], token: &str) -> Result<Self, FinderError> {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(3))
            .build()
            .map_err(|_| FinderError::Unavailable)?;
        let addresses = addresses
            .iter()
            .map(|a| a.trim_end_matches('/').to_owned())
            .collect();
        Ok(Self {
            addresses,
            token: token.to_owned(),
            client,
        })
    }

    async fn read(answer: reqwest::Response) -> Option<FindAnswer> {
        answer.error_for_status().ok()?.json().await.ok()
    }
}

#[async_trait]
impl Finder for HttpFinder {
    async fn find(&self, request: &FindRequest) -> Result<FindAnswer, FinderError> {
        let budget = Duration::from_millis(u64::from(request.budget_ms));
        for address in &self.addresses {
            let call = self
                .client
                .post(format!("{address}/v1/find"))
                .bearer_auth(&self.token)
                .timeout(budget)
                .json(request);
            if let Ok(answer) = call.send().await
                && let Some(found) = Self::read(answer).await
            {
                return Ok(found);
            }
        }
        Err(FinderError::Unavailable)
    }

    async fn rest(&self, job: &str) -> Result<FindAnswer, FinderError> {
        for address in &self.addresses {
            let call = self
                .client
                .get(format!("{address}/v1/find/{job}"))
                .bearer_auth(&self.token);
            if let Ok(answer) = call.send().await
                && let Some(found) = Self::read(answer).await
            {
                return Ok(found);
            }
        }
        Err(FinderError::Unavailable)
    }
}
