//! Stable values per session, so a page changes between sessions and never within one.

use std::hash::{Hash, Hasher};

/// A uniform value in (0, 1) from a session and a number. Uses a fixed hash, so a session keeps its
/// page across builds and toolchains.
#[must_use]
pub fn session_uniform(session: &str, id: u32) -> f64 {
    let mut hasher = rustc_hash::FxHasher::default();
    session.hash(&mut hasher);
    id.hash(&mut hasher);
    let mixed = hasher.finish().wrapping_mul(0x9E37_79B9_7F4A_7C15);
    ((mixed >> 11) as f64 + 1.0) / ((1u64 << 53) as f64 + 1.0)
}
