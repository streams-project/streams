//! The play gate: eligibility first, then the finder, then one ranking, then the card. Provider
//! names stop here, and video bytes never cross this server.

mod finder;
mod gate;
mod log;
mod marks;
mod rank;

pub use finder::{Finder, FinderError, HttpFinder, MemoryFinder};
pub use gate::{Play, PlayAsk, PlayError};
pub use log::{MemoryPlayLog, PgPlayLog, PlayLog};
pub use marks::{pick_marks, pick_reasons};
pub use rank::{Viewing, audio_score, keep_for, rank, usable_headers};
