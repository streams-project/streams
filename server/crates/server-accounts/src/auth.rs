//! The auth service, reached only from here. It checks passwords and codes and holds the password
//! hash; this server keeps its own opaque tokens, so replacing it changes no client.

use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

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

/// What the door needs from the auth service.
#[async_trait]
pub trait AuthService: Send + Sync {
    /// Make an account.
    async fn sign_up(&self, email: &str, password: &str) -> Result<Identity, AuthError>;
    /// Check a password.
    async fn check_password(&self, email: &str, password: &str) -> Result<Identity, AuthError>;
    /// Send a sign in code. Sent or not, the answer is the same.
    async fn send_code(&self, email: &str) -> Result<(), AuthError>;
    /// Check a sign in code.
    async fn check_code(&self, email: &str, code: &str) -> Result<Identity, AuthError>;
    /// Send a recovery code.
    async fn send_recovery(&self, email: &str) -> Result<(), AuthError>;
    /// Set a new password with a recovery code.
    async fn finish_recovery(
        &self,
        email: &str,
        code: &str,
        password: &str,
    ) -> Result<(), AuthError>;
    /// Read a person.
    async fn identity(&self, account: AccountId) -> Result<Identity, AuthError>;
    /// Change the password.
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

/// An auth service in memory, for tests. Codes are fixed so a test can type them.
#[derive(Debug, Default)]
pub struct MemoryAuth {
    people: Mutex<Vec<(Identity, String)>>,
    /// Sent codes, for tests to read.
    pub sent: Mutex<Vec<String>>,
    /// True makes every call fail as an outage.
    pub down: std::sync::atomic::AtomicBool,
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
        drop(self.guard()?);
        self.sent
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(email.to_owned());
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

    async fn finish_recovery(
        &self,
        email: &str,
        code: &str,
        password: &str,
    ) -> Result<(), AuthError> {
        let mut people = self.guard()?;
        let person = people
            .iter_mut()
            .find(|p| p.0.email == email)
            .filter(|_| code == MEMORY_CODE);
        person.ok_or(AuthError::Refused)?.1 = password.to_owned();
        Ok(())
    }

    async fn identity(&self, account: AccountId) -> Result<Identity, AuthError> {
        let people = self.guard()?;
        let person = people.iter().find(|p| p.0.account == account);
        person.map(|p| p.0.clone()).ok_or(AuthError::Refused)
    }

    async fn set_password(&self, account: AccountId, password: &str) -> Result<(), AuthError> {
        self.change(account, |person| password.clone_into(&mut person.1))
    }

    async fn set_email(&self, account: AccountId, email: &str) -> Result<(), AuthError> {
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

/// How long an auth call may take.
const CONNECT_S: u64 = 5;
const TOTAL_S: u64 = 8;

/// The auth service over HTTP, on an address only this server reaches.
#[derive(Clone, Debug)]
pub struct HttpAuth {
    base: String,
    admin_token: String,
    client: reqwest::Client,
}

#[derive(Deserialize)]
struct User {
    id: Uuid,
    #[serde(default)]
    email: String,
    #[serde(default)]
    phone: String,
}

impl User {
    fn identity(self) -> Identity {
        Identity {
            account: AccountId(self.id),
            email: self.email,
            phone: (!self.phone.is_empty()).then_some(self.phone),
        }
    }
}

/// A sign in answers `{ user }`, a sign up answers the user alone.
#[derive(Deserialize)]
#[serde(untagged)]
enum UserAnswer {
    Session { user: User },
    Plain(User),
}

#[derive(Deserialize)]
struct UserList {
    users: Vec<User>,
}

#[allow(clippy::needless_pass_by_value)]
fn outage(_: reqwest::Error) -> AuthError {
    AuthError::Unavailable
}

fn check(status: reqwest::StatusCode, body: &str) -> Result<(), AuthError> {
    if status.is_success() {
        return Ok(());
    }
    if status.is_server_error() {
        return Err(AuthError::Unavailable);
    }
    if body.contains("weak_password") {
        return Err(AuthError::WeakPassword);
    }
    if body.contains("email_exists") || body.contains("user_already_exists") {
        return Err(AuthError::Taken);
    }
    Err(AuthError::Refused)
}

impl HttpAuth {
    /// A client for the service at `base`.
    ///
    /// # Errors
    /// The HTTP client could not be built.
    pub fn new(base: &str, admin_token: &str) -> Result<Self, AuthError> {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(CONNECT_S))
            .timeout(Duration::from_secs(TOTAL_S))
            .build()
            .map_err(outage)?;
        Ok(Self {
            base: base.trim_end_matches('/').to_owned(),
            admin_token: admin_token.to_owned(),
            client,
        })
    }

    async fn post(&self, path: &str, body: serde_json::Value) -> Result<String, AuthError> {
        let answer = self
            .client
            .post(format!("{}{path}", self.base))
            .json(&body)
            .send()
            .await
            .map_err(outage)?;
        let status = answer.status();
        let text = answer.text().await.map_err(outage)?;
        check(status, &text)?;
        Ok(text)
    }

    async fn admin(&self, request: reqwest::RequestBuilder) -> Result<String, AuthError> {
        let answer = request
            .bearer_auth(&self.admin_token)
            .send()
            .await
            .map_err(outage)?;
        let status = answer.status();
        let text = answer.text().await.map_err(outage)?;
        check(status, &text)?;
        Ok(text)
    }

    async fn update(&self, account: AccountId, body: serde_json::Value) -> Result<(), AuthError> {
        let address = format!("{}/admin/users/{account}", self.base);
        self.admin(self.client.put(address).json(&body))
            .await
            .map(|_| ())
    }
}

fn identity_of(text: &str) -> Result<Identity, AuthError> {
    let answer: UserAnswer = serde_json::from_str(text).map_err(|_| AuthError::Unavailable)?;
    Ok(match answer {
        UserAnswer::Session { user } | UserAnswer::Plain(user) => user.identity(),
    })
}

#[async_trait]
impl AuthService for HttpAuth {
    async fn sign_up(&self, email: &str, password: &str) -> Result<Identity, AuthError> {
        identity_of(
            &self
                .post("/signup", json!({ "email": email, "password": password }))
                .await?,
        )
    }

    async fn check_password(&self, email: &str, password: &str) -> Result<Identity, AuthError> {
        let body = json!({ "email": email, "password": password });
        identity_of(&self.post("/token?grant_type=password", body).await?)
    }

    async fn send_code(&self, email: &str) -> Result<(), AuthError> {
        self.post("/otp", json!({ "email": email, "create_user": false }))
            .await
            .map(|_| ())
    }

    async fn check_code(&self, email: &str, code: &str) -> Result<Identity, AuthError> {
        let body = json!({ "type": "email", "email": email, "token": code });
        identity_of(&self.post("/verify", body).await?)
    }

    async fn send_recovery(&self, email: &str) -> Result<(), AuthError> {
        self.post("/recover", json!({ "email": email }))
            .await
            .map(|_| ())
    }

    async fn finish_recovery(
        &self,
        email: &str,
        code: &str,
        password: &str,
    ) -> Result<(), AuthError> {
        let body = json!({ "type": "recovery", "email": email, "token": code });
        let person = identity_of(&self.post("/verify", body).await?)?;
        self.set_password(person.account, password).await
    }

    async fn identity(&self, account: AccountId) -> Result<Identity, AuthError> {
        let address = format!("{}/admin/users/{account}", self.base);
        identity_of(&self.admin(self.client.get(address)).await?)
    }

    async fn set_password(&self, account: AccountId, password: &str) -> Result<(), AuthError> {
        self.update(account, json!({ "password": password })).await
    }

    async fn set_email(&self, account: AccountId, email: &str) -> Result<(), AuthError> {
        self.update(account, json!({ "email": email, "email_confirm": true }))
            .await
    }

    async fn set_phone(&self, account: AccountId, phone: &str) -> Result<(), AuthError> {
        self.update(account, json!({ "phone": phone })).await
    }

    async fn delete(&self, account: AccountId) -> Result<(), AuthError> {
        let address = format!("{}/admin/users/{account}", self.base);
        self.admin(self.client.delete(address)).await.map(|_| ())
    }

    async fn find_email(&self, email: &str) -> Result<Option<AccountId>, AuthError> {
        let address = format!("{}/admin/users", self.base);
        let query = [("filter", email)];
        let text = self.admin(self.client.get(address).query(&query)).await?;
        let list: UserList = serde_json::from_str(&text).map_err(|_| AuthError::Unavailable)?;
        Ok(list
            .users
            .into_iter()
            .find(|user| user.email == email)
            .map(|user| AccountId(user.id)))
    }
}
