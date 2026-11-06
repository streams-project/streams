//! The pages of rows: home, films, series, a genre, my list, search.

use contract::core_msg::{Action, HttpMethod, SessionState};
use contract::screen::{ImageRole, More, NavItem, Screen, ScreenName, Section, SectionKind, Tile};
use leptos::prelude::*;

use super::door::Logo;
use super::image;
use crate::host;
use crate::state::Shared;

fn address_of(item: &NavItem) -> String {
    let word = serde_json::to_value(item.screen)
        .ok()
        .and_then(|word| word.as_str().map(str::to_owned))
        .unwrap_or_default();
    match &item.genre {
        Some(genre) => format!("/genre/{genre}"),
        None => format!("/{word}"),
    }
}

fn title_address(tile: &Tile) -> String {
    let title = &tile.title;
    format!("/title/{}/{}", title.kind().word(), title.id())
}

#[component]
fn Top() -> impl IntoView {
    let shared = expect_context::<Shared>();
    let scrolled = RwSignal::new(false);
    let _listener = window_event_listener(leptos::ev::scroll, move |_| {
        let y = web_sys::window()
            .and_then(|w| w.scroll_y().ok())
            .unwrap_or(0.0);
        scrolled.set(y > 8.0);
    });
    let signed_in = move || {
        shared.session.with(|session| {
            session
                .as_ref()
                .is_some_and(|s| s.state == SessionState::SignedIn)
        })
    };
    view! {
        <header class="top" class:solid-top=scrolled>
            <Logo/>
            <nav>
                {move || shared.nav.get().into_iter().map(|item| {
                    let address = address_of(&item);
                    view! { <button class="link" on:click=move |_| host::go(&address)>{item.label}</button> }
                }).collect_view()}
            </nav>
            <div class="top-end">
                <button class="link" on:click=move |_| host::go("/search")>"Search"</button>
                {move || if signed_in() {
                    view! {
                        <button class="link" on:click=move |_| host::go("/profiles")>"Profiles"</button>
                        <button class="link" on:click=move |_| host::act(Action::SignOut { everywhere: false })>"Sign out"</button>
                    }.into_any()
                } else {
                    view! { <button class="link" on:click=move |_| host::go("/sign-in")>"Sign in"</button> }.into_any()
                }}
            </div>
        </header>
    }
}

#[component]
fn Card(tile: Tile, #[prop(optional)] rank: u32) -> impl IntoView {
    let picture = image(
        &tile.images,
        &[ImageRole::Backdrop, ImageRole::Still, ImageRole::Poster],
    );
    let address = title_address(&tile);
    let percent = tile.percent;
    let name = tile.name.clone();
    let style = picture
        .as_ref()
        .map(|url| format!("background-image:url('{url}')"));
    view! {
        <button class="card" style=style on:click=move |_| host::go(&address) aria-label=name.clone()>
            {(rank > 0).then(|| view! { <span class="rank">{rank}</span> })}
            {picture.is_none().then(|| view! { <span class="card-name">{name.clone()}</span> })}
            {percent.map(|percent| view! {
                <span class="progress"><span style=format!("width:{percent}%")></span></span>
            })}
        </button>
    }
}

#[component]
fn Hero(section: Section) -> impl IntoView {
    let Some(tile) = section.tiles.first().cloned() else {
        return ().into_any();
    };
    let picture = image(
        &tile.images,
        &[ImageRole::Backdrop, ImageRole::Still, ImageRole::Poster],
    );
    let style = picture.map(|url| format!("background-image:url('{url}')"));
    let title = &tile.title;
    let watch = format!("/watch/{}/{}", title.kind().word(), title.id());
    let details = title_address(&tile);
    view! {
        <section class="hero" style=style>
            <div class="hero-text">
                <h1>{tile.name.clone()}</h1>
                <p class="tags">{tile.tags.join(" · ")}</p>
                <div class="row-of-buttons">
                    <button class="solid" on:click=move |_| host::go(&watch)>"Play"</button>
                    <button class="glass" on:click=move |_| host::go(&details)>"More info"</button>
                </div>
            </div>
        </section>
    }
    .into_any()
}

#[component]
fn Row(section: Section) -> impl IntoView {
    let ranked = section.kind == SectionKind::Ranked;
    view! {
        <section class="row">
            {section.title.clone().map(|title| view! { <h2>{title}</h2> })}
            <div class="strip">
                {section.tiles.iter().cloned().enumerate().map(|(place, tile)| {
                    let rank = ranked.then(|| tile.badges.iter().find_map(|b| b.rank).unwrap_or(place as u32 + 1));
                    view! { <Card tile=tile rank=rank.unwrap_or(0)/> }
                }).collect_view()}
            </div>
        </section>
    }
}

#[component]
fn Grid(section: Section) -> impl IntoView {
    view! {
        <section class="row">
            {section.title.clone().map(|title| view! { <h2>{title}</h2> })}
            <div class="grid">
                {section.tiles.iter().cloned().map(|tile| view! { <Card tile=tile/> }).collect_view()}
            </div>
        </section>
    }
}

fn draw(section: Section) -> AnyView {
    match section.kind.drawn_as() {
        SectionKind::Hero => view! { <Hero section=section/> }.into_any(),
        SectionKind::Grid => view! { <Grid section=section/> }.into_any(),
        SectionKind::Notice => section
            .notice
            .map(|notice| view! { <p class="page-notice">{notice.text}</p> })
            .into_any(),
        _ => view! { <Row section=section/> }.into_any(),
    }
}

fn read_more(shared: Shared, busy: RwSignal<bool>) {
    let Some(cursor) = shared
        .screen
        .with(|screen| screen.as_ref().and_then(|s| s.cursor.clone()))
    else {
        return;
    };
    if busy.get_untracked() {
        return;
    }
    busy.set(true);
    let path = format!("/v1/more/{}", cursor.0);
    host::ask(HttpMethod::Get, &path, None, move |status, text| {
        busy.set(false);
        if !(200..300).contains(&status) {
            return;
        }
        if let Ok(more) = serde_json::from_str::<More>(&text) {
            shared.screen.update(|screen| {
                if let Some(screen) = screen {
                    screen.sections.extend(more.sections);
                    screen.cursor = more.cursor;
                }
            });
        }
    });
}

#[component]
fn SearchBox() -> impl IntoView {
    let shared = expect_context::<Shared>();
    let text = RwSignal::new(String::new());
    let run = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let words = text.get();
        let query: String = js_sys::encode_uri_component(&words).into();
        let path = format!("/v1/search?q={query}");
        host::ask(HttpMethod::Get, &path, None, move |status, body| {
            if (200..300).contains(&status)
                && let Ok(screen) = serde_json::from_str::<Screen>(&body)
            {
                shared.screen.set(Some(screen));
            }
        });
    };
    view! {
        <form class="search" on:submit=run>
            <input type="search" autofocus placeholder="Titles, people, genres" bind:value=text/>
            <button class="solid" type="submit">"Search"</button>
        </form>
    }
}

/// A page of the service.
#[component]
pub fn Browse() -> impl IntoView {
    let shared = expect_context::<Shared>();
    let busy = RwSignal::new(false);
    let _listener = window_event_listener(leptos::ev::scroll, move |_| {
        let Some(window) = web_sys::window() else {
            return;
        };
        let y = window.scroll_y().unwrap_or(0.0);
        let seen = window
            .inner_height()
            .ok()
            .and_then(|h| h.as_f64())
            .unwrap_or(0.0);
        let total = window
            .document()
            .and_then(|d| d.document_element())
            .map_or(0.0, |e| f64::from(e.scroll_height()));
        if y + seen > total - 900.0 {
            read_more(shared, busy);
        }
    });
    let searching = move || {
        shared.route.with(|route| {
            route
                .as_ref()
                .is_some_and(|route| route.screen == Some(ScreenName::Search))
        })
    };
    view! {
        <Top/>
        <main class="page">
            {move || searching().then(|| view! { <SearchBox/> })}
            {move || match shared.screen.get() {
                None if searching() => ().into_any(),
                None => view! { <div class="wait"><div class="ring"></div></div> }.into_any(),
                Some(screen) => screen.sections.into_iter().map(draw).collect_view().into_any(),
            }}
        </main>
    }
}
