#![no_std]
//! The one written promise between server, core, finder and device.
//!
//! Wire types only: no IO, no rules that need data, no address, no brand. Everything here is
//! plain JSON. Integers stay under 2^53, ids are strings, unknown fields are ignored, and an
//! unknown enum word reads as `Unknown`, so an old client keeps working when a new one appears.

extern crate alloc;

#[macro_use]
mod macros;
pub mod account;
pub mod address;
pub mod api;
pub mod client;
pub mod command;
pub mod core_msg;
pub mod driver;
pub mod error;
pub mod finder;
pub mod hello;
pub mod help;
pub mod notification;
pub mod play;
pub mod screen;
mod title;
pub mod version;

pub use title::{
    EpisodeRef, MAX_KEYS_PER_LIST, MAX_TITLE_ID, Rating, TitleKey, TitleKeyError, TitleKind,
    is_completed, watch_percent,
};
