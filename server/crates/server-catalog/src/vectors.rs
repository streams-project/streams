//! Vectors in memory as f16: one flat buffer, a key per row, and cosine helpers. The width comes
//! from the manifest, so no embedding size is built in.

use crate::det::HashMap;
use anyhow::{Result, bail};
use half::f16;
use rayon::prelude::*;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use tracing::info;

/// All vectors of a data set. One copy only.
pub struct VectorStore {
    /// Flat f16 buffer: row i starts at i * dims.
    pub data: Vec<f16>,
    /// Item keys parallel to the rows.
    pub ids: Vec<u32>,
    id_to_idx: HashMap<u32, usize>,
    pub n_vecs: usize,
    /// Values per row.
    pub dims: usize,
}

impl VectorStore {
    /// Load a raw f16 file of `dims` values per row, with a file of one u32 LE key per row.
    ///
    /// # Errors
    /// A file cannot be read, or the sizes do not fit each other.
    pub fn load(vectors: &Path, keys: &Path, dims: usize) -> Result<Arc<Self>> {
        let ids = le_u32s(&std::fs::read(keys)?);
        let n = ids.len();
        let size = std::fs::metadata(vectors)?.len();
        if n == 0 || dims == 0 || size != (n * dims * 2) as u64 {
            bail!(
                "{} holds {size} bytes, expected {n} keys x {dims} dimensions x 2 bytes",
                vectors.display()
            );
        }
        let data = read_f16s(vectors, n * dims)?;
        Ok(Arc::new(Self::from_parts(data, ids, dims)))
    }

    /// Build a store from rows already in memory.
    #[must_use]
    pub fn from_parts(data: Vec<f16>, ids: Vec<u32>, dims: usize) -> Self {
        let n = ids.len();
        let id_to_idx: HashMap<u32, usize> =
            ids.iter().enumerate().map(|(i, &id)| (id, i)).collect();
        info!(
            "Vectors loaded: {n} x {dims} dims (f16, {} MB)",
            data.len() * 2 / 1_048_576
        );
        Self {
            data,
            ids,
            id_to_idx,
            n_vecs: n,
            dims,
        }
    }

    /// The stored f16 row of vector `i`.
    #[inline]
    #[must_use]
    pub fn row(&self, i: usize) -> &[f16] {
        &self.data[i * self.dims..(i + 1) * self.dims]
    }

    /// The f32 vector of row `i`.
    #[inline]
    #[must_use]
    pub fn get_f32(&self, i: usize) -> Vec<f32> {
        self.row(i).iter().map(|x| x.to_f32()).collect()
    }

    /// Row index of a key.
    #[inline]
    #[must_use]
    pub fn index_of(&self, catalog_id: u32) -> Option<usize> {
        self.id_to_idx.get(&catalog_id).copied()
    }

    /// Top k by cosine over every row. Serves while the index builds, and ranks as the index path
    /// does when the caller applies the same rerank.
    #[must_use]
    pub fn brute_force_topk(&self, query: &[f32], k: usize) -> Vec<(u32, f32)> {
        let mut scores: Vec<(u32, f32)> = (0..self.n_vecs)
            .into_par_iter()
            .map(|i| (self.ids[i], cosine_sim_f16(query, self.row(i))))
            .collect();
        scores.sort_unstable_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        scores.truncate(k);
        scores
    }

    /// Mean of all vectors, used to centre similarity scores.
    #[must_use]
    pub fn compute_mean(&self) -> Vec<f32> {
        let n = self.n_vecs.max(1) as f32;
        let dims = self.dims;
        let mut sum = (0..self.n_vecs)
            .into_par_iter()
            .fold(
                || vec![0.0f32; dims],
                |mut acc, i| {
                    for (a, x) in acc.iter_mut().zip(self.row(i)) {
                        *a += x.to_f32();
                    }
                    acc
                },
            )
            .reduce(
                || vec![0.0f32; dims],
                |mut a, b| {
                    for (x, y) in a.iter_mut().zip(b) {
                        *x += y;
                    }
                    a
                },
            );
        sum.iter_mut().for_each(|x| *x /= n);
        sum
    }

    /// True when row `i` is the all zero placeholder of a failed embedding.
    #[must_use]
    pub fn is_zero_row(&self, i: usize) -> bool {
        self.row(i).iter().all(|x| x.to_f32() == 0.0)
    }
}

/// Read `count` little endian f16 values in steps, so the file is never held twice.
fn read_f16s(path: &Path, count: usize) -> Result<Vec<f16>> {
    let mut file = std::fs::File::open(path)?;
    let mut out = Vec::with_capacity(count);
    let mut buffer = vec![0u8; 1 << 20];
    let mut held = 0usize;
    loop {
        let read = file.read(&mut buffer[held..])?;
        if read == 0 {
            break;
        }
        held += read;
        let whole = held - held % 2;
        out.extend(
            buffer[..whole]
                .as_chunks::<2>()
                .0
                .iter()
                .map(|pair| f16::from_le_bytes(*pair)),
        );
        buffer.copy_within(whole..held, 0);
        held -= whole;
    }
    Ok(out)
}

/// Little endian f32 values of a byte slice (length a multiple of 4).
#[must_use]
pub fn le_f32s(bytes: &[u8]) -> Vec<f32> {
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|c| f32::from_le_bytes(*c))
        .collect()
}

/// Little endian u32 values of a byte slice (length a multiple of 4).
#[must_use]
pub fn le_u32s(bytes: &[u8]) -> Vec<u32> {
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|c| u32::from_le_bytes(*c))
        .collect()
}

/// Cosine similarity of an f32 query and a stored f16 row. Vectors are unit length.
#[inline]
#[must_use]
pub fn cosine_sim_f16(query: &[f32], stored: &[f16]) -> f32 {
    query.iter().zip(stored).map(|(q, s)| q * s.to_f32()).sum()
}

/// Cosine similarity of two unit f32 slices.
#[inline]
#[must_use]
pub fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
    // Sixteen independent partial sums let the compiler use SIMD.
    let mut acc = [0.0f32; 16];
    let (ca, ra) = a.as_chunks::<16>();
    let (cb, rb) = b.as_chunks::<16>();
    let tail: f32 = ra.iter().zip(rb).map(|(x, y)| x * y).sum();
    for (x, y) in ca.iter().zip(cb) {
        for (slot, (p, q)) in acc.iter_mut().zip(x.iter().zip(y)) {
            *slot += p * q;
        }
    }
    acc.iter().sum::<f32>() + tail
}

/// Normalise a vector in place.
pub fn l2_normalize(v: &mut [f32]) {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-8 {
        v.iter_mut().for_each(|x| *x /= norm);
    }
}

/// Unit sphere centroid of a set of normalised vectors, `dims` wide when the set is empty.
#[must_use]
pub fn sphere_mean(vecs: &[Vec<f32>], dims: usize) -> Vec<f32> {
    let mut sum = vec![0.0f32; dims];
    for v in vecs {
        for (s, x) in sum.iter_mut().zip(v.iter()) {
            *s += x;
        }
    }
    l2_normalize(&mut sum);
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s142_brute_force_ties_break_by_key_and_vectors_read_back() {
        let dir = std::env::temp_dir().join(format!("vectors_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (v, k) = (dir.join("v.bin"), dir.join("k.bin"));
        let row = |a: f32, b: f32| {
            [
                f16::from_f32(a).to_le_bytes(),
                f16::from_f32(b).to_le_bytes(),
            ]
            .concat()
        };
        std::fs::write(&v, [row(1.0, 0.0), row(0.0, 1.0), row(1.0, 0.0)].concat()).unwrap();
        std::fs::write(
            &k,
            [7u32, 8, 3]
                .iter()
                .flat_map(|x| x.to_le_bytes())
                .collect::<Vec<u8>>(),
        )
        .unwrap();
        let store = VectorStore::load(&v, &k, 2).unwrap();
        assert_eq!(store.index_of(8), Some(1));
        let top = store.brute_force_topk(&[1.0, 0.0], 2);
        assert_eq!(top.iter().map(|t| t.0).collect::<Vec<_>>(), vec![3, 7]);
    }

    #[test]
    fn a_short_vector_file_is_refused() {
        let dir = std::env::temp_dir().join(format!("vectors_short_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (v, k) = (dir.join("v.bin"), dir.join("k.bin"));
        std::fs::write(&v, [0u8; 3]).unwrap();
        std::fs::write(&k, [1u8, 0, 0, 0]).unwrap();
        assert!(VectorStore::load(&v, &k, 2).is_err());
    }
}
