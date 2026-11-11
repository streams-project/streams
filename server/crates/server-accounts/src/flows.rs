//! Sign in, recovery, device sign in and the account changes that go through the auth service.
//! Each attempt is metered before any outside call.

use rand::Rng;

use crate::auth::{AuthError, AuthService, Identity};
use crate::door::{Door, Member, StepUpMethod};
use crate::error::DoorError;
use crate::profile::{AccountId, NewAccount, ProfileId, SessionId, name_key};
use crate::session::IssuedTokens;
use crate::store::{AccountNames, DeviceCodeRow};
use crate::token::{hash_token, new_token};

/// A device sign in lives this long, and polls this often (RFC 8628).
pub const DEVICE_CODE_S: u32 = 900;
pub const DEVICE_POLL_S: u32 = 5;
/// Letters of a user code: no vowels, so no word appears, about 34.5 bits over 8 letters.
const USER_CODE_LETTERS: &[u8; 20] = b"BCDFGHJKLMNPQRSTVWXZ";
const USER_CODE_LENGTH: usize = 8;
/// The refusal of a code that already has an answer.
const DEVICE_DECIDED: &str = "device_decided";
/// Newest audit rows a person sees.
const EVENTS_SHOWN: usize = 10;

/// Where a sign in ended up.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedIn {
    pub identity: Identity,
    pub session: SessionId,
    pub tokens: IssuedTokens,
}

/// How a device sign in stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DevicePollState {
    Pending,
    SlowDown,
    Denied,
    Expired,
    Approved(Box<SignedIn>),
}

/// A device sign in that started.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceStart {
    pub device_code: String,
    pub user_code: String,
}

/// An account as a person reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccountView {
    pub identity: Identity,
    pub names: AccountNames,
}

/// A plain email check: one at sign, something on each side, a dot inside the domain.
#[must_use]
pub fn email_shape_ok(email: &str) -> bool {
    let email = email.trim();
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && !email.contains(char::is_whitespace)
        && domain.contains('.')
        && !domain.contains('@')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
}

fn new_user_code() -> String {
    let mut rng = rand::rng();
    (0..USER_CODE_LENGTH)
        .map(|_| char::from(USER_CODE_LETTERS[rng.random_range(0..USER_CODE_LETTERS.len())]))
        .collect()
}

fn phone_digits(phone: &str) -> usize {
    phone.chars().filter(char::is_ascii_digit).count()
}

impl From<AuthError> for DoorError {
    /// A taken address has no arm of its own on purpose. Both calls that can meet one answer the
    /// caller the same either way, so nothing should reach here; if a later call forgets, it falls
    /// in with every other refusal and still tells a stranger nothing.
    fn from(error: AuthError) -> Self {
        match error {
            AuthError::Unavailable => Self::Unavailable { retry_after_s: 5 },
            AuthError::WeakPassword => Self::BadValue("password"),
            _ => Self::Refused("sign_in"),
        }
    }
}

impl Door {
    fn auth(&self) -> Result<&dyn crate::auth::AuthService, DoorError> {
        self.auth_service().ok_or(DoorError::Refused("off.auth"))
    }

    async fn sign_in(
        &self,
        identity: Identity,
        device: &str,
        method: StepUpMethod,
    ) -> Result<SignedIn, DoorError> {
        let (session, tokens) = self.open_session(identity.account, device, method).await?;
        Ok(SignedIn {
            identity,
            session,
            tokens,
        })
    }

    /// Sign in with a password. One refusal for an unknown address and a wrong password.
    ///
    /// # Errors
    /// `RateLimited` past the meter, `Refused` for a bad check, `Unavailable` in an outage.
    pub async fn password_sign_in(
        &self,
        email: &str,
        password: &str,
        from: Origin<'_>,
    ) -> Result<SignedIn, DoorError> {
        self.meter_sign_in(email, from.address)?;
        let identity = self.auth()?.check_password(email.trim(), password).await?;
        self.sign_in(identity, from.device, StepUpMethod::Password)
            .await
    }

    /// Send a sign in code. The answer never says whether the account exists.
    ///
    /// # Errors
    /// `RateLimited` past the meter, `Unavailable` in an outage.
    pub async fn send_code(&self, email: &str, address: &str) -> Result<u32, DoorError> {
        self.meter_sign_in(email, address)?;
        match self.auth()?.send_code(email.trim()).await {
            Err(AuthError::Unavailable) => Err(DoorError::Unavailable { retry_after_s: 5 }),
            _ => Ok(self.limits().code_valid_s),
        }
    }

    /// Sign in with a code.
    ///
    /// # Errors
    /// As a password sign in.
    pub async fn code_sign_in(
        &self,
        email: &str,
        code: &str,
        from: Origin<'_>,
    ) -> Result<SignedIn, DoorError> {
        self.meter_sign_in(email, from.address)?;
        let identity = self.auth()?.check_code(email.trim(), code).await?;
        self.sign_in(identity, from.device, StepUpMethod::Code)
            .await
    }

    /// Make an account with its details, main profile and Kids profile. The answer says only that
    /// the ask was heard: free address or taken, whoever asks hears the same words, pays for the
    /// same password hash and waits on the same one letter, so this is no way to learn who has an
    /// account here. A free address gets its account and a sign in code; the person who holds a
    /// taken one is told that someone tried to use it. No tokens leave here, so the device signs
    /// in afterwards with the password it just chose.
    ///
    /// # Errors
    /// `BadValue` for a bad address, password or name, `Unavailable` in an outage.
    pub async fn sign_up(&self, new: &SignUpAsk<'_>, from: Origin<'_>) -> Result<(), DoorError> {
        self.meter_sign_in(new.email, from.address)?;
        if !email_shape_ok(new.email) {
            return Err(DoorError::BadValue("email"));
        }
        let limits = self.limits();
        if (new.password.chars().count() as u32) < limits.password_min {
            return Err(DoorError::BadValue("password"));
        }
        let (first, last) = (new.first_name.trim(), new.last_name.trim());
        let name_max = limits.name_max as usize;
        if first.is_empty()
            || last.is_empty()
            || first.chars().count() > name_max
            || last.chars().count() > name_max
        {
            return Err(DoorError::BadValue("name"));
        }
        let email = new.email.trim();
        let auth = self.auth()?;
        match auth.sign_up(email, new.password).await {
            Ok(identity) => {
                let account = NewAccount {
                    id: identity.account,
                    email: identity.email.clone(),
                    first_name: first.to_owned(),
                    last_name: last.to_owned(),
                };
                self.create_account(&account).await?;
                self.post_once(auth, email, SignUpLetter::Code).await;
            }
            // The one answer this must never give away. The letter goes to the address itself, so
            // the person who holds it learns of the attempt and nobody else learns anything.
            Err(AuthError::Taken) => self.post_once(auth, email, SignUpLetter::Taken).await,
            Err(error) => return Err(error.into()),
        }
        Ok(())
    }

    /// One letter per address per window, whatever the sign up found. Whether it went out never
    /// reaches the caller: an answer that moved with the letter would be the same leak again, and
    /// a letter an address did not ask for must not be repeatable at will.
    async fn post_once(&self, auth: &dyn AuthService, email: &str, which: SignUpLetter) {
        if self.meter_letter(email).is_err() {
            return;
        }
        let sent = match which {
            SignUpLetter::Code => auth.send_code(email).await,
            SignUpLetter::Taken => auth.send_taken_notice(email).await,
        };
        if let Err(error) = sent {
            tracing::warn!("a sign up letter did not leave: {error}");
        }
    }

    /// Prove a password or code on this session. A device session gains the power here.
    ///
    /// # Errors
    /// `Refused` for a bad proof, `BadValue` for neither.
    pub async fn confirm(
        &self,
        member: &Member,
        password: Option<&str>,
        code: Option<&str>,
        address: &str,
    ) -> Result<u32, DoorError> {
        let identity = self.auth()?.identity(member.account()).await?;
        self.meter_sign_in(&identity.email, address)?;
        let method = match (password, code) {
            (Some(password), _) => {
                self.auth()?
                    .check_password(&identity.email, password)
                    .await?;
                StepUpMethod::Password
            }
            (None, Some(code)) => {
                self.auth()?.check_code(&identity.email, code).await?;
                StepUpMethod::Code
            }
            (None, None) => return Err(DoorError::BadValue("proof")),
        };
        self.record_step_up(member, method).await
    }

    /// Send a recovery code. The same answer either way.
    ///
    /// # Errors
    /// `RateLimited` past the meter.
    pub async fn recovery_start(&self, email: &str, address: &str) -> Result<(), DoorError> {
        self.meter_sign_in(email, address)?;
        match self.auth()?.send_recovery(email.trim()).await {
            Err(AuthError::Unavailable) => Err(DoorError::Unavailable { retry_after_s: 5 }),
            _ => Ok(()),
        }
    }

    /// Set a new password with a recovery code, with no old password. Every session ends.
    ///
    /// # Errors
    /// `Refused` for a bad code, `BadValue` for a short password.
    pub async fn recovery_finish(
        &self,
        email: &str,
        code: &str,
        password: &str,
        address: &str,
    ) -> Result<(), DoorError> {
        self.meter_sign_in(email, address)?;
        if (password.chars().count() as u32) < self.limits().password_min {
            return Err(DoorError::BadValue("password"));
        }
        // The code is taken first, so a stranger who knows an address cannot end its sessions by
        // guessing, and the password is written after them, so the two cannot come apart.
        let who = self.auth()?.take_recovery(email.trim(), code).await?;
        self.credential_changed(who.account, None, password).await
    }

    /// The one way a password is set. The service is asked about the password before anything
    /// else happens, because it refuses more than the door's minimum length and a refusal after
    /// the sessions had gone would sign the other devices out for a change that never happened.
    /// Then every session that must not outlive the change ends, so a failure in the middle
    /// leaves the old password working and devices signed out, never a new password with the old
    /// sessions still alive. `keep` is the session the person is standing at: a deliberate change
    /// does not sign that one out.
    ///
    /// # Errors
    /// `BadValue` for a password the service refuses, or the store's refusal.
    async fn credential_changed(
        &self,
        account: AccountId,
        keep: Option<SessionId>,
        password: &str,
    ) -> Result<(), DoorError> {
        let auth = self.auth()?;
        auth.password_ok(password)?;
        match keep {
            Some(session) => {
                self.store().delete_other_sessions(account, session).await?;
            }
            None => self.store().delete_sessions(account).await?,
        }
        auth.set_password(account, password).await?;
        self.store().log_event(account, "password_changed").await?;
        Ok(())
    }

    /// Start a device sign in (RFC 8628). Only the hash of the device code is kept.
    ///
    /// # Errors
    /// The store's refusal.
    pub async fn device_start(&self, name: &str) -> Result<DeviceStart, DoorError> {
        let device_code = new_token();
        let user_code = new_user_code();
        let row = DeviceCodeRow {
            device_hash: hash_token(&device_code),
            user_code: user_code.clone(),
            device_name: name.chars().take(120).collect(),
            account: None,
            denied: false,
            last_poll_ms: None,
            expires_ms: self.now_ms() + u64::from(DEVICE_CODE_S) * 1000,
        };
        self.store().insert_device_code(&row).await?;
        Ok(DeviceStart {
            device_code,
            user_code,
        })
    }

    /// One poll of a device sign in.
    ///
    /// # Errors
    /// The store's refusal.
    pub async fn device_poll(&self, device_code: &str) -> Result<DevicePollState, DoorError> {
        let hash = hash_token(device_code);
        let Some(row) = self.store().device_code(&hash).await? else {
            return Ok(DevicePollState::Expired);
        };
        let now = self.now_ms();
        if row.expires_ms <= now {
            self.store().delete_device_code(&hash).await?;
            return Ok(DevicePollState::Expired);
        }
        if row.denied {
            self.store().delete_device_code(&hash).await?;
            return Ok(DevicePollState::Denied);
        }
        let too_soon = row
            .last_poll_ms
            .is_some_and(|last| now.saturating_sub(last) < u64::from(DEVICE_POLL_S) * 1000);
        if let Some(account) = row.account {
            // Only the poll that took the row signs a device in, so two polls on one approved
            // code cannot both mint a session. The loser reads as a code that is no longer there.
            if !self.store().delete_device_code(&hash).await? {
                return Ok(DevicePollState::Expired);
            }
            let (session, tokens) = self.open_device_session(account, &row.device_name).await?;
            let identity = self.auth()?.identity(account).await?;
            return Ok(DevicePollState::Approved(Box::new(SignedIn {
                identity,
                session,
                tokens,
            })));
        }
        self.store().note_device_poll(&hash, now).await?;
        Ok(if too_soon {
            DevicePollState::SlowDown
        } else {
            DevicePollState::Pending
        })
    }

    async fn waiting_code(
        &self,
        member: &Member,
        user_code: &str,
        address: &str,
    ) -> Result<DeviceCodeRow, DoorError> {
        // five tries a minute on a typed code
        self.meter_device(member.account(), address)?;
        let code = user_code.trim().to_ascii_uppercase();
        let row = self
            .store()
            .device_code_by_user(&code)
            .await?
            .ok_or(DoorError::NotFound)?;
        if row.expires_ms <= self.now_ms() {
            return Err(DoorError::NotFound);
        }
        Ok(row)
    }

    /// Approve a device from a signed in screen. The first answer to a code is the only one.
    ///
    /// # Errors
    /// `NotFound` for an unknown or lapsed code, `Refused` for a code already answered,
    /// `RateLimited` past five tries.
    pub async fn device_approve(
        &self,
        member: &Member,
        user_code: &str,
        address: &str,
    ) -> Result<(), DoorError> {
        let row = self.waiting_code(member, user_code, address).await?;
        let hash = row.device_hash;
        if self
            .store()
            .approve_device_code(&hash, member.account())
            .await?
        {
            Ok(())
        } else {
            Err(DoorError::Refused(DEVICE_DECIDED))
        }
    }

    /// Refuse a device from a signed in screen.
    ///
    /// # Errors
    /// As approve.
    pub async fn device_deny(
        &self,
        member: &Member,
        user_code: &str,
        address: &str,
    ) -> Result<(), DoorError> {
        let row = self.waiting_code(member, user_code, address).await?;
        if self.store().deny_device_code(&row.device_hash).await? {
            Ok(())
        } else {
            Err(DoorError::Refused(DEVICE_DECIDED))
        }
    }

    /// The account of the caller.
    ///
    /// # Errors
    /// The auth service or the store failed.
    pub async fn account(&self, member: &Member) -> Result<AccountView, DoorError> {
        let identity = self.auth()?.identity(member.account()).await?;
        let names = self.store().names(member.account()).await?;
        Ok(AccountView { identity, names })
    }

    /// Change the names. The main profile follows when it still carries the old first name and no
    /// other profile holds the new one. No step up is needed.
    ///
    /// # Errors
    /// `BadValue` for an empty or long name.
    pub async fn update_names(
        &self,
        member: &Member,
        first: Option<&str>,
        last: Option<&str>,
    ) -> Result<AccountView, DoorError> {
        let account = member.account();
        let old = self.store().names(account).await?;
        let max = self.limits().name_max as usize;
        let clean = |text: Option<&str>, kept: &str| -> Result<String, DoorError> {
            let name: String = text.unwrap_or(kept).trim().chars().take(max).collect();
            if name.is_empty() {
                Err(DoorError::BadValue("name"))
            } else {
                Ok(name)
            }
        };
        let names = AccountNames {
            first_name: clean(first, &old.first_name)?,
            last_name: clean(last, &old.last_name)?,
        };
        let profiles = self.store().profiles_of(account).await?;
        let new_name: String = names
            .first_name
            .chars()
            .take(self.limits().profile_name_max as usize)
            .collect();
        let taken = profiles
            .iter()
            .any(|p| name_key(&p.name) == name_key(&new_name));
        let main = profiles
            .iter()
            .find(|p| p.is_main && name_key(&p.name) == name_key(&old.first_name))
            .filter(|_| !taken)
            .map(|p| (p.id, new_name.clone()));
        self.store().set_names(account, &names, main).await?;
        self.account(member).await
    }

    /// Change the password in one request with its proof. Every other device is signed out: after
    /// a password change the person expects to be the only one holding the account, and a refresh
    /// chain a thief took would otherwise outlive the change. This session stays.
    ///
    /// # Errors
    /// `StepUpRequired`, or `BadValue` for a short password.
    pub async fn change_password(&self, member: &Member, password: &str) -> Result<(), DoorError> {
        self.require_step_up(member)?;
        if (password.chars().count() as u32) < self.limits().password_min {
            return Err(DoorError::BadValue("password"));
        }
        self.credential_changed(member.account(), Some(member.session_id()), password)
            .await
    }

    /// Change the address. As with a sign up, the answer does not say whether another account
    /// already holds the address; the person who holds it hears that someone tried to move an
    /// account onto it.
    ///
    /// # Errors
    /// `StepUpRequired`, or `BadValue` for a bad shape.
    pub async fn change_email(&self, member: &Member, email: &str) -> Result<(), DoorError> {
        self.require_step_up(member)?;
        if !email_shape_ok(email) {
            return Err(DoorError::BadValue("email"));
        }
        let email = email.trim();
        let auth = self.auth()?;
        match auth.set_email(member.account(), email).await {
            Err(AuthError::Taken) => {
                self.post_once(auth, email, SignUpLetter::Taken).await;
                Ok(())
            }
            other => Ok(other?),
        }
    }

    /// Change the phone: 6 to 15 digits.
    ///
    /// # Errors
    /// `StepUpRequired`, or `BadValue` for a bad number.
    pub async fn change_phone(&self, member: &Member, phone: &str) -> Result<(), DoorError> {
        self.require_step_up(member)?;
        let limits = self.limits();
        let digits = phone_digits(phone) as u32;
        if !(limits.phone_digits_min..=limits.phone_digits_max).contains(&digits) {
            return Err(DoorError::BadValue("phone"));
        }
        Ok(self
            .auth()?
            .set_phone(member.account(), phone.trim())
            .await?)
    }

    /// Delete the account at once. Needs a step up.
    ///
    /// # Errors
    /// `StepUpRequired`.
    pub async fn delete_account(&self, member: &Member) -> Result<(), DoorError> {
        self.require_step_up(member)?;
        self.auth()?.delete(member.account()).await?;
        Ok(self.store().delete_account(member.account()).await?)
    }

    /// Check a step up for an export, a transfer or a device removal.
    ///
    /// # Errors
    /// `StepUpRequired`.
    pub fn require_step_up(&self, member: &Member) -> Result<(), DoorError> {
        if self.has_step_up(member) {
            Ok(())
        } else {
            Err(DoorError::StepUpRequired)
        }
    }

    /// Move a profile to another account. The PIN never travels.
    ///
    /// # Errors
    /// `StepUpRequired`, `NotFound` for an unknown address, `Refused` for the main or last profile.
    pub async fn transfer_profile(
        &self,
        member: &Member,
        profile: ProfileId,
        to_email: &str,
    ) -> Result<(), DoorError> {
        self.require_step_up(member)?;
        let all = self.store().profiles_of(member.account()).await?;
        let row = all
            .iter()
            .find(|row| row.id == profile)
            .ok_or(DoorError::NotFound)?;
        if row.is_main || all.len() <= 1 {
            return Err(DoorError::Refused("main_profile"));
        }
        let to = self
            .auth()?
            .find_email(to_email.trim())
            .await?
            .ok_or(DoorError::NotFound)?;
        // The PIN and the automatic Kids flag are cleared inside the move, so a refused move
        // leaves the profile exactly as it was.
        self.store()
            .move_profile(member.account(), profile, to, self.limits().profiles_max)
            .await?;
        self.store()
            .log_event(member.account(), "profile_transferred")
            .await?;
        Ok(())
    }

    /// The newest audit rows of the caller.
    ///
    /// # Errors
    /// The store's refusal.
    pub async fn events(&self, member: &Member) -> Result<Vec<String>, DoorError> {
        Ok(self
            .store()
            .events_of(member.account(), EVENTS_SHOWN)
            .await?)
    }
}

/// The one letter a sign up or an address change sends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SignUpLetter {
    /// A sign in code, to the address of the account just made.
    Code,
    /// A word to the person who already holds the address.
    Taken,
}

/// Where a sign in comes from.
#[derive(Clone, Copy, Debug)]
pub struct Origin<'a> {
    /// The one trusted address, for the meter.
    pub address: &'a str,
    /// A name for the session list.
    pub device: &'a str,
}

/// What sign up asks.
#[derive(Clone, Copy, Debug)]
pub struct SignUpAsk<'a> {
    pub email: &'a str,
    pub password: &'a str,
    pub first_name: &'a str,
    pub last_name: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c70_c91_email_shapes() {
        assert!(email_shape_ok(" a@b.test "));
        for bad in [
            "",
            "a",
            "@b.test",
            "a@b",
            "a@.test",
            "a@test.",
            "a b@c.test",
            "a@b@c.test",
        ] {
            assert!(!email_shape_ok(bad), "{bad}");
        }
    }

    #[test]
    fn tmt11_a_user_code_is_eight_letters_with_no_vowel() {
        let code = new_user_code();
        assert_eq!(code.len(), USER_CODE_LENGTH);
        assert!(code.chars().all(|c| USER_CODE_LETTERS.contains(&(c as u8))));
    }
}
