//! The catalogue: one read only SQLite file loaded into a map behind an atomic pointer. It is read
//! once at load and at reload, never on the request path.

use crate::det::HashMap;
use anyhow::Result;
use arc_swap::ArcSwap;
use rusqlite::{Connection, OpenFlags};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tracing::{info, warn};

mod certs;
mod index;
mod item;
mod keys;
mod load;
mod overlays;

pub use certs::*;
pub use index::*;
pub use item::*;
pub use keys::*;
pub use load::{load_catalog, make_shared_catalog};
pub use overlays::*;

#[cfg(test)]
mod tests;
