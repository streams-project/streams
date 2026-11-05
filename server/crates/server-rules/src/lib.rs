//! One eligibility function and two proof types. Every list, title page and play answer goes
//! through [`Viewer`], and a response can only be built from a [`Visible`] title, so a row, hero,
//! event header or play answer cannot carry an unfiltered title.

pub mod answer;
mod block;
mod eligibility;
mod tier;
mod viewer;

pub use block::normalise_name;
pub use eligibility::{KIDS_CHANNELS, audience_fits, eligible, is_childrens_title, kid_interest};
pub use tier::{RuleError, Tier};
pub use viewer::{TierVisible, Viewer, Visible};
