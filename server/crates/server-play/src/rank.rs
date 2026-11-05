//! One ranking, here and nowhere else. A finder returns facts with no order. Unknown facts are
//! tried and fail over, so the filter uses only what is known.

use std::collections::BTreeMap;

use contract::client::CapabilityCard;
use contract::finder::FoundSource;
use contract::play::{Attempt, AttemptKind};
use sha2::{Digest, Sha256};

/// Headers a player may be told to send. Anything else from a finder is dropped.
const ALLOWED_HEADERS: [&str; 3] = ["Referer", "Origin", "User-Agent"];

/// The height an adaptive master counts as: above 720, below 1080.
const ADAPTIVE_RANK: u32 = 800;

/// Who is watching what.
#[derive(Clone, Debug, Default)]
pub struct Viewing {
    pub language: String,
    pub original_language: String,
    /// Kids tiers get verified attempts only and never an embed.
    pub kids: bool,
    /// The operator allows embeds. Off by default and in store builds.
    pub embeds: bool,
}

/// How well a source's audio suits the viewer. A multi audio source, or the viewer's language,
/// is 4; the film's own language is 5; untagged is 2; anything else is 1. Whether a finder
/// called a source original no longer decides anything (defect in C14).
#[must_use]
pub fn audio_score(audio: &[String], viewing: &Viewing) -> u8 {
    let holds = |wanted: &str| {
        !wanted.is_empty() && audio.iter().any(|tag| tag.eq_ignore_ascii_case(wanted))
    };
    if holds(&viewing.original_language) {
        5
    } else if holds("multi") || holds(&viewing.language) {
        4
    } else if audio.is_empty() {
        2
    } else {
        1
    }
}

const fn kind_rank(kind: AttemptKind) -> u8 {
    match kind {
        AttemptKind::Hls => 2,
        AttemptKind::File => 1,
        _ => 0,
    }
}

/// The container a known address suffix shows.
fn suffix(url: &str) -> Option<&'static str> {
    let path = url
        .split(['?', '#'])
        .next()
        .unwrap_or(url)
        .to_ascii_lowercase();
    if path.ends_with(".mkv") {
        Some("mkv")
    } else if path.ends_with(".webm") {
        Some("webm")
    } else {
        None
    }
}

/// True when this source may be offered to this viewer on this device.
#[must_use]
pub fn keep_for(source: &FoundSource, card: Option<&CapabilityCard>, viewing: &Viewing) -> bool {
    if source.url.trim().is_empty() || !source.url.starts_with("https://") {
        return false;
    }
    if viewing.kids && (!source.verified || source.kind == AttemptKind::Embed) {
        return false;
    }
    if source.kind == AttemptKind::Embed && !viewing.embeds {
        return false;
    }
    let Some(card) = card else {
        return source.kind != AttemptKind::Embed;
    };
    if !card.attempt_kinds.contains(&source.kind) {
        return false;
    }
    let container = source.container.as_deref().or_else(|| suffix(&source.url));
    container.is_none_or(|name| {
        card.containers
            .iter()
            .any(|known| known.eq_ignore_ascii_case(name))
    })
}

/// The allowed headers of a source, with canonical names.
#[must_use]
pub fn usable_headers(headers: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    headers
        .iter()
        .filter_map(|(name, value)| {
            let allowed = ALLOWED_HEADERS
                .iter()
                .find(|known| known.eq_ignore_ascii_case(name))?;
            Some(((*allowed).to_owned(), value.clone()))
        })
        .collect()
}

/// An opaque group, so the core can skip a dead provider and no provider name reaches a device.
fn opaque_group(group: &str, salt: &str) -> String {
    let digest = Sha256::digest(format!("{salt}|{group}"));
    digest
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Ranked attempts from sources, best first. The sort is stable, so the tie order is the finder's
/// data order. Returns the attempts and how many the card removed.
#[must_use]
pub fn rank(
    sources: Vec<FoundSource>,
    card: Option<&CapabilityCard>,
    viewing: &Viewing,
    salt: &str,
) -> (Vec<Attempt>, u32) {
    let total = sources.len();
    let mut kept: Vec<FoundSource> = sources
        .into_iter()
        .filter(|s| keep_for(s, card, viewing))
        .collect();
    let dropped = u32::try_from(total - kept.len()).unwrap_or(u32::MAX);
    let height = |source: &FoundSource| source.height.unwrap_or(ADAPTIVE_RANK);
    kept.sort_by_key(|source| {
        (
            std::cmp::Reverse(kind_rank(source.kind)),
            std::cmp::Reverse(audio_score(&source.audio, viewing)),
            std::cmp::Reverse(height(source)),
        )
    });
    let attempts = kept
        .into_iter()
        .enumerate()
        .map(|(n, source)| Attempt {
            id: format!("a{n}"),
            headers: usable_headers(&source.headers),
            group: opaque_group(&source.group, salt),
            url: source.url,
            kind: source.kind,
            verified: source.verified,
            container: source.container,
            codecs: Vec::new(),
            height: source.height,
            audio: source.audio,
            expires_at_ms: None,
        })
        .collect();
    (attempts, dropped)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(url: &str, kind: AttemptKind, height: Option<u32>, audio: &[&str]) -> FoundSource {
        FoundSource {
            url: url.to_owned(),
            kind,
            group: String::from("one"),
            headers: BTreeMap::new(),
            verified: true,
            container: None,
            height,
            audio: audio.iter().map(ToString::to_string).collect(),
        }
    }

    fn viewing() -> Viewing {
        Viewing {
            language: String::from("en"),
            original_language: String::from("fr"),
            ..Viewing::default()
        }
    }

    #[test]
    fn c14_c81_audio_prefers_the_original_then_multi_or_viewer_then_untagged() {
        let v = viewing();
        let score = |tags: &[&str]| {
            let tags: Vec<String> = tags.iter().map(ToString::to_string).collect();
            audio_score(&tags, &v)
        };
        assert_eq!(score(&["FR"]), 5);
        assert_eq!(score(&["en"]), 4);
        assert_eq!(score(&["multi"]), 4);
        assert_eq!(score(&[]), 2);
        assert_eq!(score(&["de"]), 1);
    }

    #[test]
    fn h44_u1_direct_before_embed_then_audio_then_height_and_ties_keep_finder_order() {
        let v = Viewing {
            embeds: true,
            ..viewing()
        };
        let sources = vec![
            source(
                "https://a.example/e",
                AttemptKind::Embed,
                Some(2160),
                &["fr"],
            ),
            source(
                "https://b.example/m.mp4",
                AttemptKind::File,
                Some(1080),
                &["fr"],
            ),
            source(
                "https://c.example/1.m3u8",
                AttemptKind::Hls,
                Some(720),
                &["de"],
            ),
            source(
                "https://d.example/2.m3u8",
                AttemptKind::Hls,
                Some(720),
                &["fr"],
            ),
            source("https://e.example/3.m3u8", AttemptKind::Hls, None, &["fr"]),
            source("https://f.example/4.m3u8", AttemptKind::Hls, None, &["fr"]),
        ];
        let (attempts, dropped) = rank(sources, None, &v, "salt");
        let order: Vec<&str> = attempts.iter().map(|a| a.url.as_str()).collect();
        assert_eq!(dropped, 1, "no card means no embed");
        assert_eq!(
            order[0], "https://e.example/3.m3u8",
            "the adaptive master leads"
        );
        assert_eq!(
            order[1], "https://f.example/4.m3u8",
            "a tie keeps the data order"
        );
        assert_eq!(order[2], "https://d.example/2.m3u8");
        assert_eq!(order[3], "https://c.example/1.m3u8");
        assert_eq!(order[4], "https://b.example/m.mp4");
    }

    #[test]
    fn k7_k8_kids_get_verified_attempts_and_no_embed_whatever_the_card_says() {
        let card: CapabilityCard = serde_json::from_str(include_str!(
            "../../../../platform/contract/fixtures/2026-11-05/valid/CapabilityCard.web.json"
        ))
        .unwrap();
        let v = Viewing {
            kids: true,
            embeds: true,
            ..viewing()
        };
        let mut unverified = source("https://a.example/1.m3u8", AttemptKind::Hls, None, &[]);
        unverified.verified = false;
        assert!(!keep_for(&unverified, Some(&card), &v));
        let embed = source("https://a.example/e", AttemptKind::Embed, None, &[]);
        assert!(!keep_for(&embed, Some(&card), &v));
        assert!(keep_for(
            &source("https://a.example/2.m3u8", AttemptKind::Hls, None, &[]),
            Some(&card),
            &v
        ));
    }

    #[test]
    fn s30_h41_an_embed_comes_only_with_the_operator_and_the_card() {
        let embed = source("https://a.example/e", AttemptKind::Embed, None, &[]);
        assert!(
            !keep_for(
                &embed,
                None,
                &Viewing {
                    embeds: true,
                    ..viewing()
                }
            ),
            "no card"
        );
        let mut card: CapabilityCard = serde_json::from_str(include_str!(
            "../../../../platform/contract/fixtures/2026-11-05/valid/CapabilityCard.web.json"
        ))
        .unwrap();
        card.attempt_kinds = vec![AttemptKind::Hls, AttemptKind::Embed];
        assert!(
            !keep_for(&embed, Some(&card), &viewing()),
            "the operator left it off"
        );
        assert!(keep_for(
            &embed,
            Some(&card),
            &Viewing {
                embeds: true,
                ..viewing()
            }
        ));
    }

    #[test]
    fn h5_the_filter_uses_what_is_known_and_lets_the_unknown_try() {
        let mut card: CapabilityCard = serde_json::from_str(include_str!(
            "../../../../platform/contract/fixtures/2026-11-05/valid/CapabilityCard.web.json"
        ))
        .unwrap();
        card.attempt_kinds = vec![AttemptKind::Hls, AttemptKind::File];
        card.containers = vec![String::from("mp4")];
        let v = viewing();
        assert!(!keep_for(
            &source("https://a.example/x.MKV?t=1", AttemptKind::File, None, &[]),
            Some(&card),
            &v
        ));
        assert!(keep_for(
            &source("https://a.example/stream", AttemptKind::File, None, &[]),
            Some(&card),
            &v
        ));
    }

    #[test]
    fn f8_an_address_that_is_not_https_never_reaches_a_device() {
        let v = viewing();
        assert!(!keep_for(
            &source("http://a.example/1.m3u8", AttemptKind::Hls, None, &[]),
            None,
            &v
        ));
        assert!(!keep_for(
            &source("  ", AttemptKind::Hls, None, &[]),
            None,
            &v
        ));
    }

    #[test]
    fn t4_only_three_headers_pass_and_h45_h81_no_provider_name_leaves() {
        let mut one = source("https://a.example/1.m3u8", AttemptKind::Hls, None, &[]);
        one.group = String::from("provider-name");
        one.headers
            .insert(String::from("referer"), String::from("https://b.example"));
        one.headers
            .insert(String::from("Authorization"), String::from("nope"));
        let (attempts, _) = rank(vec![one], None, &viewing(), "salt");
        assert_eq!(attempts[0].headers.len(), 1);
        assert!(attempts[0].headers.contains_key("Referer"));
        assert!(!attempts[0].group.contains("provider"));
    }
}
