//! Skip marks and content reasons from the data set. Marks are ratios of a video's length; the
//! detector that reads cues sits behind the subtitle work.

use contract::play::{TimeWindow, WindowKind};
use serde_json::{Value, json};

use crate::subtitles::detect::SkipMarks;

/// The marks of one video: the first episode of a season takes `premiere`, the last `finale`, every
/// other one `typical`, each falling back to `typical`. A film or an unknown title has none.
#[must_use]
pub fn pick_marks(entry: Option<&Value>, episode: Option<u32>, last: bool) -> Value {
    let none = json!({ "intro": null, "recap": null, "credits": null });
    let Some(entry) = entry else {
        return none;
    };
    let kind = if episode == Some(1) {
        "premiere"
    } else if last {
        "finale"
    } else {
        "typical"
    };
    let marks = entry
        .get(kind)
        .filter(|v| v.is_object())
        .or_else(|| entry.get("typical"))
        .filter(|v| v.is_object());
    match marks {
        Some(marks) => json!({
            "intro": marks.get("intro").cloned().unwrap_or(Value::Null),
            "recap": marks.get("recap").cloned().unwrap_or(Value::Null),
            "credits": marks.get("credits").cloned().unwrap_or(Value::Null),
        }),
        None => none,
    }
}

/// The windows of one video from a detector's ratios.
#[must_use]
pub fn windows_of(marks: &SkipMarks, duration_ms: u64) -> Vec<TimeWindow> {
    let at = |ratio: f64| (ratio.clamp(0.0, 1.0) * duration_ms as f64) as u64;
    let span = |pair: Option<[f64; 2]>, kind| {
        pair.map(|[start, end]| TimeWindow {
            kind,
            start_ms: at(start),
            end_ms: at(end),
            seek_to_ms: Some(at(end)),
            text: None,
        })
    };
    let mut out = Vec::new();
    out.extend(span(marks.recap, WindowKind::Recap));
    out.extend(span(marks.intro, WindowKind::Intro));
    if let Some(start) = marks.credits {
        out.push(TimeWindow {
            kind: WindowKind::Credits,
            start_ms: at(start),
            end_ms: duration_ms,
            seek_to_ms: None,
            text: None,
        });
    }
    out
}

/// The marks of a data file as the detector reads them.
#[must_use]
pub fn marks_of(entry: &Value) -> SkipMarks {
    let pair = |value: &Value| -> Option<[f64; 2]> {
        let list = value.as_array()?;
        Some([list.first()?.as_f64()?, list.get(1)?.as_f64()?])
    };
    SkipMarks {
        intro: pair(&entry["intro"]),
        recap: pair(&entry["recap"]),
        credits: entry["credits"].as_f64(),
    }
}

/// The reasons of one video, or null when there is no answer for it.
#[must_use]
pub fn pick_reasons(entry: Option<&Value>, season: Option<u32>, episode: Option<u32>) -> Value {
    let Some(entry) = entry else {
        return Value::Null;
    };
    if let Some(reasons) = entry.get("r") {
        return reasons.clone();
    }
    let key = format!("{}:{}", season.unwrap_or(1), episode.unwrap_or(1));
    entry
        .get("e")
        .and_then(|map| map.get(&key))
        .or_else(|| entry.get("d"))
        .cloned()
        .unwrap_or(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry() -> Value {
        json!({
            "k": "tv:1",
            "typical": { "intro": [0.05, 0.07], "recap": null, "credits": 0.97 },
            "premiere": { "intro": [0.1, 0.12], "recap": null, "credits": 0.95 },
            "finale": null
        })
    }

    #[test]
    fn s108_premiere_finale_and_typical() {
        let e = entry();
        assert_eq!(pick_marks(Some(&e), Some(1), false)["credits"], json!(0.95));
        assert_eq!(pick_marks(Some(&e), Some(5), false)["credits"], json!(0.97));
        assert_eq!(
            pick_marks(Some(&e), Some(9), true)["credits"],
            json!(0.97),
            "a finale falls back"
        );
    }

    #[test]
    fn s108_reasons_default_override_and_film() {
        let show = json!({ "k": "tv:1", "d": ["violence"], "e": { "1:3": ["sex", "nudity"] } });
        assert_eq!(
            pick_reasons(Some(&show), Some(1), Some(3)),
            json!(["sex", "nudity"])
        );
        assert_eq!(
            pick_reasons(Some(&show), Some(2), Some(1)),
            json!(["violence"])
        );
        let film = json!({ "k": "movie:1", "r": ["war"] });
        assert_eq!(pick_reasons(Some(&film), None, None), json!(["war"]));
        assert!(pick_reasons(None, Some(1), Some(1)).is_null());
    }

    #[test]
    fn s108_an_unknown_title_has_no_marks() {
        let marks = pick_marks(None, Some(2), false);
        assert!(marks["intro"].is_null() && marks["recap"].is_null() && marks["credits"].is_null());
    }
}
