//! The player port, unfilled.
//!
//! No film plays on this host. `DECISIONS.md` records the reason: the driver for this family is
//! libmpv, which is a system library and a surface that owns a window, and neither is here. Every
//! command is refused out loud, so the core hears `EventBody::Unsupported` instead of waiting for
//! a driver that will never report.

use contract::driver::DriverCommand;

use crate::ports::Player;

pub struct NoPlayer;

impl Player for NoPlayer {
    fn command(&mut self, _given: DriverCommand) -> bool {
        false
    }
}
