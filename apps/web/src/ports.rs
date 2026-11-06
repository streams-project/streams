//! The hexagon of the website. The core sits in the middle, and each port names what it needs from
//! the outside. This is the one place that says which adapter fills a port, and the surfaces reach
//! the world only through here. `./check` enforces that.

pub use crate::adapters::{address, player};
