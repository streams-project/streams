//! Mail: which language to write in, the wording, the layout, the check of the auth service's
//! signature and the reading of its hook. Sending is one HTTP call to the provider the operator set.

pub mod bundle;
pub mod hook;
pub mod render;
pub mod webhook;

use std::time::Duration;

use async_trait::async_trait;
use thiserror::Error;

/// Why a message did not leave.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[error("the mail provider refused: {0}")]
pub struct MailError(pub String);

/// One message, ready to send.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Letter {
    pub to: String,
    pub subject: String,
    pub html: String,
    pub text: String,
}

/// What mail needs from a provider.
#[async_trait]
pub trait MailSender: Send + Sync {
    /// Send one message.
    async fn send(&self, letter: &Letter) -> Result<(), MailError>;
}

/// A provider in memory, for tests.
#[derive(Debug, Default)]
pub struct MemoryMail {
    pub sent: std::sync::Mutex<Vec<Letter>>,
}

#[async_trait]
impl MailSender for MemoryMail {
    async fn send(&self, letter: &Letter) -> Result<(), MailError> {
        self.sent
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(letter.clone());
        Ok(())
    }
}

/// A provider over HTTP: one JSON body and a bearer token.
#[derive(Clone, Debug)]
pub struct HttpMail {
    address: String,
    token: String,
    sender: String,
    client: reqwest::Client,
}

impl HttpMail {
    /// A client for a provider.
    ///
    /// # Errors
    /// The HTTP client could not be built.
    pub fn new(address: &str, token: &str, sender: &str) -> Result<Self, MailError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|error| MailError(error.to_string()))?;
        Ok(Self {
            address: address.to_owned(),
            token: token.to_owned(),
            sender: sender.to_owned(),
            client,
        })
    }
}

#[async_trait]
impl MailSender for HttpMail {
    async fn send(&self, letter: &Letter) -> Result<(), MailError> {
        let body = serde_json::json!({
            "from": self.sender,
            "to": [letter.to],
            "subject": letter.subject,
            "html": letter.html,
            "text": letter.text,
        });
        let answer = self
            .client
            .post(&self.address)
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await
            .map_err(|error| MailError(error.to_string()))?;
        let status = answer.status();
        if status.is_success() {
            return Ok(());
        }
        let detail: String = answer
            .text()
            .await
            .unwrap_or_default()
            .chars()
            .take(200)
            .collect();
        Err(MailError(format!("{status}: {detail}")))
    }
}
