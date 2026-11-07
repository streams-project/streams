//! Subtitles from a separate program. It holds the provider and its key, so this server knows
//! neither, and the same few lines work for any provider someone adds as a pack.

use std::time::Duration;

use async_trait::async_trait;
use contract::{EpisodeRef, TitleKey};
use serde::Deserialize;

use super::source::{FoundFile, SubtitleError, SubtitleSource};

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
    client: reqwest::Client,
}

impl HttpSubtitles {
    /// A client for this program.
    ///
    /// # Errors
    /// The HTTP client could not be built.
    pub fn new(address: &str, token: &str) -> Result<Self, SubtitleError> {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(20))
            .build()
            .map_err(|_| SubtitleError::Unavailable)?;
        Ok(Self {
            address: address.trim_end_matches('/').to_owned(),
            token: token.to_owned(),
            client,
        })
    }
}

#[async_trait]
impl SubtitleSource for HttpSubtitles {
    async fn search(
        &self,
        title: TitleKey,
        episode: Option<&EpisodeRef>,
    ) -> Result<Vec<FoundFile>, SubtitleError> {
        let mut query = vec![("title", title.to_string())];
        if let Some(found) = episode {
            query.push(("season", found.season.to_string()));
            query.push(("episode", found.episode.to_string()));
        }
        let answer = self
            .client
            .get(format!("{}/v1/search", self.address))
            .bearer_auth(&self.token)
            .query(&query)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|_| SubtitleError::Unavailable)?;
        let offers: Offers = answer
            .json()
            .await
            .map_err(|_| SubtitleError::Unavailable)?;
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
        let answer = self
            .client
            .get(format!("{}/v1/file", self.address))
            .bearer_auth(&self.token)
            .query(&[("locator", locator)])
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|_| SubtitleError::Unavailable)?;
        let bytes = answer
            .bytes()
            .await
            .map_err(|_| SubtitleError::Unavailable)?;
        Ok(bytes.to_vec())
    }
}
