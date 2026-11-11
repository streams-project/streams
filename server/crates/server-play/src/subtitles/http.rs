//! Subtitles from a separate program. It holds the provider and its key, so this server knows
//! neither, and the same few lines work for any provider someone adds as a pack.

use async_trait::async_trait;
use contract::{EpisodeRef, TitleKey};
use net_guard::{Ask, Guard, Limits};
use serde::Deserialize;

use super::clean::MAX_BYTES;
use super::source::{FoundFile, SubtitleError, SubtitleSource};

/// The largest file list a pack may send. A list is a few dozen short entries.
const LIST_BYTES: usize = 256 * 1024;

#[derive(Debug, Deserialize)]
struct Offered {
    locator: String,
    language: String,
    #[serde(default)]
    hearing_impaired: bool,
}

#[derive(Debug, Deserialize)]
struct Offers {
    files: Vec<Offered>,
}

/// A subtitle program over HTTP.
#[derive(Clone, Debug)]
pub struct HttpSubtitles {
    address: String,
    token: String,
    guard: Guard,
}

impl HttpSubtitles {
    /// A client for this program.
    ///
    /// # Errors
    /// The HTTP client could not be built.
    pub fn new(address: &str, token: &str) -> Result<Self, SubtitleError> {
        let address = address.trim_end_matches('/').to_owned();
        // The pack's address is this install's own configuration, so it is allowed to be loopback,
        // which is where the launcher puts it. A locator the pack hands back is not configuration
        // and gets no such allowance.
        let guard = Guard::new(Limits::fetch(MAX_BYTES), std::slice::from_ref(&address))
            .map_err(|_| SubtitleError::Unavailable)?;
        Ok(Self {
            address,
            token: token.to_owned(),
            guard,
        })
    }

    async fn read(&self, ask: &Ask, bytes: usize) -> Result<Vec<u8>, SubtitleError> {
        let answer = self
            .guard
            .fetch_with(ask, Limits::fetch(bytes))
            .await
            .map_err(|_| SubtitleError::Unavailable)?;
        if !answer.ok() {
            return Err(SubtitleError::Unavailable);
        }
        Ok(answer.bytes)
    }
}

#[async_trait]
impl SubtitleSource for HttpSubtitles {
    async fn search(
        &self,
        title: TitleKey,
        episode: Option<&EpisodeRef>,
    ) -> Result<Vec<FoundFile>, SubtitleError> {
        let mut ask = Ask::get(format!("{}/v1/search", self.address))
            .bearer(&self.token)
            .query("title", title.to_string());
        if let Some(found) = episode {
            ask = ask
                .query("season", found.season.to_string())
                .query("episode", found.episode.to_string());
        }
        let bytes = self.read(&ask, LIST_BYTES).await?;
        let offers: Offers =
            serde_json::from_slice(&bytes).map_err(|_| SubtitleError::Unavailable)?;
        Ok(offers
            .files
            .into_iter()
            .map(|file| FoundFile {
                locator: file.locator,
                language: file.language,
                hearing_impaired: file.hearing_impaired,
            })
            .collect())
    }

    async fn fetch(&self, locator: &str) -> Result<Vec<u8>, SubtitleError> {
        // The cap is the one the reader judges a file by, so a file above it stops arriving at
        // that byte instead of being held whole and refused afterwards.
        let ask = Ask::get(format!("{}/v1/file", self.address))
            .bearer(&self.token)
            .query("locator", locator);
        self.read(&ask, MAX_BYTES).await
    }
}
