//! The hexagon of the desktop family. The core sits in the middle, and everything it needs from a
//! machine goes through one of these five. They name the same ports as every other family.

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
    /// Perform one command. `false` means this host cannot, and the core is answered
    /// `EventBody::Unsupported` rather than left waiting for a driver that will never report.
    fn command(&mut self, given: DriverCommand) -> bool;
}

/// Brand films on the way in. The title player belongs to the core, so this stays apart from it.
///
/// A reel draws into a picture that belongs to a surface's own toolkit, and this family has no
/// surface of its own, so the port is named here with the one question a host can ask without one.
pub trait Reel: Send {
    /// False while nothing here can show a brand film, so a surface never holds room for one.
    fn plays(&self) -> bool;
}
