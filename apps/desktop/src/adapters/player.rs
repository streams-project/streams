//! The player port, unfilled.
//!
//! No film plays on this host. `DECISIONS.md` records the reason: the driver for this family is
//! libmpv, which is a system library and a surface that owns a window, and neither is here. Every
//! command is dropped. The core does not wait on one: it arms the play's own start deadline and
//! fails over when it passes, whatever this port does.

use contract::driver::DriverCommand;

use crate::ports::Player;

pub struct NoPlayer;

impl Player for NoPlayer {
    fn command(&mut self, _given: DriverCommand) {}
}
