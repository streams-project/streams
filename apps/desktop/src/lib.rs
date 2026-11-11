//! The desktop host: one more host of the core, as native code.
//!
//! The core decides, this performs its effects with what a desktop has. There is no screen here.
//! The pointer surface this family draws is `apps/web`'s, and `DECISIONS.md` says why, so this
//! crate is the host and the five ports and nothing else.
//!
//! The core is linked as a Rust library, not through `platform/bridges/c`: this host is Rust too,
//! so the bridge would only turn typed messages into bytes and back for no reader.

mod adapters;
mod host;
mod ports;

pub use adapters::{FileSecrets, FileStore, HttpNetwork, NoPlayer, NoReel};
pub use host::{APP, Host, Opening, Ports, Reply, Settings, Turn};
pub use ports::{Answer, Call, Done, Network, Player, Reel, Secrets, Store};
