//! The HTTP face of the server: settings, the guard every request passes, and the router.

pub mod answer;
pub mod caller;
pub mod config;
pub mod error;
pub mod guard;
pub mod jobs;
pub mod live;
pub mod router;
pub mod routes;
pub mod state;

pub use config::Config;
pub use router::router;
pub use state::{AppState, Shared};
