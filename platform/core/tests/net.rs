//! Rules of the net module. Each test names the behaviour it covers.

use contract::core_msg::{HttpClass, HttpMethod, NoAnswerReason};
use platform_core::net::{
    Backoff, Next, Problem, budget_wait_ms, class_of, problem_of_failure, problem_of_status,
};

#[test]
fn c5_backoff_starts_at_2_s_doubles_caps_at_60_s_and_resets_on_success() {
    let mut backoff = Backoff::default();
    let waits: Vec<u64> = (0..7).map(|_| backoff.next_wait()).collect();
    assert_eq!(waits, [2_000, 4_000, 8_000, 16_000, 32_000, 60_000, 60_000]);
    backoff.reset();
    assert_eq!(backoff.next_wait(), 2_000);
}

#[test]
fn c5_a_budget_wait_is_the_server_value_clamped_to_30_to_600_s_with_300_by_default() {
    assert_eq!(budget_wait_ms(None), 300_000);
    assert_eq!(budget_wait_ms(Some(1)), 30_000);
    assert_eq!(budget_wait_ms(Some(120)), 120_000);
    assert_eq!(budget_wait_ms(Some(100_000)), 600_000);
}

#[test]
fn c111_every_status_is_data_and_only_a_missing_answer_is_a_failure() {
    for status in [200, 201, 204, 304] {
        assert_eq!(problem_of_status(status, None), None, "{status}");
    }
    assert_eq!(problem_of_status(401, None), Some(Problem::SignedOut));
    assert_eq!(problem_of_status(400, None), Some(Problem::Failed));
    assert_eq!(problem_of_status(403, None), Some(Problem::Failed));
    assert_eq!(problem_of_status(404, None), Some(Problem::Failed));
    assert_eq!(problem_of_status(500, None), Some(Problem::Slow));
    assert_eq!(problem_of_status(503, None), Some(Problem::Slow));
    assert_eq!(
        problem_of_status(429, Some(45)),
        Some(Problem::Budget { wait_ms: 45_000 })
    );
    assert_eq!(
        problem_of_failure(NoAnswerReason::Offline),
        Problem::Offline
    );
    assert_eq!(problem_of_failure(NoAnswerReason::Timeout), Problem::Slow);
    assert_eq!(problem_of_failure(NoAnswerReason::Tls), Problem::Offline);
}

#[test]
fn c6_each_problem_names_what_the_screen_does_next() {
    assert_eq!(Problem::Offline.next(), Next::Retry);
    assert_eq!(Problem::Slow.next(), Next::Retry);
    assert_eq!(
        Problem::Budget { wait_ms: 30_000 }.next(),
        Next::Wait { after_ms: 30_000 }
    );
    assert_eq!(Problem::Failed.next(), Next::Stop);
    assert_eq!(Problem::SignedOut.next(), Next::SignIn);
    assert!(Problem::Offline.is_transient());
    assert!(Problem::Slow.is_transient());
    assert!(Problem::Budget { wait_ms: 1 }.is_transient());
    assert!(!Problem::Failed.is_transient());
    assert!(!Problem::SignedOut.is_transient());
}

#[test]
fn the_core_marks_each_request_command_cacheable_or_plain() {
    use HttpClass::{Cacheable, Command, Plain};
    use HttpMethod::{Get, Post};
    assert_eq!(class_of(Post, "/v1/commands"), Command);
    assert_eq!(class_of(Get, "/v1/hello"), Cacheable);
    assert_eq!(class_of(Get, "/v1/nav"), Cacheable);
    assert_eq!(class_of(Get, "/v1/screens/home"), Cacheable);
    assert_eq!(class_of(Get, "/v1/screens/genre?genre=28"), Cacheable);
    assert_eq!(class_of(Get, "/v1/titles/movie/7"), Cacheable);
    assert_eq!(class_of(Post, "/v1/play"), Plain);
    assert_eq!(class_of(Post, "/v1/auth/password"), Plain);
    assert_eq!(class_of(Get, "/v1/changes"), Plain);
    assert_eq!(class_of(Get, "/v1/nowhere"), Plain);
}
