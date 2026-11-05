//! Row titles from what is in the row. Candidates run from most to least specific; the page builder takes the first one
//! not yet used on the page (page.rs `claim_title`) and falls back to the older generic titles in titles.rs.

use server_catalog::catalog::CatalogItem;
use server_catalog::det::HashMap;

mod fit;
mod themes;
mod titles;
mod words;

pub use fit::name_fits;
pub(crate) use themes::THEMES;
pub use titles::{candidates, mark_animated, theme_title, tone_title};
pub(crate) use words::*;
pub use words::{genre_family, genre_noun, language_label, primary_country};

#[cfg(test)]
mod tests;
