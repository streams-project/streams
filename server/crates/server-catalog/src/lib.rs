//! Title facts, vectors and the index, read through one data manifest. Nothing here decides who
//! may see a title.

pub mod attributes;
pub mod catalog;
pub mod centroid;
pub mod channels;
pub mod cluster;
pub mod coverage;
pub mod dataset;
pub mod det;
pub mod events;
pub mod fresh;
pub mod groups;
pub mod index;
pub mod manifest;
pub mod people;
pub mod profiles;
pub mod quality;
#[cfg(feature = "sample")]
pub mod sample;
pub mod support;
pub mod taxonomy;
pub mod topics;
pub mod vectors;
pub mod vibe;

pub use catalog::{CatalogItem, CatalogMap, SharedCatalog, item_key};
pub use channels::KidsChannels;
pub use dataset::{DataSet, DataSlot, SwapError};
pub use index::{HnswIndex, IndexStamp};
pub use manifest::{DataFiles, DataTrust, DatasetManifest, FileEntry, ManifestError, role};
pub use vectors::VectorStore;
