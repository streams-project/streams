//! The pages of rows: home, films, series, a genre, my list, search.

use contract::address;
use contract::core_msg::{Action, HttpMethod, SessionState, Target};
use contract::screen::{ImageRole, More, NavItem, Screen, ScreenName, Section, SectionKind, Tile};
use leptos::prelude::*;

use super::door::Logo;
use super::{image, maybe_text};
use crate::host;
use crate::state::Shared;

/// Netflix appends rows at roughly two viewports of remaining scroll (web README, section 5).
const AHEAD_VIEWPORTS: f64 = 2.0;

/// The label of the words a search with no matches shows.
const NOTHING_FOUND: &str = "search.empty";

fn address_of(item: &NavItem) -> String {
    address::of_screen(item.screen, item.genre.as_deref())
}

/// The nav item the viewer is on. Netflix draws that one at weight 500 and the rest at 400.
/// Home and the kids' home are a target of their own and carry no screen name, so they are read
/// back here; every other page names its screen.
fn here(target: Target, screen: Option<ScreenName>, genre: Option<&str>, item: &NavItem) -> bool {
    let screen = match target {
        Target::Home => Some(ScreenName::Home),
        Target::KidsHome => Some(ScreenName::Kids),
        _ => screen,
    };
    screen == Some(item.screen) && genre == item.genre.as_deref()
}

/// Two viewports of remaining scroll, the measured trigger for appending rows.
fn near_end(y: f64, seen: f64, total: f64) -> bool {
    total - (y + seen) <= seen * AHEAD_VIEWPORTS
}

/// True when a page answered with no titles at all.
fn nothing_found(screen: &Screen) -> bool {
    screen
        .sections
        .iter()
        .all(|section| section.tiles.is_empty())
}

/// A line of the words a search with no matches shows: a paragraph, or an item of its one list.
/// The server writes them, one line each, an item marked by a leading "- ".
fn lines_of(text: &str, asked: &str) -> (Vec<String>, Vec<String>) {
    let mut paragraphs = Vec::new();
    let mut items = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.is_empty() {
            continue;
        }
        match line.strip_prefix("- ") {
            Some(item) => items.push(item.replace("{query}", asked)),
            None => paragraphs.push(line.replace("{query}", asked)),
        }
    }
    (paragraphs, items)
}

fn title_address(tile: &Tile) -> String {
    let title = &tile.title;
    format!("{}{}-{}", address::TITLE, title.kind().word(), title.id())
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
                    let on_it = shared.route.with(|route| {
                        route.as_ref().is_some_and(|route| {
                            here(route.target, route.screen, route.genre.as_deref(), &item)
                        })
                    });
                    view! {
                        <button class="link" class:here=on_it on:click=move |_| host::go(&address)>
                            {item.label}
                        </button>
                    }
                }).collect_view()}
            </nav>
            <div class="top-end">
                <button class="link" on:click=move |_| host::go(address::SEARCH)>"Search"</button>
                {move || if signed_in() {
                    view! {
                        <button class="link" on:click=move |_| host::go(address::PROFILES)>"Profiles"</button>
                        <button class="link" on:click=move |_| host::act(Action::SignOut { everywhere: false })>"Sign out"</button>
                    }.into_any()
                } else {
                    view! { <button class="link" on:click=move |_| host::go(address::SIGN_IN)>"Sign in"</button> }.into_any()
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
fn SearchBox(asked: RwSignal<String>) -> impl IntoView {
    let shared = expect_context::<Shared>();
    let text = RwSignal::new(String::new());
    let run = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let words = text.get();
        asked.set(words.clone());
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

/// What a search with no matches shows: six lines of plain text, measured on Netflix web
/// 2026-11-11 - no illustration, no icon, no heading, no button. The words are the server's, so
/// nothing is drawn until the strings carry `search.empty`.
#[component]
fn NothingFound(asked: RwSignal<String>) -> impl IntoView {
    let shared = expect_context::<Shared>();
    move || {
        let text = maybe_text(shared, NOTHING_FOUND)?;
        let (paragraphs, items) = lines_of(&text, &asked.get());
        Some(view! {
            <div class="empty">
                {paragraphs.into_iter().map(|line| view! { <p>{line}</p> }).collect_view()}
                {(!items.is_empty()).then(|| view! {
                    <ul>{items.into_iter().map(|item| view! { <li>{item}</li> }).collect_view()}</ul>
                })}
            </div>
        })
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
        if near_end(y, seen, total) {
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
    let asked = RwSignal::new(String::new());
    view! {
        <Top/>
        <main class="page">
            {move || searching().then(|| view! { <SearchBox asked=asked/> })}
            {move || match shared.screen.get() {
                None if searching() => ().into_any(),
                None => view! { <div class="wait"><div class="ring"></div></div> }.into_any(),
                Some(screen) if searching() && nothing_found(&screen) => {
                    view! { <NothingFound asked=asked/> }.into_any()
                }
                Some(screen) => screen.sections.into_iter().map(draw).collect_view().into_any(),
            }}
        </main>
    }
}

#[cfg(test)]
mod tests {
    use super::{here, lines_of, near_end, nothing_found};
    use contract::core_msg::Target;
    use contract::screen::{NavItem, Screen, ScreenName, Section, SectionKind};

    fn item(screen: ScreenName, genre: Option<&str>) -> NavItem {
        NavItem {
            id: String::from("n"),
            label: String::from("Label"),
            screen,
            genre: genre.map(str::to_owned),
        }
    }

    #[test]
    fn the_nav_item_the_viewer_is_on_is_the_one_marked() {
        let home = item(ScreenName::Home, None);
        let drama = item(ScreenName::Genre, Some("drama"));
        let series = item(ScreenName::Series, None);
        // Home is a target of its own and names no screen.
        assert!(here(Target::Home, None, None, &home));
        assert!(!here(Target::Home, None, None, &series));
        assert!(here(
            Target::Screen,
            Some(ScreenName::Series),
            None,
            &series
        ));
        assert!(!here(Target::Screen, Some(ScreenName::Series), None, &home));
        assert!(here(
            Target::Screen,
            Some(ScreenName::Genre),
            Some("drama"),
            &drama
        ));
        // The same screen with another genre is another page.
        assert!(!here(
            Target::Screen,
            Some(ScreenName::Genre),
            Some("comedy"),
            &drama
        ));
        // A page with no nav item of its own marks nothing.
        assert!(!here(Target::Watch, None, None, &home));
    }

    #[test]
    fn more_rows_are_asked_for_two_viewports_from_the_end() {
        // 955 of document left to scroll, one viewport: inside the trigger.
        assert!(near_end(0.0, 955.0, 1910.0));
        // Three viewports of document with the viewer at the top: outside it.
        assert!(!near_end(0.0, 955.0, 3820.0));
        // The same page, scrolled one viewport down: inside it.
        assert!(near_end(955.0, 955.0, 3820.0));
        // A page shorter than the window is always at its end.
        assert!(near_end(0.0, 955.0, 600.0));
    }

    fn section(tiles: usize) -> Section {
        Section {
            id: String::from("s"),
            kind: SectionKind::Grid,
            title: None,
            tiles: core::iter::repeat_with(|| {
                serde_json::from_str(r#"{"title":"movie:1","name":"A"}"#).expect("a tile")
            })
            .take(tiles)
            .collect(),
            cursor: None,
            notice: None,
        }
    }

    fn screen(sections: Vec<Section>) -> Screen {
        Screen {
            name: ScreenName::Search,
            sections,
            ttl_s: 60,
            refresh_on: Vec::new(),
            cursor: None,
        }
    }

    #[test]
    fn a_page_with_no_titles_at_all_is_nothing_found() {
        assert!(nothing_found(&screen(Vec::new())));
        assert!(nothing_found(&screen(vec![section(0)])));
        assert!(!nothing_found(&screen(vec![section(0), section(1)])));
    }

    #[test]
    fn the_words_with_no_matches_echo_the_query_and_keep_their_one_list() {
        let text = "Your search for \"{query}\" did not have any matches.\nSuggestions:\n- Try different keywords\n- Try a genre\n";
        let (paragraphs, items) = lines_of(text, "qzxjvwkplmn");
        assert_eq!(
            paragraphs,
            vec![
                String::from("Your search for \"qzxjvwkplmn\" did not have any matches."),
                String::from("Suggestions:"),
            ]
        );
        assert_eq!(
            items,
            vec![
                String::from("Try different keywords"),
                String::from("Try a genre"),
            ]
        );
        // No words, no block.
        assert_eq!(lines_of("", "a"), (Vec::new(), Vec::new()));
    }
}
