//! Help text, kept once on the server. A folder holds one file per language, and every app reads
//! the same articles through these functions, so nothing is written per platform.

mod book;
mod markup;

pub use book::{Gap, Help, HelpError, Named};
pub use markup::{parse, plain};
