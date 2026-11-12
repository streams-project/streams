//! One title: its facts, seasons, and what is like it.
//!
//! A panel over the page it was opened from, which is what the reference's web page draws and what
//! this surface's own address grammar says (`contract::address::DETAILS_QUERY`). The phone draws
//! the same facts as a page of its own
//! (`apps/android/surfaces/phone/src/main/kotlin/streams/phone/title/Title.kt`), and what is taken
//! from it here is the facts and their words, not its layout.

use contract::TitleKey;
use contract::command::Command;
use contract::core_msg::{Action, HttpMethod};
use contract::screen::{AgeTone, ImageRole, Season, Tile, TitleDetail};
use leptos::prelude::*;

use super::words::{counted, filled, number};
use super::{image, label_text, maybe_text, title_address, watch_address};
use crate::host;
use crate::ports::address;
use crate::state::Shared;

fn close() {
    address::back();
}

/// Where play starts. `Title.kt:83-104`.
///
/// A film plays. A series plays where this profile left off, and a series never started plays the
/// first episode of its first season, which needs the season read first: an episode has no id of
/// its own, so there is nothing to open until the server names one.
fn play_from(detail: &TitleDetail) -> Option<String> {
    let title = detail.title;
    if title.kind() != contract::TitleKind::Tv {
        return Some(watch_address(title, None));
    }
    if let Some(episode) = detail
        .resume
        .as_ref()
        .and_then(|resume| resume.episode.as_ref())
    {
        return Some(watch_address(title, Some(episode)));
    }
    None
}

/// Play a series nothing is known about yet: ask for its first season and open its first episode.
/// `Title.kt:95-103`.
fn play_first(title: TitleKey, season: u32) {
    let path = format!(
        "/v1/titles/{}/{}/seasons/{season}",
        title.kind().word(),
        title.id()
    );
    host::ask(HttpMethod::Get, &path, None, move |status, text| {
        let first = (200..300)
            .contains(&status)
            .then(|| serde_json::from_str::<Season>(&text).ok())
            .flatten()
            .and_then(|read| read.episodes.first().map(|episode| episode.number))
            .unwrap_or(1);
        let episode = contract::EpisodeRef {
            season,
            episode: first,
        };
        host::go(&watch_address(title, Some(&episode)));
    });
}

/// The words for a length: hours and minutes, or minutes alone. `Title.kt:270-277`, and the TV's
/// `parts/Facts.kt:27-32` for the same two keys.
fn length_words(shared: Shared, minutes: u32) -> Option<String> {
    if minutes >= 60 {
        let text = maybe_text(shared, "title.length.hours")?;
        let text = super::words::fill(&text, "h", &number(shared, minutes / 60));
        Some(super::words::fill(
            &text,
            "m",
            &number(shared, minutes % 60),
        ))
    } else {
        filled(
            shared,
            "title.length.minutes",
            "m",
            &number(shared, minutes),
        )
    }
}

/// The line of facts under a title's name: the year, then how long it is or how many seasons it
/// has. `Title.kt:238-299`, less the "HD" mark, which is an English literal on that surface too and
/// has no key.
fn facts(shared: Shared, detail: &TitleDetail) -> Vec<String> {
    let mut parts = Vec::new();
    if let Some(year) = detail.year {
        parts.push(number(shared, year));
    }
    if !detail.seasons.is_empty() {
        let seasons = counted(shared, "title.season", detail.seasons.len() as u32);
        if !seasons.is_empty() {
            parts.push(seasons);
        }
    } else if let Some(minutes) = detail.runtime_min
        && let Some(words) = length_words(shared, minutes)
    {
        parts.push(words);
    }
    parts
}

/// The class an age mark wears, from the tone the server picked. Each surface paints its own; this
/// one hands the tone to the style sheet rather than deciding a colour in Rust.
fn age_tone(tone: AgeTone) -> &'static str {
    match tone {
        AgeTone::Green => "age-green",
        AgeTone::Yellow => "age-yellow",
        AgeTone::Orange => "age-orange",
        AgeTone::Pink => "age-pink",
        AgeTone::Red => "age-red",
        AgeTone::DarkRed => "age-dark-red",
        AgeTone::Blue => "age-blue",
        _ => "age-grey",
    }
}

#[component]
fn Episodes(title: TitleKey, seasons: Vec<u32>) -> impl IntoView {
    let shared = expect_context::<Shared>();
    let chosen = RwSignal::new(seasons.first().copied().unwrap_or(1));
    let season = RwSignal::new(None::<Season>);
    let (kind, id) = (title.kind().word(), title.id());
    Effect::new(move |_| {
        let number = chosen.get();
        let path = format!("/v1/titles/{kind}/{id}/seasons/{number}");
        host::ask(HttpMethod::Get, &path, None, move |status, text| {
            if (200..300).contains(&status) {
                season.set(serde_json::from_str(&text).ok());
            }
        });
    });
    let many = seasons.len() > 1;
    view! {
        <section class="episodes">
            <h2>{move || label_text(shared, "title.episodes_more")}</h2>
            {many.then(|| view! {
                <select
                    aria-label=move || label_text(shared, "say.season_pick")
                    on:change=move |event| {
                        if let Ok(number) = event_target_value(&event).parse() {
                            chosen.set(number);
                        }
                    }
                >
                    {seasons.iter().map(|place| {
                        let place = *place;
                        view! {
                            <option value=place.to_string()>
                                {move || filled(shared, "title.season.number", "n", &number(shared, place))
                                    .unwrap_or_else(|| place.to_string())}
                            </option>
                        }
                    }).collect_view()}
                </select>
            })}
            {move || season.get().map(|season| {
                let place = season.number;
                season.episodes.into_iter().map(|episode| {
                    let at = contract::EpisodeRef { season: place, episode: episode.number };
                    let address = watch_address(title, Some(&at));
                    let says = filled(shared, "title.episode", "n", &number(shared, episode.number))
                        .map_or_else(|| episode.name.clone(), |words| format!("{words}: {}", episode.name));
                    view! {
                        <button class="episode" aria-label=says on:click=move |_| host::go(&address)>
                            // The number is drawn beside the name and is already in the control's
                            // own words, so it is a decoration.
                            <span class="episode-number" aria-hidden="true">{episode.number}</span>
                            <span class="episode-text">
                                <strong>{episode.name.clone()}</strong>
                                <small>{episode.overview.clone().unwrap_or_default()}</small>
                            </span>
                        </button>
                    }
                }).collect_view()
            })}
        </section>
    }
}

fn like_this(tile: Tile) -> impl IntoView {
    let picture = image(
        &tile.images,
        &[ImageRole::Backdrop, ImageRole::Still, ImageRole::Poster],
    );
    let style = picture
        .as_ref()
        .map(|url| format!("background-image:url('{url}')"));
    let address = title_address(tile.title);
    let name = tile.name;
    let says = name.clone();
    view! {
        <button class="card" style=style aria-label=says on:click=move |_| host::go(&address)>
            {picture.is_none().then(|| view! { <span class="card-name">{name.clone()}</span> })}
        </button>
    }
}

/// The details panel, over the page it was opened from.
///
/// It is a dialog, and it says so: `role="dialog"` with `aria-modal`, named by the title's own
/// name, closed by Escape as well as by the scrim and the cross. A panel a keyboard cannot leave is
/// a trap, and that is what this was - the scrim was a `div` with a click handler and nothing else.
#[component]
pub fn Details() -> impl IntoView {
    let shared = expect_context::<Shared>();
    let listed = RwSignal::new(false);
    let reminded = RwSignal::new(false);
    let _keys = window_event_listener(leptos::ev::keydown, move |event| {
        if event.key() == "Escape" && shared.detail.with(Option::is_some) {
            close();
        }
    });
    move || {
        let detail = shared.detail.get()?;
        listed.set(detail.in_list);
        reminded.set(detail.reminder_set);
        let picture = image(
            &detail.images,
            &[ImageRole::Backdrop, ImageRole::Still, ImageRole::Poster],
        );
        let style = picture.map(|url| format!("background-image:url('{url}')"));
        let title = detail.title;
        let seasons: Vec<u32> = detail.seasons.iter().map(|season| season.number).collect();
        let first_season = seasons.first().copied().unwrap_or(1);
        let play_at = play_from(&detail);
        let resuming = detail
            .resume
            .as_ref()
            .is_some_and(|resume| resume.position_s > 0);
        let coming = detail.available_from.clone();
        let age = detail.age.clone();
        let line = facts(shared, &detail);
        let play = move |_| match &play_at {
            Some(address) => host::go(address),
            None => play_first(title, first_season),
        };
        let toggle = move |_| {
            let in_list = !listed.get();
            listed.set(in_list);
            host::act(Action::Command {
                command: Command::List { title, in_list },
            });
        };
        let remind = move |_| {
            let on = !reminded.get();
            reminded.set(on);
            host::act(Action::Command {
                command: Command::Remind { title, on },
            });
        };
        let name = detail.name.clone();
        Some(view! {
            <div class="scrim" on:click=move |_| close()>
                <article
                    class="panel"
                    role="dialog"
                    aria-modal="true"
                    aria-label=name.clone()
                    on:click=|event| event.stop_propagation()
                >
                    <button
                        class="close plain"
                        aria-label=move || label_text(shared, "say.close")
                        on:click=move |_| close()
                    >
                        // The cross is a mark and not words; the control above carries them.
                        <span aria-hidden="true">"×"</span>
                    </button>
                    <div class="panel-art" style=style>
                        <div class="panel-title">
                            <h1>{detail.name.clone()}</h1>
                            <div class="row-of-buttons">
                                {match coming.clone() {
                                    // A title that is not out yet is not played; a reminder is set
                                    // for the day it arrives (`Title.kt:300-349`).
                                    Some(day) => view! {
                                        <button class="solid" aria-pressed=move || if reminded.get() { "true" } else { "false" } on:click=remind>
                                            {move || label_text(shared, if reminded.get() { "remind.set" } else { "remind.me" })}
                                        </button>
                                        <p class="tags">
                                            {move || filled(shared, "gate.coming", "date",
                                                &shared.digits.with(|digits| super::words::in_digits(day.as_str(), digits)))}
                                        </p>
                                    }.into_any(),
                                    None => view! {
                                        <button class="solid" on:click=play>
                                            {move || label_text(shared, if resuming { "title.resume" } else { "common.play" })}
                                        </button>
                                    }.into_any(),
                                }}
                                <button
                                    class="glass"
                                    aria-pressed=move || if listed.get() { "true" } else { "false" }
                                    on:click=toggle
                                >
                                    {move || label_text(shared, if listed.get() { "list.added" } else { "list.add" })}
                                </button>
                            </div>
                        </div>
                    </div>
                    <div class="panel-body">
                        <p class="tags">
                            {age.map(|age| view! {
                                <span class=format!("age {}", age_tone(age.tone))>{age.label}</span>
                            })}
                            {line.join(" · ")}
                        </p>
                        <p>{detail.overview.clone().unwrap_or_default()}</p>
                        <p class="tags">{detail.genres.join(" · ")}</p>
                        {(!seasons.is_empty()).then(|| view! {
                            <Episodes title=title seasons=seasons.clone()/>
                        })}
                        {(!detail.more_like_this.is_empty()).then(|| view! {
                            <h2>{label_text(shared, "title.more_like")}</h2>
                            <div class="grid">
                                {detail.more_like_this.iter().cloned()
                                    .map(like_this).collect_view()}
                            </div>
                        })}
                    </div>
                </article>
            </div>
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{age_tone, play_from};
    use crate::surfaces::tests_support::english;
    use contract::Rating;
    use contract::screen::{AgeTone, TitleDetail};

    /// Every rating the contract has, so a build cannot quietly stop offering one.
    const RATINGS: [Rating; 3] = [Rating::Down, Rating::Up, Rating::Love];

    fn detail(json: &str) -> TitleDetail {
        serde_json::from_str(json).expect("a title detail")
    }

    #[test]
    fn a_film_plays_and_a_series_plays_where_the_viewer_left_off() {
        // A film has one thing to play.
        let film = detail(r#"{"title":"movie:1","name":"A"}"#);
        assert_eq!(play_from(&film).as_deref(), Some("/watch/movie/1"));
        // A series the viewer is part way through plays that episode.
        let going = detail(
            r#"{"title":"tv:2","name":"B","resume":{"episode":{"season":3,"episode":4},"position_s":90}}"#,
        );
        assert_eq!(play_from(&going).as_deref(), Some("/watch/tv/2/3/4"));
        // A series with nothing to go back to has no address until a season is read, because an
        // episode has no id of its own.
        let fresh =
            detail(r#"{"title":"tv:2","name":"B","seasons":[{"number":1,"episode_count":8}]}"#);
        assert_eq!(play_from(&fresh), None);
        // A position with no episode is a series whose season is still unknown.
        let odd = detail(r#"{"title":"tv:2","name":"B","resume":{"position_s":5}}"#);
        assert_eq!(play_from(&odd), None);
    }

    /// Every tone the contract has gets a class of its own, so no two age marks paint alike.
    #[test]
    fn every_age_tone_has_its_own_class() {
        let tones = [
            AgeTone::Green,
            AgeTone::Yellow,
            AgeTone::Orange,
            AgeTone::Pink,
            AgeTone::Red,
            AgeTone::DarkRed,
            AgeTone::Blue,
            AgeTone::Grey,
        ];
        let mut classes: Vec<&str> = tones.iter().map(|tone| age_tone(*tone)).collect();
        classes.sort_unstable();
        let held = classes.len();
        classes.dedup();
        assert_eq!(classes.len(), held, "two tones share a class");
        // A tone this build does not know falls on the plainest of them rather than on nothing.
        assert_eq!(age_tone(AgeTone::Unknown), age_tone(AgeTone::Grey));
        // And the style sheet draws every one of them.
        let css = crate::surfaces::tests_support::source("apps/web/style.css");
        for tone in tones {
            let class = age_tone(tone);
            assert!(
                css.contains(&format!(".{class}")),
                "the style sheet has no .{class}"
            );
        }
    }

    /// Every label this panel names is in the strings file, including both forms of the two
    /// controls that stand one of two ways and both plural families it counts with.
    #[test]
    fn every_label_this_panel_names_is_in_the_strings_file() {
        let book = english();
        for label in [
            "common.play",
            "gate.coming",
            "list.add",
            "list.added",
            "remind.me",
            "remind.set",
            "say.close",
            "say.season_pick",
            "title.episode",
            "title.episodes_more",
            "title.length.hours",
            "title.length.minutes",
            "title.more_like",
            "title.resume",
            "title.season.number",
        ] {
            assert!(book.contains_key(label), "strings/en.json has no {label}");
        }
        // A counted family is asked for by its base, so at least one form has to be there.
        assert!(
            book.keys().any(|key| key.starts_with("title.season.")),
            "strings/en.json has no form of title.season"
        );
        // The ratings the contract carries have words, for whoever wires the control: the phone
        // draws all three (`title/Title.kt:544-568`) and this panel draws none yet.
        for label in ["player.down", "player.up", "player.love"] {
            assert!(book.contains_key(label), "strings/en.json has no {label}");
        }
        assert_eq!(RATINGS.len(), 3);
    }
}
