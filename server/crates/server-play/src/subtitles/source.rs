//! Where subtitle files come from. A source holds the keys and calls the outside service, so the key
//! stays on the server. Its real form needs the outbound guard from `platform`, and tests use a
//! scripted one.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use contract::play::{Cue as WireCue, SubtitleBody, SubtitleFile, SubtitleLanguage, SubtitleList};
use contract::{EpisodeRef, TitleKey};
use moka::sync::Cache;
use sha2::{Digest, Sha256};
use thiserror::Error;

use super::clean::{Cue, read_file, to_vtt};
use super::lang_tag;

/// Files kept per language.
const PER_LANGUAGE: usize = 3;
/// A list is kept this long.
const LIST_TTL_S: u64 = 6 * 3600;
const LISTS_MAX: u64 = 10_000;
/// Read files kept for reuse.
const FILES_MAX: u64 = 2_000;

/// One file a source offers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FoundFile {
    /// The source's own name for this file. Never leaves the server.
    pub locator: String,
    /// The language as the source writes it.
    pub language: String,
    pub hearing_impaired: bool,
}

/// Why a source gave nothing.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum SubtitleError {
    #[error("the source did not answer")]
    Unavailable,
    #[error("no such file")]
    Unknown,
    #[error("the file is not usable")]
    Unusable,
}

/// What subtitles need from the outside.
#[async_trait]
pub trait SubtitleSource: Send + Sync {
    /// Files for a title, in the source's own order.
    async fn search(
        &self,
        title: TitleKey,
        episode: Option<&EpisodeRef>,
    ) -> Result<Vec<FoundFile>, SubtitleError>;
    /// The bytes of one file.
    async fn fetch(&self, locator: &str) -> Result<Vec<u8>, SubtitleError>;
}

/// A source with files set by a test.
#[derive(Debug, Default)]
pub struct MemorySubtitles {
    pub files: Vec<(FoundFile, Vec<u8>)>,
}

#[async_trait]
impl SubtitleSource for MemorySubtitles {
    async fn search(
        &self,
        _: TitleKey,
        _: Option<&EpisodeRef>,
    ) -> Result<Vec<FoundFile>, SubtitleError> {
        Ok(self.files.iter().map(|(file, _)| file.clone()).collect())
    }

    async fn fetch(&self, locator: &str) -> Result<Vec<u8>, SubtitleError> {
        let found = self.files.iter().find(|(file, _)| file.locator == locator);
        found
            .map(|(_, bytes)| bytes.clone())
            .ok_or(SubtitleError::Unknown)
    }
}

#[derive(Clone, Debug)]
struct Held {
    title: TitleKey,
    locator: String,
    lang: String,
}

/// The list and file service.
pub struct Subtitles {
    source: Arc<dyn SubtitleSource>,
    held: Cache<String, Held>,
    read: Cache<String, Arc<Vec<Cue>>>,
}

impl std::fmt::Debug for Subtitles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Subtitles")
    }
}

/// An opaque file id. A locator holds an address and a key may hide in it, so it never leaves.
fn file_id(locator: &str) -> String {
    Sha256::digest(locator)
        .iter()
        .take(16)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

impl Subtitles {
    /// A service over a source.
    #[must_use]
    pub fn new(source: Arc<dyn SubtitleSource>) -> Self {
        let keep = Duration::from_secs(LIST_TTL_S);
        Self {
            source,
            held: Cache::builder()
                .max_capacity(LISTS_MAX)
                .time_to_live(keep)
                .build(),
            read: Cache::builder()
                .max_capacity(FILES_MAX)
                .time_to_live(keep)
                .build(),
        }
    }

    /// One entry per language: forced files out, hearing impaired after the others, three files
    /// at most, languages sorted by tag.
    ///
    /// # Errors
    /// The source did not answer.
    pub async fn list(
        &self,
        title: TitleKey,
        episode: Option<&EpisodeRef>,
    ) -> Result<SubtitleList, SubtitleError> {
        let found = self.source.search(title, episode).await?;
        let mut languages: Vec<SubtitleLanguage> = Vec::new();
        let mut entries: Vec<(String, String, bool, String)> = found
            .into_iter()
            .filter_map(|file| {
                let parsed = lang_tag::parse(&file.language)?;
                (!parsed.forced).then(|| {
                    let hearing = file.hearing_impaired || parsed.hi;
                    (parsed.tag, file.language, hearing, file.locator)
                })
            })
            .collect();
        entries.sort_by_key(|entry| entry.2);
        for (tag, label, hearing, locator) in entries {
            let id = file_id(&locator);
            self.held.insert(
                id.clone(),
                Held {
                    title,
                    locator,
                    lang: tag.clone(),
                },
            );
            let index = languages.iter().position(|language| language.lang == tag);
            let language = match index {
                Some(at) => &mut languages[at],
                None => {
                    languages.push(SubtitleLanguage {
                        lang: tag,
                        label,
                        files: Vec::new(),
                    });
                    let last = languages.len() - 1;
                    &mut languages[last]
                }
            };
            if language.files.len() < PER_LANGUAGE {
                language.files.push(SubtitleFile {
                    id,
                    hearing_impaired: hearing,
                });
            }
        }
        languages.sort_by(|a, b| a.lang.cmp(&b.lang));
        Ok(SubtitleList { languages })
    }

    /// The title a file id was listed for, so the rules run again before it is read.
    #[must_use]
    pub fn title_of(&self, id: &str) -> Option<TitleKey> {
        self.held.get(id).map(|held| held.title)
    }

    async fn cues(&self, id: &str) -> Result<Arc<Vec<Cue>>, SubtitleError> {
        if let Some(cues) = self.read.get(id) {
            return Ok(cues);
        }
        let held = self.held.get(id).ok_or(SubtitleError::Unknown)?;
        let bytes = self.source.fetch(&held.locator).await?;
        let cues = read_file(&bytes, &held.lang).map_err(|_| SubtitleError::Unusable)?;
        let cues = Arc::new(cues);
        self.read.insert(id.to_owned(), cues.clone());
        Ok(cues)
    }

    /// Clean cues of one file. A refused file answers `Unusable`, and the core moves on.
    ///
    /// # Errors
    /// See [`SubtitleError`].
    pub async fn body(&self, id: &str) -> Result<SubtitleBody, SubtitleError> {
        let cues = self.cues(id).await?;
        let ms = |seconds: f64| (seconds.max(0.0) * 1000.0).round() as u64;
        let wire: Vec<WireCue> = cues
            .iter()
            .map(|cue| WireCue {
                start_ms: ms(cue.s),
                end_ms: ms(cue.e),
                text: cue.t.clone(),
            })
            .collect();
        let last_end_ms = wire.iter().map(|cue| cue.end_ms).max().unwrap_or(0);
        Ok(SubtitleBody {
            cues: wire,
            last_end_ms,
        })
    }

    /// WebVTT of one file, for players that load a side file.
    ///
    /// # Errors
    /// See [`SubtitleError`].
    pub async fn vtt(&self, id: &str, delay_ms: i64) -> Result<String, SubtitleError> {
        Ok(to_vtt(&self.cues(id).await?, delay_ms))
    }

    /// The cues the detector reads, for a file of this title.
    ///
    /// # Errors
    /// See [`SubtitleError`].
    pub async fn detector_cues(&self, id: &str) -> Result<Arc<Vec<Cue>>, SubtitleError> {
        self.cues(id).await
    }
}
