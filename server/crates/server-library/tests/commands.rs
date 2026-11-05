//! Every command rule on the in memory store, with two devices where the rule is about two.

use std::sync::Arc;

use contract::command::{
    Change, Command, CommandBatch, CommandOp, FeedbackEvent, Outcome, SettingKey,
};
use contract::{EpisodeRef, Rating, TitleKey, TitleKind};
use server_accounts::limits::DEFAULT_LIMITS;
use server_accounts::{
    AccountId, AccountStore, Consent, Door, ManualClock, MemoryStore, NewAccount, ProfileScope,
    StepUpMethod,
};
use server_library::{Library, LibraryStore, MemoryLibrary, TitleFacts};

struct Facts;

impl TitleFacts for Facts {
    fn genres(&self, _: TitleKey) -> Vec<String> {
        vec![String::from("Drama")]
    }
}

struct World {
    library: Library,
    clock: Arc<ManualClock>,
    scope: ProfileScope,
    next: std::cell::Cell<u32>,
}

const NOW: u64 = 1_000_000_000;

async fn world(consent: bool) -> World {
    let clock = Arc::new(ManualClock::new(NOW));
    let store: Arc<dyn AccountStore> = Arc::new(MemoryStore::default());
    let door = Door::new(store, clock.clone(), DEFAULT_LIMITS);
    let account = NewAccount {
        id: AccountId::new(),
        email: "a@b.test".into(),
        first_name: "Ada".into(),
        last_name: "L".into(),
    };
    door.create_account(&account).await.unwrap();
    let (_, tokens) = door
        .open_session(account.id, "t", StepUpMethod::Password)
        .await
        .unwrap();
    let member = door.member(&tokens.access).await.unwrap();
    door.set_consent(&member, Consent { personal: consent })
        .await
        .unwrap();
    let main = door.profiles(&member).await.unwrap().remove(0);
    door.enter_profile(&member, main.id, None).await.unwrap();
    let member = door.member(&tokens.access).await.unwrap();
    let scope = door.scope(&member).await.unwrap();
    let store: Arc<dyn LibraryStore> = Arc::new(Arc::new(MemoryLibrary::default()));
    World {
        library: Library::new(store, clock.clone()),
        clock,
        scope,
        next: std::cell::Cell::new(0),
    }
}

fn film(id: u32) -> TitleKey {
    TitleKey::new(TitleKind::Movie, id).unwrap()
}

fn series(id: u32) -> TitleKey {
    TitleKey::new(TitleKind::Tv, id).unwrap()
}

impl World {
    async fn one(&self, age_ms: Option<u64>, command: Command) -> Outcome {
        let id = self.next.replace(self.next.get() + 1);
        self.named(&format!("op{id}"), age_ms, command).await
    }

    async fn named(&self, op_id: &str, age_ms: Option<u64>, command: Command) -> Outcome {
        let batch = CommandBatch {
            ops: vec![CommandOp {
                op_id: op_id.to_owned(),
                age_ms,
                command,
            }],
        };
        self.library
            .apply(&self.scope, &Facts, &batch)
            .await
            .unwrap()
            .answers[0]
            .outcome
    }

    fn store(&self) -> &Arc<dyn LibraryStore> {
        self.library.store()
    }

    async fn changes(&self, cursor: &str) -> Vec<Change> {
        self.library
            .changes(self.scope.profile(), Some(cursor))
            .await
            .unwrap()
            .changes
    }
}

fn pos(title: TitleKey, episode: Option<EpisodeRef>, position_s: u32, duration_s: u32) -> Command {
    Command::Position {
        title,
        episode,
        position_s,
        duration_s,
    }
}

fn ep(season: u32, episode: u32) -> Option<EpisodeRef> {
    Some(EpisodeRef { season, episode })
}

#[tokio::test]
async fn h55_h24_a_rating_from_a_stale_device_leaves_the_list_alone() {
    let w = world(false).await;
    let t = film(1);
    w.one(
        None,
        Command::List {
            title: t,
            in_list: true,
        },
    )
    .await;
    // the other device has not pulled, and rates the title with an older age
    w.one(
        Some(5000),
        Command::Rate {
            title: t,
            rating: Some(Rating::Up),
        },
    )
    .await;
    assert_eq!(
        w.store().list(w.scope.profile(), 10).await.unwrap(),
        vec![t]
    );
    let rated = w.store().rated(w.scope.profile(), 10).await.unwrap();
    assert_eq!(rated, vec![(t, Rating::Up)]);
}

#[tokio::test]
async fn c18_removing_from_the_list_keeps_the_rating_and_clearing_it_keeps_the_list() {
    let w = world(false).await;
    let t = film(1);
    w.one(
        None,
        Command::Rate {
            title: t,
            rating: Some(Rating::Love),
        },
    )
    .await;
    w.one(
        None,
        Command::List {
            title: t,
            in_list: true,
        },
    )
    .await;
    w.one(
        None,
        Command::List {
            title: t,
            in_list: false,
        },
    )
    .await;
    assert_eq!(
        w.store().rated(w.scope.profile(), 9).await.unwrap().len(),
        1
    );
    w.one(
        None,
        Command::List {
            title: t,
            in_list: true,
        },
    )
    .await;
    w.one(
        None,
        Command::Rate {
            title: t,
            rating: None,
        },
    )
    .await;
    assert_eq!(w.store().list(w.scope.profile(), 9).await.unwrap(), vec![t]);
    assert!(
        w.store()
            .rated(w.scope.profile(), 9)
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn h23_h56_h86_a_real_change_is_never_dropped_for_a_stale_stamp_of_the_other_field() {
    let w = world(false).await;
    let t = film(1);
    // rate, add, then remove: the removal must land, whatever the rating's time was
    assert_eq!(
        w.one(
            None,
            Command::Rate {
                title: t,
                rating: Some(Rating::Up)
            }
        )
        .await,
        Outcome::Applied
    );
    w.clock.advance_ms(1000);
    assert_eq!(
        w.one(
            None,
            Command::List {
                title: t,
                in_list: true
            }
        )
        .await,
        Outcome::Applied
    );
    w.clock.advance_ms(1000);
    assert_eq!(
        w.one(
            None,
            Command::List {
                title: t,
                in_list: false
            }
        )
        .await,
        Outcome::Applied
    );
    assert!(
        w.store()
            .list(w.scope.profile(), 9)
            .await
            .unwrap()
            .is_empty()
    );
    // add, rate, clear the rating: the clear must land
    w.one(
        None,
        Command::List {
            title: t,
            in_list: true,
        },
    )
    .await;
    w.clock.advance_ms(1000);
    w.one(
        None,
        Command::Rate {
            title: t,
            rating: Some(Rating::Down),
        },
    )
    .await;
    w.clock.advance_ms(1000);
    assert_eq!(
        w.one(
            None,
            Command::Rate {
                title: t,
                rating: None
            }
        )
        .await,
        Outcome::Applied
    );
    assert!(
        w.store()
            .rated(w.scope.profile(), 9)
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn s1_s118_a_late_offline_write_never_moves_another_device_back() {
    let w = world(false).await;
    let t = film(1);
    assert_eq!(w.one(None, pos(t, None, 500, 1000)).await, Outcome::Applied);
    assert_eq!(
        w.one(Some(60_000), pos(t, None, 100, 1000)).await,
        Outcome::Superseded
    );
    let row = w
        .store()
        .position(w.scope.profile(), t, None)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.position_s, 500);
    // equal time overwrites
    assert_eq!(w.one(None, pos(t, None, 600, 1000)).await, Outcome::Applied);
}

#[tokio::test]
async fn s44_a_retry_changes_nothing_and_answers_the_same() {
    let w = world(false).await;
    let t = film(1);
    let first = w
        .named(
            "same",
            None,
            Command::List {
                title: t,
                in_list: true,
            },
        )
        .await;
    w.clock.advance_ms(5000);
    w.one(
        None,
        Command::List {
            title: t,
            in_list: false,
        },
    )
    .await;
    let again = w
        .named(
            "same",
            None,
            Command::List {
                title: t,
                in_list: true,
            },
        )
        .await;
    assert_eq!(first, again);
    assert!(
        w.store()
            .list(w.scope.profile(), 9)
            .await
            .unwrap()
            .is_empty(),
        "the old op did not run twice"
    );
}

#[tokio::test]
async fn s44_age_unknown_orders_by_arrival() {
    let w = world(false).await;
    let t = film(1);
    w.one(None, pos(t, None, 100, 1000)).await;
    w.clock.advance_ms(1);
    assert_eq!(w.one(None, pos(t, None, 50, 1000)).await, Outcome::Applied);
}

#[tokio::test]
async fn h61_no_device_clock_exists_to_lie() {
    let w = world(false).await;
    let t = film(1);
    // a huge age is only ever older, and never reaches a future time
    assert_eq!(
        w.one(Some(u64::MAX), pos(t, None, 10, 100)).await,
        Outcome::Applied
    );
    assert_eq!(
        w.one(Some(0), pos(t, None, 20, 100)).await,
        Outcome::Applied
    );
}

#[tokio::test]
async fn h50_h84_hiding_a_series_covers_every_episode_and_stays_hidden() {
    let w = world(false).await;
    let t = series(7);
    w.one(None, pos(t, ep(1, 1), 100, 1000)).await;
    w.one(None, pos(t, ep(1, 2), 200, 1000)).await;
    w.clock.advance_ms(1000);
    assert_eq!(
        w.one(None, Command::Hide { title: t }).await,
        Outcome::Applied
    );
    assert!(
        w.library
            .continue_watching(w.scope.profile(), 10)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        w.store()
            .recent(w.scope.profile(), 10)
            .await
            .unwrap()
            .is_empty()
    );
    // an old heartbeat that arrives late does not bring it back
    assert_eq!(
        w.one(Some(500), pos(t, ep(1, 2), 300, 1000)).await,
        Outcome::Superseded
    );
    assert!(
        w.library
            .continue_watching(w.scope.profile(), 10)
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn h21_h49_h83_removals_reach_the_other_device() {
    let w = world(false).await;
    let t = series(7);
    w.one(None, pos(t, ep(1, 1), 100, 1000)).await;
    let cursor = w
        .library
        .changes(w.scope.profile(), None)
        .await
        .unwrap()
        .cursor
        .0;
    w.one(None, Command::Hide { title: t }).await;
    let changes = w.changes(&cursor).await;
    assert!(matches!(changes[0], Change::Hidden { title } if title == t));
}

#[tokio::test]
async fn c64_clearing_history_is_one_operation_and_older_writes_stay_cleared() {
    let w = world(false).await;
    w.one(None, pos(film(1), None, 100, 1000)).await;
    w.one(None, pos(series(2), ep(1, 1), 100, 1000)).await;
    w.clock.advance_ms(1000);
    let cursor = w
        .library
        .changes(w.scope.profile(), None)
        .await
        .unwrap()
        .cursor
        .0;
    assert_eq!(w.one(None, Command::ClearHistory).await, Outcome::Applied);
    assert!(
        w.store()
            .recent(w.scope.profile(), 10)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(matches!(
        w.changes(&cursor).await[0],
        Change::HistoryCleared { .. }
    ));
    assert_eq!(
        w.one(Some(500), pos(film(1), None, 200, 1000)).await,
        Outcome::Superseded
    );
    w.clock.advance_ms(1);
    assert_eq!(
        w.one(None, pos(film(1), None, 200, 1000)).await,
        Outcome::Applied
    );
}

#[tokio::test]
async fn c33_c94_continue_watching_is_above_15_seconds_below_95_percent_newest_first() {
    let w = world(false).await;
    w.one(None, pos(film(1), None, 15, 1000)).await;
    w.clock.advance_ms(10);
    w.one(None, pos(film(2), None, 16, 1000)).await;
    w.clock.advance_ms(10);
    w.one(None, pos(film(3), None, 949, 1000)).await;
    w.clock.advance_ms(10);
    w.one(None, pos(film(4), None, 950, 1000)).await;
    let list = w
        .library
        .continue_watching(w.scope.profile(), 10)
        .await
        .unwrap();
    let ids: Vec<u32> = list.iter().map(|c| c.title.id()).collect();
    assert_eq!(ids, [3, 2]);
    assert_eq!(list[0].percent, 94);
}

#[tokio::test]
async fn c21_a_series_shows_its_newest_episode_once() {
    let w = world(false).await;
    let t = series(7);
    w.one(None, pos(t, ep(1, 1), 100, 1000)).await;
    w.clock.advance_ms(10);
    w.one(None, pos(t, ep(1, 2), 200, 1000)).await;
    let list = w
        .library
        .continue_watching(w.scope.profile(), 10)
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].episode, ep(1, 2));
}

#[tokio::test]
async fn c21_resume_is_zero_in_the_last_30_seconds_and_per_episode() {
    let w = world(false).await;
    let t = series(7);
    w.one(None, pos(t, ep(1, 1), 980, 1000)).await;
    assert_eq!(
        w.library
            .start_at_ms(w.scope.profile(), t, ep(1, 1))
            .await
            .unwrap(),
        0
    );
    w.one(None, pos(t, ep(1, 2), 300, 1000)).await;
    assert_eq!(
        w.library
            .start_at_ms(w.scope.profile(), t, ep(1, 2))
            .await
            .unwrap(),
        300_000
    );
    assert_eq!(
        w.library
            .start_at_ms(w.scope.profile(), t, ep(1, 3))
            .await
            .unwrap(),
        0
    );
}

#[tokio::test]
async fn c32_a_position_with_no_duration_or_a_film_episode_is_rejected() {
    let w = world(false).await;
    assert_eq!(
        w.one(None, pos(film(1), None, 10, 0)).await,
        Outcome::Rejected
    );
    assert_eq!(
        w.one(None, pos(film(1), ep(1, 1), 10, 100)).await,
        Outcome::Rejected
    );
}

#[tokio::test]
async fn c65_completed_is_95_percent_and_position_is_clamped() {
    let w = world(false).await;
    let t = film(1);
    w.one(None, pos(t, None, 2000, 1000)).await;
    let row = w
        .store()
        .position(w.scope.profile(), t, None)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.position_s, 1000);
    assert!(row.completed);
}

#[tokio::test]
async fn c36_with_consent_off_feedback_is_discarded_and_still_clears_from_the_queue() {
    let w = world(false).await;
    let outcome = w
        .one(
            None,
            Command::Feedback {
                title: film(1),
                event: FeedbackEvent::Like,
            },
        )
        .await;
    assert_eq!(outcome, Outcome::Applied);
    assert!(
        w.store()
            .taste_of(w.scope.profile())
            .await
            .unwrap()
            .weights
            .is_empty()
    );
}

#[tokio::test]
async fn h26_h37_with_consent_on_taste_is_durable_and_a_dislike_counts() {
    let w = world(true).await;
    w.one(
        None,
        Command::Feedback {
            title: film(1),
            event: FeedbackEvent::Like,
        },
    )
    .await;
    let liked = w.store().taste_of(w.scope.profile()).await.unwrap().weights["Drama"];
    assert!((liked - 1.5).abs() < 1e-6);
    w.one(
        None,
        Command::Rate {
            title: film(2),
            rating: Some(Rating::Down),
        },
    )
    .await;
    assert!(w.store().taste_of(w.scope.profile()).await.unwrap().weights["Drama"] < liked);
}

#[tokio::test]
async fn withdrawal_deletes_taste() {
    let w = world(true).await;
    w.one(
        None,
        Command::Feedback {
            title: film(1),
            event: FeedbackEvent::Like,
        },
    )
    .await;
    w.store().forget_taste(w.scope.profile()).await.unwrap();
    assert!(
        w.store()
            .taste_of(w.scope.profile())
            .await
            .unwrap()
            .weights
            .is_empty()
    );
}

#[tokio::test]
async fn s194_a_bad_setting_is_rejected_and_an_older_one_loses_per_field() {
    let w = world(false).await;
    let set = |key, value: &str| Command::Setting {
        key,
        value: value.to_owned(),
    };
    assert_eq!(
        w.one(None, set(SettingKey::Gender, "robot")).await,
        Outcome::Rejected
    );
    assert_eq!(
        w.one(None, set(SettingKey::Handle, "a b")).await,
        Outcome::Rejected
    );
    assert_eq!(
        w.one(None, set(SettingKey::AudioLanguage, "fr")).await,
        Outcome::Applied
    );
    assert_eq!(
        w.one(Some(5000), set(SettingKey::AudioLanguage, "de"))
            .await,
        Outcome::Superseded
    );
    assert_eq!(
        w.one(Some(5000), set(SettingKey::UiLanguage, "de")).await,
        Outcome::Applied,
        "another field is free"
    );
    let settings = w.store().settings(w.scope.profile()).await.unwrap();
    assert_eq!(settings.audio_language.as_deref(), Some("fr"));
    assert_eq!(settings.ui_language.as_deref(), Some("de"));
}

#[tokio::test]
async fn s117_a_batch_past_the_limit_is_refused_whole() {
    let w = world(false).await;
    let ops = (0..501)
        .map(|n| CommandOp {
            op_id: n.to_string(),
            age_ms: None,
            command: Command::ClearHistory,
        })
        .collect();
    assert!(
        w.library
            .apply(&w.scope, &Facts, &CommandBatch { ops })
            .await
            .is_err()
    );
    assert!(
        w.library
            .changes(w.scope.profile(), None)
            .await
            .unwrap()
            .changes
            .is_empty()
    );
}

#[tokio::test]
async fn s45_changes_page_by_cursor_and_repeat_nothing_after_it() {
    let w = world(false).await;
    for n in 1..=3 {
        w.one(
            None,
            Command::List {
                title: film(n),
                in_list: true,
            },
        )
        .await;
    }
    let first = w.library.changes(w.scope.profile(), None).await.unwrap();
    assert_eq!(first.changes.len(), 3);
    assert!(!first.has_more);
    let later = w
        .library
        .changes(w.scope.profile(), Some(&first.cursor.0))
        .await
        .unwrap();
    assert!(later.changes.is_empty());
    assert_eq!(later.cursor, first.cursor, "S45: a cursor never moves back");
}

#[tokio::test]
async fn s51_old_signals_are_pruned_by_the_job() {
    let w = world(true).await;
    w.one(
        None,
        Command::Feedback {
            title: film(1),
            event: FeedbackEvent::Click,
        },
    )
    .await;
    assert_eq!(w.store().prune_feed(NOW + 1).await.unwrap(), 1);
    assert_eq!(w.store().prune_feed(NOW + 1).await.unwrap(), 0);
}

#[tokio::test]
async fn h22_an_unknown_command_is_rejected_and_the_rest_of_the_batch_runs() {
    let w = world(false).await;
    let batch: CommandBatch = serde_json::from_value(serde_json::json!({
        "ops": [
            { "op_id": "a", "command": "from_a_newer_app", "x": 1 },
            { "op_id": "b", "command": "list", "title": "movie:5", "in_list": true }
        ]
    }))
    .unwrap();
    let answers = w
        .library
        .apply(&w.scope, &Facts, &batch)
        .await
        .unwrap()
        .answers;
    assert_eq!(answers[0].outcome, Outcome::Rejected);
    assert_eq!(answers[1].outcome, Outcome::Applied);
}
