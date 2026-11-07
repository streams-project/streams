//! The play gate: eligibility first, then the finder, then one ranking, then the card. Provider
//! names stop here, and video bytes never cross this server.

mod finder;
mod gate;
mod log;
mod marks;
mod rank;
pub mod subtitles;

pub use finder::{Finder, FinderError, HttpFinder, MemoryFinder};
pub use gate::{Play, PlayAsk, PlayError};
pub use log::{MemoryPlayLog, PgPlayLog, PlayLog};
pub use marks::{marks_of, pick_marks, pick_reasons, windows_of};
pub use rank::{Viewing, audio_score, keep_for, rank, usable_headers};
pub use subtitles::{
    FoundFile, HttpSubtitles, MemorySubtitles, SubtitleError, SubtitleSource, Subtitles,
};
