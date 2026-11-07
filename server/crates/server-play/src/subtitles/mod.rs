//! Subtitles and skip marks. One clean result for every device: the server lists, fetches, reads
//! and judges files, and devices compare player time to windows.

pub mod clean;
pub mod detect;
mod http;
pub mod lang_tag;
mod source;

pub use http::HttpSubtitles;
pub use source::{FoundFile, MemorySubtitles, SubtitleError, SubtitleSource, Subtitles};
