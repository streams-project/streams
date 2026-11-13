//! The desktop host and the screens that come before a film.
//!
//! The core decides, the host performs its effects with what a desktop has, and `surfaces` lays
//! the screens out. The owner decided on 2026-11-13 that the desktop is its own app and not a web
//! page in a frame, which supersedes `DECISIONS.md` decision 2: the pointer screens before
//! playback are this family's now, measured against `reference/netflix-on-the-web.md`.
//!
//! No toolkit is linked here, and no dependency was added for the screens. A surface lays out
//! into `surfaces::scene::Scene` - rectangles, corners, colours by their brand role, words,
//! pictures and the caret - and a painter turns a scene into a window. Which painter is the
//! owner's to decide, and the screens do not change when it is decided. The fifth port, `reel`,
//! draws into a picture a surface's toolkit owns, so it is still named by whichever shell owns
//! one.
//!
//! The core is linked as a Rust library, not through `platform/bridges/c`: this host is Rust too,
//! so the bridge would only turn typed messages into bytes and back for no reader.

mod adapters;
mod host;
mod ports;
pub mod surfaces;

pub use adapters::{FileSecrets, FileStore, HttpNetwork, NoPlayer};
pub use host::{APP, Host, Opening, Ports, Reply, Settings, Turn};
pub use ports::{Answer, Call, Done, Network, Player, Secrets, Store};
pub use surfaces::{Ask, Face, Surface, Window};
