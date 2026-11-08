//! A loaded data set behind one atomic pointer. A new set is validated and loaded beside the old
//! one, and the old one keeps serving when the new one is refused.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::Result;
use arc_swap::{ArcSwap, ArcSwapOption};
use thiserror::Error;
use tracing::{info, warn};

use crate::catalog::{CatalogMap, load_catalog};
use crate::centroid::CentroidVault;
use crate::channels::KidsChannels;
use crate::cluster::ClusterSeeds;
use crate::det::HashMap;
use crate::events::EventStore;
use crate::fresh::FreshStore;
use crate::groups::{self, GroupMap, GroupNames};
use crate::index::{HnswIndex, read_stamp};
use crate::manifest::{DataFiles, DataTrust, DatasetManifest, ManifestError, role};
use crate::people::PeopleIndex;
use crate::profiles::ProfileStore;
use crate::taxonomy::TaxonomyStore;
use crate::topics::TopicStore;
use crate::vectors::VectorStore;
use crate::vibe::{self, VibeIndex};

/// One whole data set, immutable once loaded apart from the index, which arrives when built.
pub struct DataSet {
    pub manifest: DatasetManifest,
    /// Hash of the manifest file. Shown in `/v1/ready`.
    pub data_hash: String,
    pub files: DataFiles,
    pub catalog: CatalogMap,
    pub vectors: Arc<VectorStore>,
    pub profiles: ProfileStore,
    /// Channels the set names as children's channels. Empty when the file is absent.
    pub kids_channels: KidsChannels,
    pub mean: Vec<f32>,
    pub seeds: ClusterSeeds,
    pub vault: CentroidVault,
    pub groups: GroupMap,
    pub group_names: GroupNames,
    pub vibe: VibeIndex,
    pub topics: TopicStore,
    pub fresh: FreshStore,
    pub people: PeopleIndex,
    pub events: EventStore,
    pub play_facts: PlayFacts,
    /// Row definitions, centred on the corpus mean. Empty when the files are absent.
    pub taxonomy: Arc<TaxonomyStore>,
    /// Row definitions of the Kids pages.
    pub kids_taxonomy: Arc<TaxonomyStore>,
    /// Empty while the index builds. Pages then use the exact scan.
    pub index: ArcSwapOption<HnswIndex>,
    /// Optional files that failed their check.
    pub dropped: Vec<String>,
    /// Titles' names and stories in other languages, read from the catalog file when asked.
    pub texts: Arc<crate::texts::TitleTexts>,
    /// Index searches that failed and fell back to the exact scan.
    pub index_errors: AtomicU64,
}

/// Per title facts of a video, as lines of JSON keyed by title key.
#[derive(Debug, Default)]
pub struct PlayFacts {
    /// Skip marks as ratios of a video's length.
    pub marks: HashMap<String, serde_json::Value>,
    /// Content reasons.
    pub reasons: HashMap<String, serde_json::Value>,
}

fn keyed_lines(path: Option<&Path>) -> HashMap<String, serde_json::Value> {
    let text = path
        .and_then(|path| std::fs::read_to_string(path).ok())
        .unwrap_or_default();
    text.lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter_map(|value| Some((value.get("k")?.as_str()?.to_owned(), value)))
        .collect()
}

impl PlayFacts {
    fn load(files: &DataFiles) -> Self {
        Self {
            marks: keyed_lines(files.get(role::SKIP_MARKS)),
            reasons: keyed_lines(files.get(role::CONTENT_REASONS)),
        }
    }
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
    /// Validate and load one set against the keys this server holds. Reads files; call on a
    /// blocking thread. No load skips the signature: a trust with no key refuses every set, so an
    /// unset `DATA_PUBLIC_KEYS` cannot turn the check off.
    ///
    /// # Errors
    /// No key is held, the signature does not fit, a file is refused, or a required file does not
    /// load.
    pub fn load(manifest_path: &Path, trust: &DataTrust) -> Result<Self, SwapError> {
        trust.check(manifest_path)?;
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
        let coverage = crate::coverage::Coverage::of(&catalog, &vectors);
        if let Some(advice) = coverage.advice() {
            warn!("{advice}");
        }
        let mut dropped = dropped;
        let seeds = load_seeds(&files, dims, &mut dropped);
        let vault = CentroidVault::compute(&catalog, &vectors);
        let db = need(role::CATALOG)?;
        require_text_index(&db)?;
        let texts = Arc::new(crate::texts::TitleTexts::new(&db));
        let group_map = groups::load(&db);
        let group_names = groups::load_names(&db);
        let vibe = vibe::load(&db, groups::vibe_members(&group_map));
        let taxonomy = load_taxonomy(
            &files,
            (role::TAXONOMY_VECTORS, role::TAXONOMY_ENTRIES),
            dims,
            &mean,
            &mut dropped,
        );
        let kids_taxonomy = load_taxonomy(
            &files,
            (role::KIDS_VECTORS, role::KIDS_ENTRIES),
            dims,
            &mean,
            &mut dropped,
        );
        let kids_channels = load_kids_channels(&files, &mut dropped);
        Ok(Self {
            manifest,
            data_hash,
            topics: files
                .get(role::TOPICS_META)
                .map_or_else(TopicStore::default, TopicStore::load),
            fresh: files
                .get(role::FRESH)
                .map_or_else(FreshStore::default, FreshStore::load),
            events: files
                .get(role::EVENTS)
                .map_or_else(EventStore::default, EventStore::load),
            play_facts: PlayFacts::load(&files),
            kids_channels,
            people: PeopleIndex::load(&db),
            files,
            catalog,
            vectors,
            profiles,
            mean,
            seeds,
            vault,
            groups: group_map,
            group_names,
            vibe,
            taxonomy,
            kids_taxonomy,
            index: ArcSwapOption::empty(),
            dropped,
            texts,
            index_errors: AtomicU64::new(0),
        })
    }

    /// A title's name, story and line in a language (a code such as `ar`), or None for English,
    /// an unknown title or no translation.
    #[must_use]
    pub fn title_text(&self, key: u32, lang: &str) -> Option<Arc<crate::texts::TitleText>> {
        let item = self.catalog.get(&key)?;
        self.texts.get(key, &item.media_type, item.title_id, lang)
    }

    /// A title's name in a language, from the names read once for that language; None for
    /// English or no translation.
    #[must_use]
    pub fn title_name(&self, key: u32, lang: &str) -> Option<String> {
        self.texts.name(key, lang)
    }

    /// True when the index holds and has not failed.
    #[must_use]
    pub fn index_healthy(&self) -> bool {
        self.index.load().is_some() && self.index_errors.load(Ordering::Relaxed) == 0
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
        let found = guard
            .as_ref()
            .and_then(|index| match index.search(query, k) {
                Ok(found) => Some(found),
                Err(error) => {
                    // counted and logged, so a broken index cannot pass for a ready one
                    warn!("Index search failed, the exact scan answers: {error}");
                    self.index_errors.fetch_add(1, Ordering::Relaxed);
                    None
                }
            });
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

/// Search reads a full text table. With none its text step would quietly find nothing, so a set
/// without one is refused at load.
fn require_text_index(db: &Path) -> Result<(), SwapError> {
    use rusqlite::{Connection, OpenFlags};
    let refused = |error: rusqlite::Error| SwapError::Load(error.to_string());
    let conn =
        Connection::open_with_flags(db, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(refused)?;
    let found: i64 = conn
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE name = 'catalog_fts'",
            [],
            |row| row.get(0),
        )
        .map_err(refused)?;
    if found == 0 {
        return Err(SwapError::Load(String::from(
            "the catalogue has no text index",
        )));
    }
    Ok(())
}

/// Seeds are optional, so a bad file is dropped and never stops the set.
fn load_seeds(files: &DataFiles, dims: usize, dropped: &mut Vec<String>) -> ClusterSeeds {
    let Some(path) = files.get(role::SEEDS) else {
        return ClusterSeeds::default();
    };
    let mut seeds = ClusterSeeds::load(path, dims).unwrap_or_else(|error| {
        warn!("Seeds left out: {error}");
        dropped.push(role::SEEDS.to_owned());
        ClusterSeeds::default()
    });
    seeds.read_names(files.get(role::SEED_NAMES), files.get(role::SEED_GROUPS));
    seeds
}

/// The set's children's channel names. The data builder writes this file for every set and marks
/// it required, so its bytes are checked like any other. A set that names no such file at all
/// leaves the list empty: the channel clause of the children's title rule then does not fire, the
/// role is named in `dropped` and the reason is logged, so the loss is stated and not silent.
fn load_kids_channels(files: &DataFiles, dropped: &mut Vec<String>) -> KidsChannels {
    let Some(path) = files.get(role::KIDS_CHANNELS) else {
        warn!(
            "The set names no children's channels, so that part of the children's title rule is off"
        );
        dropped.push(role::KIDS_CHANNELS.to_owned());
        return KidsChannels::default();
    };
    let channels = KidsChannels::load(path);
    if channels.is_empty() {
        dropped.push(role::KIDS_CHANNELS.to_owned());
    }
    channels
}

/// Row definitions, centred on the corpus mean, with an empty store when absent or bad.
fn load_taxonomy(
    files: &DataFiles,
    (vectors, entries): (&str, &str),
    dims: usize,
    mean: &[f32],
    dropped: &mut Vec<String>,
) -> Arc<TaxonomyStore> {
    let (Some(bin), Some(json)) = (files.get(vectors), files.get(entries)) else {
        return TaxonomyStore::from_raw(Vec::new());
    };
    match TaxonomyStore::load(bin, json, dims) {
        Ok(mut store) => {
            if let Some(owned) = Arc::get_mut(&mut store) {
                owned.center_with(mean);
            }
            store
        }
        Err(error) => {
            warn!("Row definitions left out: {error}");
            dropped.push(entries.to_owned());
            TaxonomyStore::from_raw(Vec::new())
        }
    }
}

/// The one pointer every request reads. The swap is atomic and keeps the last good set.
pub struct DataSlot {
    current: ArcSwap<DataSet>,
    manifest_path: PathBuf,
    trust: DataTrust,
    swap_lock: Mutex<()>,
}

impl DataSlot {
    /// Load the first set against the keys this server holds.
    ///
    /// # Errors
    /// The set was refused, a key is missing included. With no good set there is nothing to serve.
    pub fn open(manifest_path: &Path, trust: DataTrust) -> Result<Self, SwapError> {
        Ok(Self {
            current: ArcSwap::from_pointee(DataSet::load(manifest_path, &trust)?),
            manifest_path: manifest_path.to_path_buf(),
            trust,
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
        let next = DataSet::load(&self.manifest_path, &self.trust)?;
        info!("Data set {} in use", next.manifest.dataset_version);
        self.current.store(Arc::new(next));
        Ok(true)
    }
}
