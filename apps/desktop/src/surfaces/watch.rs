//! The player screen: transport controls, age card, and `TimeWindow` skip marks.
//!
//! The core holds the play session and hands its active [`PlayView`] to the surface. As the player
//! advances, the shell reports the current position in milliseconds (`Surface::at`), and this
//! screen compares that position against `PlayView.windows` (`WindowKind::Intro`,
//! `WindowKind::Recap`, `WindowKind::Credits`, and `WindowKind::AgeCard`), matching the television
//! and web watch surfaces (`Moments.kt:33-34`, `Watch.kt:257-260,373-376,488-494`,
//! `Watch.swift:209-211,277,400-403`).

use contract::address;
use contract::core_msg::{Action, PlayState, PlayView};
use contract::play::{TimeWindow, WindowKind};
use contract::{EpisodeRef, TitleKey};

use super::door;
use super::scene::{Corner, Ink, Part, Rect, Scene};
use super::{Ask, Surface, sizes};

/// What a skip control asks the shell to do when pressed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum SkipTarget {
    /// Jump to the end of an intro or recap window (or its `seek_to_ms` target).
    Seek(u64),
    /// Open the next episode during closing credits.
    Next(String),
}

impl SkipTarget {
    #[must_use]
    pub(super) fn ask(&self) -> Ask {
        match self {
            Self::Seek(to_ms) => Ask::Act(Action::Seek { to_ms: *to_ms }),
            Self::Next(path) => Ask::Go(path.clone()),
        }
    }
}

/// The skip control offered at a moment of the film: its `strings/` key and target action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SkipOffer {
    pub label: &'static str,
    pub target: SkipTarget,
}

/// The first window of one of `kinds` that covers `at_ms`. Half-open (`start_ms..end_ms`),
/// matching `Moments.kt:33-34` and `Moments.swift:8-10`.
#[must_use]
pub(super) fn window_now<'a>(
    windows: &'a [TimeWindow],
    at_ms: u64,
    kinds: &[WindowKind],
) -> Option<&'a TimeWindow> {
    windows.iter().find(|window| {
        kinds.contains(&window.kind) && (window.start_ms..window.end_ms).contains(&at_ms)
    })
}

/// The watch address for an episode of a series, built from `contract::address::WATCH`.
#[must_use]
pub(super) fn watch_address(title: TitleKey, episode: &EpisodeRef) -> String {
    format!(
        "{}{}/{}/{}",
        address::WATCH,
        title.to_string().replace(':', "/"),
        episode.season,
        episode.episode
    )
}

/// The active skip offer for `play` at `at_ms`. Intro and recap windows seek to
/// `seek_to_ms.unwrap_or(end_ms)`; credits windows offer the next episode when one exists.
#[must_use]
pub(super) fn skip_offer(play: &PlayView, at_ms: u64) -> Option<SkipOffer> {
    if play.state == PlayState::Failed {
        return None;
    }
    if let Some(part) = window_now(
        &play.windows,
        at_ms,
        &[WindowKind::Intro, WindowKind::Recap],
    ) {
        let label = match part.kind {
            WindowKind::Intro => "player.skip_intro",
            _ => "player.skip_recap",
        };
        return Some(SkipOffer {
            label,
            target: SkipTarget::Seek(part.seek_to_ms.unwrap_or(part.end_ms)),
        });
    }
    let episode = play.next_episode.as_ref()?;
    window_now(&play.windows, at_ms, &[WindowKind::Credits]).map(|_| SkipOffer {
        label: "player.next",
        target: SkipTarget::Next(watch_address(play.title, episode)),
    })
}

/// How wide a control pill in a corner of the player may be without colliding with its neighbour
/// across the window.
fn control_wide(wide: f64) -> f64 {
    ((sizes::content(wide) - sizes::GUTTER) / 2.0).clamp(0.0, sizes::READING / 2.0)
}

/// The age rating card shown once at the start of play (`WindowKind::AgeCard`).
fn age_card(scene: &mut Scene, wide: f64, label: &str) {
    let margin = sizes::margin(wide);
    let top = margin + sizes::pill(wide) + sizes::HERO_GAP;
    let (size, line_high) = sizes::row_title(wide);
    let bar_wide = 6.0;
    let gap = sizes::GUTTER * 2.0;
    let text_wide = (sizes::content(wide) - bar_wide - gap).max(0.0);
    scene.add(Part::plate(
        Rect::new(margin, top, bar_wide, line_high),
        Ink::Accent,
        Corner::Square,
    ));
    scene.add(
        Part::words(
            Rect::new(margin + bar_wide + gap, top, text_wide, line_high),
            Ink::Text,
            label,
            size,
            500,
        )
        .displayed(),
    );
}

/// The "Are you still watching?" card shown when several episodes autoplayed untouched.
fn still_asking(surface: &Surface, scene: &mut Scene, play: &PlayView) {
    let wide = scene.window.wide;
    let (_, head_high) = sizes::row_title(wide);
    let pill = sizes::pill(wide);
    let inset = sizes::hero_inset(wide);
    let high = head_high + 2.0 * pill + 2.0 * sizes::GUTTER + 2.0 * inset;
    let card = door::card(scene, high);
    let mut at = Rect::new(
        card.x + inset,
        card.y + inset,
        (card.wide - 2.0 * inset).max(0.0),
        0.0,
    );
    at.y += door::heading(scene, at, &surface.text("player.still")) + sizes::GUTTER;
    let yes_ask = match &play.next_episode {
        Some(episode) => Ask::Go(watch_address(play.title, episode)),
        None => Ask::Act(Action::Unpause),
    };
    at.y += door::button(
        scene,
        at,
        Ink::Accent,
        Ink::AccentText,
        &surface.text("player.still.yes"),
        yes_ask,
    ) + sizes::GUTTER;
    door::button(
        scene,
        at,
        Ink::Surface,
        Ink::Text,
        &surface.text("player.still.no"),
        Ask::Act(Action::Stop),
    );
}

pub(super) fn draw(surface: &Surface, scene: &mut Scene) {
    let wide = scene.window.wide;
    let high = scene.window.high;
    let margin = sizes::margin(wide);
    let pill = sizes::pill(wide);
    let button_wide = control_wide(wide);

    // Top-left back control.
    door::button(
        scene,
        Rect::new(margin, margin, button_wide, pill),
        Ink::Surface,
        Ink::Text,
        &surface.text("player.back"),
        Ask::Act(Action::Stop),
    );

    let failed = surface
        .play
        .as_ref()
        .is_some_and(|play| play.state == PlayState::Failed);
    if failed {
        let (_, line_high) = sizes::body_type(wide);
        let inset = sizes::hero_inset(wide);
        let card = door::card(scene, line_high + 2.0 * inset);
        let said = surface
            .notice
            .as_ref()
            .filter(|notice| !notice.text.is_empty())
            .map(|notice| notice.text.clone())
            .or_else(|| surface.maybe_text("error.no_source"))
            .unwrap_or_default();
        door::line(
            scene,
            Rect::new(
                card.x + inset,
                card.y + inset,
                (card.wide - 2.0 * inset).max(0.0),
                line_high,
            ),
            Ink::Text,
            &said,
        );
        return;
    }

    let bottom_y = (high - margin - pill).max(margin + pill + sizes::GUTTER);
    let playing = surface
        .play
        .as_ref()
        .is_some_and(|play| play.state == PlayState::Playing);
    let transport_word = surface.text(if playing { "say.pause" } else { "common.play" });
    let transport_ask = Ask::Act(if playing {
        Action::Pause
    } else {
        Action::Unpause
    });
    door::button(
        scene,
        Rect::new(margin, bottom_y, button_wide, pill),
        Ink::Surface,
        Ink::Text,
        &transport_word,
        transport_ask,
    );

    let Some(play) = &surface.play else {
        return;
    };

    if let Some(age) = window_now(&play.windows, surface.position_ms, &[WindowKind::AgeCard])
        .and_then(|window| window.text.as_deref())
        .filter(|text| !text.is_empty())
    {
        age_card(scene, wide, age);
    }

    if let Some(offer) = skip_offer(play, surface.position_ms) {
        let words = surface.text(offer.label);
        if !words.is_empty() {
            let skip_x = (wide - margin - button_wide).max(margin);
            door::button(
                scene,
                Rect::new(skip_x, bottom_y, button_wide, pill),
                Ink::Accent,
                Ink::AccentText,
                &words,
                offer.target.ask(),
            );
        }
    }

    if play.still_asking {
        still_asking(surface, scene, play);
    }
}

#[cfg(test)]
mod tests {
    use contract::core_msg::{Action, PlayState, PlayView, Target, View};
    use contract::play::{TimeWindow, WindowKind};
    use contract::{EpisodeRef, TitleKey, TitleKind};

    use super::super::reading::english;
    use super::super::tests::route;
    use super::super::{Ask, Face, Surface, Window};
    use super::{SkipOffer, SkipTarget, skip_offer, window_now};
    use crate::host::Turn;

    const WINDOWS: [(f64, f64); 5] = [
        (390.0, 844.0),
        (768.0, 899.0),
        (1280.0, 899.0),
        (1440.0, 900.0),
        (1920.0, 899.0),
    ];

    fn sample_play() -> PlayView {
        PlayView {
            state: PlayState::Playing,
            title: TitleKey::new(TitleKind::Tv, 42).expect("valid title key"),
            episode: Some(EpisodeRef {
                season: 1,
                episode: 2,
            }),
            attempt: Some("a1".to_owned()),
            windows: vec![
                TimeWindow {
                    kind: WindowKind::AgeCard,
                    start_ms: 0,
                    end_ms: 4_000,
                    seek_to_ms: None,
                    text: Some("16+".to_owned()),
                },
                TimeWindow {
                    kind: WindowKind::Recap,
                    start_ms: 5_000,
                    end_ms: 25_000,
                    seek_to_ms: None,
                    text: None,
                },
                TimeWindow {
                    kind: WindowKind::Intro,
                    start_ms: 30_000,
                    end_ms: 90_000,
                    seek_to_ms: Some(92_000),
                    text: None,
                },
                TimeWindow {
                    kind: WindowKind::Credits,
                    start_ms: 1_200_000,
                    end_ms: 1_260_000,
                    seek_to_ms: None,
                    text: None,
                },
            ],
            next_episode: Some(EpisodeRef {
                season: 1,
                episode: 3,
            }),
            still_asking: false,
        }
    }

    fn watching(play: PlayView) -> Surface {
        let mut surface = Surface::new();
        route(&mut surface, Target::Watch);
        let mut turn = Turn::default();
        turn.views.push(View {
            play: Some(play),
            ..View::default()
        });
        surface.apply(&turn);
        assert_eq!(surface.face(), Face::Watch);
        surface
    }

    #[test]
    fn window_now_matches_half_open_bounds_and_requested_kinds() {
        let play = sample_play();
        assert_eq!(
            window_now(
                &play.windows,
                4_999,
                &[WindowKind::Recap, WindowKind::Intro]
            ),
            None
        );
        assert_eq!(
            window_now(
                &play.windows,
                5_000,
                &[WindowKind::Recap, WindowKind::Intro]
            )
            .map(|window| window.kind),
            Some(WindowKind::Recap)
        );
        // End boundary is exclusive (`start_ms..end_ms`).
        assert_eq!(
            window_now(
                &play.windows,
                25_000,
                &[WindowKind::Recap, WindowKind::Intro]
            ),
            None
        );
        assert_eq!(
            window_now(&play.windows, 30_000, &[WindowKind::Intro]).map(|window| window.kind),
            Some(WindowKind::Intro)
        );
    }

    #[test]
    fn skip_window_visibility_and_seek_behavior_across_recap_intro_and_credits() {
        let book = english();
        let window = Window::new(1440.0, 900.0);
        let mut surface = watching(sample_play());

        // Before recap starts (and inside age card): age card is drawn, no skip pill is visible.
        surface.at(2_000);
        assert_eq!(
            surface
                .window_now(&[WindowKind::AgeCard])
                .and_then(|w| w.text.as_deref()),
            Some("16+")
        );
        assert_eq!(
            surface.window_now(&[WindowKind::Intro, WindowKind::Recap]),
            None
        );
        let scene = surface.draw(window);
        assert!(scene.says("16+"));
        assert!(!scene.says(&book["player.skip_recap"]));
        assert!(!scene.says(&book["player.skip_intro"]));
        assert!(!scene.says(&book["player.next"]));

        // Inside recap (`5_000..25_000`, `seek_to_ms: None`): shows "Skip Recap" and seeks to `end_ms`.
        surface.at(5_000);
        let scene = surface.draw(window);
        assert!(scene.says(&book["player.skip_recap"]));
        assert!(!scene.says(&book["player.skip_intro"]));
        let asks = surface.press(window, &book["player.skip_recap"]);
        assert_eq!(asks, vec![Ask::Act(Action::Seek { to_ms: 25_000 })]);

        // At exact end of recap (`25_000`): skip pill disappears.
        surface.at(25_000);
        let scene = surface.draw(window);
        assert!(!scene.says(&book["player.skip_recap"]));
        assert!(!scene.says(&book["player.skip_intro"]));

        // Inside intro (`30_000..90_000`, `seek_to_ms: Some(92_000)`): shows "Skip Intro" and seeks to `seek_to_ms`.
        for (wide, high) in WINDOWS {
            let win = Window::new(wide, high);
            surface.at(45_000);
            let scene = surface.draw(win);
            assert!(scene.says(&book["player.skip_intro"]), "at {wide}");
            assert!(scene.outside().is_empty(), "a part is outside at {wide}");
            let asks = surface.press(win, &book["player.skip_intro"]);
            assert_eq!(
                asks,
                vec![Ask::Act(Action::Seek { to_ms: 92_000 })],
                "at {wide}"
            );
        }

        // Inside credits with `next_episode`: shows "Next Episode" and navigates to `/watch/tv/42/1/3`.
        surface.at(1_210_000);
        let scene = surface.draw(window);
        assert!(scene.says(&book["player.next"]));
        let asks = surface.press(window, &book["player.next"]);
        assert_eq!(asks, vec![Ask::Go("/watch/tv/42/1/3".to_owned())]);
    }

    #[test]
    fn credits_without_next_episode_and_failed_play_hide_skip_controls() {
        let book = english();
        let window = Window::new(1440.0, 900.0);

        let mut finale = sample_play();
        finale.next_episode = None;
        assert_eq!(skip_offer(&finale, 1_210_000), None);
        let mut surface = watching(finale);
        surface.at(1_210_000);
        assert!(!surface.draw(window).says(&book["player.next"]));

        let mut failed = sample_play();
        failed.state = PlayState::Failed;
        assert_eq!(
            skip_offer(&failed, 45_000),
            None,
            "failed play must not offer skip"
        );
        let mut failed_surface = watching(failed);
        failed_surface.at(45_000);
        let scene = failed_surface.draw(window);
        assert!(scene.says(&book["error.no_source"]));
        assert!(!scene.says(&book["player.skip_intro"]));
        assert_eq!(
            SkipOffer {
                label: "player.skip_intro",
                target: SkipTarget::Seek(92_000),
            }
            .target
            .ask(),
            Ask::Act(Action::Seek { to_ms: 92_000 })
        );
    }

    #[test]
    fn every_label_this_player_names_is_in_the_strings_file() {
        let book = english();
        for label in [
            "common.play",
            "error.no_source",
            "player.back",
            "player.next",
            "player.skip_intro",
            "player.skip_recap",
            "player.still",
            "player.still.no",
            "player.still.yes",
            "say.pause",
        ] {
            assert!(book.contains_key(label), "strings/en.json has no {label}");
        }
    }
}
