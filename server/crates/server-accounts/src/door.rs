//! The one door. Reads the bearer token, checks the session row, and mints [`ProfileScope`].

use std::sync::Arc;

use crate::auth::AuthService;
use crate::clock::Clock;
use crate::error::DoorError;
use crate::limits::AccountLimits;
use crate::meter::{Meter, MeterKey};
use crate::pin::{self, PinCheck, PinOutcome};
use crate::profile::{
    AccountId, NewAccount, Profile, ProfileId, ProfileKind, ProfileLimitsRow, ProfilePatch,
    SessionId, clean_profile_name, starting_profiles,
};
use crate::session::{Consent, DeviceSession, IssuedTokens, RefreshOutcome};
use crate::store::AccountStore;
use crate::token::{hash_token, new_token};

/// How a step up was made.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepUpMethod {
    Password,
    Code,
}

impl StepUpMethod {
    const fn word(self) -> &'static str {
        match self {
            Self::Password => "password",
            Self::Code => "code",
        }
    }
}

/// A caller with a live session. Holds no profile yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Member {
    session: DeviceSession,
}

impl Member {
    /// The account.
    #[must_use]
    pub const fn account(&self) -> AccountId {
        self.session.account
    }

    /// The session.
    #[must_use]
    pub const fn session_id(&self) -> SessionId {
        self.session.id
    }

    /// The active profile, if one was entered.
    #[must_use]
    pub const fn active_profile(&self) -> Option<ProfileId> {
        self.session.active_profile
    }
}

/// Proof that this request acts for one profile of one account. The fields are private and no
/// function outside the door builds one, so a query that takes a scope can only be reached
/// through the session row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileScope {
    account: AccountId,
    session: SessionId,
    profile: ProfileId,
    kind: ProfileKind,
    age_limit: String,
    maturity: String,
    blocked_names: Vec<String>,
    consent: Consent,
}

impl ProfileScope {
    /// The account.
    #[must_use]
    pub const fn account(&self) -> AccountId {
        self.account
    }

    /// The session.
    #[must_use]
    pub const fn session(&self) -> SessionId {
        self.session
    }

    /// The profile.
    #[must_use]
    pub const fn profile(&self) -> ProfileId {
        self.profile
    }

    /// The kind of profile.
    #[must_use]
    pub const fn kind(&self) -> ProfileKind {
        self.kind
    }

    /// The age limit word as saved.
    #[must_use]
    pub fn age_limit(&self) -> &str {
        &self.age_limit
    }

    /// The older coarse level.
    #[must_use]
    pub fn maturity(&self) -> &str {
        &self.maturity
    }

    /// Names this profile never sees, as typed.
    #[must_use]
    pub fn blocked_names(&self) -> &[String] {
        &self.blocked_names
    }

    /// Consent of this session.
    #[must_use]
    pub const fn consent(&self) -> Consent {
        self.consent
    }
}

/// The door over one store.
#[derive(Clone)]
pub struct Door {
    store: Arc<dyn AccountStore>,
    clock: Arc<dyn Clock>,
    limits: AccountLimits,
    calls: Arc<Meter>,
    sign_ins: Arc<Meter>,
    device_tries: Arc<Meter>,
    auth: Option<Arc<dyn AuthService>>,
}

impl std::fmt::Debug for Door {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Door")
            .field("limits", &self.limits)
            .finish()
    }
}

/// Calls per minute from one address, before any token lookup.
const CALLS_PER_MINUTE: u32 = 600;
/// Password or code tries per minute on one account name or address.
const SIGN_INS_PER_MINUTE: u32 = 10;
/// Typed device codes per minute.
const DEVICE_TRIES_PER_MINUTE: u32 = 5;

impl Door {
    /// A door with the default meters.
    #[must_use]
    pub fn new(store: Arc<dyn AccountStore>, clock: Arc<dyn Clock>, limits: AccountLimits) -> Self {
        Self {
            store,
            clock,
            limits,
            calls: Arc::new(Meter::new(CALLS_PER_MINUTE)),
            sign_ins: Arc::new(Meter::new(SIGN_INS_PER_MINUTE)),
            device_tries: Arc::new(Meter::new(DEVICE_TRIES_PER_MINUTE)),
            auth: None,
        }
    }

    /// This door, with the auth service it signs people in through.
    #[must_use]
    pub fn with_auth(mut self, auth: Arc<dyn AuthService>) -> Self {
        self.auth = Some(auth);
        self
    }

    /// The auth service, when one is set.
    #[must_use]
    pub fn auth_service(&self) -> Option<&dyn AuthService> {
        self.auth.as_deref()
    }

    /// The time this door reads.
    #[must_use]
    pub fn now_ms(&self) -> u64 {
        self.clock.now_ms()
    }

    /// Count a typed device code.
    ///
    /// # Errors
    /// `RateLimited` past five tries a minute.
    pub fn meter_device(&self, account: AccountId, address: &str) -> Result<(), DoorError> {
        let now = self.now();
        for key in [
            MeterKey::Account(account.to_string()),
            MeterKey::Address(address.to_owned()),
        ] {
            self.device_tries
                .check(key, now)
                .map_err(|retry_after_s| DoorError::RateLimited { retry_after_s })?;
        }
        Ok(())
    }

    /// The limits this door applies.
    #[must_use]
    pub const fn limits(&self) -> &AccountLimits {
        &self.limits
    }

    /// The store behind this door.
    #[must_use]
    pub fn store(&self) -> &Arc<dyn AccountStore> {
        &self.store
    }

    fn now(&self) -> u64 {
        self.clock.now_ms()
    }

    /// Count a call from an address. Runs before any lookup, so invented tokens cost a hash at
    /// most. The address is the one header the front proxy sets.
    ///
    /// # Errors
    /// `RateLimited` past the limit.
    pub fn meter_call(&self, address: &str) -> Result<(), DoorError> {
        self.calls
            .check(MeterKey::Address(address.to_owned()), self.now())
            .map_err(|retry_after_s| DoorError::RateLimited { retry_after_s })
    }

    /// Count a sign in try, by account name and by address. Runs before any outside call.
    ///
    /// # Errors
    /// `RateLimited` past either limit.
    pub fn meter_sign_in(&self, name: &str, address: &str) -> Result<(), DoorError> {
        let now = self.now();
        for key in [
            MeterKey::Account(name.trim().to_lowercase()),
            MeterKey::Address(address.to_owned()),
        ] {
            self.sign_ins
                .check(key, now)
                .map_err(|retry_after_s| DoorError::RateLimited { retry_after_s })?;
        }
        Ok(())
    }

    fn fresh_session(
        &self,
        account: AccountId,
        device_name: &str,
    ) -> (DeviceSession, IssuedTokens) {
        let now = self.now();
        let id = SessionId::new();
        let (access, current, salt) = (new_token(), format!("{id}.{}", new_token()), new_token());
        let mut session = DeviceSession {
            id,
            account,
            device_name: device_name.to_owned(),
            access_hash: hash_token(&access),
            access_expires_ms: now + u64::from(self.limits.access_s) * 1000,
            refresh_current: hash_token(&current),
            refresh_next: hash_token(&current),
            refresh_salt: salt,
            refresh_first_used_ms: None,
            active_profile: None,
            entered_at_ms: None,
            step_up_at_ms: None,
            step_up_method: None,
            consent: Consent::default(),
            last_seen_ms: now,
            created_ms: now,
        };
        session.refresh_next = hash_token(&session.next_for(&current));
        let tokens = IssuedTokens {
            access,
            refresh: current,
            expires_in: self.limits.access_s,
        };
        (session, tokens)
    }

    /// Make the rows of a new account.
    ///
    /// # Errors
    /// The store's refusal.
    pub async fn create_account(&self, account: &NewAccount) -> Result<(), DoorError> {
        let profiles = starting_profiles(account, &self.limits);
        self.store.create_account(account, &profiles).await?;
        Ok(())
    }

    /// Open a session after the auth service accepted a password or a code. The check that just
    /// passed also counts as a step up.
    ///
    /// # Errors
    /// The store's refusal.
    pub async fn open_session(
        &self,
        account: AccountId,
        device_name: &str,
        method: StepUpMethod,
    ) -> Result<(SessionId, IssuedTokens), DoorError> {
        let (mut session, tokens) = self.fresh_session(account, device_name);
        session.step_up_at_ms = Some(self.now());
        session.step_up_method = Some(method.word().to_owned());
        self.store.insert_session(&session).await?;
        Ok((session.id, tokens))
    }

    /// Open a session approved from another device. It has no step up power.
    ///
    /// # Errors
    /// The store's refusal.
    pub async fn open_device_session(
        &self,
        account: AccountId,
        device_name: &str,
    ) -> Result<(SessionId, IssuedTokens), DoorError> {
        let (session, tokens) = self.fresh_session(account, device_name);
        self.store.insert_session(&session).await?;
        Ok((session.id, tokens))
    }

    /// The caller of a bearer token. A failed read answers 503, never a sign out.
    ///
    /// # Errors
    /// `Unauthorized` for an unknown or lapsed token, `Unavailable` when the store is down.
    pub async fn member(&self, bearer: &str) -> Result<Member, DoorError> {
        let session = self
            .store
            .session_by_access(&hash_token(bearer))
            .await?
            .ok_or(DoorError::Unauthorized)?;
        let now = self.now();
        let idle_ms = u64::from(self.limits.idle_s) * 1000;
        if session.access_expires_ms <= now || now.saturating_sub(session.last_seen_ms) > idle_ms {
            return Err(DoorError::Unauthorized);
        }
        Ok(Member { session })
    }

    /// Mint the scope of this request from the session row. Nothing the client sent is read, so
    /// a profile header means nothing, and a session with no profile is refused.
    ///
    /// # Errors
    /// `ProfileRequired` with no active profile, or when the profile is gone.
    pub async fn scope(&self, member: &Member) -> Result<ProfileScope, DoorError> {
        let active = member
            .session
            .active_profile
            .ok_or(DoorError::ProfileRequired)?;
        let profile = self
            .store
            .profile(member.session.account, active)
            .await?
            .ok_or(DoorError::ProfileRequired)?;
        Ok(ProfileScope {
            account: profile.account,
            session: member.session.id,
            profile: profile.id,
            kind: profile.kind,
            age_limit: profile.age_limit,
            maturity: profile.maturity,
            blocked_names: profile.blocked_names,
            consent: member.session.consent,
        })
    }

    /// Enter a profile: check the PIN on the server, then write the active profile to the
    /// session row. A profile of another account is simply not found.
    ///
    /// # Errors
    /// `PinRequired`, `PinWrong`, `PinLocked` or `NotFound`.
    pub async fn enter_profile(
        &self,
        member: &Member,
        profile: ProfileId,
        pin: Option<&str>,
    ) -> Result<Profile, DoorError> {
        let row = self
            .store
            .profile(member.account(), profile)
            .await?
            .ok_or(DoorError::NotFound)?;
        if row.has_pin {
            let check = PinCheck {
                pin,
                now_ms: self.now(),
            };
            match self.store.pin_attempt(profile, check).await? {
                PinOutcome::Pass => {}
                PinOutcome::Missing => return Err(DoorError::PinRequired),
                PinOutcome::Wrong => return Err(DoorError::PinWrong),
                PinOutcome::Locked { retry_after_s } => {
                    return Err(DoorError::PinLocked { retry_after_s });
                }
            }
        }
        self.store
            .set_active_profile(member.session_id(), profile, self.now())
            .await?;
        Ok(row)
    }

    /// Turn a refresh token into tokens. A lost reply is safe: the same current gets the same
    /// next. A dead token, or a current past the reuse window, ends the session.
    ///
    /// # Errors
    /// `SessionEnded` when the token is refused for good; `Unavailable` keeps the tokens.
    pub async fn refresh(&self, token: &str) -> Result<IssuedTokens, DoorError> {
        let hash = hash_token(token);
        let (id, _) = token.split_once('.').ok_or(DoorError::SessionEnded)?;
        let id = SessionId::parse(id).map_err(|_| DoorError::SessionEnded)?;
        let session = self
            .store
            .session(id)
            .await?
            .ok_or(DoorError::SessionEnded)?;
        let now = self.now();
        let reuse_ms = u64::from(self.limits.refresh_reuse_s) * 1000;
        let idle_ms = u64::from(self.limits.idle_s) * 1000;
        if now.saturating_sub(session.last_seen_ms) > idle_ms {
            self.store
                .delete_session(session.account, session.id)
                .await?;
            return Err(DoorError::SessionEnded);
        }
        let mut next = session.clone();
        let refresh = match session.classify_refresh(&hash, now, reuse_ms) {
            RefreshOutcome::End => {
                self.store
                    .delete_session(session.account, session.id)
                    .await?;
                return Err(DoorError::SessionEnded);
            }
            RefreshOutcome::OfferNext => {
                next.refresh_first_used_ms = next.refresh_first_used_ms.or(Some(now));
                session.next_for(token)
            }
            RefreshOutcome::Promote => {
                next.refresh_current = hash;
                next.refresh_first_used_ms = Some(now);
                let offered = session.next_for(token);
                next.refresh_next = hash_token(&offered);
                offered
            }
        };
        let access = new_token();
        next.access_hash = hash_token(&access);
        next.access_expires_ms = now + u64::from(self.limits.access_s) * 1000;
        next.last_seen_ms = now;
        self.store.update_tokens(&next).await?;
        Ok(IssuedTokens {
            access,
            refresh,
            expires_in: self.limits.access_s,
        })
    }

    /// Refresh, naming the account the tokens belong to.
    ///
    /// # Errors
    /// As [`Door::refresh`].
    pub async fn refresh_for(&self, token: &str) -> Result<(AccountId, IssuedTokens), DoorError> {
        let tokens = self.refresh(token).await?;
        let (id, _) = token.split_once('.').ok_or(DoorError::SessionEnded)?;
        let id = SessionId::parse(id).map_err(|_| DoorError::SessionEnded)?;
        let session = self
            .store
            .session(id)
            .await?
            .ok_or(DoorError::SessionEnded)?;
        Ok((session.account, tokens))
    }

    /// Record a step up on this session. Refresh never renews it.
    ///
    /// # Errors
    /// The store's refusal.
    pub async fn record_step_up(
        &self,
        member: &Member,
        method: StepUpMethod,
    ) -> Result<u32, DoorError> {
        self.store
            .set_step_up(member.session_id(), self.now(), method.word())
            .await?;
        Ok(self.limits.step_up_s)
    }

    /// True when the session proved a password or code inside the window.
    #[must_use]
    pub fn has_step_up(&self, member: &Member) -> bool {
        member.session.step_up_at_ms.is_some_and(|at| {
            self.now().saturating_sub(at) <= u64::from(self.limits.step_up_s) * 1000
        })
    }

    /// Set the age limit and blocked names.
    ///
    /// # Errors
    /// `StepUpRequired`, or `BadValue` past the limits.
    pub async fn set_limits(
        &self,
        member: &Member,
        profile: ProfileId,
        limits: ProfileLimitsRow,
    ) -> Result<(), DoorError> {
        if !self.has_step_up(member) {
            return Err(DoorError::StepUpRequired);
        }
        let too_many = limits.blocked_names.len() > self.limits.blocked_names_max as usize;
        let too_long = limits
            .blocked_names
            .iter()
            .any(|name| name.chars().count() > self.limits.blocked_name_max as usize);
        if too_many || too_long {
            return Err(DoorError::BadValue("blocked_names"));
        }
        self.store
            .set_limits(member.account(), profile, &limits)
            .await?;
        Ok(())
    }

    /// Add a profile. A Kids profile starts at the Kids limit.
    ///
    /// # Errors
    /// `BadValue`, or the store's limit or name refusal.
    pub async fn create_profile(
        &self,
        member: &Member,
        name: &str,
        kids: bool,
    ) -> Result<Profile, DoorError> {
        let name = clean_profile_name(name, &self.limits)?;
        let existing = self.store.profiles_of(member.account()).await?;
        let sort_order = i16::try_from(existing.len()).unwrap_or(i16::MAX);
        let (kind, age_limit, maturity) = if kids {
            (ProfileKind::Kids, "PG", "older_kids")
        } else {
            (ProfileKind::Adult, "18", "adult")
        };
        let profile = Profile {
            id: ProfileId::new(),
            account: member.account(),
            name,
            avatar: String::from(if kids { "kids" } else { "default" }),
            kind,
            is_main: false,
            auto_kids: false,
            has_pin: false,
            sort_order,
            age_limit: age_limit.to_owned(),
            maturity: maturity.to_owned(),
            blocked_names: Vec::new(),
        };
        self.store
            .insert_profile(&profile, self.limits.profiles_max)
            .await?;
        self.store
            .log_event(member.account(), "profile_created")
            .await?;
        Ok(profile)
    }

    /// Change a profile. Moving between Kids and adult needs a step up, and a main profile can
    /// not be Kids.
    ///
    /// # Errors
    /// `StepUpRequired`, `BadValue`, `Refused`, or the store's refusal.
    pub async fn update_profile(
        &self,
        member: &Member,
        profile: ProfileId,
        mut patch: ProfilePatch,
    ) -> Result<Profile, DoorError> {
        let row = self
            .store
            .profile(member.account(), profile)
            .await?
            .ok_or(DoorError::NotFound)?;
        if let Some(name) = &patch.name {
            patch.name = Some(clean_profile_name(name, &self.limits)?);
        }
        if let Some(kind) = patch.kind.filter(|kind| *kind != row.kind) {
            if !self.has_step_up(member) {
                return Err(DoorError::StepUpRequired);
            }
            if kind == ProfileKind::Kids && row.is_main {
                return Err(DoorError::Refused("main_profile_kids"));
            }
        }
        Ok(self
            .store
            .update_profile(member.account(), profile, &patch)
            .await?)
    }

    /// Delete a profile. The last and the main profile stay, and any session on it loses its
    /// active profile, so the picker and the PIN return.
    ///
    /// # Errors
    /// `Refused` for the main or last profile.
    pub async fn delete_profile(
        &self,
        member: &Member,
        profile: ProfileId,
    ) -> Result<(), DoorError> {
        let all = self.store.profiles_of(member.account()).await?;
        let row = all
            .iter()
            .find(|row| row.id == profile)
            .ok_or(DoorError::NotFound)?;
        if all.len() <= 1 {
            return Err(DoorError::Refused("last_profile"));
        }
        if row.is_main {
            return Err(DoorError::Refused("main_profile"));
        }
        self.store.delete_profile(member.account(), profile).await?;
        self.store
            .log_event(member.account(), "profile_deleted")
            .await?;
        Ok(())
    }

    /// Move the main flag. Needs a step up and refuses a Kids profile.
    ///
    /// # Errors
    /// `StepUpRequired`, `Refused` or `NotFound`.
    pub async fn set_main_profile(
        &self,
        member: &Member,
        profile: ProfileId,
    ) -> Result<(), DoorError> {
        if !self.has_step_up(member) {
            return Err(DoorError::StepUpRequired);
        }
        let row = self
            .store
            .profile(member.account(), profile)
            .await?
            .ok_or(DoorError::NotFound)?;
        if row.kind == ProfileKind::Kids {
            return Err(DoorError::Refused("main_profile_kids"));
        }
        self.store
            .set_main_profile(member.account(), profile)
            .await?;
        self.store
            .log_event(member.account(), "main_profile_changed")
            .await?;
        Ok(())
    }

    /// Set or clear a PIN. Needs a step up, which also stands for the current PIN.
    ///
    /// # Errors
    /// `StepUpRequired`, `BadValue` or `NotFound`.
    pub async fn set_pin(
        &self,
        member: &Member,
        profile: ProfileId,
        pin: Option<&str>,
    ) -> Result<(), DoorError> {
        if !self.has_step_up(member) {
            return Err(DoorError::StepUpRequired);
        }
        self.store
            .profile(member.account(), profile)
            .await?
            .ok_or(DoorError::NotFound)?;
        let event = match pin {
            Some(pin) => {
                if !pin::pin_shape_ok(pin, &self.limits) {
                    return Err(DoorError::BadValue("pin"));
                }
                self.store
                    .set_pin(profile, Some(pin::hash_pin(pin)))
                    .await?;
                "pin_set"
            }
            None => {
                self.store.set_pin(profile, None).await?;
                "pin_cleared"
            }
        };
        self.store.log_event(member.account(), event).await?;
        Ok(())
    }

    /// End this session, or all of them.
    ///
    /// # Errors
    /// The store's refusal.
    pub async fn sign_out(&self, member: &Member, everywhere: bool) -> Result<(), DoorError> {
        if everywhere {
            self.store.delete_sessions(member.account()).await?;
        } else {
            self.store
                .delete_session(member.account(), member.session_id())
                .await?;
        }
        Ok(())
    }

    /// Set consent on this session.
    ///
    /// # Errors
    /// The store's refusal.
    pub async fn set_consent(&self, member: &Member, consent: Consent) -> Result<(), DoorError> {
        self.store.set_consent(member.session_id(), consent).await?;
        Ok(())
    }

    /// Profiles of the caller's account.
    ///
    /// # Errors
    /// The store's refusal.
    pub async fn profiles(&self, member: &Member) -> Result<Vec<Profile>, DoorError> {
        Ok(self.store.profiles_of(member.account()).await?)
    }

    /// One profile of the caller's account.
    ///
    /// # Errors
    /// `NotFound` for a profile of another account.
    pub async fn profile(&self, member: &Member, id: ProfileId) -> Result<Profile, DoorError> {
        self.store
            .profile(member.account(), id)
            .await?
            .ok_or(DoorError::NotFound)
    }

    /// Sessions of the caller's account, newest use first.
    ///
    /// # Errors
    /// The store's refusal.
    pub async fn devices(&self, member: &Member) -> Result<Vec<DeviceSession>, DoorError> {
        Ok(self.store.sessions_of(member.account()).await?)
    }

    /// Sign out one device of this account. Another account's device is simply not found.
    ///
    /// # Errors
    /// The store's refusal.
    pub async fn remove_device(&self, member: &Member, id: SessionId) -> Result<(), DoorError> {
        let sessions = self.store.sessions_of(member.account()).await?;
        if !sessions.iter().any(|session| session.id == id) {
            return Err(DoorError::NotFound);
        }
        Ok(self.store.delete_session(member.account(), id).await?)
    }

    /// Sign out every device but this one.
    ///
    /// # Errors
    /// The store's refusal.
    pub async fn remove_other_devices(&self, member: &Member) -> Result<(), DoorError> {
        for session in self.store.sessions_of(member.account()).await? {
            if session.id != member.session_id() {
                self.store
                    .delete_session(member.account(), session.id)
                    .await?;
            }
        }
        Ok(())
    }

    /// The device card of this session, as sent.
    ///
    /// # Errors
    /// The store's refusal.
    pub async fn card(&self, session: SessionId) -> Result<Option<serde_json::Value>, DoorError> {
        Ok(self.store.card(session).await?)
    }

    /// Save a device card (opaque JSON). True asks the device for the whole card.
    ///
    /// # Errors
    /// The store's refusal.
    pub async fn save_card(
        &self,
        member: &Member,
        card_hash: &str,
        card: Option<&serde_json::Value>,
    ) -> Result<bool, DoorError> {
        Ok(self
            .store
            .save_card(member.session_id(), card_hash, card)
            .await?)
    }
}
