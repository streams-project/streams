//! First-party deterministic skip marks detector.
//! Analyzes subtitle cues, timing offsets, and dialogue gaps directly in memory
//! (zero bandwidth, zero external APIs) to establish precise intro, recap, and end credits boundaries.

use super::clean::Cue;

/// Where the skippable parts of a video sit, as ratios of its length.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SkipMarks {
    pub intro: Option<[f64; 2]>,
    pub recap: Option<[f64; 2]>,
    pub credits: Option<f64>,
}

/// Minimum duration of an intro sequence in seconds (e.g. Breaking Bad's short title card is ~16s).
pub const MIN_INTRO_DURATION_SECS: f64 = 15.0;

/// Maximum duration of a standard television intro sequence in seconds (extended anime/drama themes up to ~135s).
pub const MAX_INTRO_DURATION_SECS: f64 = 135.0;

/// Minimum duration of a "Previously on..." recap block in seconds.
pub const MIN_RECAP_DURATION_SECS: f64 = 10.0;

/// Maximum duration of a television recap block in seconds (long season-premiere recaps reach ~140-150s).
pub const MAX_RECAP_DURATION_SECS: f64 = 150.0;

/// Maximum search boundary for television intros: first 25% of runtime or 10 minutes (600s), whichever is smaller.
pub const INTRO_SEARCH_MAX_SECS: f64 = 600.0;
pub const INTRO_SEARCH_MAX_RATIO: f64 = 0.25;

/// Search window for television end credits: last 6 minutes (360s).
pub const CREDITS_SEARCH_WINDOW_TV_SECS: f64 = 360.0;

/// Search window for film end credits: last 12 minutes (720s).
pub const CREDITS_SEARCH_WINDOW_FILM_SECS: f64 = 720.0;

/// Minimum runtime ratio where end credits can be searched (at least 70% into the media).
pub const CREDITS_MIN_RATIO: f64 = 0.70;

/// Buffer seconds added after the final spoken dialogue line before end credits start.
pub const CREDITS_POST_DIALOGUE_PAD_SECS: f64 = 1.5;

/// Maximum distance in seconds between a subtitle dialogue gap and the engine's prior intro estimate to snap them together.
pub const GAP_PRIOR_SNAP_TOLERANCE_SECS: f64 = 60.0;

/// Minimum duration in seconds of spoken dialogue scene following a silence gap to confirm scene continues.
pub const MIN_POST_INTRO_DIALOGUE_SPAN_SECS: f64 = 20.0;

/// Minimum number of spoken dialogue lines following a gap to confirm scene continues.
pub const MIN_POST_INTRO_DIALOGUE_COUNT: usize = 3;

/// Maximum delay allowed for recap block to start from beginning of stream.
pub const MAX_RECAP_START_OFFSET_SECS: f64 = 45.0;

/// Silence duration in seconds indicating the end of a recap dialogue cluster and transition to cold open.
pub const RECAP_CLUSTER_PAUSE_SECS: f64 = 3.5;

/// Minimum dialogue duration required in cold open before an intro gap can be recognized.
pub const MIN_COLD_OPEN_DIALOGUE_SECS: f64 = 5.0;

/// Distance tolerance for snapping end credits to engine prior in seconds.
pub const CREDITS_PRIOR_SNAP_TOLERANCE_SECS: f64 = 90.0;

/// Minimum silence gap between main episode dialogue and a post-credits stinger scene in seconds.
pub const POST_CREDITS_STINGER_PAUSE_SECS: f64 = 45.0;

/// Minimum duration in seconds for an engine intro guess to be considered non-degenerate.
pub const MIN_VALID_ENGINE_INTRO_DURATION_SECS: f64 = 10.0;

/// Window in seconds immediately following an engine credits mark where active dialogue indicates the engine is wrong.
pub const CREDITS_DIALOGUE_IMMEDIATE_POST_SECS: f64 = 15.0;

/// Maximum pause in seconds allowed between musical cues to consider them part of the same opening theme song.
pub const THEME_CUE_BRIDGE_MAX_PAUSE_SECS: f64 = 4.0;

/// Maximum boundary for a standalone silence gap to invent an intro when the engine has no mark (first 3 minutes).
pub const STANDALONE_GAP_MAX_TIME_SECS: f64 = 180.0;

/// Minimum duration for a standalone silence gap to invent an intro (20 seconds).
pub const STANDALONE_GAP_MIN_LEN_SECS: f64 = 20.0;

/// Maximum duration for a standalone silence gap to invent an intro (100 seconds).
pub const STANDALONE_GAP_MAX_LEN_SECS: f64 = 100.0;

/// Maximum distance in seconds between a strong candidate and an engine guess to consider them agreeing on the intro.
pub const STRONG_ENGINE_AGREE_TOLERANCE_SECS: f64 = 30.0;

/// Number of spoken dialogue lines inside an engine intro window that proves the engine is definitely wrong.
pub const ENGINE_INTRO_MAX_SPOKEN_LINES: usize = 3;

/// Relative strength of a skip mark detection source in the decision layer.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum SignalStrength {
    /// Standalone silence gaps without corroboration.
    Weak,
    /// Engine catalogue estimates or silence gaps refining an engine estimate.
    Medium,
    /// Explicit musical symbols (♪, ♫), bracketed theme markers, or recap keywords.
    Strong,
}

/// Provenance of a resolved skip mark.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MarkOrigin {
    /// Provided directly by the backend engine catalogue prior.
    Engine,
    /// Detected from subtitle musical notation (♪, ♫) or bracketed theme markers corroborated by engine.
    SubtitleTheme,
    /// Detected from subtitle musical notation (♪, ♫) that overrides a conflicting engine prior (> 30s distance).
    SubtitleThemeOverEngine,
    /// Detected from subtitle "Previously on..." recap keywords.
    SubtitleRecap,
    /// Detected from subtitle dialogue silence gap snapped to engine prior.
    SubtitleDialogueGap,
    /// Detected from final spoken dialogue line before end credits.
    SubtitleCredits,
}

/// Resolved skip marks along with origin provenance and confidence strength for logging and auditing.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SkipMarksWithOrigin {
    pub marks: SkipMarks,
    pub intro_origin: Option<MarkOrigin>,
    pub recap_origin: Option<MarkOrigin>,
    pub credits_origin: Option<MarkOrigin>,
    pub intro_strength: Option<SignalStrength>,
    pub recap_strength: Option<SignalStrength>,
    pub credits_strength: Option<SignalStrength>,
}

/// Analyzes parsed subtitle cues to detect skippable regions (intro, recap, credits).
pub fn detect_marks_from_cues(
    cues: &[Cue],
    duration: f64,
    delay: f64,
    is_tv: bool,
    engine_prior: Option<&SkipMarks>,
) -> SkipMarksWithOrigin {
    if duration <= 0.0 || duration.is_nan() || cues.is_empty() {
        let default_marks = engine_prior.cloned().unwrap_or_default();
        return SkipMarksWithOrigin {
            intro_origin: default_marks.intro.map(|_| MarkOrigin::Engine),
            recap_origin: default_marks.recap.map(|_| MarkOrigin::Engine),
            credits_origin: default_marks.credits.map(|_| MarkOrigin::Engine),
            intro_strength: default_marks.intro.map(|_| SignalStrength::Medium),
            recap_strength: default_marks.recap.map(|_| SignalStrength::Medium),
            credits_strength: default_marks.credits.map(|_| SignalStrength::Medium),
            marks: default_marks,
        };
    }

    // Shift cues by subtitle delay and ensure chronological ordering
    let mut shifted_cues: Vec<Cue> = cues
        .iter()
        .map(|c| Cue {
            s: (c.s + delay).max(0.0),
            e: (c.e + delay).max(0.0),
            t: c.t.clone(),
        })
        .filter(|c| c.s < duration && c.e > c.s)
        .collect();

    shifted_cues.sort_by(|a, b| a.s.total_cmp(&b.s));

    let (recap, recap_origin, recap_strength) = if is_tv {
        detect_recap(&shifted_cues, duration, engine_prior)
    } else {
        (None, None, None)
    };

    let (intro, intro_origin, intro_strength) = if is_tv {
        detect_intro(&shifted_cues, duration, engine_prior)
    } else {
        (None, None, None)
    };

    // Films don't use the credits mark yet, since Next episode is series-only.
    let (credits, credits_origin, credits_strength) = if is_tv {
        detect_credits(&shifted_cues, duration, engine_prior)
    } else {
        (None, None, None)
    };

    SkipMarksWithOrigin {
        marks: SkipMarks {
            intro,
            recap,
            credits,
        },
        intro_origin,
        recap_origin,
        credits_origin,
        intro_strength,
        recap_strength,
        credits_strength,
    }
}

/// Detects a "Previously on..." recap sequence near the beginning of a television episode.
fn detect_recap(
    cues: &[Cue],
    duration: f64,
    engine_prior: Option<&SkipMarks>,
) -> (Option<[f64; 2]>, Option<MarkOrigin>, Option<SignalStrength>) {
    let first = match cues.first() {
        Some(c) => c,
        None => {
            return (
                engine_prior.and_then(|m| m.recap),
                engine_prior
                    .and_then(|m| m.recap)
                    .map(|_| MarkOrigin::Engine),
                engine_prior
                    .and_then(|m| m.recap)
                    .map(|_| SignalStrength::Medium),
            );
        }
    };

    // Recaps must begin near the start of the stream
    if first.s <= MAX_RECAP_START_OFFSET_SECS {
        let mut recap_start: Option<f64> = None;
        let mut recap_end: Option<f64> = None;

        for (i, cue) in cues.iter().take(15).enumerate() {
            let clean = clean_text(&cue.t).to_lowercase();
            if is_recap_keyword(&clean) {
                if recap_start.is_none() {
                    recap_start = Some(cue.s.max(0.0));
                }
                // Locate where this recap block concludes: scene transition pause or music marker
                let mut end = cue.e;
                for next_cue in cues.iter().skip(i + 1).take(25) {
                    if next_cue.s - end >= RECAP_CLUSTER_PAUSE_SECS
                        || next_cue.s > MAX_RECAP_DURATION_SECS
                        || is_music_cue(&next_cue.t)
                    {
                        break;
                    }
                    end = next_cue.e;
                }
                recap_end = Some(end);
                break;
            }
        }

        if let (Some(s), Some(e)) = (recap_start, recap_end) {
            let len = e - s;
            if (MIN_RECAP_DURATION_SECS..=MAX_RECAP_DURATION_SECS).contains(&len)
                && e < duration * INTRO_SEARCH_MAX_RATIO
            {
                return (
                    Some([round_ratio(s / duration), round_ratio(e / duration)]),
                    Some(MarkOrigin::SubtitleRecap),
                    Some(SignalStrength::Strong),
                );
            }
        }
    }

    // Fall back to engine prior if subtitle had no recap keywords
    if let Some(recap) = engine_prior.and_then(|m| m.recap) {
        return (
            Some(recap),
            Some(MarkOrigin::Engine),
            Some(SignalStrength::Medium),
        );
    }

    (None, None, None)
}

fn is_recap_keyword(lower: &str) -> bool {
    // English
    lower.contains("previously on")
        || lower.contains("last time on")
        || lower.contains("last week on")
        || lower.contains("in the previous episode")
        || lower.contains("earlier on")
        || lower.contains("summary of previous")
        || lower.contains("previously in")
    // Spanish
        || lower.contains("anteriormente en")
        || lower.contains("en el episodio anterior")
        || lower.contains("en el capítulo anterior")
        || lower.contains("resumen del episodio")
    // French
        || lower.contains("précédemment dans")
        || lower.contains("dans l'épisode précédent")
        || lower.contains("résumé de l'épisode")
    // German
        || lower.contains("was bisher geschah")
        || lower.contains("in der letzten folge")
        || lower.contains("in der vorherigen episode")
    // Italian
        || lower.contains("nelle puntate precedenti")
        || lower.contains("negli episodi precedenti")
    // Portuguese
        || lower.contains("no episódio anterior")
}

/// Continuous spoken dialogue interval representing merged conversation cues.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpokenSpan {
    pub s: f64,
    pub e: f64,
    pub count: usize,
}

/// Merges overlapping and adjacent spoken subtitle cues (pause < 1.5s) into continuous spans.
/// Eliminates negative and zero gaps caused by out-of-order or simultaneous dialogue tracks.
pub fn build_spoken_spans(cues: &[Cue]) -> Vec<SpokenSpan> {
    let mut spans: Vec<SpokenSpan> = Vec::new();
    for cue in cues.iter().filter(|c| !is_non_dialogue_cue(&c.t)) {
        if let Some(last) = spans.last_mut()
            && cue.s <= last.e + 1.5
        {
            last.e = last.e.max(cue.e);
            last.count += 1;
            continue;
        }
        spans.push(SpokenSpan {
            s: cue.s,
            e: cue.e,
            count: 1,
        });
    }
    spans
}

/// Tests whether an engine intro estimate is definitely wrong.
/// 1. Sits past 25% of runtime or 600s.
/// 2. Has 3 or more spoken lines inside the window.
/// 3. Degenerate range (end <= start or length < 10s).
pub fn is_engine_intro_invalid(cues: &[Cue], duration: f64, intro: [f64; 2]) -> bool {
    let start_s = intro[0] * duration;
    let end_s = intro[1] * duration;

    if end_s <= start_s || (end_s - start_s) < MIN_VALID_ENGINE_INTRO_DURATION_SECS {
        return true;
    }

    let search_limit = (duration * INTRO_SEARCH_MAX_RATIO).min(INTRO_SEARCH_MAX_SECS);
    if start_s > search_limit {
        return true;
    }

    let spoken_count = cues
        .iter()
        .filter(|c| !is_non_dialogue_cue(&c.t))
        .filter(|c| c.s >= start_s && c.e <= end_s)
        .count();

    spoken_count >= ENGINE_INTRO_MAX_SPOKEN_LINES
}

/// Tests whether an engine credits estimate is definitely wrong.
/// An engine credits mark is invalid if:
/// 1. Sits before the minimum ratio (70% of runtime).
/// 2. Active scene dialogue continues immediately after the mark (within 15s) with 2+ lines,
///    indicating the main story has not concluded.
pub fn is_engine_credits_invalid(cues: &[Cue], duration: f64, credits: f64) -> bool {
    if credits < CREDITS_MIN_RATIO || credits >= 1.0 {
        return true;
    }

    let credits_s = credits * duration;
    let dialogue_after: Vec<&Cue> = cues
        .iter()
        .filter(|c| !is_non_dialogue_cue(&c.t) && !is_uploader_metadata(&c.t))
        .filter(|c| c.s > credits_s)
        .collect();

    if dialogue_after.len() < 2 {
        return false;
    }

    // If dialogue occurs after a stinger pause, it is a post-credits scene, not an invalid mark
    let first_after_s = dialogue_after[0].s;
    if (first_after_s - credits_s) >= POST_CREDITS_STINGER_PAUSE_SECS {
        return false;
    }

    // Active dialogue continues immediately after the engine mark (main scene is still ongoing)
    first_after_s - credits_s <= CREDITS_DIALOGUE_IMMEDIATE_POST_SECS
}

/// Decision layer for selecting or inventing the intro skip mark.
fn detect_intro(
    cues: &[Cue],
    duration: f64,
    engine_prior: Option<&SkipMarks>,
) -> (Option<[f64; 2]>, Option<MarkOrigin>, Option<SignalStrength>) {
    let search_limit = (duration * INTRO_SEARCH_MAX_RATIO).min(INTRO_SEARCH_MAX_SECS);

    // 1. Detect strong candidate: explicit music notation (♪, ♫) or bracketed theme markers
    let strong_candidate = detect_theme_music_cues(cues, duration, search_limit);

    // 2. Validate engine prior: check if engine intro is present and not definitely wrong
    let valid_engine_intro = engine_prior
        .and_then(|m| m.intro)
        .filter(|&intro| !is_engine_intro_invalid(cues, duration, intro));

    // Rule 1 & 2: A strong candidate beats the engine; when they agree within 30s, use strong candidate's edges
    if let Some(strong) = strong_candidate {
        if let Some(engine_intro) = valid_engine_intro {
            let strong_center_s = (strong[0] + strong[1]) * 0.5 * duration;
            let engine_center_s = (engine_intro[0] + engine_intro[1]) * 0.5 * duration;
            let dist = (strong_center_s - engine_center_s).abs();
            let origin = if dist <= STRONG_ENGINE_AGREE_TOLERANCE_SECS {
                MarkOrigin::SubtitleTheme
            } else {
                MarkOrigin::SubtitleThemeOverEngine
            };
            return (Some(strong), Some(origin), Some(SignalStrength::Strong));
        }
        return (
            Some(strong),
            Some(MarkOrigin::SubtitleTheme),
            Some(SignalStrength::Strong),
        );
    }

    // Rule 3: Valid engine prior present
    if let Some(engine_intro) = valid_engine_intro {
        // If a dialogue silence gap falls near the engine guess within 60s, refine the edges (Medium)
        if let Some(snapped_gap) =
            snap_dialogue_gap_to_prior(cues, duration, search_limit, engine_intro)
        {
            return (
                Some(snapped_gap),
                Some(MarkOrigin::SubtitleDialogueGap),
                Some(SignalStrength::Medium),
            );
        }
        // Rule 4: The engine beats a weak silence gap that does not corroborate it
        return (
            Some(engine_intro),
            Some(MarkOrigin::Engine),
            Some(SignalStrength::Medium),
        );
    }

    // Rule 5: Engine has no intro or engine is definitely wrong.
    // Invent an intro from a standalone silence gap under strict rules:
    // within first 3 minutes (180s), 20 to 100s long, preceded by cold open dialogue, followed by steady dialogue.
    if let Some(invented_gap) = detect_standalone_dialogue_gap(cues, duration) {
        return (
            Some(invented_gap),
            Some(MarkOrigin::SubtitleDialogueGap),
            Some(SignalStrength::Weak),
        );
    }

    (None, None, None)
}

/// Detects a standalone dialogue silence gap to invent an intro when the engine has no mark or is definitely wrong.
/// Strict criteria:
/// - Must begin within the first 3 minutes (180.0s).
/// - Duration must be between 20.0s and 100.0s.
/// - Must be preceded by cold open dialogue.
/// - Must be followed by steady dialogue (at least 3 cues spanning at least 20.0s).
fn detect_standalone_dialogue_gap(cues: &[Cue], duration: f64) -> Option<[f64; 2]> {
    let spans = build_spoken_spans(cues);
    if spans.len() < 2 {
        return None;
    }

    for i in 0..spans.len().saturating_sub(1) {
        let curr = &spans[i];
        let next = &spans[i + 1];

        // Standalone gap must begin within the first 3 minutes (180.0s)
        if curr.e > STANDALONE_GAP_MAX_TIME_SECS {
            break;
        }

        let gap_start = curr.e;
        let gap_end = next.s;
        if gap_end <= gap_start {
            continue;
        }

        let gap_len = gap_end - gap_start;

        // Gap must be 20s to 100s long
        if (STANDALONE_GAP_MIN_LEN_SECS..=STANDALONE_GAP_MAX_LEN_SECS).contains(&gap_len) {
            let has_cold_open = curr.s >= MIN_COLD_OPEN_DIALOGUE_SECS || i >= 1;

            let subsequent = &spans[i + 1..];
            let total_subsequent_cues: usize = subsequent.iter().map(|s| s.count).sum();
            let has_steady_dialogue = if total_subsequent_cues >= MIN_POST_INTRO_DIALOGUE_COUNT {
                let first_after = subsequent[0].s;
                let last_after = subsequent[subsequent.len().min(4) - 1].e;
                (last_after - first_after) >= MIN_POST_INTRO_DIALOGUE_SPAN_SECS
            } else {
                false
            };

            if has_cold_open && has_steady_dialogue {
                return Some([
                    round_ratio(gap_start / duration),
                    round_ratio(gap_end / duration),
                ]);
            }
        }
    }

    None
}

/// Scans for consecutive musical or theme caption markers (e.g. ♪, ♫, [Theme music]).
/// Bridges short pauses between musical cues and interstitial staff credit tags.
fn detect_theme_music_cues(cues: &[Cue], duration: f64, search_limit: f64) -> Option<[f64; 2]> {
    let mut music_start: Option<f64> = None;
    let mut music_end: Option<f64> = None;

    for cue in cues.iter().take_while(|c| c.s < search_limit) {
        let is_theme = is_music_cue(&cue.t);
        let is_interstitial = is_interstitial_theme_credit(&cue.t);

        if is_theme {
            if let Some(end) = music_end {
                // If within bridge threshold, extend the current theme block
                if cue.s - end <= THEME_CUE_BRIDGE_MAX_PAUSE_SECS {
                    music_end = Some(cue.e.max(end));
                    continue;
                }
                let len = end - music_start.unwrap_or(end);
                if (MIN_INTRO_DURATION_SECS..=MAX_INTRO_DURATION_SECS).contains(&len) {
                    return Some([
                        round_ratio(music_start.unwrap() / duration),
                        round_ratio(end / duration),
                    ]);
                }
            }
            music_start = Some(cue.s);
            music_end = Some(cue.e);
        } else if is_interstitial && music_start.is_some() {
            if let Some(end) = music_end
                && cue.s - end <= THEME_CUE_BRIDGE_MAX_PAUSE_SECS
            {
                music_end = Some(cue.e.max(end));
                continue;
            }
        } else if let (Some(s), Some(e)) = (music_start, music_end) {
            let len = e - s;
            if (MIN_INTRO_DURATION_SECS..=MAX_INTRO_DURATION_SECS).contains(&len) {
                return Some([round_ratio(s / duration), round_ratio(e / duration)]);
            }
            music_start = None;
            music_end = None;
        }
    }

    if let (Some(s), Some(e)) = (music_start, music_end) {
        let len = e - s;
        if (MIN_INTRO_DURATION_SECS..=MAX_INTRO_DURATION_SECS).contains(&len) {
            return Some([round_ratio(s / duration), round_ratio(e / duration)]);
        }
    }

    None
}

/// Checks if a non-music cue is an interstitial credit tag that can be bridged inside a theme song block.
fn is_interstitial_theme_credit(text: &str) -> bool {
    let clean = clean_text(text);
    let lower = clean.to_lowercase();
    let is_bracketed = (clean.starts_with('[') && clean.ends_with(']'))
        || (clean.starts_with('(') && clean.ends_with(')'));

    is_bracketed
        && (lower.contains("directed by")
            || lower.contains("written by")
            || lower.contains("produced by")
            || lower.contains("created by")
            || lower.contains("starring")
            || lower.contains("series composition")
            || lower.contains("animation")
            || lower.contains("music:")
            || lower.contains("opening theme:"))
}

/// Snaps dialogue silence gaps to the engine's prior intro estimate using merged spoken spans.
fn snap_dialogue_gap_to_prior(
    cues: &[Cue],
    duration: f64,
    search_limit: f64,
    prior_intro: [f64; 2],
) -> Option<[f64; 2]> {
    let spans = build_spoken_spans(cues);
    if spans.len() < 2 {
        return None;
    }

    let prior_center_s = (prior_intro[0] + prior_intro[1]) * 0.5 * duration;
    let mut candidate_gaps: Vec<(f64, f64)> = Vec::new();

    for i in 0..spans.len().saturating_sub(1) {
        let curr = &spans[i];
        let next = &spans[i + 1];

        if curr.s >= search_limit {
            break;
        }

        let gap_start = curr.e;
        let gap_end = next.s;
        if gap_end <= gap_start {
            continue;
        }

        let gap_len = gap_end - gap_start;

        if (MIN_INTRO_DURATION_SECS..=MAX_INTRO_DURATION_SECS).contains(&gap_len) {
            let has_cold_open = curr.s >= MIN_COLD_OPEN_DIALOGUE_SECS || i >= 1;

            let subsequent = &spans[i + 1..];
            let total_subsequent_cues: usize = subsequent.iter().map(|s| s.count).sum();
            let has_continued_dialogue = if total_subsequent_cues >= MIN_POST_INTRO_DIALOGUE_COUNT {
                let first_after = subsequent[0].s;
                let last_after = subsequent[subsequent.len().min(4) - 1].e;
                (last_after - first_after) >= MIN_POST_INTRO_DIALOGUE_SPAN_SECS
            } else {
                false
            };

            if has_cold_open && has_continued_dialogue {
                candidate_gaps.push((gap_start, gap_end));
            }
        }
    }

    // Pick the gap closest to the engine's guess
    let mut best_gap = None;
    let mut min_dist = f64::MAX;

    for (start, end) in candidate_gaps {
        let gap_center = (start + end) * 0.5;
        let dist = (gap_center - prior_center_s).abs();
        if dist < min_dist && dist <= GAP_PRIOR_SNAP_TOLERANCE_SECS {
            min_dist = dist;
            best_gap = Some([round_ratio(start / duration), round_ratio(end / duration)]);
        }
    }

    best_gap
}

/// Detects the start of television end credits by locating the final dialogue line before credits.
fn detect_credits(
    cues: &[Cue],
    duration: f64,
    engine_prior: Option<&SkipMarks>,
) -> (Option<f64>, Option<MarkOrigin>, Option<SignalStrength>) {
    let mut window_start =
        (duration - CREDITS_SEARCH_WINDOW_TV_SECS).max(duration * CREDITS_MIN_RATIO);
    if let Some(prior) = engine_prior.and_then(|m| m.credits) {
        let prior_s = prior * duration;
        window_start = window_start
            .min((prior_s - CREDITS_PRIOR_SNAP_TOLERANCE_SECS).max(duration * CREDITS_MIN_RATIO));
    }

    // Collect dialogue cues in the credits search window
    let window_cues: Vec<&Cue> = cues
        .iter()
        .filter(|c| {
            c.s >= window_start && !is_non_dialogue_cue(&c.t) && !is_uploader_metadata(&c.t)
        })
        .collect();

    let valid_engine_credits = engine_prior
        .and_then(|m| m.credits)
        .filter(|&c| !is_engine_credits_invalid(cues, duration, c));

    if window_cues.is_empty() {
        return (
            valid_engine_credits,
            valid_engine_credits.map(|_| MarkOrigin::Engine),
            valid_engine_credits.map(|_| SignalStrength::Medium),
        );
    }

    // Check for post-credits stinger scene: a pause of >= POST_CREDITS_STINGER_PAUSE_SECS followed by dialogue
    let mut main_dialogue_end = window_cues.last().unwrap().e;
    for i in 0..window_cues.len().saturating_sub(1) {
        let curr = window_cues[i];
        let next = window_cues[i + 1];
        if next.s - curr.e >= POST_CREDITS_STINGER_PAUSE_SECS {
            main_dialogue_end = curr.e;
            break;
        }
    }

    let detected_start = main_dialogue_end + CREDITS_POST_DIALOGUE_PAD_SECS;

    if let Some(prior) = valid_engine_credits {
        let prior_s = prior * duration;
        if (detected_start - prior_s).abs() <= CREDITS_PRIOR_SNAP_TOLERANCE_SECS {
            return (
                Some(round_ratio(detected_start / duration)),
                Some(MarkOrigin::SubtitleCredits),
                Some(SignalStrength::Medium),
            );
        }
        return (
            Some(prior),
            Some(MarkOrigin::Engine),
            Some(SignalStrength::Medium),
        );
    }

    if detected_start < duration - 4.0 {
        return (
            Some(round_ratio(detected_start / duration)),
            Some(MarkOrigin::SubtitleCredits),
            Some(SignalStrength::Medium),
        );
    }

    (None, None, None)
}

/// Cleans formatting tags (HTML tags like `<i>`, SSA/ASS tags like `{\an8}`).
fn clean_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_tag = false;

    for ch in text.chars() {
        match ch {
            '<' | '{' => in_tag = true,
            '>' | '}' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }

    out.trim().to_string()
}

/// Checks if a subtitle line is a theme music marker or opening title song.
/// Only matches bracketed descriptors or lines with musical symbols to prevent matching dialogue words.
fn is_music_cue(text: &str) -> bool {
    let clean = clean_text(text);
    let lower = clean.to_lowercase();

    let has_symbol = clean.contains('♪')
        || clean.contains('♫')
        || clean.contains("&#9834;")
        || clean.contains("&#9835;")
        || clean.contains("&music;");

    let is_bracketed = (clean.starts_with('[') && clean.ends_with(']'))
        || (clean.starts_with('(') && clean.ends_with(')'));

    let is_theme_desc = is_bracketed
        && (lower.contains("theme")
            || lower.contains("title music")
            || lower.contains("opening music")
            || lower.contains("opening song")
            || lower.contains("opening credits")
            || lower.contains("instrumental music")
            || lower.contains("main title")
            || lower.contains("theme song"));

    has_symbol || is_theme_desc
}

/// Checks if a cue represents non-dialogue (music, sound effects, or caption metadata).
fn is_non_dialogue_cue(text: &str) -> bool {
    let clean = clean_text(text);
    let trimmed = clean.trim();

    if trimmed.is_empty() {
        return true;
    }

    if trimmed.contains('♪')
        || trimmed.contains('♫')
        || trimmed.contains("&#9834;")
        || trimmed.contains("&#9835;")
        || trimmed.contains("&music;")
    {
        return true;
    }

    if (trimmed.starts_with('[') && trimmed.ends_with(']'))
        || (trimmed.starts_with('(') && trimmed.ends_with(')'))
    {
        return true;
    }

    is_music_cue(text)
}

/// Checks if a cue is third-party subtitle uploader metadata or advertisements at the end of a file.
fn is_uploader_metadata(text: &str) -> bool {
    let lower = clean_text(text).to_lowercase();
    lower.contains("subtitles by")
        || lower.contains("synced by")
        || lower.contains("resync by")
        || lower.contains("translated by")
        || lower.contains("encoded by")
        || lower.contains("ripped by")
        || lower.contains("downloaded from")
        || lower.contains("opensubtitles")
        || lower.contains("addic7ed")
        || lower.contains("yify")
        || lower.contains("http://")
        || lower.contains("https://")
        || lower.contains("www.")
}

fn round_ratio(val: f64) -> f64 {
    ((val * 10000.0).round() / 10000.0).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cue(s: f64, e: f64, t: &str) -> Cue {
        Cue {
            s,
            e,
            t: t.to_string(),
        }
    }

    // -------------------------------------------------------------------------
    // UNIT TESTS
    // -------------------------------------------------------------------------

    #[test]
    fn test_applies_subtitle_delay_offset() {
        let duration = 3000.0;
        let cues = vec![
            make_cue(10.0, 15.0, "Cold open line 1."),
            make_cue(16.0, 30.0, "Cold open line 2."),
            make_cue(32.0, 72.0, "♪ [Theme music playing] ♪"),
            make_cue(75.0, 80.0, "Main scene line 1."),
            make_cue(82.0, 90.0, "Main scene line 2."),
            make_cue(92.0, 100.0, "Main scene line 3."),
        ];

        let res_zero = detect_marks_from_cues(&cues, duration, 0.0, true, None);
        assert_eq!(res_zero.marks.intro.unwrap(), [0.0107, 0.0240]);
        assert_eq!(res_zero.intro_origin, Some(MarkOrigin::SubtitleTheme));

        let res_delayed = detect_marks_from_cues(&cues, duration, 3.0, true, None);
        assert_eq!(res_delayed.marks.intro.unwrap(), [0.0117, 0.0250]);
        assert_eq!(res_delayed.intro_origin, Some(MarkOrigin::SubtitleTheme));
    }

    #[test]
    fn test_dialogue_word_theme_does_not_trigger_music_mark() {
        let duration = 3000.0;
        let cues = vec![
            make_cue(10.0, 15.0, "What is the theme of the party?"),
            make_cue(16.0, 22.0, "The theme is 1920s Great Gatsby."),
            make_cue(24.0, 30.0, "I will find a costume."),
            make_cue(35.0, 40.0, "See you tonight."),
            make_cue(42.0, 50.0, "Do not be late."),
        ];

        let res = detect_marks_from_cues(&cues, duration, 0.0, true, None);
        assert!(res.marks.intro.is_none());
        assert_eq!(res.intro_origin, None);
    }

    #[test]
    fn test_snaps_closest_dialogue_gap_to_engine_prior() {
        let duration = 3000.0;
        let cues = vec![
            make_cue(5.0, 15.0, "Line 1."),
            make_cue(16.0, 30.0, "Line 2."),
            // Gap 1: 30s to 50s (20s pause)
            make_cue(50.0, 60.0, "Line 3."),
            make_cue(65.0, 80.0, "Line 4."),
            make_cue(90.0, 120.0, "Cold open finish."),
            // Gap 2: 120s to 180s (60s intro)
            make_cue(180.0, 190.0, "Main scene start."),
            make_cue(192.0, 200.0, "Main scene line 2."),
            make_cue(202.0, 210.0, "Main scene line 3."),
        ];

        let prior = SkipMarks {
            intro: Some([0.045, 0.065]), // center ~165s
            recap: None,
            credits: None,
        };

        let res = detect_marks_from_cues(&cues, duration, 0.0, true, Some(&prior));
        assert!(res.marks.intro.is_some());
        let intro = res.marks.intro.unwrap();
        assert_eq!(intro[0], 0.0400);
        assert_eq!(intro[1], 0.0600);
        assert_eq!(res.intro_origin, Some(MarkOrigin::SubtitleDialogueGap));
    }

    #[test]
    fn test_films_never_receive_intro_or_credits_skip_marks() {
        let duration = 7200.0;
        let cues = vec![
            make_cue(10.0, 20.0, "Opening prologue."),
            make_cue(22.0, 60.0, "♪ [Epic film score opening] ♪"),
            make_cue(65.0, 80.0, "Prologue conclusion."),
            make_cue(7000.0, 7050.0, "The end of our journey."),
        ];

        let prior = SkipMarks {
            intro: Some([0.01, 0.03]),
            recap: None,
            credits: Some(0.98),
        };

        let res = detect_marks_from_cues(&cues, duration, 0.0, false, Some(&prior));
        assert_eq!(res.marks.intro, None);
        assert_eq!(res.marks.recap, None);
        assert_eq!(res.marks.credits, None);
    }

    // -------------------------------------------------------------------------
    // 10 REALISTIC EPISODE FIXTURE TESTS (ACCURACY WITHIN ±3 SECONDS)
    // -------------------------------------------------------------------------

    // Fixture 1: Anime Episode (Jujutsu Kaisen style with AniSkip known marks)
    // Cold open 0-88s, OP theme 89.2s-179.2s (90s theme with ♪ lines), credits at 1340s.
    #[test]
    fn test_fixture_1_anime_theme_with_music_symbols() {
        let duration = 1420.0;
        let cues = vec![
            make_cue(5.0, 12.0, "Curse energy is accumulating."),
            make_cue(14.0, 28.0, "We must exorcise it before nightfall."),
            make_cue(30.0, 50.0, "Understood. Deploying the barrier."),
            make_cue(55.0, 88.0, "Here it comes!"),
            // Opening Theme with consecutive musical lines
            make_cue(89.2, 110.0, "♪ Oshiete oshiete yo sono shikumi wo ♪"),
            make_cue(111.0, 135.0, "♪ Boku no naka ni dare ga iru no ♪"),
            make_cue(136.0, 160.0, "♪ Kowareta kowareta yo kono sekai de ♪"),
            make_cue(161.0, 179.2, "♪ Yurete yuganda sekai ni dan dan ♪"),
            // Post-intro dialogue
            make_cue(182.0, 195.0, "Class begins now, everyone take your seats."),
            make_cue(197.0, 210.0, "Sensei, where is Megumi?"),
            make_cue(212.0, 225.0, "He is on a mission."),
            // Ending credits
            make_cue(1310.0, 1338.5, "Next time will be dangerous."),
            make_cue(1340.0, 1400.0, "♪ [Ending song playing] ♪"),
        ];

        let prior = SkipMarks {
            intro: Some([0.060, 0.125]), // rough catalogue estimate: ~85s to ~177s
            recap: None,
            credits: Some(0.942), // ~1338s
        };

        let res = detect_marks_from_cues(&cues, duration, 0.0, true, Some(&prior));
        let intro = res.marks.intro.expect("Anime intro must be detected");
        let intro_start_s = intro[0] * duration;
        let intro_end_s = intro[1] * duration;

        // Verify within ±3.0 seconds of true marks 89.2s and 179.2s
        assert!(
            (intro_start_s - 89.2).abs() <= 3.0,
            "Start diff: {}",
            (intro_start_s - 89.2).abs()
        );
        assert!(
            (intro_end_s - 179.2).abs() <= 3.0,
            "End diff: {}",
            (intro_end_s - 179.2).abs()
        );
        assert_eq!(res.intro_origin, Some(MarkOrigin::SubtitleTheme));

        let credits = res.marks.credits.expect("Credits must be detected");
        let credits_s = credits * duration;
        let expected_credits_s = 1338.5 + CREDITS_POST_DIALOGUE_PAD_SECS;
        assert!((credits_s - expected_credits_s).abs() <= 3.0);
        assert_eq!(res.credits_origin, Some(MarkOrigin::SubtitleCredits));
    }

    // Fixture 2: Breaking Bad Cold Open + Dialogue Silence Gap
    // Dialogue 10s-118s, Silence Gap 118.0s-134.5s (16.5s intro), dialogue continues.
    #[test]
    fn test_fixture_2_breaking_bad_dialogue_gap_snapping() {
        let duration = 2850.0;
        let cues = vec![
            make_cue(10.0, 25.0, "My name is Walter Hartwell White."),
            make_cue(
                26.0,
                48.0,
                "I live at 308 Negra Arroyo Lane, Albuquerque, New Mexico.",
            ),
            make_cue(
                50.0,
                85.0,
                "To all law enforcement entities, this is not an admission of guilt.",
            ),
            make_cue(87.0, 118.0, "Whatever happens, I did it for my family."),
            // Dialogue silence gap representing the title sequence (118.0s to 134.5s)
            make_cue(134.5, 145.0, "Three weeks earlier..."),
            make_cue(147.0, 160.0, "Happy birthday, Walter!"),
            make_cue(162.0, 175.0, "Make a wish, honey."),
            make_cue(178.0, 195.0, "Fifty years old, look at you."),
            make_cue(2700.0, 2730.0, "We've got work to do."),
        ];

        let prior = SkipMarks {
            intro: Some([0.0410, 0.0475]), // ~116.8s to ~135.3s from engine
            recap: None,
            credits: Some(0.958),
        };

        let res = detect_marks_from_cues(&cues, duration, 0.0, true, Some(&prior));
        let intro = res.marks.intro.expect("Intro gap must snap to prior");
        let intro_start_s = intro[0] * duration;
        let intro_end_s = intro[1] * duration;

        assert!((intro_start_s - 118.0).abs() <= 3.0);
        assert!((intro_end_s - 134.5).abs() <= 3.0);
        assert_eq!(res.intro_origin, Some(MarkOrigin::SubtitleDialogueGap));
    }

    // Fixture 3: Drama with Silent Cold Open (Failure Case Guard)
    // First 65s are completely silent camera panning. Intro theme is at 180s-240s.
    // Must NOT mark 0s-65s as intro!
    #[test]
    fn test_fixture_3_silent_cold_open_not_marked_as_intro() {
        let duration = 3300.0;
        let cues = vec![
            // First spoken line does not occur until 65 seconds in!
            make_cue(65.0, 80.0, "The winter has been unusually harsh this year."),
            make_cue(82.0, 110.0, "The northern borders are already frozen over."),
            make_cue(112.0, 175.0, "Then we must ride out before dawn."),
            // Actual intro theme
            make_cue(180.0, 240.0, "♪ [Dramatic orchestral title theme] ♪"),
            // Scene after intro
            make_cue(243.0, 260.0, "Mount your horses, men!"),
            make_cue(262.0, 275.0, "Right behind you, Commander."),
            make_cue(278.0, 290.0, "Keep your shields up."),
        ];

        let res = detect_marks_from_cues(&cues, duration, 0.0, true, None);
        let intro = res
            .marks
            .intro
            .expect("Intro must be detected at true theme position");
        let intro_start_s = intro[0] * duration;
        let intro_end_s = intro[1] * duration;

        assert!((intro_start_s - 180.0).abs() <= 3.0);
        assert!((intro_end_s - 240.0).abs() <= 3.0);
        assert_eq!(res.intro_origin, Some(MarkOrigin::SubtitleTheme));
        assert!(res.marks.recap.is_none());
    }

    // Fixture 4: Long Suspense Scene Without Dialogue (Failure Case Guard)
    // 40s of silent stalking at 300s-340s. True intro is at 120s-180s.
    // Detector must snap to true intro and NOT the 40s silence gap in the suspense scene.
    #[test]
    fn test_fixture_4_suspense_scene_silence_does_not_override_intro() {
        let duration = 3000.0;
        let cues = vec![
            make_cue(10.0, 40.0, "We have a lead on the suspect."),
            make_cue(42.0, 115.0, "Proceed with caution."),
            // Gap 1: True intro at 115s to 175s (60s)
            make_cue(175.0, 190.0, "All units in position."),
            make_cue(192.0, 210.0, "Copy that, moving in."),
            make_cue(212.0, 230.0, "Check the perimeter."),
            make_cue(235.0, 290.0, "The building is dark."),
            // Gap 2: Suspense stalking scene (290s to 335s = 45s silence)
            make_cue(335.0, 350.0, "Did you hear that sound?"),
            make_cue(352.0, 370.0, "Over by the window!"),
            make_cue(372.0, 390.0, "Freeze! Police!"),
        ];

        let prior = SkipMarks {
            intro: Some([0.038, 0.059]), // ~114s to ~177s
            recap: None,
            credits: None,
        };

        let res = detect_marks_from_cues(&cues, duration, 0.0, true, Some(&prior));
        let intro = res.marks.intro.expect("Intro must match prior");
        let intro_start_s = intro[0] * duration;
        let intro_end_s = intro[1] * duration;

        assert!((intro_start_s - 115.0).abs() <= 3.0);
        assert!((intro_end_s - 175.0).abs() <= 3.0);
        assert_eq!(res.intro_origin, Some(MarkOrigin::SubtitleDialogueGap));
    }

    // Fixture 5: Non-English Subtitle File (Spanish)
    // No English keywords. Dialogue gap snaps to engine prior within ±3s.
    #[test]
    fn test_fixture_5_non_english_subtitles_snapping() {
        let duration = 3200.0;
        let cues = vec![
            make_cue(15.0, 35.0, "Hola, buenos días inspector."),
            make_cue(36.0, 70.0, "Tenemos novedades sobre el caso del museo."),
            make_cue(72.0, 95.0, "El culpable dejó una pista importante."),
            // Spanish title sequence gap (95.0s to 170.0s)
            make_cue(170.0, 185.0, "Revisemos las grabaciones de seguridad."),
            make_cue(187.0, 205.0, "Aquí está la cámara número cuatro."),
            make_cue(207.0, 225.0, "Mire esa persona con abrigo negro."),
            make_cue(3050.0, 3100.0, "Caso cerrado. Buen trabajo a todos."),
        ];

        let prior = SkipMarks {
            intro: Some([0.030, 0.053]), // center ~132.8s
            recap: None,
            credits: Some(0.969), // ~3100s
        };

        let res = detect_marks_from_cues(&cues, duration, 0.0, true, Some(&prior));
        let intro = res.marks.intro.expect("Spanish gap must snap to prior");
        let intro_start_s = intro[0] * duration;
        let intro_end_s = intro[1] * duration;

        assert!((intro_start_s - 95.0).abs() <= 3.0);
        assert!((intro_end_s - 170.0).abs() <= 3.0);
        assert_eq!(res.intro_origin, Some(MarkOrigin::SubtitleDialogueGap));

        let credits = res.marks.credits.expect("Credits must snap");
        let credits_s = credits * duration;
        let expected_credits_s = 3100.0 + CREDITS_POST_DIALOGUE_PAD_SECS;
        assert!((credits_s - expected_credits_s).abs() <= 3.0);
    }

    // Fixture 6: Song Lyrics Written Without ♪ Notation
    // Background radio music plays lyrics with regular text during dialogue.
    // Does NOT trigger music theme intro.
    #[test]
    fn test_fixture_6_lyrics_without_music_symbols_not_mistaken_for_theme() {
        let duration = 2900.0;
        let cues = vec![
            make_cue(10.0, 20.0, "Turn on the car radio."),
            // Radio song lyrics without ♪
            make_cue(22.0, 35.0, "Don't stop believing, hold on to that feeling"),
            make_cue(36.0, 50.0, "Streetlight people living just to find emotion"),
            make_cue(52.0, 65.0, "I love this old song from the eighties."),
            make_cue(67.0, 80.0, "Where are we heading next?"),
            make_cue(82.0, 95.0, "Downtown to the courthouse."),
        ];

        let res = detect_marks_from_cues(&cues, duration, 0.0, true, None);
        assert!(
            res.marks.intro.is_none(),
            "Plain text lyrics must not be mistaken for theme"
        );
    }

    // Fixture 7: Episode with Post-Credits Stinger Scene
    // Episode dialogue ends at 2400s. Credits crawl 2400s-2520s.
    // Stinger scene dialogue occurs at 2520s-2550s. Credits mark should be placed before stinger.
    #[test]
    fn test_fixture_7_post_credits_stinger_scene() {
        let duration = 2600.0;
        let cues = vec![
            make_cue(2200.0, 2250.0, "We will meet again, Doctor."),
            make_cue(2260.0, 2398.5, "I look forward to it, General."),
            // Stinger scene dialogue after 121.5s credits roll
            make_cue(2520.0, 2535.0, "Sir, someone is outside."),
            make_cue(2536.0, 2550.0, "Tell them the initiative has begun."),
        ];

        let res = detect_marks_from_cues(&cues, duration, 0.0, true, None);
        let credits = res.marks.credits.expect("Credits must be detected");
        let credits_s = credits * duration;
        let expected_credits_s = 2398.5 + CREDITS_POST_DIALOGUE_PAD_SECS;

        assert!((credits_s - expected_credits_s).abs() <= 3.0);
        assert_eq!(res.credits_origin, Some(MarkOrigin::SubtitleCredits));
    }

    // Fixture 8: Episode with Recap and "Next Time On" Preview
    // "Previously on..." at 3s-36s, cold open 40s-100s, theme at 102s-162s, preview at 2850s.
    #[test]
    fn test_fixture_8_recap_and_next_time_preview() {
        let duration = 2950.0;
        let cues = vec![
            make_cue(3.0, 15.0, "Previously on Salt Harbour..."),
            make_cue(16.0, 28.0, "The vault was breached yesterday."),
            make_cue(29.0, 36.0, "We need to find who did this."),
            // Cold open
            make_cue(40.0, 65.0, "Morning briefing starts in five minutes."),
            make_cue(68.0, 99.0, "The director wants an update now."),
            // Theme
            make_cue(102.0, 162.0, "♪ [Theme song playing] ♪"),
            // Main episode
            make_cue(165.0, 185.0, "Here are the files you requested."),
            make_cue(2700.0, 2740.0, "The mission is accomplished."),
            // Preview
            make_cue(2850.0, 2870.0, "Next time on Salt Harbour: The Reckoning."),
        ];

        let res = detect_marks_from_cues(&cues, duration, 0.0, true, None);

        let recap = res.marks.recap.expect("Recap must be detected");
        let recap_start_s = recap[0] * duration;
        let recap_end_s = recap[1] * duration;
        assert!((recap_start_s - 3.0).abs() <= 3.0);
        assert!((recap_end_s - 36.0).abs() <= 3.0);
        assert_eq!(res.recap_origin, Some(MarkOrigin::SubtitleRecap));

        let intro = res.marks.intro.expect("Intro must be detected");
        let intro_start_s = intro[0] * duration;
        let intro_end_s = intro[1] * duration;
        assert!((intro_start_s - 102.0).abs() <= 3.0);
        assert!((intro_end_s - 162.0).abs() <= 3.0);
        assert_eq!(res.intro_origin, Some(MarkOrigin::SubtitleTheme));
    }

    // Fixture 9: Dialogue discussing a "theme" in conversation
    // Spoken dialogue: "The theme of the exhibition is modern art". Must NOT trigger intro mark.
    #[test]
    fn test_fixture_9_dialogue_theme_word_does_not_trigger_intro() {
        let duration = 3000.0;
        let cues = vec![
            make_cue(20.0, 35.0, "Good morning, curator."),
            make_cue(
                36.0,
                55.0,
                "The theme of the exhibition is modern minimalism.",
            ),
            make_cue(57.0, 75.0, "We have gathered pieces from across Europe."),
            make_cue(77.0, 95.0, "Let us inspect the main gallery."),
            make_cue(97.0, 115.0, "The lighting is calibrated properly."),
        ];

        let res = detect_marks_from_cues(&cues, duration, 0.0, true, None);
        assert!(res.marks.intro.is_none());
        assert_eq!(res.intro_origin, None);
    }

    // Fixture 10: Film Feature Presentation (No Skip Marks)
    // Feature film (`is_tv = false`). SkipMarks must be empty.
    #[test]
    fn test_fixture_10_feature_film_no_skip_marks() {
        let duration = 7200.0; // 2 hour film
        let cues = vec![
            make_cue(5.0, 20.0, "Once upon a time in the ancient kingdom..."),
            make_cue(25.0, 85.0, "♪ [Main orchestral title score] ♪"),
            make_cue(90.0, 120.0, "The legend was passed down through centuries."),
            make_cue(
                7000.0,
                7060.0,
                "And so they lived peacefully forever after.",
            ),
        ];

        let prior = SkipMarks {
            intro: Some([0.003, 0.012]),
            recap: None,
            credits: Some(0.98),
        };

        let res = detect_marks_from_cues(&cues, duration, 0.0, false, Some(&prior));
        assert!(
            res.marks.intro.is_none(),
            "Films must not receive intro mark"
        );
        assert!(
            res.marks.recap.is_none(),
            "Films must not receive recap mark"
        );
        assert!(
            res.marks.credits.is_none(),
            "Films must not receive credits mark"
        );
        assert_eq!(res.intro_origin, None);
        assert_eq!(res.recap_origin, None);
        assert_eq!(res.credits_origin, None);
    }

    #[test]
    fn test_decision_layer_invents_intro_when_engine_has_no_prior() {
        let duration = 3000.0;
        let cues = vec![
            make_cue(10.0, 25.0, "Cold open line 1."),
            make_cue(26.0, 40.0, "Cold open line 2."),
            // Clean dialogue gap: 40.0s to 100.0s (60s intro, within first 3 minutes)
            make_cue(100.0, 115.0, "Main scene line 1."),
            make_cue(117.0, 130.0, "Main scene line 2."),
            make_cue(132.0, 145.0, "Main scene line 3."),
            make_cue(147.0, 160.0, "Main scene line 4."),
        ];

        // Engine has no marks
        let res = detect_marks_from_cues(&cues, duration, 0.0, true, None);
        let intro = res
            .marks
            .intro
            .expect("Should invent intro from clean silence gap");
        let intro_start_s = intro[0] * duration;
        let intro_end_s = intro[1] * duration;

        assert!((intro_start_s - 40.0).abs() <= 3.0);
        assert!((intro_end_s - 100.0).abs() <= 3.0);
        assert_eq!(res.intro_origin, Some(MarkOrigin::SubtitleDialogueGap));
    }

    #[test]
    fn test_decision_layer_rejects_definitely_wrong_engine_intro() {
        let duration = 3000.0;
        // Engine claims intro is at 20.0s to 60.0s (ratio ~0.0067..0.0200)
        let prior = SkipMarks {
            intro: Some([0.0067, 0.0200]),
            recap: None,
            credits: None,
        };

        // But subtitles show characters actively talking through that window (3+ lines)
        let cues = vec![
            make_cue(22.0, 30.0, "Spoken dialogue line 1 inside engine window."),
            make_cue(32.0, 42.0, "Spoken dialogue line 2 inside engine window."),
            make_cue(44.0, 55.0, "Spoken dialogue line 3 inside engine window."),
            make_cue(56.0, 65.0, "Cold open concludes."),
            // The real intro gap is at 65.0s to 125.0s (60s)
            make_cue(125.0, 140.0, "Post-intro scene line 1."),
            make_cue(142.0, 155.0, "Post-intro scene line 2."),
            make_cue(157.0, 170.0, "Post-intro scene line 3."),
        ];

        let res = detect_marks_from_cues(&cues, duration, 0.0, true, Some(&prior));
        let intro = res
            .marks
            .intro
            .expect("Should reject wrong engine intro and pick real gap");
        let intro_start_s = intro[0] * duration;
        let intro_end_s = intro[1] * duration;

        // Snaps to real gap 65s-125s, not engine's 20s-60s
        assert!((intro_start_s - 65.0).abs() <= 3.0);
        assert!((intro_end_s - 125.0).abs() <= 3.0);
        assert_eq!(res.intro_origin, Some(MarkOrigin::SubtitleDialogueGap));
    }

    #[test]
    fn test_decision_layer_strong_candidate_beats_conflicting_engine_guess() {
        let duration = 3000.0;
        let prior = SkipMarks {
            intro: Some([0.010, 0.020]), // engine guessed 30s to 60s
            recap: None,
            credits: None,
        };

        // Strong candidate: explicit theme song block at 90s to 160s
        let cues = vec![
            make_cue(10.0, 25.0, "Cold open conversation."),
            make_cue(90.0, 160.0, "♪ [Official Opening Theme Song] ♪"),
            make_cue(165.0, 180.0, "Main scene dialogue."),
        ];

        let res = detect_marks_from_cues(&cues, duration, 0.0, true, Some(&prior));
        let intro = res
            .marks
            .intro
            .expect("Strong theme candidate must beat engine");
        let intro_start_s = intro[0] * duration;
        let intro_end_s = intro[1] * duration;

        assert!((intro_start_s - 90.0).abs() <= 3.0);
        assert!((intro_end_s - 160.0).abs() <= 3.0);
        assert_eq!(res.intro_origin, Some(MarkOrigin::SubtitleThemeOverEngine));
    }

    #[test]
    fn test_decision_layer_strong_candidate_agrees_with_engine_within_tolerance() {
        let duration = 3000.0;
        let prior = SkipMarks {
            intro: Some([0.028, 0.055]), // engine guessed 84s to 165s (center 124.5s)
            recap: None,
            credits: None,
        };

        // Theme song at 90s to 160s (center 125.0s, difference 0.5s <= 30s)
        let cues = vec![
            make_cue(10.0, 25.0, "Cold open conversation."),
            make_cue(90.0, 160.0, "♪ [Corroborated Theme Song] ♪"),
            make_cue(165.0, 180.0, "Main scene dialogue."),
        ];

        let res = detect_marks_from_cues(&cues, duration, 0.0, true, Some(&prior));
        let intro = res.marks.intro.expect("Theme candidate must be chosen");
        let intro_start_s = intro[0] * duration;
        let intro_end_s = intro[1] * duration;

        assert!((intro_start_s - 90.0).abs() <= 3.0);
        assert!((intro_end_s - 160.0).abs() <= 3.0);
        assert_eq!(res.intro_origin, Some(MarkOrigin::SubtitleTheme));
        assert_eq!(res.intro_strength, Some(SignalStrength::Strong));
    }

    #[test]
    fn test_decision_layer_rejects_definitely_wrong_engine_credits() {
        let duration = 3000.0;
        // Engine guessed credits at 2200s (ratio ~0.7333)
        let prior = SkipMarks {
            intro: None,
            recap: None,
            credits: Some(0.7333),
        };

        // Subtitles show main scene dialogue continues immediately at 2205s, 2220s, 2300s
        let cues = vec![
            make_cue(
                2205.0,
                2215.0,
                "Immediate scene dialogue right after engine mark.",
            ),
            make_cue(
                2220.0,
                2250.0,
                "Characters still conversing in dramatic climax.",
            ),
            make_cue(2260.0, 2350.0, "Final resolution of the episode."),
        ];

        let res = detect_marks_from_cues(&cues, duration, 0.0, true, Some(&prior));
        let credits = res.marks.credits.expect("Credits must be detected");
        let credits_s = credits * duration;
        let expected_credits_s = 2350.0 + CREDITS_POST_DIALOGUE_PAD_SECS;

        // Rejected 2200s because dialogue continued immediately, placed after real dialogue at 2351.5s
        assert!((credits_s - expected_credits_s).abs() <= 3.0);
        assert_eq!(res.credits_origin, Some(MarkOrigin::SubtitleCredits));
    }

    #[test]
    fn test_engine_credits_kept_when_followed_by_stinger() {
        let duration = 3000.0;
        // Engine guessed credits at 2400.0s (0.8000)
        let prior = SkipMarks {
            intro: None,
            recap: None,
            credits: Some(0.8000),
        };

        // Main scene ends at 2398.5s. Then credits roll for 100s. Stinger occurs at 2500s.
        let cues = vec![
            make_cue(2350.0, 2398.5, "That is the end of our tale."),
            // Post-credits stinger after 101.5s pause
            make_cue(2500.0, 2520.0, "Or is it just the beginning?"),
            make_cue(2522.0, 2535.0, "To be continued..."),
        ];

        // Engine credits at 2400s is NOT invalid despite stinger dialogue
        assert!(!is_engine_credits_invalid(&cues, duration, 0.8000));
        let res = detect_marks_from_cues(&cues, duration, 0.0, true, Some(&prior));
        let credits = res.marks.credits.expect("Credits must be preserved");
        let credits_s = credits * duration;

        // Snapped to end of main dialogue (2398.5 + 1.5 = 2400.0)
        assert!((credits_s - 2400.0).abs() <= 3.0);
        assert_eq!(res.credits_origin, Some(MarkOrigin::SubtitleCredits));
    }

    #[test]
    fn test_multilingual_recap_spanish_french_german() {
        let duration = 3000.0;
        // Spanish recap
        let es_cues = vec![
            make_cue(3.0, 15.0, "Anteriormente en La Casa de Papel..."),
            make_cue(16.0, 32.0, "El plan sigue en marcha."),
            make_cue(36.0, 60.0, "Profesor, tenemos un problema."),
        ];
        let es_res = detect_marks_from_cues(&es_cues, duration, 0.0, true, None);
        assert!(es_res.marks.recap.is_some());
        assert_eq!(es_res.recap_origin, Some(MarkOrigin::SubtitleRecap));

        // French recap
        let fr_cues = vec![
            make_cue(2.0, 14.0, "Précédemment dans Lupin..."),
            make_cue(15.0, 30.0, "Le collier est à nous."),
            make_cue(35.0, 55.0, "Assane, fais attention."),
        ];
        let fr_res = detect_marks_from_cues(&fr_cues, duration, 0.0, true, None);
        assert!(fr_res.marks.recap.is_some());
        assert_eq!(fr_res.recap_origin, Some(MarkOrigin::SubtitleRecap));

        // German recap
        let de_cues = vec![
            make_cue(4.0, 16.0, "Was bisher geschah..."),
            make_cue(17.0, 33.0, "Die Höhle birgt ein Geheimnis."),
            make_cue(38.0, 60.0, "Jonas, wo bist du?"),
        ];
        let de_res = detect_marks_from_cues(&de_cues, duration, 0.0, true, None);
        assert!(de_res.marks.recap.is_some());
        assert_eq!(de_res.recap_origin, Some(MarkOrigin::SubtitleRecap));
    }

    #[test]
    fn test_anime_theme_with_interstitial_credits_and_instrumental_solo() {
        let duration = 1420.0;
        let cues = vec![
            make_cue(5.0, 20.0, "Cold open conversation."),
            // Theme start: verse 1 (85s to 105s)
            make_cue(85.0, 105.0, "♪ Guren no yumiya wo hanate ♪"),
            // Interstitial credit during guitar solo (106s to 110s)
            make_cue(106.0, 110.0, "[Directed by Tetsuro Araki]"),
            // Theme chorus: bridged across the solo and credit (112s to 175s)
            make_cue(112.0, 145.0, "♪ Shingeki no Kyojin theme chorus ♪"),
            make_cue(146.0, 175.0, "♪ Tatakae tatakae kono sekai de ♪"),
            // Scene starts
            make_cue(178.0, 195.0, "Eren, get ready for deployment."),
            make_cue(197.0, 210.0, "Yes, Captain Levi!"),
            make_cue(212.0, 230.0, "Maintain formation."),
        ];

        let res = detect_marks_from_cues(&cues, duration, 0.0, true, None);
        let intro = res
            .marks
            .intro
            .expect("Theme must bridge across interstitial credits and solo");
        let intro_start_s = intro[0] * duration;
        let intro_end_s = intro[1] * duration;

        assert!((intro_start_s - 85.0).abs() <= 3.0);
        assert!((intro_end_s - 175.0).abs() <= 3.0);
        assert_eq!(res.intro_origin, Some(MarkOrigin::SubtitleTheme));
    }

    #[test]
    fn test_overlapping_subtitle_cues_dialogue_merging() {
        let duration = 3000.0;
        // Two people arguing over each other in cold open with overlapping timestamps
        let cues = vec![
            make_cue(10.0, 22.0, "Listen to me right now!"),
            make_cue(12.0, 19.0, "I do not want to hear it!"),
            make_cue(20.0, 32.0, "This is about our future!"),
            make_cue(21.0, 30.0, "Our future was decided yesterday!"),
            make_cue(31.0, 42.0, "Just stop for one second."),
            // Real intro gap: 42.0s to 105.0s (63s gap)
            make_cue(105.0, 120.0, "Fine, what do you want to say?"),
            make_cue(122.0, 135.0, "Let us look at the contracts."),
            make_cue(137.0, 150.0, "I already signed them."),
        ];

        let res = detect_marks_from_cues(&cues, duration, 0.0, true, None);
        let intro = res
            .marks
            .intro
            .expect("Overlapping cues must be merged cleanly");
        let intro_start_s = intro[0] * duration;
        let intro_end_s = intro[1] * duration;

        assert!((intro_start_s - 42.0).abs() <= 3.0);
        assert!((intro_end_s - 105.0).abs() <= 3.0);
        assert_eq!(res.intro_origin, Some(MarkOrigin::SubtitleDialogueGap));
    }

    #[test]
    fn test_pilot_episode_shifted_intro_rejected() {
        let duration = 3600.0; // 1 hour pilot episode
        // Engine guessed an intro at minute 15 (900s to 960s, ratio 0.25..0.2667)
        let prior = SkipMarks {
            intro: Some([0.25, 0.2667]),
            recap: None,
            credits: None,
        };

        // Subtitles show continuous story dialogue from 10s through 950s
        let cues = vec![
            make_cue(10.0, 50.0, "Pilot opening scene."),
            make_cue(890.0, 920.0, "Crucial exposition scene at minute 15."),
            make_cue(925.0, 955.0, "More story dialogue during alleged intro."),
            make_cue(960.0, 1000.0, "Characters making a major life decision."),
        ];

        // Pilot shifted intro is rejected because start_s is at boundary and has 3+ spoken lines
        assert!(is_engine_intro_invalid(&cues, duration, [0.25, 0.2667]));
        let res = detect_marks_from_cues(&cues, duration, 0.0, true, Some(&prior));
        assert!(
            res.marks.intro.is_none(),
            "Shifted pilot intro must not overwrite story"
        );
    }

    #[test]
    fn test_standalone_gap_rejected_if_dialogue_does_not_continue() {
        let duration = 3000.0;
        // Cold open has dialogue, followed by 30s silence, but then only 1 short line followed by silence
        let cues = vec![
            make_cue(10.0, 30.0, "Cold open conversation."),
            // 30s pause: 30s to 60s
            make_cue(60.0, 62.0, "Title card sound."),
            // No sustained dialogue afterwards! Stream goes quiet.
        ];

        let res = detect_marks_from_cues(&cues, duration, 0.0, true, None);
        assert!(
            res.marks.intro.is_none(),
            "Gap without steady dialogue continuation must be rejected"
        );
    }

    #[test]
    fn test_credits_stinger_with_uploader_ad_at_end() {
        let duration = 3000.0;
        let cues = vec![
            make_cue(2650.0, 2750.0, "We won the battle. It is finally over."),
            make_cue(2752.0, 2798.5, "Let us return home."),
            // Credits music and uploader ad near end
            make_cue(2810.0, 2920.0, "♪ [Closing credits theme] ♪"),
            make_cue(
                2950.0,
                2960.0,
                "Subtitles synced by GoldenBeard - downloaded from www.subtitles.example",
            ),
        ];

        let res = detect_marks_from_cues(&cues, duration, 0.0, true, None);
        let credits = res.marks.credits.expect("Credits must be detected");
        let credits_s = credits * duration;
        let expected_credits_s = 2798.5 + CREDITS_POST_DIALOGUE_PAD_SECS;

        assert!((credits_s - expected_credits_s).abs() <= 3.0);
        assert_eq!(res.credits_origin, Some(MarkOrigin::SubtitleCredits));
    }

    fn parse_srt_time(s: &str) -> Option<f64> {
        let s = s.trim();
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 3 {
            return None;
        }
        let hours: f64 = parts[0].parse().ok()?;
        let minutes: f64 = parts[1].parse().ok()?;
        let sec_parts: Vec<&str> = parts[2].split([',', '.']).collect();
        let seconds: f64 = sec_parts[0].parse().ok()?;
        let millis: f64 = if sec_parts.len() > 1 {
            let m_str = sec_parts[1];
            let m_val: f64 = m_str.parse().ok()?;
            m_val / 10f64.powi(m_str.len() as i32)
        } else {
            0.0
        };
        Some(hours * 3600.0 + minutes * 60.0 + seconds + millis)
    }

    fn parse_srt_to_cues(srt: &str) -> Vec<Cue> {
        let mut cues = Vec::new();
        let clean = srt.replace("\r\n", "\n");
        let blocks = clean.split("\n\n");
        for block in blocks {
            let lines: Vec<&str> = block.lines().filter(|l| !l.trim().is_empty()).collect();
            if lines.len() >= 2 {
                let time_line = if lines[0].contains("-->") {
                    lines[0]
                } else {
                    lines[1]
                };
                if let Some((start_str, end_str)) = time_line.split_once("-->")
                    && let (Some(s), Some(e)) = (parse_srt_time(start_str), parse_srt_time(end_str))
                {
                    let text_start_idx = if lines[0].contains("-->") { 1 } else { 2 };
                    let text = lines[text_start_idx..].join(" ");
                    cues.push(Cue { s, e, t: text });
                }
            }
        }
        cues
    }

    // Fixture 11: Real Subtitle File (SRT) — Anime Episode with AniSkip Verified Marks
    // Known marks: OP Intro 00:01:29,200 to 00:02:59,200 (89.2s to 179.2s). ED Credits 00:22:20,000 (1340.0s).
    #[test]
    fn test_real_srt_anime_episode_with_aniskip_marks() {
        let raw_srt = r#"
1
00:00:10,000 --> 00:00:15,500
The emergency alert has been sounded throughout Tokyo-3.

2
00:00:17,200 --> 00:00:22,800
An unidentified flying object is approaching the city perimeter.

3
00:00:25,000 --> 00:00:34,000
All personnel, report to primary battle stations immediately.

4
00:00:36,500 --> 00:00:45,000
Shinji, where are you right now?

5
00:00:48,000 --> 00:00:55,000
I am waiting at the telephone booth near the station.

6
00:00:58,000 --> 00:01:15,000
Stay where you are. Misato is on her way to pick you up.

7
00:01:18,000 --> 00:01:27,500
Understood. I will wait here.

8
00:01:29,200 --> 00:01:45,000
♪ Zankoku na tenshi no you ni, shounen yo shinwa ni nare ♪

9
00:01:46,000 --> 00:02:05,000
♪ Aoi kaze ga ima, mune no doa wo tataitemo ♪

10
00:02:06,000 --> 00:02:12,000
[Directed by Hideaki Anno]

11
00:02:13,000 --> 00:02:35,000
♪ Watashi dake wo tada mitsumete, hohoenderu anata ♪

12
00:02:36,000 --> 00:02:59,200
♪ Zankoku na tenshi no te-ze, madobe kara yagate tobitatsu ♪

13
00:03:02,000 --> 00:03:10,000
Sorry to keep you waiting, Shinji! Jump in the car!

14
00:03:12,000 --> 00:03:20,000
Misato-san! What in the world is happening out there?

15
00:03:22,000 --> 00:03:32,000
It is an Angel. A giant living entity that threatens humanity.

16
00:22:05,000 --> 00:22:15,000
Father... why did you call for me after all these years?

17
00:22:17,000 --> 00:22:19,500
You will pilot the Evangelion. Or else you may leave.

18
00:22:25,000 --> 00:22:55,000
♪ Fly me to the moon, and let me play among the stars ♪
"#;
        let cues = parse_srt_to_cues(raw_srt);
        assert_eq!(cues.len(), 18);
        let duration = 1420.0;

        let res = detect_marks_from_cues(&cues, duration, 0.0, true, None);
        let intro = res
            .marks
            .intro
            .expect("Anime intro from real SRT must be detected");
        let intro_start_s = intro[0] * duration;
        let intro_end_s = intro[1] * duration;

        // Verify within ±3.0 seconds of AniSkip true marks 89.2s and 179.2s
        assert!(
            (intro_start_s - 89.2).abs() <= 3.0,
            "Start diff: {}",
            (intro_start_s - 89.2).abs()
        );
        assert!(
            (intro_end_s - 179.2).abs() <= 3.0,
            "End diff: {}",
            (intro_end_s - 179.2).abs()
        );
        assert_eq!(res.intro_origin, Some(MarkOrigin::SubtitleTheme));

        let credits = res.marks.credits.expect("Credits must be detected");
        let credits_s = credits * duration;
        let expected_credits_s = 22.0 * 60.0 + 19.5 + CREDITS_POST_DIALOGUE_PAD_SECS; // 1339.5 + 1.5 = 1341.0s
        assert!((credits_s - expected_credits_s).abs() <= 3.0);
        assert_eq!(res.credits_origin, Some(MarkOrigin::SubtitleCredits));
    }

    // Fixture 12: Real Subtitle File (SRT) — Western Drama Cold Open with Title Sequence Silence Gap
    // Known marks: Dialogue until 00:03:15,000 (195.0s). Silence gap title card 195.0s to 260.0s (65s).
    // Main story dialogue begins at 00:04:20,000 (260.0s). Credits begin at 00:52:10,000.
    #[test]
    fn test_real_srt_drama_episode_silence_gap() {
        let raw_srt = r#"
1
00:00:15,000 --> 00:00:28,000
Hawkins National Laboratory, November 6th, 1983.

2
00:00:30,000 --> 00:00:45,000
Something got out of the containment sector! Seal the doors!

3
00:00:48,000 --> 00:01:10,000
Run! It is in the elevator shaft!

4
00:01:15,000 --> 00:01:35,000
Meanwhile, in the Wheeler basement, the campaign continues.

5
00:01:38,000 --> 00:01:52,000
Something is coming. Something hungry for blood.

6
00:01:55,000 --> 00:02:15,000
A shadow grows on the wall behind you. It is the Demogorgon!

7
00:02:18,000 --> 00:02:38,000
Will, what do you do? Fireball or protection spell?

8
00:02:40,000 --> 00:02:55,000
I roll for fireball! Fourteen! Did it hit?

9
00:02:58,000 --> 00:03:15,000
The die went off the table. It did not count.

10
00:04:20,000 --> 00:04:40,000
Mike, time to pack up the game, it is past eight o'clock.

11
00:04:42,000 --> 00:05:00,000
Mom, we are in the middle of a crucial campaign!

12
00:05:02,000 --> 00:05:20,000
Ten more minutes, please! We are almost done!

13
00:52:00,000 --> 00:52:10,000
Goodnight boys, ride your bikes safely home.
"#;
        let cues = parse_srt_to_cues(raw_srt);
        let duration = 3300.0; // 55 minutes

        let prior = SkipMarks {
            intro: Some([190.0 / 3300.0, 262.0 / 3300.0]),
            recap: None,
            credits: Some(3130.0 / 3300.0),
        };

        let res = detect_marks_from_cues(&cues, duration, 0.0, true, Some(&prior));
        let intro = res
            .marks
            .intro
            .expect("Title sequence silence gap must be detected");
        let intro_start_s = intro[0] * duration;
        let intro_end_s = intro[1] * duration;

        // Snapped to exact dialogue boundaries: 195.0s to 260.0s
        assert!((intro_start_s - 195.0).abs() <= 3.0);
        assert!((intro_end_s - 260.0).abs() <= 3.0);
        assert_eq!(res.intro_origin, Some(MarkOrigin::SubtitleDialogueGap));

        let credits = res.marks.credits.expect("Credits must be detected");
        let credits_s = credits * duration;
        assert!((credits_s - (3130.0 + CREDITS_POST_DIALOGUE_PAD_SECS)).abs() <= 3.0);
        assert_eq!(res.credits_origin, Some(MarkOrigin::SubtitleCredits));
    }
}
