//! Cluster seeds: groups found offline over all vectors. A truncated or wrong width file is
//! refused as a whole, so an optional file can never stop a start.

use anyhow::{Context, Result, bail};
use std::path::Path;
use tracing::{info, warn};

use crate::vectors::{le_f32s, le_u32s};

/// One group found offline.
#[derive(Debug, Clone)]
pub struct ClusterSeed {
    /// Unit length centre.
    pub centroid: Vec<f32>,
    /// Item keys of its members.
    pub member_ids: Vec<u32>,
    /// Row title written for the group, empty when none was.
    pub name: String,
    /// World level 2 group this seed is, or -1.
    pub group: i32,
}

/// All seeds of a data set.
#[derive(Debug, Default)]
pub struct ClusterSeeds {
    pub seeds: Vec<ClusterSeed>,
}

/// A byte reader that refuses to run past the end of the file.
struct Reader<'b> {
    bytes: &'b [u8],
    at: usize,
}

impl<'b> Reader<'b> {
    fn take(&mut self, len: usize) -> Result<&'b [u8]> {
        let end = self
            .at
            .checked_add(len)
            .context("seed file sizes overflow")?;
        let part = self
            .bytes
            .get(self.at..end)
            .context("seed file is cut short")?;
        self.at = end;
        Ok(part)
    }

    fn u32(&mut self) -> Result<usize> {
        let part = self.take(4)?;
        Ok(u32::from_le_bytes([part[0], part[1], part[2], part[3]]) as usize)
    }
}

impl ClusterSeeds {
    /// Read a seed file of this width.
    ///
    /// # Errors
    /// The file cannot be read, is cut short, or has another width.
    pub fn load(path: &Path, dims: usize) -> Result<Self> {
        let bytes = std::fs::read(path)?;
        let mut reader = Reader {
            bytes: &bytes,
            at: 0,
        };
        let count = reader.u32()?;
        let width = reader.u32()?;
        if width != dims {
            bail!("seed file has {width} dimensions, the vectors have {dims}");
        }
        let mut seeds = Vec::new();
        for _ in 0..count {
            let centroid = le_f32s(reader.take(width * 4)?);
            let members = reader.u32()?;
            let member_ids = le_u32s(reader.take(members * 4)?);
            seeds.push(ClusterSeed {
                centroid,
                member_ids,
                name: String::new(),
                group: -1,
            });
        }
        info!("Seeds loaded: {}", seeds.len());
        Ok(Self { seeds })
    }

    /// Add the names and groups of the seeds. A file of another length is left out.
    pub fn read_names(&mut self, names: Option<&Path>, groups: Option<&Path>) {
        if let Some(found) = names
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|text| serde_json::from_str::<Vec<String>>(&text).ok())
        {
            if found.len() == self.seeds.len() {
                for (seed, name) in self.seeds.iter_mut().zip(found) {
                    seed.name = name;
                }
            } else {
                warn!(
                    "Seed names left out: {} for {} seeds",
                    found.len(),
                    self.seeds.len()
                );
            }
        }
        if let Some(found) = groups
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|text| serde_json::from_str::<Vec<i32>>(&text).ok())
            && found.len() == self.seeds.len()
        {
            for (seed, id) in self.seeds.iter_mut().zip(found) {
                seed.group = id;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(values: &[u32]) -> std::path::PathBuf {
        let name = format!("seeds_{}_{}", std::process::id(), values.len());
        let path = std::env::temp_dir().join(name);
        let bytes: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        std::fs::write(&path, bytes).unwrap();
        path
    }

    #[test]
    fn h69_a_truncated_seed_file_is_an_error_and_no_panic() {
        assert!(ClusterSeeds::load(&file(&[9, 2, 1, 1]), 2).is_err());
    }

    #[test]
    fn h69_a_wrong_width_is_refused() {
        assert!(ClusterSeeds::load(&file(&[1, 3]), 2).is_err());
    }

    #[test]
    fn a_whole_file_reads() {
        let one = 1.0f32.to_bits();
        let set = ClusterSeeds::load(&file(&[1, 2, one, 0, 2, 7, 8]), 2).unwrap();
        assert_eq!(set.seeds[0].member_ids, [7, 8]);
        assert!((set.seeds[0].centroid[0] - 1.0).abs() < 1e-6);
    }
}
