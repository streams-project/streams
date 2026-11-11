//! The account store in memory, for tests and for checks with no database.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::clock::{Clock, SystemClock};
use crate::pin::{self, PinCheck, PinOutcome, PinRow};
use crate::profile::{
    AccountId, NewAccount, Profile, ProfileId, ProfileKind, ProfileLimitsRow, ProfilePatch,
    SessionId, name_key,
};
use crate::session::{Consent, DeviceSession};
use crate::store::{
    AccountNames, AccountStore, DeviceCard, DeviceCodeRow, StoreError, StoreResult,
};
use crate::token::TokenHash;

#[derive(Default)]
struct Rows {
    accounts: HashMap<AccountId, NewAccount>,
    sessions: HashMap<SessionId, DeviceSession>,
    cards: HashMap<SessionId, (String, Option<serde_json::Value>)>,
    profiles: HashMap<ProfileId, Profile>,
    pins: HashMap<ProfileId, PinRow>,
    /// The key each device keeps for itself, as Postgres keeps it in `account_devices`.
    devices: HashMap<(AccountId, String), SessionId>,
    /// The audit log: the account, the event and the time it was written, so the retention this
    /// store is asked for can be the retention it keeps.
    events: Vec<(AccountId, String, u64)>,
    device_codes: Vec<DeviceCodeRow>,
    names: HashMap<AccountId, AccountNames>,
    /// `account_details.profile_transfer`: absent counts as the column's default, true.
    transfer_off: HashMap<AccountId, bool>,
}

/// Rows in a mutex. A test can stop it to prove an outage is not read as a sign out.
pub struct MemoryStore {
    rows: Mutex<Rows>,
    down: AtomicBool,
    /// Stamps the audit rows. Postgres stamps them with `now()`; this store is given a clock so
    /// the same prune call means the same thing on both.
    clock: Arc<dyn Clock>,
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self {
            rows: Mutex::default(),
            down: AtomicBool::default(),
            clock: Arc::new(SystemClock),
        }
    }
}

impl std::fmt::Debug for MemoryStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MemoryStore")
    }
}

impl MemoryStore {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// An empty store whose audit rows are stamped by this clock.
    #[must_use]
    pub fn with_clock(clock: Arc<dyn Clock>) -> Self {
        Self {
            clock,
            ..Self::default()
        }
    }

    /// Turn the account holder's answer about moving a profile on or off. The column exists and
    /// no route writes it, so this stands for an operator setting it by hand, which is the only
    /// way it is set today.
    pub fn set_profile_transfer(&self, account: AccountId, allowed: bool) {
        self.rows
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .transfer_off
            .insert(account, allowed);
    }

    /// Make every call fail as an outage, or work again.
    pub fn set_down(&self, down: bool) {
        self.down.store(down, Ordering::SeqCst);
    }

    /// The session a device key was last seen on, as the store kept it.
    #[must_use]
    pub fn device_session(&self, account: AccountId, device_key: &str) -> Option<SessionId> {
        let rows = self
            .rows
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        rows.devices.get(&(account, device_key.to_owned())).copied()
    }

    /// Audit rows of an account, oldest first.
    #[must_use]
    pub fn events(&self, account: AccountId) -> Vec<String> {
        let rows = self
            .rows
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        rows.events
            .iter()
            .filter(|(owner, _, _)| *owner == account)
            .map(|(_, event, _)| event.clone())
            .collect()
    }

    fn guard(&self) -> StoreResult<std::sync::MutexGuard<'_, Rows>> {
        if self.down.load(Ordering::SeqCst) {
            return Err(StoreError::Unavailable);
        }
        Ok(self
            .rows
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner))
    }
}

fn name_taken(rows: &Rows, account: AccountId, name: &str, except: Option<ProfileId>) -> bool {
    rows.profiles.values().any(|row| {
        row.account == account && Some(row.id) != except && name_key(&row.name) == name_key(name)
    })
}

#[async_trait]
impl AccountStore for MemoryStore {
    async fn create_account(&self, account: &NewAccount, profiles: &[Profile]) -> StoreResult<()> {
        let mut rows = self.guard()?;
        rows.accounts.insert(account.id, account.clone());
        rows.names.insert(
            account.id,
            AccountNames {
                first_name: account.first_name.clone(),
                last_name: account.last_name.clone(),
            },
        );
        for profile in profiles {
            rows.profiles.insert(profile.id, profile.clone());
        }
        Ok(())
    }

    async fn insert_session(&self, session: &DeviceSession) -> StoreResult<()> {
        self.guard()?.sessions.insert(session.id, session.clone());
        Ok(())
    }

    async fn session_by_access(&self, hash: &TokenHash) -> StoreResult<Option<DeviceSession>> {
        let rows = self.guard()?;
        Ok(rows
            .sessions
            .values()
            .find(|row| row.access_hash == *hash)
            .cloned())
    }

    async fn session(&self, id: SessionId) -> StoreResult<Option<DeviceSession>> {
        Ok(self.guard()?.sessions.get(&id).cloned())
    }

    async fn update_tokens(
        &self,
        session: &DeviceSession,
        from_current: &TokenHash,
        from_next: &TokenHash,
    ) -> StoreResult<bool> {
        let mut rows = self.guard()?;
        let row = rows
            .sessions
            .get_mut(&session.id)
            .ok_or(StoreError::NotFound)?;
        // The same test as the Postgres where: a snapshot that lost the race writes nothing.
        if row.refresh_current != *from_current || row.refresh_next != *from_next {
            return Ok(false);
        }
        row.access_hash = session.access_hash;
        row.access_expires_ms = session.access_expires_ms;
        row.refresh_current = session.refresh_current;
        row.refresh_next = session.refresh_next;
        row.refresh_first_used_ms = session.refresh_first_used_ms;
        row.last_seen_ms = session.last_seen_ms;
        Ok(true)
    }

    async fn set_active_profile(
        &self,
        session: SessionId,
        profile: ProfileId,
        at_ms: u64,
    ) -> StoreResult<()> {
        let mut rows = self.guard()?;
        let row = rows
            .sessions
            .get_mut(&session)
            .ok_or(StoreError::NotFound)?;
        row.active_profile = Some(profile);
        row.entered_at_ms = Some(at_ms);
        Ok(())
    }

    async fn set_step_up(&self, session: SessionId, at_ms: u64, method: &str) -> StoreResult<()> {
        let mut rows = self.guard()?;
        let row = rows
            .sessions
            .get_mut(&session)
            .ok_or(StoreError::NotFound)?;
        row.step_up_at_ms = Some(at_ms);
        row.step_up_method = Some(method.to_owned());
        Ok(())
    }

    async fn delete_session(&self, account: AccountId, id: SessionId) -> StoreResult<bool> {
        let mut rows = self.guard()?;
        if rows
            .sessions
            .get(&id)
            .is_some_and(|row| row.account == account)
        {
            rows.sessions.remove(&id);
            return Ok(true);
        }
        Ok(false)
    }

    async fn delete_sessions(&self, account: AccountId) -> StoreResult<()> {
        self.guard()?
            .sessions
            .retain(|_, row| row.account != account);
        Ok(())
    }

    async fn delete_other_sessions(&self, account: AccountId, keep: SessionId) -> StoreResult<()> {
        self.guard()?
            .sessions
            .retain(|id, row| row.account != account || *id == keep);
        Ok(())
    }

    async fn sessions_of(&self, account: AccountId) -> StoreResult<Vec<DeviceSession>> {
        let rows = self.guard()?;
        let mut list: Vec<_> = rows
            .sessions
            .values()
            .filter(|row| row.account == account)
            .cloned()
            .collect();
        list.sort_by_key(|row| std::cmp::Reverse(row.last_seen_ms));
        Ok(list)
    }

    async fn profile(&self, account: AccountId, id: ProfileId) -> StoreResult<Option<Profile>> {
        let rows = self.guard()?;
        Ok(rows
            .profiles
            .get(&id)
            .filter(|row| row.account == account)
            .cloned())
    }

    async fn profiles_of(&self, account: AccountId) -> StoreResult<Vec<Profile>> {
        let rows = self.guard()?;
        let mut list: Vec<_> = rows
            .profiles
            .values()
            .filter(|row| row.account == account)
            .cloned()
            .collect();
        list.sort_by_key(|row| row.sort_order);
        Ok(list)
    }

    async fn insert_profile(&self, profile: &Profile, max: u32) -> StoreResult<()> {
        let mut rows = self.guard()?;
        let count = rows
            .profiles
            .values()
            .filter(|row| row.account == profile.account)
            .count();
        if count >= max as usize {
            return Err(StoreError::LimitReached);
        }
        if name_taken(&rows, profile.account, &profile.name, None) {
            return Err(StoreError::NameTaken);
        }
        rows.profiles.insert(profile.id, profile.clone());
        Ok(())
    }

    async fn update_profile(
        &self,
        account: AccountId,
        id: ProfileId,
        patch: &ProfilePatch,
    ) -> StoreResult<Profile> {
        let mut rows = self.guard()?;
        if let Some(name) = &patch.name
            && name_taken(&rows, account, name, Some(id))
        {
            return Err(StoreError::NameTaken);
        }
        let row = rows
            .profiles
            .get_mut(&id)
            .filter(|row| row.account == account)
            .ok_or(StoreError::NotFound)?;
        if let Some(name) = &patch.name {
            row.name.clone_from(name);
        }
        if let Some(avatar) = &patch.avatar {
            row.avatar.clone_from(avatar);
        }
        if let Some(kind) = patch.kind {
            row.kind = kind;
            if kind == ProfileKind::Kids {
                row.maturity = String::from("older_kids");
                row.age_limit = String::from("PG");
            } else {
                row.maturity = String::from("adult");
            }
        }
        Ok(row.clone())
    }

    async fn delete_profile(&self, account: AccountId, id: ProfileId) -> StoreResult<()> {
        let mut rows = self.guard()?;
        if !rows
            .profiles
            .get(&id)
            .is_some_and(|row| row.account == account)
        {
            return Err(StoreError::NotFound);
        }
        rows.profiles.remove(&id);
        rows.pins.remove(&id);
        for session in rows.sessions.values_mut() {
            if session.active_profile == Some(id) {
                session.active_profile = None;
                session.entered_at_ms = None;
            }
        }
        Ok(())
    }

    async fn set_main_profile(&self, account: AccountId, id: ProfileId) -> StoreResult<()> {
        let mut rows = self.guard()?;
        for row in rows
            .profiles
            .values_mut()
            .filter(|row| row.account == account)
        {
            row.is_main = row.id == id;
        }
        Ok(())
    }

    async fn pin_attempt(
        &self,
        profile: ProfileId,
        check: PinCheck<'_>,
    ) -> StoreResult<PinOutcome> {
        let mut rows = self.guard()?;
        let Some(row) = rows.pins.get(&profile).cloned() else {
            return Ok(PinOutcome::Pass);
        };
        let at = self.clock.now_ms();
        let (outcome, next) = pin::decide(&row, check);
        if let Some(next) = next {
            let locked =
                next.locked_until_ms != row.locked_until_ms && next.locked_until_ms.is_some();
            rows.pins.insert(profile, next);
            if locked {
                let owner = rows.profiles.get(&profile).map(|row| row.account);
                if let Some(owner) = owner {
                    rows.events.push((owner, String::from("pin_locked"), at));
                }
            }
        }
        Ok(outcome)
    }

    async fn set_pin(&self, profile: ProfileId, hash: Option<String>) -> StoreResult<()> {
        let mut rows = self.guard()?;
        let row = rows
            .profiles
            .get_mut(&profile)
            .ok_or(StoreError::NotFound)?;
        row.has_pin = hash.is_some();
        match hash {
            Some(hash) => {
                rows.pins.insert(
                    profile,
                    PinRow {
                        hash,
                        failed: 0,
                        locked_until_ms: None,
                        locks: 0,
                    },
                );
            }
            None => {
                rows.pins.remove(&profile);
            }
        }
        Ok(())
    }

    async fn set_limits(
        &self,
        account: AccountId,
        profile: ProfileId,
        limits: &ProfileLimitsRow,
    ) -> StoreResult<()> {
        let mut rows = self.guard()?;
        let row = rows
            .profiles
            .get_mut(&profile)
            .filter(|row| row.account == account)
            .ok_or(StoreError::NotFound)?;
        row.age_limit.clone_from(&limits.age_limit);
        row.blocked_names.clone_from(&limits.blocked_names);
        Ok(())
    }

    async fn set_consent(&self, session: SessionId, consent: Consent) -> StoreResult<()> {
        let mut rows = self.guard()?;
        let row = rows
            .sessions
            .get_mut(&session)
            .ok_or(StoreError::NotFound)?;
        row.consent = consent;
        Ok(())
    }

    async fn log_event(&self, account: AccountId, event: &str) -> StoreResult<()> {
        let at = self.clock.now_ms();
        self.guard()?.events.push((account, event.to_owned(), at));
        Ok(())
    }

    async fn events_of(&self, account: AccountId, limit: usize) -> StoreResult<Vec<String>> {
        let rows = self.guard()?;
        let mine = rows
            .events
            .iter()
            .rev()
            .filter(|(owner, _, _)| *owner == account);
        Ok(mine
            .take(limit)
            .map(|(_, event, _)| event.clone())
            .collect())
    }

    async fn names(&self, account: AccountId) -> StoreResult<AccountNames> {
        let rows = self.guard()?;
        Ok(rows.names.get(&account).cloned().unwrap_or_default())
    }

    async fn profile_transfer_allowed(&self, account: AccountId) -> StoreResult<bool> {
        Ok(*self.guard()?.transfer_off.get(&account).unwrap_or(&true))
    }

    async fn set_names(
        &self,
        account: AccountId,
        names: &AccountNames,
        main: Option<(ProfileId, String)>,
    ) -> StoreResult<()> {
        let mut rows = self.guard()?;
        rows.names.insert(account, names.clone());
        if let Some((id, name)) = main
            && let Some(row) = rows.profiles.get_mut(&id)
        {
            row.name = name;
        }
        Ok(())
    }

    /// Every row of the account goes, which is what the foreign keys do in Postgres. The rows
    /// keyed to a profile or a session of this account have to be found first, because nothing in
    /// this store holds the account on them: `profile_pins` is keyed by profile, and the card in
    /// `device_sessions` by session. Leaving either behind made the double answer differently
    /// from the database it stands in for, and account deletion is only ever tested on the
    /// double.
    async fn delete_account(&self, account: AccountId) -> StoreResult<()> {
        let mut rows = self.guard()?;
        let profiles: Vec<ProfileId> = rows
            .profiles
            .values()
            .filter(|row| row.account == account)
            .map(|row| row.id)
            .collect();
        let sessions: Vec<SessionId> = rows
            .sessions
            .values()
            .filter(|row| row.account == account)
            .map(|row| row.id)
            .collect();
        rows.accounts.remove(&account);
        rows.names.remove(&account);
        rows.transfer_off.remove(&account);
        for profile in profiles {
            rows.pins.remove(&profile);
        }
        for session in sessions {
            rows.cards.remove(&session);
        }
        rows.sessions.retain(|_, row| row.account != account);
        rows.profiles.retain(|_, row| row.account != account);
        rows.events.retain(|(owner, _, _)| *owner != account);
        rows.devices.retain(|(owner, _), _| *owner != account);
        rows.device_codes.retain(|row| row.account != Some(account));
        Ok(())
    }

    async fn move_profile(
        &self,
        from: AccountId,
        profile: ProfileId,
        to: AccountId,
        max: u32,
    ) -> StoreResult<()> {
        let mut rows = self.guard()?;
        let held = rows
            .profiles
            .values()
            .filter(|row| row.account == to)
            .count();
        if held >= max as usize {
            return Err(StoreError::LimitReached);
        }
        let name = rows
            .profiles
            .get(&profile)
            .map(|row| row.name.clone())
            .ok_or(StoreError::NotFound)?;
        if name_taken(&rows, to, &name, Some(profile)) {
            return Err(StoreError::NameTaken);
        }
        let row = rows
            .profiles
            .get_mut(&profile)
            .ok_or(StoreError::NotFound)?;
        if row.account != from {
            return Err(StoreError::NotFound);
        }
        row.account = to;
        // As Postgres does it in its transaction: the PIN belongs to the account that set it, and
        // the automatic Kids flag is one per account, so neither travels with the row.
        row.auto_kids = false;
        row.has_pin = false;
        rows.pins.remove(&profile);
        for session in rows.sessions.values_mut() {
            if session.active_profile == Some(profile) {
                session.active_profile = None;
            }
        }
        Ok(())
    }

    async fn insert_device_code(&self, row: &DeviceCodeRow) -> StoreResult<()> {
        self.guard()?.device_codes.push(row.clone());
        Ok(())
    }

    async fn device_code(&self, hash: &TokenHash) -> StoreResult<Option<DeviceCodeRow>> {
        let rows = self.guard()?;
        Ok(rows
            .device_codes
            .iter()
            .find(|row| row.device_hash == *hash)
            .cloned())
    }

    async fn device_code_by_user(&self, user_code: &str) -> StoreResult<Option<DeviceCodeRow>> {
        let rows = self.guard()?;
        Ok(rows
            .device_codes
            .iter()
            .find(|row| row.user_code == user_code)
            .cloned())
    }

    async fn note_device_poll(&self, hash: &TokenHash, at_ms: u64) -> StoreResult<()> {
        let mut rows = self.guard()?;
        if let Some(held) = rows
            .device_codes
            .iter_mut()
            .find(|held| held.device_hash == *hash)
        {
            held.last_poll_ms = Some(at_ms);
        }
        Ok(())
    }

    async fn approve_device_code(&self, hash: &TokenHash, account: AccountId) -> StoreResult<bool> {
        let mut rows = self.guard()?;
        let held = rows
            .device_codes
            .iter_mut()
            .find(|held| held.device_hash == *hash && held.account.is_none() && !held.denied);
        Ok(held.map(|held| held.account = Some(account)).is_some())
    }

    async fn deny_device_code(&self, hash: &TokenHash) -> StoreResult<bool> {
        let mut rows = self.guard()?;
        let held = rows
            .device_codes
            .iter_mut()
            .find(|held| held.device_hash == *hash && held.account.is_none() && !held.denied);
        Ok(held.map(|held| held.denied = true).is_some())
    }

    async fn delete_device_code(&self, hash: &TokenHash) -> StoreResult<bool> {
        let mut rows = self.guard()?;
        let before = rows.device_codes.len();
        rows.device_codes.retain(|row| row.device_hash != *hash);
        Ok(rows.device_codes.len() < before)
    }

    /// Sessions unused since a time, audit rows older than a time, and device codes past their
    /// life. The audit row half was taken and dropped before, so the 365 day rule was proved on
    /// no store that CI runs.
    async fn prune(
        &self,
        idle_before_ms: u64,
        events_before_ms: u64,
        now_ms: u64,
    ) -> StoreResult<u64> {
        let mut rows = self.guard()?;
        let before = rows.sessions.len() + rows.device_codes.len() + rows.events.len();
        rows.sessions
            .retain(|_, row| row.last_seen_ms >= idle_before_ms);
        rows.device_codes.retain(|row| row.expires_ms > now_ms);
        rows.events.retain(|(_, _, at)| *at >= events_before_ms);
        let after = rows.sessions.len() + rows.device_codes.len() + rows.events.len();
        Ok((before - after) as u64)
    }

    async fn card(&self, session: SessionId) -> StoreResult<Option<serde_json::Value>> {
        let rows = self.guard()?;
        Ok(rows.cards.get(&session).and_then(|(_, card)| card.clone()))
    }

    async fn save_card(&self, seen: &DeviceCard<'_>) -> StoreResult<bool> {
        let mut rows = self.guard()?;
        let known = rows
            .cards
            .get(&seen.session)
            .is_some_and(|(hash, stored)| hash == seen.card_hash && stored.is_some());
        if !known {
            rows.cards.insert(
                seen.session,
                (seen.card_hash.to_owned(), seen.card.cloned()),
            );
        }
        rows.devices
            .insert((seen.account, seen.device_key.to_owned()), seen.session);
        Ok(!known && seen.card.is_none())
    }
}
