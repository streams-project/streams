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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    /// A subtitle pack over raw TCP. Raw, because what is proved here is this module's own caps
    /// and the guard behind them, not what any library does with a URL. It answers the same list
    /// and the same file whatever it is asked, so each test reads one size.
    async fn pack(list: Vec<u8>, file: Vec<u8>) -> HttpSubtitles {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("a port");
        let address = format!("http://{}", listener.local_addr().expect("an address"));
        tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                let (list, file) = (list.clone(), file.clone());
                tokio::spawn(async move {
                    let mut head = [0u8; 4096];
                    let read = stream.read(&mut head).await.unwrap_or(0);
                    let asked = String::from_utf8_lossy(&head[..read]).into_owned();
                    let body = if asked.contains("/v1/search") {
                        list
                    } else {
                        file
                    };
                    let answer = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
                        body.len()
                    );
                    let _ = stream.write_all(answer.as_bytes()).await;
                    let _ = stream.write_all(&body).await;
                    let _ = stream.flush().await;
                });
            }
        });
        // The pack's address is this install's own configuration, which is what lets it be
        // loopback. Nothing a pack hands back gets that allowance.
        HttpSubtitles::new(&address, "token").expect("a client")
    }

    fn a_list(entries: usize) -> Vec<u8> {
        let files: Vec<String> = (0..entries)
            .map(|n| format!(r#"{{"locator":"file-{n}","language":"en"}}"#))
            .collect();
        format!(r#"{{"files":[{}]}}"#, files.join(",")).into_bytes()
    }

    fn title() -> TitleKey {
        TitleKey::parse("movie:603").expect("a title")
    }

    /// The caps are the point of this module and nothing was pinning them: a regression to a
    /// whole-body read, or to a different number here from the one the reader judges a file by,
    /// would have passed the suite in silence.
    #[tokio::test]
    async fn h74_a_list_over_the_cap_is_not_read() {
        let small = pack(a_list(20), Vec::new()).await;
        assert_eq!(
            small.search(title(), None).await.expect("a list").len(),
            20,
            "a list well under the cap did not arrive"
        );
        // Each entry is about 44 bytes, so this is comfortably past the 256 kB a list may be.
        let big = a_list(7_000);
        assert!(
            big.len() > LIST_BYTES,
            "the test's own list is not over the cap"
        );
        let over = pack(big, Vec::new()).await;
        assert_eq!(
            over.search(title(), None).await,
            Err(SubtitleError::Unavailable),
            "a list of any size was read"
        );
    }

    /// `clean::MAX_BYTES` is the one number: it stops the fetch and it judges the file. The
    /// comment that said so was false once already, which is this project's own H75.
    #[tokio::test]
    async fn c43_h75_a_file_is_fetched_up_to_the_byte_the_reader_judges_it_by() {
        let at_the_cap = pack(Vec::new(), vec![b'x'; MAX_BYTES]).await;
        assert_eq!(
            at_the_cap.fetch("f").await.expect("the file").len(),
            MAX_BYTES,
            "a file at the cap was refused"
        );
        let over = pack(Vec::new(), vec![b'x'; MAX_BYTES + 1]).await;
        assert_eq!(
            over.fetch("f").await,
            Err(SubtitleError::Unavailable),
            "one byte past the cap was read"
        );
    }
}
