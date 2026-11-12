//! The player page. The core runs the session; this draws its state and passes on the controls.

use contract::core_msg::{Action, PlayState};
use leptos::html::Div;
use leptos::prelude::*;

use super::words::{fill, in_digits};
use super::{label_text, maybe_text, watch_address};
use crate::host;
use crate::ports::{address, player};
use crate::state::Shared;

const CONTROLS_STAY_MS: i32 = 3000;

/// How far a key moves the film. The phone's `SEEK_MS` (`watch/Watch.kt:43`), which is the only
/// surface in this project that states one.
const SEEK_MS: u64 = 10_000;

/// A position as a clock, in this language's digits.
fn clock_text(ms: u64, digits: &str) -> String {
    let seconds = ms / 1000;
    let (hours, minutes, seconds) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
    let plain = if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    };
    in_digits(&plain, digits)
}

fn leave() {
    host::act(Action::Stop);
    address::back();
}

#[component]
pub fn Watch() -> impl IntoView {
    let shared = expect_context::<Shared>();
    let stage = NodeRef::<Div>::new();
    let silent = RwSignal::new(false);
    let shown = RwSignal::new(true);
    let idle = RwSignal::new(0_u32);
    stage.on_load(move |element| player::attach(&element, shared));
    Effect::new(move |_| {
        shared.clock.track();
        silent.set(player::muted());
    });
    let state = Memo::new(move |_| {
        shared
            .play
            .with(|play| play.as_ref().map(|play| play.state))
    });
    let playing = move || state.get() == Some(PlayState::Playing);
    let flip = move || {
        if playing() {
            host::act(Action::Pause);
        } else {
            host::act(Action::Unpause);
        }
    };
    let wake = move || {
        shown.set(true);
        let count = idle.get_untracked() + 1;
        idle.set(count);
        set_timeout(
            move || {
                if idle.get_untracked() == count && playing() {
                    shown.set(false);
                }
            },
            std::time::Duration::from_millis(CONTROLS_STAY_MS as u64),
        );
    };
    let seek = move |event: leptos::ev::Event| {
        if let Ok(to_ms) = event_target_value(&event).parse() {
            host::act(Action::Seek { to_ms });
        }
    };
    // The keys the phone's player answers (`watch/Watch.kt:69-79`): play and pause, and ten seconds
    // either way. A page that answered a tap and no key at all was a player a keyboard could start
    // and never pause, which is what this was: the whole surface was one `div` with a click
    // handler, reachable by no key and announced to no reader.
    let _keys = window_event_listener(leptos::ev::keydown, move |event| {
        let at = shared.clock.get_untracked().time_ms;
        match event.key().as_str() {
            " " | "Enter" | "MediaPlayPause" | "k" => flip(),
            "ArrowLeft" => host::act(Action::Seek {
                to_ms: at.saturating_sub(SEEK_MS),
            }),
            "ArrowRight" => host::act(Action::Seek {
                to_ms: at.saturating_add(SEEK_MS),
            }),
            "Escape" => leave(),
            _ => return,
        }
        event.prevent_default();
        wake();
    });
    let next = move || {
        shared.play.with(|play| {
            play.as_ref().and_then(|play| {
                let episode = play.next_episode.as_ref()?;
                Some(watch_address(play.title, Some(episode)))
            })
        })
    };
    let position_says = move || {
        let clock = shared.clock.get();
        let digits = shared.digits.get();
        let said = maybe_text(shared, "say.position")?;
        let said = fill(&said, "at", &clock_text(clock.time_ms, &digits));
        Some(fill(
            &said,
            "length",
            &clock_text(clock.duration_ms, &digits),
        ))
    };
    view! {
        <div class="watch" class:bare=move || !shown.get() on:mousemove=move |_| wake()>
            // The picture is the film, and the film is not words. The controls below are what a
            // reader reaches, so the stage is marked rather than left as a silent stop.
            <div class="stage" node_ref=stage aria-hidden="true"></div>
            {move || matches!(state.get(), None | Some(PlayState::Resolving | PlayState::Starting | PlayState::Stalled | PlayState::Switching))
                .then(|| view! {
                    <div class="ring" role="status" aria-label=move || label_text(shared, "say.loading")></div>
                })}
            {move || (state.get() == Some(PlayState::Failed)).then(|| view! {
                <p class="watch-fail" role="alert">{label_text(shared, "error.no_source")}</p>
            })}
            <div class="watch-top">
                <button
                    class="plain"
                    aria-label=move || label_text(shared, "player.back")
                    on:click=move |_| leave()
                >
                    {move || label_text(shared, "player.back")}
                </button>
            </div>
            <div class="watch-bottom">
                // The bar is a slider, and a slider has to say where it is: a reader hears a bare
                // number from a range otherwise, with no idea what of.
                <input type="range" min="0" step="1000"
                    aria-label=move || label_text(shared, "say.position")
                    aria-valuetext=position_says
                    prop:max=move || shared.clock.get().duration_ms.to_string()
                    prop:value=move || shared.clock.get().time_ms.to_string()
                    on:change=seek/>
                <div class="watch-row">
                    <button
                        class="plain"
                        aria-label=move || label_text(shared, if playing() { "say.pause" } else { "common.play" })
                        on:click=move |_| flip()
                    >
                        {move || label_text(shared, if playing() { "say.pause" } else { "common.play" })}
                    </button>
                    <span>{position_says}</span>
                    <span class="grow"></span>
                    {move || silent.get().then(|| view! {
                        // A film that had to start without sound, and the one control that turns it
                        // on. `strings/` has no key for those words; `player.audio` is the nearest
                        // it has, and **`player.unmute` is in the report**.
                        <button class="plain" on:click=move |_| {
                            player::unmute();
                            silent.set(false);
                        }>{label_text(shared, "player.audio")}</button>
                    })}
                    {move || (shared.audio.with(Vec::len) > 1).then(|| view! {
                        <select aria-label=label_text(shared, "player.audio") on:change=move |event| {
                            host::act(Action::SelectAudio { track: event_target_value(&event) });
                        }>
                            {shared.audio.get().into_iter().map(|track| {
                                let name = track.label.or(track.lang).unwrap_or_else(|| track.id.clone());
                                view! { <option value=track.id>{name}</option> }
                            }).collect_view()}
                        </select>
                    })}
                    {move || (!shared.text.with(Vec::is_empty)).then(|| view! {
                        <select aria-label=label_text(shared, "player.subtitles") on:change=move |event| {
                            let track = event_target_value(&event);
                            let track = (!track.is_empty()).then_some(track);
                            host::act(Action::SelectText { track });
                        }>
                            <option value="">{label_text(shared, "player.subtitles_off")}</option>
                            {shared.text.get().into_iter().map(|track| {
                                let name = track.label.or(track.lang).unwrap_or_else(|| track.id.clone());
                                view! { <option value=track.id>{name}</option> }
                            }).collect_view()}
                        </select>
                    })}
                    {move || next().map(|address| view! {
                        <button class="plain" on:click=move |_| host::go(&address)>
                            {label_text(shared, "player.next")}
                        </button>
                    })}
                    // Full screen has no key in `strings/` and no surface with an equivalent: a
                    // television is already full screen and a phone's player fills its window.
                    // **`player.full_screen` is in the report.** Until it exists the control is
                    // drawn with the brand's own mark and named by `say.show_more`, which is the
                    // nearest the strings have to "give this more room".
                    <button
                        class="plain"
                        aria-label=move || label_text(shared, "say.show_more")
                        on:click=move |_| player::fullscreen()
                    >
                        <span aria-hidden="true">"⛶"</span>
                    </button>
                </div>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::{SEEK_MS, clock_text};
    use crate::surfaces::tests_support::{english, numbers, source};

    #[test]
    fn a_position_reads_as_a_clock_in_the_language_s_own_digits() {
        assert_eq!(clock_text(0, "latn"), "0:00");
        assert_eq!(clock_text(65_000, "latn"), "1:05");
        // An hour in, so the hours appear; and the minutes and seconds keep their two places.
        assert_eq!(clock_text(3_605_000, "latn"), "1:00:05");
        assert_eq!(clock_text(7_384_000, "latn"), "2:03:04");
        // Arabic writes its own digits, and the colon stays where it is.
        assert_eq!(clock_text(65_000, "arab"), "١:٠٥");
        assert_eq!(clock_text(3_605_000, "arabext"), "۱:۰۰:۰۵");
    }

    /// A key moves the film as far as it moves it on the phone.
    #[test]
    fn a_key_moves_the_film_the_phone_s_distance() {
        let phone =
            source("apps/android/surfaces/phone/src/main/kotlin/streams/phone/watch/Watch.kt");
        // The phone writes it with an underscore between the thousands, which is part of the
        // number and not the end of it.
        let said = numbers(&["SEEK_MS = ", "L"], &phone, "the phone's seek step")[0];
        assert_eq!(said, SEEK_MS as f64);
    }

    #[test]
    fn every_label_this_player_names_is_in_the_strings_file() {
        let book = english();
        for label in [
            "common.play",
            "error.no_source",
            "player.audio",
            "player.back",
            "player.next",
            "player.subtitles",
            "player.subtitles_off",
            "say.loading",
            "say.pause",
            "say.position",
            "say.show_more",
        ] {
            assert!(book.contains_key(label), "strings/en.json has no {label}");
        }
        // The two the player asks for that no language has yet.
        for missing in ["player.unmute", "player.full_screen"] {
            assert!(
                !book.contains_key(missing),
                "{missing} is in strings/en.json now: wire it and move it to the list above"
            );
        }
    }
}
