//! Topic rows: rows named for what a title is about, from judged memberships.
use super::*;
use serde_json::json;
use server_catalog::topics::Topic;

impl PageContext<'_> {
    /// Up to `want` topic rows for this page, none whose title is in `skip_titles`; each is at least 12 titles that no earlier row of the page has.
    pub(crate) fn build_topic_rows(
        &self,
        taken: &HashSet<u32>,
        want: usize,
        skip_titles: &HashSet<String>,
    ) -> Vec<Row> {
        if want == 0 || self.topics.topics.is_empty() {
            return Vec::new();
        }
        let mut order: Vec<(&String, &Topic)> = self.topics.topics.iter().collect();
        order.sort_by(|a, b| {
            entry_noise(
                self.session,
                31_000 + b.0.len() * 7 + b.0.bytes().map(|x| x as usize).sum::<usize>(),
            )
            .partial_cmp(&entry_noise(
                self.session,
                31_000 + a.0.len() * 7 + a.0.bytes().map(|x| x as usize).sum::<usize>(),
            ))
            .unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut rows: Vec<Row> = Vec::new();
        let mut used = taken.clone();
        for (slug, topic) in order {
            if rows.len() >= want {
                break;
            }
            if skip_titles.contains(&topic.name.to_lowercase()) {
                continue;
            }
            if self.surface != "both" && !topic.media.iter().any(|m| m == self.surface) {
                continue;
            }
            let Some(keys) = self.catalog.topic_keys.get(slug) else {
                continue;
            };
            let items = self.eligible(keys, &used, false);
            if items.len() < 12 {
                continue;
            }
            if let Some(mut r) = self.row_of(&topic.name, "topic", &items) {
                r.meta = Some(
                    json!({ "title_key": format!("topic.{slug}"), "title_args": {}, "reason": { "kind": "topic", "topic": slug } }),
                );
                r.items.iter().for_each(|i| {
                    used.insert(i.id);
                });
                rows.push(r);
            }
        }
        rows
    }
}
