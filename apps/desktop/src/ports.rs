//! The hexagon of the desktop family. The core sits in the middle, and everything it needs from a
//! machine goes through one of these four.
//!
//! The names are every other family's: network, secrets, store, player. Where a shape departs from
//! `apps/android/ports/src/main/kotlin/streams/ports/Ports.kt` the departure is written here rather
//! than left to be found. `Store::put` hands back `false` where Android's returns nothing, because the
//! offline layer reads a refused write and marks itself not durable (`self.durable = false` on an
//! `EventBody::Unsupported`, `platform/offline/src/lib.rs:239`), which Android's `Unit` cannot say. `Player` carries `command` alone where Android's also has `listen`,
//! `picture`, `now`, `lift` and `wordsLook`; every one of those four others needs a picture a
//! surface's toolkit owns, and this family has no surface. The fifth port every other family names,
//! `reel`, is not named here at all for that same reason - its shape is a toolkit picture - and it
//! arrives with the shell that owns one.

use std::collections::BTreeMap;

use contract::core_msg::{HttpMethod, NoAnswerReason};
use contract::driver::DriverCommand;

/// The one answer header the core and the offline layer read. Nothing else is carried in, so no
/// surface can come to depend on a header the other hosts do not pass.
pub(crate) const ETAG: &str = "etag";

/// One request. The core only ever names a path, so the port is handed the whole address.
pub struct Call {
    pub method: HttpMethod,
    pub url: String,
    pub headers: BTreeMap<String, String>,
    pub body: Option<String>,
    pub deadline_ms: u32,
}

/// What came back. Any status at all is an answer; `NoReply` means none came.
pub enum Answer {
    Reply {
        status: u32,
        /// The one answer header the core and the offline layer read.
        etag: Option<String>,
        text: String,
    },
    NoReply {
        reason: NoAnswerReason,
    },
}

/// Called once with the answer, from whichever thread the send ran on.
pub type Done = Box<dyn FnOnce(Answer) + Send + 'static>;

pub trait Network: Send + Sync {
    fn send(&self, call: Call, done: Done);

    /// Says when the machine gains or loses its connection. Without it the core never hears the
    /// network come back and a waiting call sits out its whole backoff.
    fn watch(&self, changed: Box<dyn Fn(bool) + Send + Sync + 'static>);
}

/// What the core keeps between runs: the session and the device key, by name.
pub trait Secrets: Send + Sync {
    fn get(&self, name: &str) -> Option<String>;

    /// False when the store refused the write. The caller must say so rather than pretend: a
    /// session token that was never written signs the viewer out at the next launch with no
    /// signal, and the core keeps the token in memory instead when it hears the truth.
    fn put(&self, name: &str, value: &str) -> bool;

    fn delete(&self, name: &str);
}

/// What the core's offline layer keeps between runs: waiting commands and kept pages.
pub trait Store: Send + Sync {
    fn get(&self, key: &str) -> Option<String>;

    /// False when the write was refused, which tells the layer it is not durable here.
    fn put(&self, key: &str, value: &str) -> bool;

    fn delete(&self, key: &str);

    fn keys(&self, prefix: &str) -> Vec<String>;
}

/// Plays films, reports facts, and decides nothing.
pub trait Player: Send {
    /// Perform one command. Nothing is handed back, as on the other families: the core takes a
    /// player effect's id from its counter and registers it in no waiting map
    /// (`platform/core/src/lib.rs:496`), so there is no answer to a command for it to match.
    fn command(&mut self, given: DriverCommand);
}

// This port has no way back, and that is open rather than settled. A driver reports through
// `EventBody::Driver { generation, event }` (`platform/contract/src/core_msg.rs:318`), which
// Android's `Player.listen(report)` carries and this trait does not. Until something here plays a
// film, a shell that attaches a driver hands those events to `Host::step` itself, the same escape
// `Network::watch` describes for a connection change. Commands would then leave through the port
// and reports arrive beside it, which wants a decision before a driver lands.
