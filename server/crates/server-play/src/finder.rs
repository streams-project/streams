//! The finder, reached over a private route with a service token. Tests use a scripted one.

use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use contract::finder::{FindAnswer, FindOutcome, FindRequest, FoundSource};
use net_guard::{Ask, Guard, Limits};
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

/// The largest answer a finder may send. A source list is facts about a few dozen addresses.
const ANSWER_BYTES: usize = 1_000_000;

/// A backstop under the request deadline the API holds a play to, so a finder that answers its
/// headers and then stops writing cannot hold the request open to the end of that deadline. The
/// per-call budget below is what normally ends a search; this only catches the rest.
const BACKSTOP: Duration = Duration::from_secs(12);

/// How long the late attempts of a job may take. A job is already running, so reading it is quick.
const REST_BUDGET: Duration = Duration::from_secs(3);

/// Finders over HTTP. Every address is asked at once, and one budget covers the whole fan-out.
///
/// Asking them in turn and taking the first that answered was a defect, not a shortcut: a finder
/// that answers two hundred with a real not found had the same effect as one that holds the title,
/// so the list's first entry decided every play and every other provider was never asked. Asking
/// in turn also spent the budget once per address, which the request deadline above cannot afford.
#[derive(Clone, Debug)]
pub struct HttpFinder {
    addresses: Vec<String>,
    token: String,
    guard: Guard,
}

impl HttpFinder {
    /// A client for these finders.
    ///
    /// # Errors
    /// The HTTP client could not be built.
    pub fn new(addresses: &[String], token: &str) -> Result<Self, FinderError> {
        let addresses: Vec<String> = addresses
            .iter()
            .map(|a| a.trim_end_matches('/').to_owned())
            .collect();
        // These addresses come from this install's own configuration, which is why they are the
        // hosts allowed to sit on a private address: an operator's finder packs listen on
        // loopback. Nothing a provider or a viewer names gets that.
        let guard = Guard::new(Limits::provider(ANSWER_BYTES).within(BACKSTOP), &addresses)
            .map_err(|_| FinderError::Unavailable)?;
        Ok(Self {
            addresses,
            token: token.to_owned(),
            guard,
        })
    }

    /// Every call at once, and whatever came back inside the budget.
    ///
    /// The budget covers the fan-out rather than each call, so two slow finders cost what one
    /// does. Calls still running when it ends are dropped with the set.
    async fn all(&self, asks: Vec<Ask>, budget: Duration) -> Vec<FindAnswer> {
        let mut calling = tokio::task::JoinSet::new();
        for ask in asks {
            let guard = self.guard.clone();
            calling.spawn(async move { guard.fetch(&ask).await });
        }
        let until = tokio::time::Instant::now() + budget;
        let mut answers = Vec::new();
        while !calling.is_empty() {
            let Ok(Some(done)) = tokio::time::timeout_at(until, calling.join_next()).await else {
                break;
            };
            // A finder that answered with an error, or with something that is not an answer, said
            // nothing. A two hundred that says not found did say something, and merges in below.
            if let Ok(Ok(found)) = done
                && found.ok()
                && let Ok(answer) = serde_json::from_slice::<FindAnswer>(&found.bytes)
            {
                answers.push(answer);
            }
        }
        answers
    }
}

/// Everything the finders said, as one answer, or `None` when not one of them answered.
///
/// The two rules are the ones the finder service already uses across its own providers: a link two
/// of them both hold counts once, and the outcome follows from whether anything is still coming
/// and whether anything was found.
fn merge(answers: Vec<FindAnswer>) -> Option<FindAnswer> {
    if answers.is_empty() {
        return None;
    }
    let mut sources: Vec<FoundSource> = Vec::new();
    let mut job = None;
    for answer in answers {
        for source in answer.sources {
            if !sources.iter().any(|seen| seen.url == source.url) {
                sources.push(source);
            }
        }
        job = job.or(answer.job);
    }
    // Only a job can be followed up, so a job is what "still coming" means here.
    let outcome = match (job.is_some(), sources.is_empty()) {
        (false, true) => FindOutcome::NotFound,
        (false, false) => FindOutcome::Found,
        (true, true) => FindOutcome::Pending,
        (true, false) => FindOutcome::Partial,
    };
    Some(FindAnswer {
        outcome,
        sources,
        job,
    })
}

#[async_trait]
impl Finder for HttpFinder {
    async fn find(&self, request: &FindRequest) -> Result<FindAnswer, FinderError> {
        let budget = Duration::from_millis(u64::from(request.budget_ms));
        let body = serde_json::to_vec(request).map_err(|_| FinderError::Unavailable)?;
        let asks = self
            .addresses
            .iter()
            .map(|address| {
                Ask::post_json(format!("{address}/v1/find"), body.clone()).bearer(&self.token)
            })
            .collect();
        merge(self.all(asks, budget).await).ok_or(FinderError::Unavailable)
    }

    async fn rest(&self, job: &str) -> Result<FindAnswer, FinderError> {
        // A job belongs to one finder, so the others answer not found and merge to nothing.
        let asks = self
            .addresses
            .iter()
            .map(|address| Ask::get(format!("{address}/v1/find/{job}")).bearer(&self.token))
            .collect();
        merge(self.all(asks, REST_BUDGET).await).ok_or(FinderError::Unavailable)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    /// A finder over HTTP that answers this body after this wait. Raw TCP, because what is proved
    /// here is which addresses were asked and when, not what any library does with them.
    async fn finder_at(body: String, after: Duration) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("a port");
        let address = format!("http://{}", listener.local_addr().expect("an address"));
        tokio::spawn(async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            while let Ok((mut stream, _)) = listener.accept().await {
                let body = body.clone();
                tokio::spawn(async move {
                    let mut ignored = [0u8; 4096];
                    let _ = stream.read(&mut ignored).await;
                    tokio::time::sleep(after).await;
                    let reply = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\
                         Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    );
                    let _ = stream.write_all(reply.as_bytes()).await;
                    let _ = stream.flush().await;
                });
            }
        });
        address
    }

    fn nothing() -> String {
        String::from(r#"{"outcome":"not_found","sources":[]}"#)
    }

    fn holding(url: &str) -> String {
        format!(
            r#"{{"outcome":"found","sources":[{{"url":"{url}","kind":"hls",
            "group":"g","headers":{{}},"verified":false}}]}}"#
        )
    }

    fn ask(budget_ms: u32) -> FindRequest {
        serde_json::from_value(serde_json::json!({
            "title": "movie:7", "name": "A Film", "budget_ms": budget_ms,
        }))
        .expect("a request")
    }

    #[tokio::test]
    async fn s167_a_finder_that_answers_not_found_does_not_hide_the_one_holding_the_title() {
        // The order the launcher writes: the open film pack first, which answers for everything.
        let first = finder_at(nothing(), Duration::ZERO).await;
        let second = finder_at(holding("https://real.example/master.m3u8"), Duration::ZERO).await;
        let finder = HttpFinder::new(&[first, second], "token").expect("a finder");
        let found = finder.find(&ask(2000)).await.expect("an answer");
        assert_eq!(found.outcome, FindOutcome::Found);
        assert_eq!(found.sources.len(), 1);
        assert_eq!(found.sources[0].url, "https://real.example/master.m3u8");
    }

    #[tokio::test]
    async fn s167_the_same_link_from_two_finders_counts_once() {
        let body = holding("https://same.example/a.m3u8");
        let first = finder_at(body.clone(), Duration::ZERO).await;
        let second = finder_at(body, Duration::ZERO).await;
        let finder = HttpFinder::new(&[first, second], "token").expect("a finder");
        assert_eq!(
            finder
                .find(&ask(2000))
                .await
                .expect("an answer")
                .sources
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn s167_h78_one_budget_covers_the_whole_fan_out() {
        let slow = Duration::from_millis(300);
        let first = finder_at(holding("https://a.example/1.m3u8"), slow).await;
        let second = finder_at(holding("https://b.example/2.m3u8"), slow).await;
        let finder = HttpFinder::new(&[first, second], "token").expect("a finder");
        let began = Instant::now();
        let found = finder.find(&ask(400)).await.expect("an answer");
        // Asked in turn, the second would start after the first finished and the budget would be
        // gone, so both sources here are what proves they were asked together.
        assert_eq!(found.sources.len(), 2, "both finders inside one budget");
        assert!(
            began.elapsed() < Duration::from_millis(400),
            "{:?}",
            began.elapsed()
        );
    }

    #[tokio::test]
    async fn s167_with_nothing_answering_the_play_fails_and_does_not_read_a_not_found() {
        let finder =
            HttpFinder::new(&[String::from("http://127.0.0.1:1")], "token").expect("a finder");
        assert_eq!(finder.find(&ask(500)).await, Err(FinderError::Unavailable));
    }

    #[test]
    fn s167_an_answer_still_coming_keeps_its_job_and_merges_as_partial() {
        let pending = FindAnswer {
            outcome: FindOutcome::Pending,
            sources: Vec::new(),
            job: Some(String::from("j1")),
        };
        let found: FindAnswer =
            serde_json::from_str(&holding("https://a.example/1.m3u8")).expect("an answer");
        let merged = merge(vec![pending, found]).expect("merged");
        assert_eq!(merged.outcome, FindOutcome::Partial);
        assert_eq!(merged.job.as_deref(), Some("j1"));
        assert_eq!(merged.sources.len(), 1);
        assert_eq!(merge(Vec::new()), None, "nothing answered");
    }
}
