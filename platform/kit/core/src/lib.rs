//! Proof kit for the core: scripted sessions run through the JSON boundary, with the offline
//! layer on and off, plus the gates that keep the core free of std, async, a clock and IO.

pub mod boundary;
pub mod host;
pub mod scan;
pub mod sessions;
pub mod wasm;
