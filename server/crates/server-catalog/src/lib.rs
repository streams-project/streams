//! Title facts, vectors and the index, read through one data manifest. Nothing here decides who
//! may see a title.

pub mod attributes;
pub mod catalog;
pub mod dataset;
pub mod det;
pub mod index;
pub mod manifest;
pub mod profiles;
pub mod quality;
#[cfg(feature = "sample")]
pub mod sample;
pub mod vectors;

pub use catalog::{CatalogItem, CatalogMap, SharedCatalog, item_key};
pub use dataset::{DataSet, DataSlot, SwapError};
pub use index::{HnswIndex, IndexStamp};
pub use manifest::{DataFiles, DatasetManifest, FileEntry, ManifestError, role};
pub use vectors::VectorStore;
