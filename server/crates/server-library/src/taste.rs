//! Durable taste: genre weights learned from what a profile does, kept across restarts.

use std::collections::BTreeMap;

use contract::command::FeedbackEvent;
use serde::{Deserialize, Serialize};

/// Each event scales every old weight by this.
const DECAY: f32 = 0.95;
/// A weight below this is forgotten.
const FLOOR: f32 = 0.01;

/// Genre weights of a profile.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Taste {
    pub weights: BTreeMap<String, f32>,
    pub version: u32,
}

/// What one signal is worth. A dislike counts against the title's genres.
#[must_use]
pub const fn event_weight(event: FeedbackEvent) -> Option<f32> {
    match event {
        FeedbackEvent::Click => Some(0.3),
        FeedbackEvent::Start => Some(0.6),
        FeedbackEvent::Complete => Some(1.0),
        FeedbackEvent::Like => Some(1.5),
        FeedbackEvent::Skip => Some(-0.5),
        _ => None,
    }
}

/// Weight of a dislike rating, the same as a skip.
pub const DISLIKE_WEIGHT: f32 = -0.5;

/// Fold one signal into the taste: old weights decay, the title's genres share the new weight.
pub fn learn(taste: &mut Taste, genres: &[String], weight: f32) {
    for value in taste.weights.values_mut() {
        *value *= DECAY;
    }
    if !genres.is_empty() {
        let share = weight / genres.len() as f32;
        for genre in genres {
            *taste.weights.entry(genre.clone()).or_insert(0.0) += share;
        }
    }
    taste.weights.retain(|_, value| value.abs() >= FLOOR);
    taste.version += 1;
}

impl Taste {
    /// The top positive genres, normalised to sum to 1.
    #[must_use]
    pub fn top(&self, count: usize) -> Vec<(String, f32)> {
        let mut positive: Vec<(String, f32)> = self
            .weights
            .iter()
            .filter(|(_, weight)| **weight > 0.0)
            .map(|(genre, weight)| (genre.clone(), *weight))
            .collect();
        positive.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        positive.truncate(count);
        let total: f32 = positive.iter().map(|(_, weight)| weight).sum();
        if total > 0.0 {
            for (_, weight) in &mut positive {
                *weight /= total;
            }
        }
        positive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn genres(list: &[&str]) -> Vec<String> {
        list.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn u4_weights_are_shared_across_a_titles_genres_and_decay_per_event() {
        let mut taste = Taste::default();
        learn(&mut taste, &genres(&["Drama", "Crime"]), 1.0);
        assert!((taste.weights["Drama"] - 0.5).abs() < 1e-6);
        learn(&mut taste, &genres(&["Drama"]), 1.0);
        assert!((taste.weights["Drama"] - (0.5 * 0.95 + 1.0)).abs() < 1e-6);
        assert!((taste.weights["Crime"] - 0.475).abs() < 1e-6);
    }

    #[test]
    fn h37_a_dislike_lowers_a_genre_and_a_tiny_weight_is_forgotten() {
        let mut taste = Taste::default();
        learn(&mut taste, &genres(&["Horror"]), 0.3);
        learn(&mut taste, &genres(&["Horror"]), DISLIKE_WEIGHT);
        assert!(taste.weights["Horror"] < 0.0);
        let mut small = Taste::default();
        learn(&mut small, &genres(&["War"]), 0.0101);
        learn(&mut small, &genres(&["Drama"]), 1.0);
        assert!(!small.weights.contains_key("War"));
    }

    #[test]
    fn u4_top_keeps_five_positive_genres_and_sums_to_one() {
        let mut taste = Taste::default();
        for (genre, weight) in [
            ("A", 6.0),
            ("B", 5.0),
            ("C", 4.0),
            ("D", 3.0),
            ("E", 2.0),
            ("F", 1.0),
        ] {
            taste.weights.insert(genre.to_owned(), weight);
        }
        taste.weights.insert(String::from("G"), -1.0);
        let top = taste.top(5);
        assert_eq!(top.len(), 5);
        assert_eq!(top[0].0, "A");
        assert!((top.iter().map(|t| t.1).sum::<f32>() - 1.0).abs() < 1e-5);
    }
}
