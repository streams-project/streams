//! Sign in inside the server, so a deployment needs no separate auth service. Passwords are bcrypt
//! hashes in `auth.users`, the same table and hash the outside service kept, so accounts carry over.
//! Codes are random, hashed, single use, expire in ten minutes and allow five tries.

use std::sync::Arc;

use async_trait::async_trait;
use rand::Rng;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::auth::{AuthError, AuthService, Identity};
use crate::mail::render::{Brand, Kind, Request, render};
use crate::mail::{Letter, MailSender};
use crate::profile::AccountId;

/// Digits in a code.
pub const CODE_DIGITS: u32 = 8;
const CODE_MINUTES: i32 = 10;
const MAX_TRIES: i32 = 5;
const PASSWORD_MIN: usize = 8;
/// bcrypt reads no further than this, so a longer password would be silently cut.
const PASSWORD_MAX_BYTES: usize = 72;
const SIGN_IN: &str = "sign_in";
const RECOVERY: &str = "recovery";

/// A hash to check when the address is unknown, so a wrong address takes as long as a wrong password.
const DECOY_HASH: &str = "$2b$12$C6UzMDM.H6dfI/f/IKcEeO7ZjfnSJIxbJqfTvRCzNgKqHmOqK6c8C";

/// How a code reaches a person.
#[async_trait]
pub trait CodePost: Send + Sync {
    /// Send this code. `recovery` picks the wording.
    async fn deliver(&self, email: &str, code: &str, recovery: bool) -> Result<(), AuthError>;
}

/// Codes by the deployment's mail service. With mail off a code has no way out, so asking for one
/// answers unavailable and people sign in with their password.
pub struct MailPost {
    pub mail: Option<Arc<dyn MailSender>>,
    pub name: String,
    pub help_email: String,
    pub site_url: String,
    pub help_url: String,
}

#[async_trait]
impl CodePost for MailPost {
    async fn deliver(&self, email: &str, code: &str, recovery: bool) -> Result<(), AuthError> {
        let Some(mail) = &self.mail else {
            return Err(AuthError::Unavailable);
        };
        let brand = Brand {
            name: &self.name,
            help_email: &self.help_email,
            site_url: &self.site_url,
            help_url: &self.help_url,
        };
        let page = render(
            &brand,
            &Request {
                kind: if recovery { Kind::Recovery } else { Kind::Code },
                lang: "en".to_owned(),
                to: email.to_owned(),
                name: None,
                profile: None,
                link: None,
                code: Some(code.to_owned()),
                minutes: CODE_MINUTES.unsigned_abs(),
                new_email: None,
                device: None,
            },
        );
        let letter = Letter {
            to: email.to_owned(),
            subject: page.subject,
            html: page.html,
            text: page.text,
        };
        mail.send(&letter).await.map_err(|_| AuthError::Unavailable)
    }
}

/// Sign in over the server's own database.
pub struct BuiltInAuth {
    pool: PgPool,
    post: Arc<dyn CodePost>,
}

impl BuiltInAuth {
    /// Sign in on this database, sending codes through `post`.
    #[must_use]
    pub fn new(pool: PgPool, post: Arc<dyn CodePost>) -> Self {
        Self { pool, post }
    }

    async fn issue(&self, email: &str, purpose: &str) -> Result<(), AuthError> {
        let email = address(email);
        let known = self.find_email(&email).await?.is_some();
        // Sent or not, the answer is the same.
        if !known {
            return Ok(());
        }
        let code = new_code();
        sqlx::query(
            "insert into auth.login_codes (email, purpose, code_hash, expires_at) \
             values ($1, $2, $3, now() + make_interval(mins => $4)) \
             on conflict (email, purpose) do update \
             set code_hash = excluded.code_hash, expires_at = excluded.expires_at, tries = 0",
        )
        .bind(&email)
        .bind(purpose)
        .bind(code_hash(&email, &code))
        .bind(CODE_MINUTES)
        .execute(&self.pool)
        .await
        .map_err(gone)?;
        self.post.deliver(&email, &code, purpose == RECOVERY).await
    }

    /// True once for a right code. A wrong one spends a try, and the last try ends the code.
    async fn spend(&self, email: &str, purpose: &str, code: &str) -> Result<bool, AuthError> {
        let row = sqlx::query(
            "update auth.login_codes set tries = tries + 1 \
             where email = $1 and purpose = $2 and expires_at > now() and tries < $3 \
             returning code_hash",
        )
        .bind(email)
        .bind(purpose)
        .bind(MAX_TRIES)
        .fetch_optional(&self.pool)
        .await
        .map_err(gone)?;
        let Some(row) = row else { return Ok(false) };
        let right = row.get::<String, _>("code_hash") == code_hash(email, code);
        if right {
            sqlx::query("delete from auth.login_codes where email = $1 and purpose = $2")
                .bind(email)
                .bind(purpose)
                .execute(&self.pool)
                .await
                .map_err(gone)?;
        }
        Ok(right)
    }

    async fn by_email(&self, email: &str) -> Result<Option<(Identity, Option<String>)>, AuthError> {
        let row = sqlx::query(
            "select id, email, phone, encrypted_password from auth.users where lower(email) = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(gone)?;
        Ok(row.map(|row| (person(&row), row.get("encrypted_password"))))
    }
}

fn person(row: &sqlx::postgres::PgRow) -> Identity {
    Identity {
        account: AccountId(row.get("id")),
        email: row.get::<Option<String>, _>("email").unwrap_or_default(),
        phone: row.get("phone"),
    }
}

/// A database that did not answer is an outage, never a sign out.
fn gone(_: sqlx::Error) -> AuthError {
    AuthError::Unavailable
}

fn address(email: &str) -> String {
    email.trim().to_lowercase()
}

fn new_code() -> String {
    let mut random = rand::rng();
    (0..CODE_DIGITS)
        .map(|_| char::from(b'0' + random.random_range(0..10u8)))
        .collect()
}

fn code_hash(email: &str, code: &str) -> String {
    let digest = Sha256::digest(format!("{email}\n{code}"));
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn strong_enough(password: &str) -> Result<(), AuthError> {
    let length = password.chars().count();
    if length < PASSWORD_MIN || password.len() > PASSWORD_MAX_BYTES {
        return Err(AuthError::WeakPassword);
    }
    Ok(())
}

async fn hash(password: &str) -> Result<String, AuthError> {
    let password = password.to_owned();
    tokio::task::spawn_blocking(move || bcrypt::hash(password, bcrypt::DEFAULT_COST))
        .await
        .map_err(|_| AuthError::Unavailable)?
        .map_err(|_| AuthError::Unavailable)
}

async fn verify(password: &str, hash: &str) -> bool {
    let password = password.to_owned();
    let hash = hash.to_owned();
    tokio::task::spawn_blocking(move || bcrypt::verify(password, &hash).unwrap_or(false))
        .await
        .unwrap_or(false)
}

#[async_trait]
impl AuthService for BuiltInAuth {
    async fn sign_up(&self, email: &str, password: &str) -> Result<Identity, AuthError> {
        strong_enough(password)?;
        let email = address(email);
        let hashed = hash(password).await?;
        let row = sqlx::query(
            "insert into auth.users (id, email, encrypted_password) values ($1, $2, $3) \
             on conflict do nothing returning id, email, phone",
        )
        .bind(Uuid::new_v4())
        .bind(&email)
        .bind(hashed)
        .fetch_optional(&self.pool)
        .await
        .map_err(gone)?;
        row.map(|row| person(&row)).ok_or(AuthError::Taken)
    }

    async fn check_password(&self, email: &str, password: &str) -> Result<Identity, AuthError> {
        let Some((identity, Some(stored))) = self.by_email(&address(email)).await? else {
            let _ = verify(password, DECOY_HASH).await;
            return Err(AuthError::Refused);
        };
        if verify(password, &stored).await {
            Ok(identity)
        } else {
            Err(AuthError::Refused)
        }
    }

    async fn send_code(&self, email: &str) -> Result<(), AuthError> {
        self.issue(email, SIGN_IN).await
    }

    async fn check_code(&self, email: &str, code: &str) -> Result<Identity, AuthError> {
        let email = address(email);
        if !self.spend(&email, SIGN_IN, code).await? {
            return Err(AuthError::Refused);
        }
        let found = self.by_email(&email).await?;
        found
            .map(|(identity, _)| identity)
            .ok_or(AuthError::Refused)
    }

    async fn send_recovery(&self, email: &str) -> Result<(), AuthError> {
        self.issue(email, RECOVERY).await
    }

    async fn finish_recovery(
        &self,
        email: &str,
        code: &str,
        password: &str,
    ) -> Result<(), AuthError> {
        strong_enough(password)?;
        let email = address(email);
        if !self.spend(&email, RECOVERY, code).await? {
            return Err(AuthError::Refused);
        }
        let hashed = hash(password).await?;
        sqlx::query("update auth.users set encrypted_password = $1 where lower(email) = $2")
            .bind(hashed)
            .bind(&email)
            .execute(&self.pool)
            .await
            .map_err(gone)?;
        Ok(())
    }

    async fn identity(&self, account: AccountId) -> Result<Identity, AuthError> {
        let row = sqlx::query("select id, email, phone from auth.users where id = $1")
            .bind(account.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(gone)?;
        row.map(|row| person(&row)).ok_or(AuthError::Refused)
    }

    async fn set_password(&self, account: AccountId, password: &str) -> Result<(), AuthError> {
        strong_enough(password)?;
        let hashed = hash(password).await?;
        sqlx::query("update auth.users set encrypted_password = $1 where id = $2")
            .bind(hashed)
            .bind(account.0)
            .execute(&self.pool)
            .await
            .map_err(gone)?;
        Ok(())
    }

    async fn set_email(&self, account: AccountId, email: &str) -> Result<(), AuthError> {
        let done = sqlx::query("update auth.users set email = $1 where id = $2")
            .bind(address(email))
            .bind(account.0)
            .execute(&self.pool)
            .await;
        match done {
            Ok(_) => Ok(()),
            Err(sqlx::Error::Database(error)) if error.is_unique_violation() => {
                Err(AuthError::Taken)
            }
            Err(error) => Err(gone(error)),
        }
    }

    async fn set_phone(&self, account: AccountId, phone: &str) -> Result<(), AuthError> {
        sqlx::query("update auth.users set phone = $1 where id = $2")
            .bind(phone)
            .bind(account.0)
            .execute(&self.pool)
            .await
            .map_err(gone)?;
        Ok(())
    }

    async fn delete(&self, account: AccountId) -> Result<(), AuthError> {
        sqlx::query("delete from auth.users where id = $1")
            .bind(account.0)
            .execute(&self.pool)
            .await
            .map_err(gone)?;
        Ok(())
    }

    async fn find_email(&self, email: &str) -> Result<Option<AccountId>, AuthError> {
        let found = self.by_email(&address(email)).await?;
        Ok(found.map(|(identity, _)| identity.account))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_code_is_eight_digits_and_they_vary() {
        let first = new_code();
        assert_eq!(first.len(), 8);
        assert!(first.chars().all(|c| c.is_ascii_digit()));
        assert!((0..20).any(|_| new_code() != first));
    }

    #[test]
    fn a_code_is_kept_only_as_a_hash_tied_to_its_address() {
        let hashed = code_hash("a@example.test", "12345678");
        assert!(!hashed.contains("12345678"));
        assert_ne!(hashed, code_hash("b@example.test", "12345678"));
    }

    #[test]
    fn a_short_password_and_one_bcrypt_would_cut_are_refused() {
        assert_eq!(strong_enough("short"), Err(AuthError::WeakPassword));
        assert_eq!(strong_enough(&"x".repeat(73)), Err(AuthError::WeakPassword));
        assert_eq!(strong_enough("long-enough"), Ok(()));
    }

    #[test]
    fn the_decoy_is_a_real_hash_so_an_unknown_address_costs_the_same_time() {
        assert!(!bcrypt::verify("anything", DECOY_HASH).unwrap());
    }

    #[test]
    fn an_address_is_compared_without_case_or_spaces() {
        assert_eq!(address("  Ada@Example.TEST "), "ada@example.test");
    }
}
