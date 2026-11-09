//! The one place a command changes a profile's library. Both stores run this code.

use std::sync::Arc;

use contract::command::Command;
use contract::command::{
    Change, Changes, CommandAnswer, CommandBatch, FeedbackEvent, OpAnswer, Outcome, SettingKey,
};
use contract::screen::Cursor;
use contract::{EpisodeRef, Rating, TitleKey, TitleKind, is_completed, watch_percent};
use server_accounts::{Clock, ProfileId, ProfileScope};

use crate::model::{
    Continue, EpisodeProgress, FeedRow, HiddenTitle, Limits, Reached, TitleReminder, TitleState,
    age_to_time,
};
use crate::store::{LibraryError, LibraryStore, LibraryTx, Row};
use crate::taste::{DISLIKE_WEIGHT, event_weight, learn};

/// Positions read to find a title's newest one.
const RECENT_SCAN: usize = 500;

/// What the library may ask about a title. The server passes this in, so this crate never reads
/// the catalogue.
pub trait TitleFacts: Send + Sync {
    /// The genres of a title. Empty when the title is unknown.
    fn genres(&self, title: TitleKey) -> Vec<String>;
}

/// The library over one store.
#[derive(Clone)]
pub struct Library {
    store: Arc<dyn LibraryStore>,
    clock: Arc<dyn Clock>,
    limits: Limits,
}

impl std::fmt::Debug for Library {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Library")
            .field("limits", &self.limits)
            .finish()
    }
}

/// True for a setting value the server accepts.
fn setting_ok(key: SettingKey, value: &str) -> bool {
    match key {
        SettingKey::AutoplayNext | SettingKey::AutoplayPreviews => {
            matches!(value, "true" | "false")
        }
        SettingKey::Handle => {
            (3..=16).contains(&value.len()) && value.chars().all(|c| c.is_ascii_alphanumeric())
        }
        SettingKey::Gender => matches!(value, "woman" | "man" | "none"),
        SettingKey::SubtitleStyle => {
            serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(value).is_ok()
        }
        SettingKey::AudioLanguage | SettingKey::SubtitleLanguage | SettingKey::UiLanguage => {
            value.len() <= 20
        }
        _ => false,
    }
}

/// True when a command's ids and duration are ones the server keeps.
fn position_ok(title: TitleKey, episode: Option<&EpisodeRef>, duration_s: u32) -> bool {
    let series = title.kind() == TitleKind::Tv;
    duration_s > 0 && (series || episode.is_none())
}

impl Library {
    /// A library with the default limits.
    #[must_use]
    pub fn new(store: Arc<dyn LibraryStore>, clock: Arc<dyn Clock>) -> Self {
        Self {
            store,
            clock,
            limits: Limits::default(),
        }
    }

    /// The store, for reads.
    #[must_use]
    pub fn store(&self) -> &Arc<dyn LibraryStore> {
        &self.store
    }

    /// Apply a batch with one answer per op, in order. An op id seen before gets its old answer,
    /// so a retry changes nothing. Consent decides whether signals are kept.
    ///
    /// # Errors
    /// `BatchTooLarge` for a batch past the limit, or the store's failure. Nothing is kept then.
    pub async fn apply(
        &self,
        scope: &ProfileScope,
        facts: &dyn TitleFacts,
        batch: &CommandBatch,
    ) -> Row<CommandAnswer> {
        if batch.ops.len() > self.limits.batch_max {
            return Err(LibraryError::BatchTooLarge);
        }
        let now = self.clock.now_ms();
        let consent = scope.consent().personal;
        let mut tx = self.store.begin(scope.profile()).await?;
        let mut answers = Vec::with_capacity(batch.ops.len());
        for op in &batch.ops {
            let outcome = if op.op_id.is_empty() || op.op_id.len() > self.limits.op_id_max {
                Outcome::Rejected
            } else if let Some(old) = tx.seen_op(&op.op_id).await? {
                old
            } else {
                let at = age_to_time(now, op.age_ms);
                let outcome = run(tx.as_mut(), &op.command, at, consent, facts).await?;
                tx.record_op(&op.op_id, outcome, now).await?;
                outcome
            };
            answers.push(OpAnswer {
                op_id: op.op_id.clone(),
                outcome,
            });
        }
        tx.commit().await?;
        Ok(CommandAnswer { answers })
    }

    /// Changes after a cursor. The cursor is a change number, so a slow writer can never be
    /// skipped the way a time cursor could; repeats are harmless.
    ///
    /// # Errors
    /// The store's failure.
    pub async fn changes(&self, profile: ProfileId, cursor: Option<&str>) -> Row<Changes> {
        let after = cursor.and_then(|text| text.parse().ok()).unwrap_or(0);
        let limit = self.limits.changes_page;
        let (changes, last) = self.store.changes_after(profile, after, limit).await?;
        let has_more = changes.len() >= limit;
        Ok(Changes {
            changes,
            cursor: Cursor(last.max(after).to_string()),
            has_more,
        })
    }

    /// Continue Watching: position above 15 s and below 95 percent, newest first, one entry per
    /// title, hidden titles left out.
    ///
    /// # Errors
    /// The store's failure.
    pub async fn continue_watching(&self, profile: ProfileId, limit: usize) -> Row<Vec<Continue>> {
        let recent = self.store.recent(profile, limit.saturating_mul(8)).await?;
        let mut seen: Vec<TitleKey> = Vec::new();
        let mut out = Vec::new();
        for row in recent {
            if seen.contains(&row.title) {
                continue;
            }
            seen.push(row.title);
            if row.position_s > self.limits.continue_min_s && !row.completed {
                let percent = watch_percent(row.position_s.into(), row.duration_s.into());
                out.push(Continue {
                    title: row.title,
                    episode: row.episode.clone(),
                    percent: percent.map_or(0, u32::from),
                    changed_ms: row.changed_ms,
                });
            }
            if out.len() >= limit {
                break;
            }
        }
        Ok(out)
    }

    /// The newest place this profile reached in a title, hidden titles left out.
    ///
    /// # Errors
    /// The store's failure.
    pub async fn last_watched(&self, profile: ProfileId, title: TitleKey) -> Row<Option<Reached>> {
        let recent = self.store.recent(profile, RECENT_SCAN).await?;
        Ok(recent
            .into_iter()
            .find(|row| row.title == title)
            .map(|row| Reached {
                finished: row.completed || near_end(row.position_s, row.duration_s),
                position_s: row.position_s,
                episode: row.episode,
            }))
    }

    /// Where to start: 0 when a video longer than a minute is within its last 30 s; a series
    /// resumes only for the same episode.
    ///
    /// # Errors
    /// The store's failure.
    pub async fn start_at_ms(
        &self,
        profile: ProfileId,
        title: TitleKey,
        episode: Option<EpisodeRef>,
    ) -> Row<u64> {
        let Some(row) = self.store.position(profile, title, episode).await? else {
            return Ok(0);
        };
        Ok(if near_end(row.position_s, row.duration_s) {
            0
        } else {
            u64::from(row.position_s) * 1000
        })
    }
}

/// True for a video longer than a minute within its last 30 s.
const fn near_end(position_s: u32, duration_s: u32) -> bool {
    duration_s > 60 && position_s + 30 >= duration_s
}

async fn run(
    tx: &mut dyn LibraryTx,
    command: &Command,
    at: u64,
    consent: bool,
    facts: &dyn TitleFacts,
) -> Row<Outcome> {
    match command {
        Command::List { title, in_list } => set_list(tx, *title, *in_list, at).await,
        Command::Rate { title, rating } => {
            set_rating(tx, *title, *rating, at, consent, facts).await
        }
        Command::Hide { title } => hide(tx, *title, at).await,
        Command::ClearHistory => clear(tx, at).await,
        Command::Position {
            title,
            episode,
            position_s,
            duration_s,
        } => position(tx, *title, episode.clone(), *position_s, *duration_s, at).await,
        Command::Setting { key, value } => setting(tx, *key, value, at).await,
        Command::Feedback { title, event } => {
            feedback(tx, *title, *event, at, consent, facts).await
        }
        Command::TrailerRefused { title } => {
            tx.add_refused_trailer(*title, at).await?;
            Ok(Outcome::Applied)
        }
        Command::Remind { title, on } => set_remind(tx, *title, *on, at).await,
        _ => Ok(Outcome::Rejected),
    }
}

async fn set_remind(tx: &mut dyn LibraryTx, title: TitleKey, on: bool, at: u64) -> Row<Outcome> {
    let mut row = tx.reminder(title).await?.unwrap_or(TitleReminder {
        title,
        remind: false,
        version: 0,
        changed_ms: 0,
        seq: 0,
    });
    if at < row.changed_ms {
        return Ok(Outcome::Superseded);
    }
    row.remind = on;
    row.version += 1;
    row.changed_ms = at;
    row.seq = tx.next_seq().await?;
    tx.put_reminder(&row).await?;
    Ok(Outcome::Applied)
}

/// List and rating keep apart: a write here never reads or touches the rating. A time equal
/// to the stored one overwrites; an older one changes nothing.
async fn set_list(tx: &mut dyn LibraryTx, title: TitleKey, in_list: bool, at: u64) -> Row<Outcome> {
    let mut row = tx
        .title(title)
        .await?
        .unwrap_or_else(|| TitleState::empty(title));
    if at < row.list_changed_ms {
        return Ok(Outcome::Superseded);
    }
    row.in_list = in_list;
    row.list_version += 1;
    row.list_changed_ms = at;
    if in_list {
        row.added_ms = row.added_ms.or(Some(at));
    } else {
        row.added_ms = None;
    }
    row.seq = tx.next_seq().await?;
    tx.put_title(&row).await?;
    Ok(Outcome::Applied)
}

async fn set_rating(
    tx: &mut dyn LibraryTx,
    title: TitleKey,
    rating: Option<Rating>,
    at: u64,
    consent: bool,
    facts: &dyn TitleFacts,
) -> Row<Outcome> {
    if matches!(rating, Some(Rating::Unknown)) {
        return Ok(Outcome::Rejected);
    }
    let mut row = tx
        .title(title)
        .await?
        .unwrap_or_else(|| TitleState::empty(title));
    if at < row.rating_changed_ms {
        return Ok(Outcome::Superseded);
    }
    row.rating = rating;
    row.rating_version += 1;
    row.rating_changed_ms = at;
    row.seq = tx.next_seq().await?;
    tx.put_title(&row).await?;
    if consent && rating == Some(Rating::Down) {
        let mut taste = tx.taste().await?;
        learn(&mut taste, &facts.genres(title), DISLIKE_WEIGHT);
        tx.put_taste(&taste).await?;
    }
    Ok(Outcome::Applied)
}

async fn hide(tx: &mut dyn LibraryTx, title: TitleKey, at: u64) -> Row<Outcome> {
    if tx
        .hidden(title)
        .await?
        .is_some_and(|row| at < row.hidden_ms)
    {
        return Ok(Outcome::Superseded);
    }
    let seq = tx.next_seq().await?;
    tx.put_hidden(&HiddenTitle {
        title,
        hidden_ms: at,
        seq,
    })
    .await?;
    Ok(Outcome::Applied)
}

async fn clear(tx: &mut dyn LibraryTx, at: u64) -> Row<Outcome> {
    if tx.cleared_at().await?.is_some_and(|old| at < old) {
        return Ok(Outcome::Superseded);
    }
    let seq = tx.next_seq().await?;
    tx.clear_history(at, seq).await?;
    Ok(Outcome::Applied)
}

/// Save a position. A write from before history was cleared stays cleared, and a later
/// position brings a hidden title back, as a new viewing of it.
async fn position(
    tx: &mut dyn LibraryTx,
    title: TitleKey,
    episode: Option<EpisodeRef>,
    position_s: u32,
    duration_s: u32,
    at: u64,
) -> Row<Outcome> {
    if !position_ok(title, episode.as_ref(), duration_s) {
        return Ok(Outcome::Rejected);
    }
    if tx.cleared_at().await?.is_some_and(|cleared| at < cleared) {
        return Ok(Outcome::Superseded);
    }
    let old = tx.progress(title, episode.clone()).await?;
    if old.as_ref().is_some_and(|row| at < row.changed_ms) {
        return Ok(Outcome::Superseded);
    }
    if let Some(hidden) = tx.hidden(title).await? {
        if at < hidden.hidden_ms {
            return Ok(Outcome::Superseded);
        }
        tx.drop_hidden(title).await?;
    }
    let position_s = position_s.min(duration_s);
    let row = EpisodeProgress {
        title,
        episode,
        position_s,
        duration_s,
        completed: is_completed(position_s.into(), duration_s.into()),
        version: old.map_or(0, |row| row.version) + 1,
        changed_ms: at,
        seq: tx.next_seq().await?,
    };
    tx.put_progress(&row).await?;
    Ok(Outcome::Applied)
}

async fn setting(tx: &mut dyn LibraryTx, key: SettingKey, value: &str, at: u64) -> Row<Outcome> {
    if !setting_ok(key, value) {
        return Ok(Outcome::Rejected);
    }
    let seq = tx.next_seq().await?;
    let written = tx.put_setting(key, value, at, seq).await?;
    Ok(if written {
        Outcome::Applied
    } else {
        Outcome::Superseded
    })
}

/// With consent off nothing is kept and the op still clears from the device's queue.
async fn feedback(
    tx: &mut dyn LibraryTx,
    title: TitleKey,
    event: FeedbackEvent,
    at: u64,
    consent: bool,
    facts: &dyn TitleFacts,
) -> Row<Outcome> {
    let Some(weight) = event_weight(event) else {
        return Ok(Outcome::Rejected);
    };
    if consent {
        tx.add_feed(FeedRow {
            title,
            event,
            at_ms: at,
        })
        .await?;
        let mut taste = tx.taste().await?;
        learn(&mut taste, &facts.genres(title), weight);
        tx.put_taste(&taste).await?;
    }
    Ok(Outcome::Applied)
}

/// Turn a row into the change a device reads.
#[must_use]
pub(crate) fn title_change(row: &TitleState) -> Change {
    use contract::command::{ListState, RatingState};
    Change::Title {
        title: row.title,
        list: ListState {
            in_list: row.in_list,
            version: row.list_version,
            changed_at_ms: row.list_changed_ms,
        },
        rating: RatingState {
            rating: row.rating,
            version: row.rating_version,
            changed_at_ms: row.rating_changed_ms,
        },
    }
}

/// Turn a position into the change a device reads.
#[must_use]
pub(crate) fn progress_change(row: &EpisodeProgress, hidden: bool) -> Change {
    use contract::command::ProgressState;
    Change::Progress {
        state: ProgressState {
            title: row.title,
            episode: row.episode.clone(),
            position_s: row.position_s,
            duration_s: row.duration_s,
            completed: row.completed,
            hidden,
            version: row.version,
            changed_at_ms: row.changed_ms,
        },
    }
}
