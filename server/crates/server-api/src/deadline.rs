//! Deadlines inside one request.
//!
//! The guard gives a whole request [`crate::state::REQUEST_DEADLINE_MS`]. A call on the database
//! is given less than that, so a database that has stopped answering ends the call rather than
//! the request.
//!
//! Why it is here: with Postgres down, `POST /v1/auth/password` answered `deadline_exceeded`
//! after 15.003 s and `POST /v1/auth/signup` after 15.017 s, both the request deadline to the
//! millisecond (measured 2026-11-11). The answer this crate already has for a store that is not
//! there - [`server_accounts::DoorError::Unavailable`], which `error.rs` turns into "busy, try
//! again in 5 s" - was never reached, because the call did not fail, it waited. The deadline
//! below is what turns that wait into that failure.

use std::future::Future;
use std::time::Duration;

use server_accounts::DoorError;

use crate::error::{ApiError, ApiResult};

/// Longest one call on the database may take: a third of the request deadline.
///
/// Written as that division and not as a number, so the one thing this file rests on - that a
/// store call ends before the request does - cannot come apart when either is changed. A third
/// leaves a handler room for two calls and the work between them, and a client hears "busy" in a
/// third of the time it used to wait for nothing.
pub const STORE_DEADLINE_MS: u64 = crate::state::REQUEST_DEADLINE_MS / 3;

/// Seconds a client is told to wait when a call on the database ran out of time. The same number
/// `error.rs` gives a store read that failed, because it is the same condition.
const RETRY_AFTER_S: u32 = 5;

/// Run one call on the database under [`STORE_DEADLINE_MS`].
///
/// Past the deadline the future is dropped - which releases the pool connection it was waiting
/// on instead of holding it for the rest of the request - and the answer is the one a store
/// error already gives: `Unavailable`, so the client hears "busy, try again in 5 s" and keeps
/// its tokens.
pub async fn store<T, E, F>(call: F) -> ApiResult<T>
where
    F: Future<Output = Result<T, E>>,
    ApiError: From<E>,
{
    match tokio::time::timeout(Duration::from_millis(STORE_DEADLINE_MS), call).await {
        Ok(answer) => answer.map_err(ApiError::from),
        Err(_) => Err(<ApiError as From<DoorError>>::from(
            DoorError::Unavailable {
                retry_after_s: RETRY_AFTER_S,
            },
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contract::error::ErrorCode;

    #[tokio::test(start_paused = true)]
    async fn a_call_that_never_answers_becomes_busy_rather_than_waiting() {
        let answer: ApiResult<()> = store(std::future::pending::<Result<(), DoorError>>()).await;
        let error = answer.expect_err("a call that never answers must fail");
        assert_eq!(error.code, ErrorCode::Overloaded);
        assert_eq!(error.label, "error.try_again");
        assert_eq!(error.retry_after_s, Some(RETRY_AFTER_S));
        assert_eq!(error.status, 503);
    }

    #[tokio::test(start_paused = true)]
    async fn a_call_that_answers_in_time_is_passed_through_unchanged() {
        let quick = async { Ok::<u32, DoorError>(7) };
        assert_eq!(store(quick).await.expect("the answer"), 7);
        let refused = async { Err::<u32, DoorError>(DoorError::Unauthorized) };
        assert_eq!(
            store(refused).await.expect_err("refused").code,
            ErrorCode::Unauthorized,
            "a real store error keeps its own answer"
        );
    }
}
