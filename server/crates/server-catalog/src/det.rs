//! Deterministic hash collections for everything the engine builds from its own data (titles, groups, rows).
use rustc_hash::FxBuildHasher;

pub type HashMap<K, V> = std::collections::HashMap<K, V, FxBuildHasher>;
pub type HashSet<K> = std::collections::HashSet<K, FxBuildHasher>;

/// A map with room for `n` entries.
pub fn map_with_capacity<K, V>(n: usize) -> HashMap<K, V> {
    HashMap::with_capacity_and_hasher(n, FxBuildHasher)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_keys_iterate_in_the_same_order_in_separate_maps() {
        let build = || {
            let mut m: HashMap<u32, u32> = HashMap::default();
            for k in (0..2000).map(|i| i * 7919 % 10007) {
                m.insert(k, k);
            }
            m.keys().copied().collect::<Vec<u32>>()
        };
        assert_eq!(build(), build());
    }
}
