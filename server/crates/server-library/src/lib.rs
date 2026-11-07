//! Commands, changes, list, rating, position and taste. Clients send commands and read results.

mod apply;
mod memory;
mod model;
mod postgres;
mod reads;
mod store;
mod taste;

pub use apply::{Library, TitleFacts};
pub use memory::MemoryLibrary;
pub use model::{
    Continue, EpisodeProgress, FeedRow, HiddenTitle, Limits, Reached, TitleState, age_to_time,
};
pub use postgres::PgLibrary;
pub use reads::SettingsView;
pub use store::{LibraryError, LibraryRead, LibraryStore, LibraryTx, Row};
pub use taste::{Taste, event_weight, learn};
