//! Finding titles for a row: nearest neighbours of a vector, eligible titles of a list of keys, and rows made of them.
use super::*;

impl<'a> PageContext<'a> {
    /// Nearest titles of a query, reranked by popularity, by the index or by the exact scan.
    pub(crate) fn retrieve_candidates(&self, query: &[f32], k: usize) -> Vec<(u32, f32)> {
        self.data.nearest(query, k)
    }

    /// The titles of a list of keys that this page may show, best quality first; `any_genre` skips the page's genre rule.
    pub(crate) fn eligible(
        &self,
        keys: &[u32],
        taken: &HashSet<u32>,
        any_genre: bool,
    ) -> Vec<&CatalogItem> {
        let mut v: Vec<&CatalogItem> = keys
            .iter()
            .filter_map(|k| self.catalog.get(k))
            .filter(|i| {
                !taken.contains(&i.id)
                    && (if any_genre {
                        self.matches_filters_any_genre(i)
                    } else {
                        self.matches_filters(i)
                    })
                    && self.member_ok(i)
                    && self.reliable(i)
            })
            .collect();
        server_catalog::quality::sort_by_quality(&mut v);
        v
    }

    /// A row of these titles in the order given (no re-ordering for variety): the profile's own lists, the new releases, the lead titles of a collection.
    pub(crate) fn row_of_any(
        &self,
        title: &str,
        kind: &str,
        items: &[&CatalogItem],
    ) -> Option<Row> {
        self.row_of_at_least(3, title, kind, items)
    }

    /// A row with a floor of its own. The viewer's own lists show from their first title.
    pub(crate) fn row_of_at_least(
        &self,
        least: usize,
        title: &str,
        kind: &str,
        items: &[&CatalogItem],
    ) -> Option<Row> {
        let row_items: Vec<RowItem> = items
            .iter()
            .take(ROW_SIZE)
            .map(|i| RowItem::from_catalog(i))
            .collect();
        if row_items.len() < least {
            return None;
        }
        Some(Row {
            title: title.to_string(),
            archetype: "thematic".to_string(),
            kind: kind.to_string(),
            items: row_items,
            meta: None,
        })
    }

    pub(crate) fn row_of(&self, title: &str, kind: &str, items: &[&CatalogItem]) -> Option<Row> {
        let ids: Vec<u32> = items.iter().take(60).map(|i| i.id).collect();
        let row_items = self.diverse_titles(&ids, ROW_SIZE);
        if row_items.len() < COHERENCE_MIN_ITEMS {
            return None;
        }
        Some(Row {
            title: title.to_string(),
            archetype: "thematic".to_string(),
            kind: kind.to_string(),
            items: row_items,
            meta: None,
        })
    }

    /// Build row items from ranked ids; no significant title word is shared by more than FRANCHISE_CAP titles.
    pub(crate) fn diverse_titles(&self, ids: &[u32], n: usize) -> Vec<RowItem> {
        let cap = FRANCHISE_CAP;
        let mut per: HashMap<String, usize> = HashMap::default();
        let mut out: Vec<RowItem> = Vec::with_capacity(n);
        for id in ids {
            let Some(item) = self.catalog.get(id) else {
                continue;
            };
            let words = title_words(&item.title);
            if words
                .iter()
                .any(|w| per.get(w).copied().unwrap_or(0) >= cap)
            {
                continue;
            }
            for w in words {
                *per.entry(w).or_insert(0) += 1;
            }
            out.push(RowItem::from_catalog(item));
            if out.len() >= n {
                break;
            }
        }
        out
    }
}
