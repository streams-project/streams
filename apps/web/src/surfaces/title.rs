//! One title: its facts, seasons, and what is like it.

use contract::TitleKey;
use contract::command::Command;
use contract::core_msg::{Action, HttpMethod};
use contract::screen::{ImageRole, Season, Tile, TitleDetail};
use leptos::prelude::*;

use super::{image, label_text};
use crate::host;
use crate::ports::address;
use crate::state::Shared;

fn close() {
    address::back();
}

#[component]
fn Episodes(title: TitleKey, seasons: Vec<u32>) -> impl IntoView {
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
    view! {
        <section class="episodes">
            <select on:change=move |event| {
                if let Ok(number) = event_target_value(&event).parse() {
                    chosen.set(number);
                }
            }>
                {seasons.iter().map(|number| view! {
                    <option value=number.to_string()>{format!("Season {number}")}</option>
                }).collect_view()}
            </select>
            {move || season.get().map(|season| season.episodes.into_iter().map(|episode| {
                let number = season.number;
                let address = format!("/watch/{kind}/{id}/{number}/{}", episode.number);
                view! {
                    <button class="episode" on:click=move |_| host::go(&address)>
                        <span class="episode-number">{episode.number}</span>
                        <span class="episode-text">
                            <strong>{episode.name.clone()}</strong>
                            <small>{episode.overview.clone().unwrap_or_default()}</small>
                        </span>
                    </button>
                }
            }).collect_view())}
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
    let title = &tile.title;
    let address = format!(
        "{}{}-{}",
        contract::address::TITLE,
        title.kind().word(),
        title.id()
    );
    let name = tile.name;
    view! {
        <button class="card" style=style on:click=move |_| host::go(&address)>
            {picture.is_none().then(|| view! { <span class="card-name">{name.clone()}</span> })}
        </button>
    }
}

fn facts(detail: &TitleDetail) -> String {
    let mut parts = Vec::new();
    if let Some(year) = detail.year {
        parts.push(year.to_string());
    }
    if let Some(minutes) = detail.runtime_min {
        parts.push(format!("{}h {}m", minutes / 60, minutes % 60));
    }
    if !detail.seasons.is_empty() {
        parts.push(format!("{} seasons", detail.seasons.len()));
    }
    parts.join(" · ")
}

/// The details panel, over the page it was opened from.
#[component]
pub fn Details() -> impl IntoView {
    let shared = expect_context::<Shared>();
    let listed = RwSignal::new(false);
    move || {
        let detail = shared.detail.get()?;
        listed.set(detail.in_list);
        let picture = image(
            &detail.images,
            &[ImageRole::Backdrop, ImageRole::Still, ImageRole::Poster],
        );
        let style = picture.map(|url| format!("background-image:url('{url}')"));
        let title = detail.title;
        let watch = format!("/watch/{}/{}", title.kind().word(), title.id());
        let seasons: Vec<u32> = detail.seasons.iter().map(|season| season.number).collect();
        let keyed = title;
        let toggle = move |_| {
            let in_list = !listed.get();
            listed.set(in_list);
            host::act(Action::Command {
                command: Command::List {
                    title: keyed,
                    in_list,
                },
            });
        };
        Some(view! {
            <div class="scrim" on:click=move |_| close()>
                <article class="panel" on:click=|event| event.stop_propagation()>
                    <button class="close plain" aria-label="Close" on:click=move |_| close()>"×"</button>
                    <div class="panel-art" style=style>
                        <div class="panel-title">
                            <h1>{detail.name.clone()}</h1>
                            <div class="row-of-buttons">
                                <button class="solid" on:click=move |_| host::go(&watch)>"Play"</button>
                                <button class="glass" on:click=toggle>
                                    {move || label_text(shared, if listed.get() { "list.added" } else { "list.add" })}
                                </button>
                            </div>
                        </div>
                    </div>
                    <div class="panel-body">
                        <p class="tags">{facts(&detail)}</p>
                        <p>{detail.overview.clone().unwrap_or_default()}</p>
                        <p class="tags">{detail.genres.join(" · ")}</p>
                        {(!seasons.is_empty()).then(|| view! { <Episodes title=title seasons=seasons.clone()/> })}
                        {(!detail.more_like_this.is_empty()).then(|| view! {
                            <h2>"More like this"</h2>
                            <div class="grid">
                                {detail.more_like_this.iter().cloned().map(like_this).collect_view()}
                            </div>
                        })}
                    </div>
                </article>
            </div>
        })
    }
}
