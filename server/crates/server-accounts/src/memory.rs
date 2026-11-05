//! The account store in memory, for tests and for checks with no database.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;

use crate::pin::{self, PinCheck, PinOutcome, PinRow};
use crate::profile::{
    AccountId, NewAccount, Profile, ProfileId, ProfileKind, ProfileLimitsRow, ProfilePatch,
    SessionId, name_key,
};
use crate::session::{Consent, DeviceSession};
use crate::store::{AccountStore, StoreError, StoreResult};
use crate::token::TokenHash;

#[derive(Default)]
struct Rows {
    accounts: HashMap<AccountId, NewAccount>,
    sessions: HashMap<SessionId, DeviceSession>,
    cards: HashMap<SessionId, (String, Option<serde_json::Value>)>,
    profiles: HashMap<ProfileId, Profile>,
    pins: HashMap<ProfileId, PinRow>,
    events: Vec<(AccountId, String)>,
}

/// Rows in a mutex. A test can stop it to prove an outage is not read as a sign out.
#[derive(Default)]
pub struct MemoryStore {
    rows: Mutex<Rows>,
    down: AtomicBool,
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

    /// Make every call fail as an outage, or work again.
    pub fn set_down(&self, down: bool) {
        self.down.store(down, Ordering::SeqCst);
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
            .filter(|(owner, _)| *owner == account)
            .map(|(_, event)| event.clone())
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

    async fn update_session(&self, session: &DeviceSession) -> StoreResult<()> {
        let mut rows = self.guard()?;
        let row = rows
            .sessions
            .get_mut(&session.id)
            .ok_or(StoreError::NotFound)?;
        *row = session.clone();
        Ok(())
    }

    async fn delete_session(&self, account: AccountId, id: SessionId) -> StoreResult<()> {
        let mut rows = self.guard()?;
        if rows
            .sessions
            .get(&id)
            .is_some_and(|row| row.account == account)
        {
            rows.sessions.remove(&id);
        }
        Ok(())
    }

    async fn delete_sessions(&self, account: AccountId) -> StoreResult<()> {
        self.guard()?
            .sessions
            .retain(|_, row| row.account != account);
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
        let (outcome, next) = pin::decide(&row, check);
        if let Some(next) = next {
            let locked =
                next.locked_until_ms != row.locked_until_ms && next.locked_until_ms.is_some();
            rows.pins.insert(profile, next);
            if locked {
                let owner = rows.profiles.get(&profile).map(|row| row.account);
                if let Some(owner) = owner {
                    rows.events.push((owner, String::from("pin_locked")));
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
        self.guard()?.events.push((account, event.to_owned()));
        Ok(())
    }

    async fn save_card(
        &self,
        session: SessionId,
        card_hash: &str,
        card: Option<&serde_json::Value>,
    ) -> StoreResult<bool> {
        let mut rows = self.guard()?;
        let known = rows
            .cards
            .get(&session)
            .is_some_and(|(hash, stored)| hash == card_hash && stored.is_some());
        if !known {
            rows.cards
                .insert(session, (card_hash.to_owned(), card.cloned()));
        }
        Ok(!known && card.is_none())
    }
}
