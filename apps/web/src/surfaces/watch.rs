//! The player page. The core runs the session; this draws its state and passes on the controls.

use contract::core_msg::{Action, PlayState};
use leptos::html::Div;
use leptos::prelude::*;

use crate::host;
use crate::ports::{address, player};
use crate::state::Shared;

const CONTROLS_STAY_MS: i32 = 3000;

fn clock_text(ms: u64) -> String {
    let seconds = ms / 1000;
    let (hours, minutes, seconds) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
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
    let toggle = move |_| {
        if playing() {
            host::act(Action::Pause);
        } else {
            host::act(Action::Unpause);
        }
    };
    let waking = move |_| {
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
    let next = move || {
        shared.play.with(|play| {
            play.as_ref().and_then(|play| {
                let episode = play.next_episode.as_ref()?;
                let title = &play.title;
                Some(format!(
                    "/watch/{}/{}/{}/{}",
                    title.kind().word(),
                    title.id(),
                    episode.season,
                    episode.episode
                ))
            })
        })
    };
    view! {
        <div class="watch" class:bare=move || !shown.get() on:mousemove=waking on:click=toggle>
            <div class="stage" node_ref=stage></div>
            {move || matches!(state.get(), None | Some(PlayState::Resolving | PlayState::Starting | PlayState::Stalled | PlayState::Switching))
                .then(|| view! { <div class="ring"></div> })}
            {move || (state.get() == Some(PlayState::Failed)).then(|| view! {
                <p class="watch-fail">"This title cannot play right now."</p>
            })}
            <div class="watch-top" on:click=|event| event.stop_propagation()>
                <button class="plain" on:click=move |_| leave()>"Back"</button>
            </div>
            <div class="watch-bottom" on:click=|event| event.stop_propagation()>
                <input type="range" min="0" step="1000"
                    prop:max=move || shared.clock.get().duration_ms.to_string()
                    prop:value=move || shared.clock.get().time_ms.to_string()
                    on:change=seek/>
                <div class="watch-row">
                    <button class="plain" on:click=toggle>{move || if playing() { "Pause" } else { "Play" }}</button>
                    <span>{move || {
                        let clock = shared.clock.get();
                        format!("{} / {}", clock_text(clock.time_ms), clock_text(clock.duration_ms))
                    }}</span>
                    <span class="grow"></span>
                    {move || silent.get().then(|| view! {
                        <button class="plain" on:click=move |_| {
                            player::unmute();
                            silent.set(false);
                        }>"Turn on sound"</button>
                    })}
                    {move || (shared.audio.with(Vec::len) > 1).then(|| view! {
                        <select aria-label="Audio" on:change=move |event| {
                            host::act(Action::SelectAudio { track: event_target_value(&event) });
                        }>
                            {shared.audio.get().into_iter().map(|track| {
                                let name = track.label.or(track.lang).unwrap_or_else(|| track.id.clone());
                                view! { <option value=track.id>{name}</option> }
                            }).collect_view()}
                        </select>
                    })}
                    {move || (!shared.text.with(Vec::is_empty)).then(|| view! {
                        <select aria-label="Subtitles" on:change=move |event| {
                            let track = event_target_value(&event);
                            let track = (!track.is_empty()).then_some(track);
                            host::act(Action::SelectText { track });
                        }>
                            <option value="">"Subtitles off"</option>
                            {shared.text.get().into_iter().map(|track| {
                                let name = track.label.or(track.lang).unwrap_or_else(|| track.id.clone());
                                view! { <option value=track.id>{name}</option> }
                            }).collect_view()}
                        </select>
                    })}
                    {move || next().map(|address| view! {
                        <button class="plain" on:click=move |_| host::go(&address)>"Next episode"</button>
                    })}
                    <button class="plain" on:click=move |_| player::fullscreen()>"Full screen"</button>
                </div>
            </div>
        </div>
    }
}
