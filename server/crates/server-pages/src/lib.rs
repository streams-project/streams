//! Pages, rows, search and title pages. Rows are built from the catalogue and the library, and
//! every title passes the rules once more when a row is turned into an answer.

mod build;
pub mod cursor;
pub mod gate;
pub mod genre_pages;
pub mod genre_spec;
pub mod hero;
pub mod hooks;
pub mod mmr;
pub mod more_like;
pub mod new_popular;
pub mod noise;
pub mod page;
pub mod personal;
pub mod plan;
mod reading;
pub mod requests;
pub mod row_label;
pub mod search;
pub mod signals;
pub mod suggest;
pub mod tags;
pub mod titles;
pub mod words;

pub use build::{PageCheck, PageError, Pages, Personal, ScreenAsk};
pub use reading::personal_of;
