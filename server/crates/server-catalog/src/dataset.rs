//! A loaded data set behind one atomic pointer. A new set is validated and loaded beside the old
//! one, and the old one keeps serving when the new one is refused.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::Result;
use arc_swap::{ArcSwap, ArcSwapOption};
use thiserror::Error;
use tracing::{info, warn};

use crate::catalog::{CatalogMap, load_catalog};
use crate::index::{HnswIndex, read_stamp};
use crate::manifest::{DataFiles, DatasetManifest, ManifestError, role};
use crate::profiles::ProfileStore;
use crate::vectors::VectorStore;

/// One whole data set, immutable once loaded apart from the index, which arrives when built.
pub struct DataSet {
    pub manifest: DatasetManifest,
    /// Hash of the manifest file. Shown in `/v1/ready`.
    pub data_hash: String,
    pub files: DataFiles,
    pub catalog: CatalogMap,
    pub vectors: Arc<VectorStore>,
    pub profiles: ProfileStore,
    pub mean: Vec<f32>,
    /// Empty while the index builds. Pages then use the exact scan.
    pub index: ArcSwapOption<HnswIndex>,
    /// Optional files that failed their check.
    pub dropped: Vec<String>,
}

/// Why a swap was refused. The last good set keeps serving.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SwapError {
    #[error(transparent)]
    Manifest(#[from] ManifestError),
    #[error("the data set did not load: {0}")]
    Load(String),
}

impl DataSet {
    /// Validate and load one set. Reads files; call on a blocking thread.
    ///
    /// # Errors
    /// The manifest refuses a file, or a required file does not load.
    pub fn load(manifest_path: &Path) -> Result<Self, SwapError> {
        let (manifest, data_hash) = DatasetManifest::read(manifest_path)?;
        let dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
        let (files, dropped) = manifest.validate(dir)?;
        for note in &dropped {
            warn!("Optional data file left out: {note}");
        }
        let need = |name: &'static str| {
            files
                .get(name)
                .ok_or(ManifestError::MissingRole(name))
                .map(Path::to_path_buf)
        };
        let dims = manifest.dimensions as usize;
        let vectors = VectorStore::load(&need(role::VECTORS)?, &need(role::VECTOR_KEYS)?, dims)
            .map_err(|e| SwapError::Load(e.to_string()))?;
        let catalog = load_catalog(&files).map_err(|e| SwapError::Load(e.to_string()))?;
        let profiles = files
            .get(role::PROFILES)
            .map_or_else(ProfileStore::empty, ProfileStore::load);
        let mean = vectors.compute_mean();
        Ok(Self {
            manifest,
            data_hash,
            files,
            catalog,
            vectors,
            profiles,
            mean,
            index: ArcSwapOption::empty(),
            dropped,
        })
    }

    /// Width of the vectors.
    #[must_use]
    pub fn dims(&self) -> usize {
        self.vectors.dims
    }

    /// Open the index file when its stamp matches this set exactly. The one sided size test and
    /// the trust switch are gone.
    pub fn open_index(&self, path: &Path) -> bool {
        let good =
            read_stamp(path).is_some_and(|stamp| stamp.matches(&self.data_hash, self.dims(), path));
        if !good {
            return false;
        }
        match HnswIndex::load(path, self.dims()) {
            Ok(index) => {
                self.index.store(Some(Arc::new(index)));
                true
            }
            Err(error) => {
                warn!("Index not opened: {error}");
                false
            }
        }
    }

    /// Build the index, save it with its stamp, and start using it. Blocking.
    ///
    /// # Errors
    /// The build or the save failed.
    pub fn build_index(&self, path: &Path) -> Result<()> {
        let index = HnswIndex::build_from_store(&self.vectors)?;
        index.save(path, &self.data_hash, self.dims())?;
        self.index.store(Some(Arc::new(index)));
        Ok(())
    }

    /// Nearest titles by cosine, reranked by popularity, by the index or by the exact scan. Both
    /// paths rank the same way.
    #[must_use]
    pub fn nearest(&self, query: &[f32], k: usize) -> Vec<(u32, f32)> {
        let guard = self.index.load();
        let found = guard.as_ref().and_then(|index| index.search(query, k).ok());
        let mut results = found.unwrap_or_else(|| self.vectors.brute_force_topk(query, k));
        for (id, sim) in &mut results {
            if let Some(item) = self.catalog.get(id) {
                *sim = crate::quality::rerank_score(*sim, item.popularity);
            }
        }
        results.sort_unstable_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        results
    }
}

/// The one pointer every request reads. The swap is atomic and keeps the last good set.
pub struct DataSlot {
    current: ArcSwap<DataSet>,
    manifest_path: PathBuf,
    swap_lock: Mutex<()>,
}

impl DataSlot {
    /// Load the first set.
    ///
    /// # Errors
    /// The set was refused. With no good set there is nothing to serve.
    pub fn open(manifest_path: &Path) -> Result<Self, SwapError> {
        Ok(Self {
            current: ArcSwap::from_pointee(DataSet::load(manifest_path)?),
            manifest_path: manifest_path.to_path_buf(),
            swap_lock: Mutex::new(()),
        })
    }

    /// The set in use.
    #[must_use]
    pub fn load(&self) -> Arc<DataSet> {
        self.current.load_full()
    }

    /// Read the manifest again. A changed set is validated and loaded first; a refused one leaves
    /// the current set in place. True when the set changed.
    ///
    /// # Errors
    /// The new set was refused.
    pub fn reload(&self) -> Result<bool, SwapError> {
        let _swap = self
            .swap_lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let (_, hash) = DatasetManifest::read(&self.manifest_path)?;
        if hash == self.load().data_hash {
            return Ok(false);
        }
        let next = DataSet::load(&self.manifest_path)?;
        info!("Data set {} in use", next.manifest.dataset_version);
        self.current.store(Arc::new(next));
        Ok(true)
    }
}
