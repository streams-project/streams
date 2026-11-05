#![no_std]
//! The one written promise between server, core, finder and device.
//!
//! Wire types only: no IO, no rules that need data, no address, no brand. Everything here is
//! plain JSON. Integers stay under 2^53, ids are strings, unknown fields are ignored, and an
//! unknown enum word reads as `Unknown`, so an old client keeps working when a new one appears.

extern crate alloc;

#[macro_use]
mod macros;
mod title;

pub use title::{
    EpisodeRef, MAX_KEYS_PER_LIST, MAX_TITLE_ID, Rating, TitleKey, TitleKeyError, TitleKind,
    is_completed, watch_percent,
};
