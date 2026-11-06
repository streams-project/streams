//! Rows made from requests. A request is a name and fact rules: the catalog is filtered by the rules, what passes is ranked by closeness to the centre of the
//! request's titles, and the row carries the request's own name, so no row is named from examples. The page builder is `page/requests.rs`.
mod rules;
mod store;

pub use rules::Rules;
pub use store::{Request, RequestDef, RequestStore};
