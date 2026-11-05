//! Whether the catalogue and the vectors describe the same titles, in words a builder can act on.
//! A title with no vector can never join a row, and a vector outside the catalogue costs memory.

use crate::catalog::CatalogMap;
use crate::vectors::VectorStore;

/// Counts of one data set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Coverage {
    pub titles: usize,
    pub vectors: usize,
    pub titles_with_vector: usize,
}

impl Coverage {
    /// Count a set.
    #[must_use]
    pub fn of(catalog: &CatalogMap, vectors: &VectorStore) -> Self {
        let titles_with_vector = catalog
            .values()
            .filter(|item| vectors.index_of(item.id).is_some())
            .count();
        Self {
            titles: catalog.len(),
            vectors: vectors.n_vecs,
            titles_with_vector,
        }
    }

    /// Titles that can never appear in a row.
    #[must_use]
    pub const fn titles_without_vector(&self) -> usize {
        self.titles - self.titles_with_vector
    }

    /// Vectors of titles the catalogue does not show.
    #[must_use]
    pub const fn vectors_outside_catalog(&self) -> usize {
        self.vectors.saturating_sub(self.titles_with_vector)
    }

    /// What to do, when something should be done. Missing titles are checked first.
    #[must_use]
    pub fn advice(&self) -> Option<String> {
        let missing = self.titles_without_vector();
        if missing * 20 > self.titles {
            return Some(format!(
                "{missing} of {} titles have no vector and can never appear in a row: run the export",
                self.titles
            ));
        }
        let extra = self.vectors_outside_catalog();
        if extra * 4 > self.vectors {
            return Some(format!(
                "{extra} of {} vectors belong to titles the catalogue does not show: run the export",
                self.vectors
            ));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn counts(titles: usize, vectors: usize, titles_with_vector: usize) -> Coverage {
        Coverage {
            titles,
            vectors,
            titles_with_vector,
        }
    }

    #[test]
    fn s140_a_set_that_matches_needs_nothing() {
        assert_eq!(counts(100, 100, 100).advice(), None);
    }

    #[test]
    fn s140_more_than_one_title_in_20_without_a_vector_is_named_first() {
        assert!(
            counts(100, 400, 94)
                .advice()
                .unwrap()
                .contains("6 of 100 titles")
        );
        assert_eq!(
            counts(100, 100, 95).advice(),
            None,
            "exactly one in 20 passes"
        );
    }

    #[test]
    fn s140_more_than_one_vector_in_4_outside_the_catalogue_is_named() {
        assert!(
            counts(100, 134, 100)
                .advice()
                .unwrap()
                .contains("34 of 134 vectors")
        );
        assert_eq!(counts(100, 133, 100).advice(), None);
    }
}
