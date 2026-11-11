//! The in memory store, for tests and for running with no database. It holds one lock per call,
//! and a batch works on a copy that replaces the original at commit, as a transaction would.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use contract::command::{Change, Outcome, SettingKey};
use contract::{EpisodeRef, Rating, TitleKey};
use server_accounts::ProfileId;
use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};

use crate::apply::{progress_change, title_change};
use crate::model::{
    EpisodeProgress, FeedRow, HiddenTitle, NotificationReadState, TitleReminder, TitleState,
};
use crate::reads::SettingsView;
use crate::store::{LibraryRead, LibraryStore, LibraryTx, Row};
use crate::taste::Taste;

type ProgressKey = (TitleKey, u32, u32);

fn progress_key(title: TitleKey, episode: Option<EpisodeRef>) -> ProgressKey {
    let (season, number) = episode.map_or((0, 0), |e| (e.season, e.episode));
    (title, season, number)
}

/// Everything of one profile.
#[derive(Clone, Debug, Default)]
struct Profile {
    seq: u64,
    ops: HashMap<String, Outcome>,
    titles: BTreeMap<TitleKey, TitleState>,
    progress: BTreeMap<ProgressKey, EpisodeProgress>,
    hidden: BTreeMap<TitleKey, HiddenTitle>,
    reminders: BTreeMap<TitleKey, TitleReminder>,
    notification_reads: NotificationReadState,
    cleared: Option<(u64, u64)>,
    taste: Taste,
    feed: Vec<FeedRow>,
    settings: SettingsView,
    setting_times: HashMap<SettingKey, (u64, u64)>,
    setting_seq: u64,
    refused_trailers: Vec<(TitleKey, u64)>,
}

/// One library held in memory.
#[derive(Debug, Default)]
pub struct MemoryLibrary {
    profiles: Mutex<HashMap<ProfileId, Profile>>,
    writers: Mutex<HashMap<ProfileId, Arc<AsyncMutex<()>>>>,
}

impl MemoryLibrary {
    fn with<T>(&self, profile: ProfileId, read: impl FnOnce(&Profile) -> T) -> T {
        let mut all = self
            .profiles
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        read(all.entry(profile).or_default())
    }
}

struct MemoryTx {
    store: Arc<MemoryLibrary>,
    id: ProfileId,
    work: Profile,
    _writer: OwnedMutexGuard<()>,
}

fn apply_setting(settings: &mut SettingsView, key: SettingKey, value: &str) {
    let text = || (!value.is_empty()).then(|| value.to_owned());
    match key {
        SettingKey::AutoplayNext => settings.autoplay_next = value == "true",
        SettingKey::AutoplayPreviews => settings.autoplay_previews = value == "true",
        SettingKey::AudioLanguage => settings.audio_language = text(),
        SettingKey::SubtitleLanguage => settings.subtitle_language = text(),
        SettingKey::UiLanguage => settings.ui_language = text(),
        SettingKey::Handle => settings.handle = text(),
        SettingKey::Gender => settings.gender = text(),
        SettingKey::SubtitleStyle => {
            settings.subtitle_style = serde_json::from_str(value).unwrap_or_default();
        }
        _ => {}
    }
}

#[async_trait]
impl LibraryTx for MemoryTx {
    async fn next_seq(&mut self) -> Row<u64> {
        self.work.seq += 1;
        Ok(self.work.seq)
    }
    async fn seen_op(&mut self, op_id: &str) -> Row<Option<Outcome>> {
        Ok(self.work.ops.get(op_id).copied())
    }
    async fn record_op(&mut self, op_id: &str, outcome: Outcome, _now_ms: u64) -> Row<()> {
        self.work.ops.insert(op_id.to_owned(), outcome);
        Ok(())
    }
    async fn title(&mut self, title: TitleKey) -> Row<Option<TitleState>> {
        Ok(self.work.titles.get(&title).cloned())
    }
    async fn put_title(&mut self, row: &TitleState) -> Row<()> {
        self.work.titles.insert(row.title, row.clone());
        Ok(())
    }
    async fn progress(
        &mut self,
        title: TitleKey,
        episode: Option<EpisodeRef>,
    ) -> Row<Option<EpisodeProgress>> {
        Ok(self
            .work
            .progress
            .get(&progress_key(title, episode))
            .cloned())
    }
    async fn put_progress(&mut self, row: &EpisodeProgress) -> Row<()> {
        self.work
            .progress
            .insert(progress_key(row.title, row.episode.clone()), row.clone());
        Ok(())
    }
    async fn hidden(&mut self, title: TitleKey) -> Row<Option<HiddenTitle>> {
        Ok(self.work.hidden.get(&title).copied())
    }
    async fn put_hidden(&mut self, row: &HiddenTitle) -> Row<()> {
        self.work.hidden.insert(row.title, *row);
        Ok(())
    }
    async fn drop_hidden(&mut self, title: TitleKey) -> Row<()> {
        self.work.hidden.remove(&title);
        Ok(())
    }
    async fn clear_history(&mut self, at_ms: u64, seq: u64) -> Row<()> {
        self.work.progress.retain(|_, row| row.changed_ms > at_ms);
        self.work.hidden.retain(|_, row| row.hidden_ms > at_ms);
        self.work.cleared = Some((at_ms, seq));
        Ok(())
    }
    async fn cleared_at(&mut self) -> Row<Option<u64>> {
        Ok(self.work.cleared.map(|(at, _)| at))
    }
    async fn put_setting(
        &mut self,
        key: SettingKey,
        value: &str,
        at_ms: u64,
        seq: u64,
    ) -> Row<bool> {
        if self
            .work
            .setting_times
            .get(&key)
            .is_some_and(|(old, _)| at_ms < *old)
        {
            return Ok(false);
        }
        self.work.setting_times.insert(key, (at_ms, seq));
        apply_setting(&mut self.work.settings, key, value);
        self.work.setting_seq = seq;
        Ok(true)
    }
    async fn taste(&mut self) -> Row<Taste> {
        Ok(self.work.taste.clone())
    }
    async fn put_taste(&mut self, taste: &Taste) -> Row<()> {
        self.work.taste = taste.clone();
        Ok(())
    }
    async fn add_feed(&mut self, row: FeedRow) -> Row<()> {
        self.work.feed.push(row);
        Ok(())
    }
    async fn add_refused_trailer(&mut self, title: TitleKey, at_ms: u64) -> Row<()> {
        self.work.refused_trailers.push((title, at_ms));
        Ok(())
    }
    async fn reminder(&mut self, title: TitleKey) -> Row<Option<TitleReminder>> {
        Ok(self.work.reminders.get(&title).copied())
    }
    async fn put_reminder(&mut self, row: &TitleReminder) -> Row<()> {
        self.work.reminders.insert(row.title, *row);
        Ok(())
    }
    async fn mark_notifications_read(&mut self, up_to: &str, now_ms: u64) -> Row<()> {
        self.work.notification_reads.read_up_to_ms = now_ms;
        if !up_to.is_empty()
            && !self
                .work
                .notification_reads
                .dismissed
                .contains(&up_to.to_string())
        {
            self.work
                .notification_reads
                .dismissed
                .push(up_to.to_string());
        }
        Ok(())
    }
    async fn commit(self: Box<Self>) -> Row<()> {
        let this = *self;
        let mut all = this
            .store
            .profiles
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        all.insert(this.id, this.work);
        Ok(())
    }
}

#[async_trait]
impl LibraryRead for Arc<MemoryLibrary> {
    async fn list(&self, profile: ProfileId, limit: usize) -> Row<Vec<TitleKey>> {
        Ok(self.with(profile, |p| {
            let mut rows: Vec<&TitleState> = p.titles.values().filter(|r| r.in_list).collect();
            rows.sort_by_key(|r| std::cmp::Reverse((r.added_ms, r.seq)));
            rows.into_iter().take(limit).map(|r| r.title).collect()
        }))
    }
    async fn rated(&self, profile: ProfileId, limit: usize) -> Row<Vec<(TitleKey, Rating)>> {
        Ok(self.with(profile, |p| {
            let mut rows: Vec<&TitleState> =
                p.titles.values().filter(|r| r.rating.is_some()).collect();
            rows.sort_by_key(|r| std::cmp::Reverse((r.rating_changed_ms, r.seq)));
            rows.into_iter()
                .take(limit)
                .filter_map(|r| Some((r.title, r.rating?)))
                .collect()
        }))
    }
    async fn recent(&self, profile: ProfileId, limit: usize) -> Row<Vec<EpisodeProgress>> {
        Ok(self.with(profile, |p| {
            let mut rows: Vec<&EpisodeProgress> = p
                .progress
                .values()
                .filter(|r| !p.hidden.contains_key(&r.title))
                .collect();
            rows.sort_by_key(|r| std::cmp::Reverse((r.changed_ms, r.seq)));
            rows.into_iter().take(limit).cloned().collect()
        }))
    }
    async fn positions(&self, profile: ProfileId, limit: usize) -> Row<Vec<EpisodeProgress>> {
        Ok(self.with(profile, |p| {
            let mut rows: Vec<&EpisodeProgress> = p.progress.values().collect();
            rows.sort_by_key(|r| std::cmp::Reverse((r.changed_ms, r.seq)));
            rows.into_iter().take(limit).cloned().collect()
        }))
    }
    async fn hidden_titles(&self, profile: ProfileId, limit: usize) -> Row<Vec<HiddenTitle>> {
        Ok(self.with(profile, |p| {
            let mut rows: Vec<&HiddenTitle> = p.hidden.values().collect();
            rows.sort_by_key(|r| std::cmp::Reverse((r.hidden_ms, r.seq)));
            rows.into_iter().take(limit).cloned().collect()
        }))
    }
    async fn position(
        &self,
        profile: ProfileId,
        title: TitleKey,
        episode: Option<EpisodeRef>,
    ) -> Row<Option<EpisodeProgress>> {
        Ok(self.with(profile, |p| {
            p.progress.get(&progress_key(title, episode)).cloned()
        }))
    }
    async fn taste_of(&self, profile: ProfileId) -> Row<Taste> {
        Ok(self.with(profile, |p| p.taste.clone()))
    }
    async fn seen(&self, profile: ProfileId, limit: usize) -> Row<Vec<TitleKey>> {
        let rows = self.recent(profile, limit).await?;
        Ok(rows.into_iter().map(|r| r.title).collect())
    }
    async fn settings(&self, profile: ProfileId) -> Row<SettingsView> {
        Ok(self.with(profile, |p| p.settings.clone()))
    }
    async fn changes_after(
        &self,
        profile: ProfileId,
        after: u64,
        limit: usize,
    ) -> Row<(Vec<Change>, u64)> {
        Ok(self.with(profile, |p| {
            let mut rows: Vec<(u64, Change)> = Vec::new();
            for row in p.titles.values().filter(|r| r.seq > after) {
                rows.push((row.seq, title_change(row)));
            }
            for row in p.progress.values().filter(|r| r.seq > after) {
                rows.push((
                    row.seq,
                    progress_change(row, p.hidden.contains_key(&row.title)),
                ));
            }
            for row in p.hidden.values().filter(|r| r.seq > after) {
                rows.push((row.seq, Change::Hidden { title: row.title }));
            }
            for row in p.reminders.values().filter(|r| r.seq > after) {
                rows.push((
                    row.seq,
                    Change::Remind {
                        title: row.title,
                        on: row.remind,
                    },
                ));
            }
            if let Some((at, seq)) = p.cleared.filter(|(_, seq)| *seq > after) {
                rows.push((seq, Change::HistoryCleared { changed_at_ms: at }));
            }
            rows.sort_by_key(|(seq, _)| *seq);
            rows.truncate(limit);
            let last = rows.last().map_or(after, |(seq, _)| *seq);
            (rows.into_iter().map(|(_, change)| change).collect(), last)
        }))
    }
    async fn forget_taste(&self, profile: ProfileId) -> Row<()> {
        let mut all = self
            .profiles
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(p) = all.get_mut(&profile) {
            p.taste = Taste::default();
            p.feed.clear();
        }
        Ok(())
    }
    async fn prune_feed(&self, before_ms: u64) -> Row<u64> {
        let mut all = self
            .profiles
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut gone = 0;
        for p in all.values_mut() {
            let before = p.feed.len();
            p.feed.retain(|row| row.at_ms >= before_ms);
            gone += (before - p.feed.len()) as u64;
        }
        Ok(gone)
    }
    async fn reminders(&self, profile: ProfileId, limit: usize) -> Row<Vec<TitleKey>> {
        Ok(self.with(profile, |p| {
            let mut items: Vec<_> = p.reminders.values().filter(|r| r.remind).collect();
            items.sort_by_key(|r| std::cmp::Reverse(r.changed_ms));
            items.into_iter().take(limit).map(|r| r.title).collect()
        }))
    }
    async fn is_reminded(&self, profile: ProfileId, title: TitleKey) -> Row<bool> {
        Ok(self.with(profile, |p| {
            p.reminders.get(&title).is_some_and(|r| r.remind)
        }))
    }
    async fn notification_reads(&self, profile: ProfileId) -> Row<NotificationReadState> {
        Ok(self.with(profile, |p| p.notification_reads.clone()))
    }
}

#[async_trait]
impl LibraryStore for Arc<MemoryLibrary> {
    async fn begin(&self, profile: ProfileId) -> Row<Box<dyn LibraryTx>> {
        let lock = {
            let mut writers = self
                .writers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            writers.entry(profile).or_default().clone()
        };
        let writer = lock.lock_owned().await;
        Ok(Box::new(MemoryTx {
            store: self.clone(),
            id: profile,
            work: self.with(profile, Clone::clone),
            _writer: writer,
        }))
    }

    /// The whole profile goes: one entry holds its list, ratings, positions, hidden titles,
    /// settings, taste, signals, reminders, read marks, refused trailers and op ids, which is the
    /// cascade Postgres does in the schema. The writer lock is dropped with it, so a later batch
    /// for the same id starts from nothing rather than from a counter the rows no longer have.
    async fn delete_profile(&self, profile: ProfileId) -> Row<()> {
        self.profiles
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&profile);
        self.writers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&profile);
        Ok(())
    }
}
