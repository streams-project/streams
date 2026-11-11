//! What sign in must do. It checks passwords and codes and holds the password
//! hash; this server keeps its own opaque tokens, so replacing it changes no client.

use std::sync::Mutex;

use async_trait::async_trait;
use thiserror::Error;

use crate::profile::AccountId;

/// A person the auth service knows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    pub account: AccountId,
    pub email: String,
    pub phone: Option<String>,
}

/// Why the auth service said no.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum AuthError {
    /// It did not answer. A 503 here, never a sign out.
    #[error("the auth service did not answer")]
    Unavailable,
    /// The password, code or address was refused.
    #[error("refused")]
    Refused,
    /// The password is too weak.
    #[error("weak password")]
    WeakPassword,
    /// The address is taken. Never shown to a caller as this.
    #[error("the address is taken")]
    Taken,
}

/// bcrypt reads no further than this many bytes, so a service of this trait refuses a longer
/// password rather than storing one whose tail means nothing.
pub const PASSWORD_MAX_BYTES: usize = 72;

/// What the door needs from the auth service.
#[async_trait]
pub trait AuthService: Send + Sync {
    /// Make an account.
    async fn sign_up(&self, email: &str, password: &str) -> Result<Identity, AuthError>;
    /// Check a password.
    async fn check_password(&self, email: &str, password: &str) -> Result<Identity, AuthError>;
    /// Send a sign in code. Sent or not, known address or not, the answer is the same. An `Err`
    /// means no address could have one — there is no mail, or the store did not answer — which is
    /// as true of an address with no account as of one with an account.
    async fn send_code(&self, email: &str) -> Result<(), AuthError>;
    /// Check a sign in code.
    async fn check_code(&self, email: &str, code: &str) -> Result<Identity, AuthError>;
    /// Send a recovery code.
    async fn send_recovery(&self, email: &str) -> Result<(), AuthError>;
    /// Tell the holder of an address that someone tried to make an account with it, or to move an
    /// account onto it. An address with no account hears nothing, and the caller is told the same
    /// either way.
    async fn send_taken_notice(&self, email: &str) -> Result<(), AuthError>;
    /// Take a recovery code, leaving the password alone. The door sets the password itself, so the
    /// sessions of the account can end between the two steps.
    async fn take_recovery(&self, email: &str, code: &str) -> Result<Identity, AuthError>;
    /// Set a new password with a recovery code, in one call for a caller that wants both steps.
    async fn finish_recovery(
        &self,
        email: &str,
        code: &str,
        password: &str,
    ) -> Result<(), AuthError> {
        let who = self.take_recovery(email, code).await?;
        self.set_password(who.account, password).await
    }
    /// Read a person.
    async fn identity(&self, account: AccountId) -> Result<Identity, AuthError>;
    /// Whether this service would take this password, asked before anything else changes. A
    /// service refuses more than the door's minimum length, and the door ends sessions on the way
    /// to a new password, so the refusal has to be known before the first session goes.
    /// `WeakPassword` for a password this service will not store.
    fn password_ok(&self, _password: &str) -> Result<(), AuthError> {
        Ok(())
    }
    /// Change the password. It refuses the same passwords [`AuthService::password_ok`] does.
    async fn set_password(&self, account: AccountId, password: &str) -> Result<(), AuthError>;
    /// Change the address.
    async fn set_email(&self, account: AccountId, email: &str) -> Result<(), AuthError>;
    /// Change the phone.
    async fn set_phone(&self, account: AccountId, phone: &str) -> Result<(), AuthError>;
    /// Delete a person. Every table of the account cascades.
    async fn delete(&self, account: AccountId) -> Result<(), AuthError>;
    /// The person with this address, for moving a profile.
    async fn find_email(&self, email: &str) -> Result<Option<AccountId>, AuthError>;
}

/// An auth service in memory, for tests. Codes are fixed so a test can type them. It answers an
/// unknown address the way the built in service does, so a test of that rule has the same teeth
/// here as in production.
#[derive(Debug, Default)]
pub struct MemoryAuth {
    people: Mutex<Vec<(Identity, String)>>,
    /// Addresses a code went to, for tests to read.
    pub sent: Mutex<Vec<String>>,
    /// Addresses told that they are taken, for tests to read.
    pub warned: Mutex<Vec<String>>,
    /// True makes every call fail as an outage.
    pub down: std::sync::atomic::AtomicBool,
    /// True makes every letter fail the way a mail provider that refuses does. The caller must
    /// still hear the same answer, whatever the address.
    pub mail_down: std::sync::atomic::AtomicBool,
    /// True is a deployment with no mail at all: a fact about the server, so every address hears
    /// the same refusal.
    pub mail_off: std::sync::atomic::AtomicBool,
}

/// The code `MemoryAuth` accepts.
pub const MEMORY_CODE: &str = "12345678";

impl MemoryAuth {
    fn guard(&self) -> Result<std::sync::MutexGuard<'_, Vec<(Identity, String)>>, AuthError> {
        if self.down.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(AuthError::Unavailable);
        }
        Ok(self
            .people
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner))
    }

    fn delivery(&self) -> Result<(), AuthError> {
        if self.mail_down.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(AuthError::Unavailable);
        }
        Ok(())
    }

    fn known(&self, email: &str) -> Result<bool, AuthError> {
        let people = self.guard()?;
        Ok(people.iter().any(|p| p.0.email == email))
    }

    fn change(
        &self,
        account: AccountId,
        edit: impl FnOnce(&mut (Identity, String)),
    ) -> Result<(), AuthError> {
        let mut people = self.guard()?;
        let person = people
            .iter_mut()
            .find(|p| p.0.account == account)
            .ok_or(AuthError::Refused)?;
        edit(person);
        Ok(())
    }
}

#[async_trait]
impl AuthService for MemoryAuth {
    async fn sign_up(&self, email: &str, password: &str) -> Result<Identity, AuthError> {
        let mut people = self.guard()?;
        if people.iter().any(|p| p.0.email == email) {
            return Err(AuthError::Taken);
        }
        let identity = Identity {
            account: AccountId::new(),
            email: email.to_owned(),
            phone: None,
        };
        people.push((identity.clone(), password.to_owned()));
        Ok(identity)
    }

    async fn check_password(&self, email: &str, password: &str) -> Result<Identity, AuthError> {
        let people = self.guard()?;
        let person = people
            .iter()
            .find(|p| p.0.email == email && p.1 == password);
        person.map(|p| p.0.clone()).ok_or(AuthError::Refused)
    }

    async fn send_code(&self, email: &str) -> Result<(), AuthError> {
        // The shape the built in service answers in: whether letters leave at all is settled
        // before the address is read, only an address with an account is written to, and a
        // provider that refuses never changes the answer.
        if self.mail_off.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(AuthError::Unavailable);
        }
        if !self.known(email)? {
            return Ok(());
        }
        self.sent
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(email.to_owned());
        let _refused = self.delivery();
        Ok(())
    }

    async fn send_taken_notice(&self, email: &str) -> Result<(), AuthError> {
        if self.mail_off.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(AuthError::Unavailable);
        }
        if !self.known(email)? {
            return Ok(());
        }
        self.warned
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(email.to_owned());
        // As `send_code` does, and as the built in service does: the provider's result never
        // reaches the caller, or a refusing provider would answer a taken address differently
        // from a free one.
        let _refused = self.delivery();
        Ok(())
    }

    async fn check_code(&self, email: &str, code: &str) -> Result<Identity, AuthError> {
        let people = self.guard()?;
        let person = people
            .iter()
            .find(|p| p.0.email == email)
            .filter(|_| code == MEMORY_CODE);
        person.map(|p| p.0.clone()).ok_or(AuthError::Refused)
    }

    async fn send_recovery(&self, email: &str) -> Result<(), AuthError> {
        self.send_code(email).await
    }

    async fn take_recovery(&self, email: &str, code: &str) -> Result<Identity, AuthError> {
        let people = self.guard()?;
        let person = people
            .iter()
            .find(|p| p.0.email == email)
            .filter(|_| code == MEMORY_CODE);
        person.map(|p| p.0.clone()).ok_or(AuthError::Refused)
    }

    async fn identity(&self, account: AccountId) -> Result<Identity, AuthError> {
        let people = self.guard()?;
        let person = people.iter().find(|p| p.0.account == account);
        person.map(|p| p.0.clone()).ok_or(AuthError::Refused)
    }

    fn password_ok(&self, password: &str) -> Result<(), AuthError> {
        // The same rule the built in service holds, so a test of what the door does with a
        // refused password has the same teeth here as in production.
        if password.len() > PASSWORD_MAX_BYTES {
            return Err(AuthError::WeakPassword);
        }
        Ok(())
    }

    async fn set_password(&self, account: AccountId, password: &str) -> Result<(), AuthError> {
        self.password_ok(password)?;
        self.change(account, |person| password.clone_into(&mut person.1))
    }

    async fn set_email(&self, account: AccountId, email: &str) -> Result<(), AuthError> {
        {
            // One address belongs to one person here as well: the real table holds it once, and a
            // double that took two would leave that rule untested.
            let people = self.guard()?;
            if people
                .iter()
                .any(|p| p.0.email == email && p.0.account != account)
            {
                return Err(AuthError::Taken);
            }
        }
        self.change(account, |person| email.clone_into(&mut person.0.email))
    }

    async fn set_phone(&self, account: AccountId, phone: &str) -> Result<(), AuthError> {
        self.change(account, |person| person.0.phone = Some(phone.to_owned()))
    }

    async fn delete(&self, account: AccountId) -> Result<(), AuthError> {
        self.guard()?.retain(|p| p.0.account != account);
        Ok(())
    }

    async fn find_email(&self, email: &str) -> Result<Option<AccountId>, AuthError> {
        let people = self.guard()?;
        Ok(people
            .iter()
            .find(|p| p.0.email == email)
            .map(|p| p.0.account))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering::SeqCst;

    use super::*;

    async fn with_ada() -> MemoryAuth {
        let auth = MemoryAuth::default();
        auth.sign_up("ada@example.test", "long-enough")
            .await
            .unwrap();
        auth
    }

    #[tokio::test]
    async fn a_taken_address_notice_tells_the_caller_nothing_about_the_address() {
        // The caller is told the same either way, which means a provider that refuses must not
        // answer an address that has an account differently from one that has none. Letting the
        // delivery's result through was the sign up leak again by another road.
        let auth = with_ada().await;
        auth.mail_down.store(true, SeqCst);
        assert_eq!(auth.send_taken_notice("ada@example.test").await, Ok(()));
        assert_eq!(auth.send_taken_notice("nobody@example.test").await, Ok(()));
        assert_eq!(
            auth.warned.lock().unwrap().clone(),
            ["ada@example.test"],
            "and only the holder of the address was written to"
        );
        // No mail at all is a fact about the server, so every address hears that refusal.
        auth.mail_off.store(true, SeqCst);
        for email in ["ada@example.test", "nobody@example.test"] {
            assert_eq!(
                auth.send_taken_notice(email).await,
                Err(AuthError::Unavailable)
            );
        }
    }

    #[tokio::test]
    async fn a_password_this_service_will_not_store_is_refused_before_it_is_stored() {
        // The door asks this before it ends any session, so the two answers have to agree.
        let auth = with_ada().await;
        let long = "x".repeat(PASSWORD_MAX_BYTES + 1);
        let account = auth.find_email("ada@example.test").await.unwrap().unwrap();
        assert_eq!(auth.password_ok(&long), Err(AuthError::WeakPassword));
        assert_eq!(
            auth.set_password(account, &long).await,
            Err(AuthError::WeakPassword)
        );
        assert_eq!(auth.password_ok("long-enough"), Ok(()));
        assert_eq!(auth.set_password(account, "long-enough").await, Ok(()));
    }
}
