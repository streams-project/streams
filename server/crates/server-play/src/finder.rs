//! The finder, reached over a private route with a service token. Tests use a scripted one.

use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use contract::finder::{FindAnswer, FindOutcome, FindRequest, FoundSource};
use moka::sync::Cache;
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

/// Job handles remembered, and for how long. A handle is followed up only while the play session
/// holding it lives, so these are that cache's own numbers (`gate.rs`: six hours, a hundred
/// thousand plays), and a handle is written here at the moment its session is written there.
/// A follow-up that superseded an earlier handle leaves the earlier one behind, so under enough
/// churn a live handle can still be dropped by capacity; then the follow-up falls back to asking
/// every finder, which is what every handle used to get, and never to answering nothing.
const JOBS_MAX: u64 = 100_000;
const JOB_TTL: Duration = Duration::from_secs(6 * 3600);

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
    /// Which finder issued each job handle. A handle belongs to the pack that made it, so the
    /// follow-up is sent to that one and to nothing else: asking every configured pack handed one
    /// pack another pack's handle and spent the whole follow-up budget on packs that have no such
    /// job, which is the isolation the finder layer promises per pack.
    issued: Cache<String, String>,
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
            issued: Cache::builder()
                .max_capacity(JOBS_MAX)
                .time_to_live(JOB_TTL)
                .build(),
        })
    }

    /// Remember which finder a handle came from, including the fresh handle a follow-up answers
    /// with, so the second follow-up goes to the same one as the first.
    fn remember(&self, answer: &FindAnswer, address: &str) {
        if let Some(job) = &answer.job {
            self.issued.insert(job.clone(), address.to_owned());
        }
    }

    /// Every call at once, and whatever came back inside the budget.
    ///
    /// The budget covers the fan-out rather than each call, so two slow finders cost what one
    /// does. Calls still running when it ends are dropped with the set.
    async fn all(&self, asks: Vec<(String, Ask)>, budget: Duration) -> Vec<FindAnswer> {
        let mut calling = tokio::task::JoinSet::new();
        for (address, ask) in asks {
            let guard = self.guard.clone();
            calling.spawn(async move { (address, guard.fetch(&ask).await) });
        }
        let until = tokio::time::Instant::now() + budget;
        let mut answers = Vec::new();
        while !calling.is_empty() {
            let Ok(Some(done)) = tokio::time::timeout_at(until, calling.join_next()).await else {
                break;
            };
            // A finder that answered with an error, or with something that is not an answer, said
            // nothing. A two hundred that says not found did say something, and merges in below.
            if let Ok((address, Ok(found))) = done
                && found.ok()
                && let Ok(answer) = serde_json::from_slice::<FindAnswer>(&found.bytes)
            {
                // Who issued the handle is known here and nowhere later: the merged answer holds
                // one job string and cannot say whose it was.
                self.remember(&answer, &address);
                answers.push(answer);
            }
        }
        answers
    }

    /// The finders a call goes to: the one that issued this handle, or all of them for a call that
    /// has no handle yet.
    fn asking(&self, job: Option<&str>) -> Vec<String> {
        match job.and_then(|job| self.issued.get(job)) {
            Some(address) => vec![address],
            None => self.addresses.clone(),
        }
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
            .asking(None)
            .into_iter()
            .map(|address| {
                let ask =
                    Ask::post_json(format!("{address}/v1/find"), body.clone()).bearer(&self.token);
                (address, ask)
            })
            .collect();
        merge(self.all(asks, budget).await).ok_or(FinderError::Unavailable)
    }

    async fn rest(&self, job: &str) -> Result<FindAnswer, FinderError> {
        // A handle belongs to the finder that issued it, and `find` wrote down which that was. A
        // handle this process never issued is the only case that still asks them all, which is
        // the behaviour every handle used to get.
        let asks = self
            .asking(Some(job))
            .into_iter()
            .map(|address| {
                let ask = Ask::get(format!("{address}/v1/find/{job}")).bearer(&self.token);
                (address, ask)
            })
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

    /// A finder that answers this body and writes down the first line of every request it is
    /// sent, so a test can prove which finders were asked and which were not.
    async fn finder_noting(body: String) -> (String, std::sync::Arc<Mutex<Vec<String>>>) {
        let seen = std::sync::Arc::new(Mutex::new(Vec::new()));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("a port");
        let address = format!("http://{}", listener.local_addr().expect("an address"));
        let noting = std::sync::Arc::clone(&seen);
        tokio::spawn(async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            while let Ok((mut stream, _)) = listener.accept().await {
                let (body, noting) = (body.clone(), std::sync::Arc::clone(&noting));
                tokio::spawn(async move {
                    let mut head = [0u8; 4096];
                    let read = stream.read(&mut head).await.unwrap_or(0);
                    let line = String::from_utf8_lossy(&head[..read])
                        .lines()
                        .next()
                        .unwrap_or_default()
                        .to_owned();
                    noting
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .push(line);
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
        (address, seen)
    }

    fn asked(seen: &Mutex<Vec<String>>) -> Vec<String> {
        seen.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// A handle names a job inside one pack, and the follow-up used to be sent to every pack
    /// configured: each one was handed another pack's handle with the service token, and the
    /// whole follow-up budget could be spent on packs that have no such job. One operator with
    /// one pack never saw it, which is why nothing caught it.
    #[tokio::test]
    async fn h78_a_job_handle_goes_only_to_the_finder_that_issued_it() {
        let holder = r#"{"outcome":"partial","sources":[],"job":"job-of-the-first"}"#;
        let (first, first_seen) = finder_noting(String::from(holder)).await;
        let (second, second_seen) = finder_noting(nothing()).await;
        let finder = HttpFinder::new(&[first.clone(), second.clone()], "token").expect("a finder");

        let found = finder.find(&ask(2000)).await.expect("an answer");
        assert_eq!(found.job.as_deref(), Some("job-of-the-first"));
        // Both were asked to search, which is what a search is.
        assert_eq!(asked(&first_seen).len(), 1);
        assert_eq!(asked(&second_seen).len(), 1);
        first_seen
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        second_seen
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();

        finder.rest("job-of-the-first").await.expect("the rest");
        assert_eq!(
            asked(&first_seen),
            vec![String::from("GET /v1/find/job-of-the-first HTTP/1.1")],
            "the finder that issued the handle was not the one asked"
        );
        assert!(
            asked(&second_seen).is_empty(),
            "another pack was handed this pack's job handle: {:?}",
            asked(&second_seen)
        );

        // A handle this process never issued has nowhere to go, so it is asked of all of them,
        // which is what every handle used to get. It is reachable only after a restart, which
        // loses the plays that hold handles as well.
        finder.rest("job-nobody-issued").await.expect("the rest");
        assert_eq!(asked(&first_seen).len(), 2);
        assert_eq!(asked(&second_seen).len(), 1);
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
