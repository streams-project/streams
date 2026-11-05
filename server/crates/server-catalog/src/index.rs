//! The nearest neighbour index. The server is its only writer, so its
//! options live here once, and a stamp ties an index file to the data set it was built from.

use crate::manifest::sha256_file;
use crate::vectors::VectorStore;
use anyhow::Result;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::info;
use usearch::{Index, IndexOptions, MetricKind, ScalarKind};

/// Edges per node.
const HNSW_M: usize = 16;
const HNSW_EF_CONSTRUCTION: usize = 128;
const HNSW_EF_SEARCH: usize = 64;

/// What an index file was built from. Written beside the index by this server only.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexStamp {
    /// Hash of the data manifest.
    pub data_hash: String,
    /// Index library and options, so a change of either rebuilds.
    pub options: String,
    pub index_sha256: String,
}

/// The options text for a width.
#[must_use]
pub fn options_text(dims: usize) -> String {
    format!(
        "usearch-2 cos f16 dims={dims} m={HNSW_M} efc={HNSW_EF_CONSTRUCTION} efs={HNSW_EF_SEARCH}"
    )
}

impl IndexStamp {
    /// True when this stamp belongs to the data set, and the file still has its hash.
    #[must_use]
    pub fn matches(&self, data_hash: &str, dims: usize, index: &Path) -> bool {
        self.data_hash == data_hash
            && self.options == options_text(dims)
            && sha256_file(index).is_ok_and(|hash| hash == self.index_sha256)
    }
}

/// A usearch index with persistence.
pub struct HnswIndex {
    inner: Index,
}

impl HnswIndex {
    fn make_options(dims: usize) -> IndexOptions {
        IndexOptions {
            dimensions: dims,
            metric: MetricKind::Cos,
            quantization: ScalarKind::F16,
            connectivity: HNSW_M,
            expansion_add: HNSW_EF_CONSTRUCTION,
            expansion_search: HNSW_EF_SEARCH,
            ..Default::default()
        }
    }

    /// Open an index from disk, mapped where possible: opening takes milliseconds.
    ///
    /// # Errors
    /// The file cannot be read as an index of this width.
    pub fn load(path: &Path, dims: usize) -> Result<Self> {
        let inner = Index::new(&Self::make_options(dims))?;
        let text = path.to_string_lossy();
        if inner.view(&text).is_err() {
            inner.load(&text)?;
        }
        info!("Index loaded from {text} ({} vectors)", inner.size());
        Ok(Self { inner })
    }

    /// Number of vectors in the index.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.size()
    }

    /// True for an empty index.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Build over every vector in the store, in parallel. Zero rows are skipped. Blocking.
    ///
    /// # Errors
    /// The index library refused.
    pub fn build_from_store(store: &VectorStore) -> Result<Self> {
        let inner = Index::new(&Self::make_options(store.dims))?;
        inner.reserve_capacity_and_threads(store.n_vecs, rayon::current_num_threads())?;
        (0..store.n_vecs)
            .into_par_iter()
            .try_for_each(|i| -> Result<()> {
                if !store.is_zero_row(i) {
                    inner.add(u64::from(store.ids[i]), &store.get_f32(i))?;
                }
                Ok(())
            })?;
        info!("Index built: {} vectors", inner.size());
        Ok(Self { inner })
    }

    /// Save atomically, with the stamp beside it.
    ///
    /// # Errors
    /// A write failed.
    pub fn save(&self, path: &Path, data_hash: &str, dims: usize) -> Result<IndexStamp> {
        let tmp = path.with_extension("tmp");
        self.inner.save(&tmp.to_string_lossy())?;
        std::fs::rename(&tmp, path)?;
        let stamp = IndexStamp {
            data_hash: data_hash.to_owned(),
            options: options_text(dims),
            index_sha256: sha256_file(path)?,
        };
        std::fs::write(stamp_path(path), serde_json::to_vec(&stamp)?)?;
        Ok(stamp)
    }

    /// Top k neighbours as (key, cosine).
    ///
    /// # Errors
    /// The index library refused the query.
    pub fn search(&self, query: &[f32], k: usize) -> Result<Vec<(u32, f32)>> {
        let matches = self.inner.search(query, k)?;
        Ok(matches
            .keys
            .into_iter()
            .zip(matches.distances)
            .map(|(key, dist)| (u32::try_from(key).unwrap_or(0), 1.0 - dist))
            .collect())
    }
}

/// Where the stamp of an index file lives.
#[must_use]
pub fn stamp_path(index: &Path) -> std::path::PathBuf {
    let mut name = index.as_os_str().to_owned();
    name.push(".stamp");
    name.into()
}

/// The stamp beside an index file, when it can be read.
#[must_use]
pub fn read_stamp(index: &Path) -> Option<IndexStamp> {
    let bytes = std::fs::read(stamp_path(index)).ok()?;
    serde_json::from_slice(&bytes).ok()
}
