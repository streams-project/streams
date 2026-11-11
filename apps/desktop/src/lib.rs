//! The desktop host: one more host of the core, as native code.
//!
//! The core decides, this performs its effects with what a desktop has. There is no screen here.
//! The pointer surface this family draws is `apps/web`'s, and `DECISIONS.md` says why, so this
//! crate is the host and four of the five ports and nothing else. The fifth, `reel`, draws into a
//! picture a surface's toolkit owns, so it is named by whichever shell owns one.
//!
//! The core is linked as a Rust library, not through `platform/bridges/c`: this host is Rust too,
//! so the bridge would only turn typed messages into bytes and back for no reader.

mod adapters;
mod host;
mod ports;

pub use adapters::{FileSecrets, FileStore, HttpNetwork, NoPlayer};
pub use host::{APP, Host, Opening, Ports, Reply, Settings, Turn};
pub use ports::{Answer, Call, Done, Network, Player, Secrets, Store};
