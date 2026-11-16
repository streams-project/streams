//! The pages of rows: home, films, series, a genre, my list, search.

use std::collections::BTreeMap;

use contract::address;
use contract::command::Command;
use contract::core_msg::{Action, HttpMethod, SessionState, Target};
use contract::screen::{
    BadgeKind, Cursor, ImageRole, More, NavItem, Screen, ScreenName, Section, SectionKind, Tile,
};
use leptos::prelude::*;

use super::door::Logo;
use super::search::SearchBox;
use super::words::{fill, filled, number};
use super::{empty, image, label_text, maybe_text, notifications, title_address, watch_address};
use crate::host;
use crate::state::Shared;

/// How much remaining scroll triggers the next part of a list, in viewports of that list.
///
/// Measured on the reference's web page: the DOM is appended in blocks of 7 to 16 titles at about
/// two viewports of remaining scroll (`notes/netflix-reference/00-gaps.md:29` of the pstream
/// repository). The citation here read "web README, section 5" until 2026-11-12; that README has
/// nine lines and no sections, so the number had no source anybody could open.
///
/// It is used on both axes, because it is the same rule both times. The phone states the sideways
/// half as a count of tiles - "A row shows 3.29 tiles, so two rows' worth of travel is 6.6 tiles,
/// so seven" (`browse/Paging.kt:43-47`) - and two rows' worth of travel is two viewports of the
/// row, so one constant covers both. The test below shows this rule gives the phone's seven when it
/// is fed the phone's own 3.29.
const AHEAD_VIEWPORTS: f64 = 2.0;

/// The label of the words a search with no matches shows.
const NOTHING_FOUND: &str = "search.empty";

/// The rows a page draws. `browse/Rows.kt:73`.
const ROW_KINDS: [SectionKind; 4] = [
    SectionKind::Row,
    SectionKind::Ranked,
    SectionKind::Continue,
    SectionKind::Grid,
];

fn address_of(item: &NavItem) -> String {
    address::of_screen(item.screen, item.genre.as_deref())
}

/// The nav item the viewer is on. Netflix draws that one at weight 500 and the rest at 400.
/// Home and the kids' home are a target of their own and carry no screen name, so they are read
/// back here; every other page names its screen.
///
/// A title is the browse page with a modal over it and its address carries no screen, so the
/// marking comes from `behind`: the page still loaded underneath. That page's name carries no
/// genre, so a genre item stays unmarked while its modal is open rather than marking a different
/// genre's item.
fn here(
    target: Target,
    screen: Option<ScreenName>,
    genre: Option<&str>,
    behind: Option<ScreenName>,
    item: &NavItem,
) -> bool {
    let screen = match target {
        Target::Home => Some(ScreenName::Home),
        Target::KidsHome => Some(ScreenName::Kids),
        Target::Title => screen.or(behind),
        _ => screen,
    };
    screen == Some(item.screen) && genre == item.genre.as_deref()
}

/// Two viewports of remaining scroll, the measured trigger for appending to a list. The same
/// function down a page and along a row.
fn near_end(at: f64, seen: f64, total: f64) -> bool {
    total - (at + seen) <= seen * AHEAD_VIEWPORTS
}

/// How many more titles of a row are read before the viewer reaches its end, in titles.
///
/// The phone's `TILES_AHEAD` written as the rule that produced it rather than as its answer: a row
/// shows `visible` tiles and the trigger is `AHEAD_VIEWPORTS` of them, rounded up, because a part
/// of a tile is a tile that has to be there (`browse/Paging.kt:43-47`). The web does not need the
/// count - `near_end` works in pixels - and it is here because it is the one form of this number
/// the phone wrote down, so the two can be held together.
#[must_use]
pub fn tiles_ahead(visible: f64) -> usize {
    (visible * AHEAD_VIEWPORTS).ceil().max(1.0) as usize
}

/// True when a page answered with no titles at all. A notice section carries the server's words and
/// never any tiles (`server-rules/src/answer.rs`), so it is not an empty answer: `draw` shows it
/// instead of these words. Unreachable from a search today, which answers with one grid.
fn nothing_found(screen: &Screen) -> bool {
    screen.sections.iter().all(|section| {
        section.tiles.is_empty() && !matches!(section.kind.drawn_as(), SectionKind::Notice)
    })
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
            Some(item) => items.push(fill(item, "query", asked)),
            None => paragraphs.push(fill(line, "query", asked)),
        }
    }
    (paragraphs, items)
}

/// Where a list's reading is up to: the cursor it would ask for next, and whether it is asking now.
///
/// The two are one value on purpose. Held apart, the end of a read wrote them one after the other,
/// and a draw that fell between the two writes saw "not asking" beside a cursor that had already
/// been spent and asked the server for it a second time. That was watched happening on a device:
/// two requests for `page-...-36` and two for `-48` in one scroll to the bottom
/// (`browse/Paging.kt:50-56`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Reading {
    cursor: Option<Cursor>,
    asking: bool,
}

/// What the answer carried followed by every part read since, each thing once.
///
/// Two parts could name one row, or one title, if a page were built again in the middle of a read,
/// and a list with the same key twice throws in the toolkit the phone draws with, so the key
/// decides (`browse/Paging.kt:99-101` and `:122`). The first copy stays, because it is the one the
/// viewer is already looking at.
fn joined<T: Clone, K: Ord>(answer: &[T], parts: &[T], key: impl Fn(&T) -> K) -> Vec<T> {
    let mut all: Vec<T> = answer.iter().chain(parts).cloned().collect();
    let mut seen = BTreeMap::new();
    all.retain(|thing| seen.insert(key(thing), ()).is_none());
    all
}

/// How deep each row of this page has been read.
///
/// The port of `browse/Paging.kt:104-142`. A row arrives with the fifteen titles one answer
/// carries, and the reference's own row runs to seventy while this project's server holds a hundred
/// and twenty; the rest is read through the row's own cursor as the viewer travels along it. Until
/// today this surface read more *rows* through the screen's cursor and never more titles of a row,
/// so every row on the website stopped at fifteen.
#[derive(Clone, Copy)]
struct Depth {
    parts: RwSignal<BTreeMap<String, Vec<Tile>>>,
    reading: RwSignal<BTreeMap<String, Reading>>,
}

impl Depth {
    fn new() -> Self {
        Self {
            parts: RwSignal::new(BTreeMap::new()),
            reading: RwSignal::new(BTreeMap::new()),
        }
    }

    /// Every title of this row the viewer can reach.
    fn tiles(self, section: &Section) -> Vec<Tile> {
        self.parts.with(|parts| {
            let read = parts.get(&section.id).map_or(&[][..], Vec::as_slice);
            joined(&section.tiles, read, |tile| tile.title)
        })
    }

    /// Start every row of this page again, on the answer it arrived with.
    fn start(self) {
        self.parts.set(BTreeMap::new());
        self.reading.set(BTreeMap::new());
    }

    /// How many titles of this row the viewer can reach, read without being tracked: this is
    /// answered inside the row's own scroll handler, which is not a draw.
    fn held(self, section: &Section) -> usize {
        let read = self
            .parts
            .with_untracked(|parts| parts.get(&section.id).map_or(0, Vec::len));
        section.tiles.len() + read
    }

    /// The row says it is near its end; the next part is read unless one is already on its way.
    fn reach(self, section: &Section) {
        let id = section.id.clone();
        let next = self.reading.with_untracked(|reading| {
            let now = reading.get(&id).cloned().unwrap_or_else(|| Reading {
                cursor: section.cursor.clone(),
                asking: false,
            });
            if now.asking { None } else { now.cursor }
        });
        let Some(next) = next else { return };
        self.reading.update(|reading| {
            reading.insert(
                id.clone(),
                Reading {
                    cursor: Some(next.clone()),
                    asking: true,
                },
            );
        });
        let path = format!("/v1/more/{}", next.0);
        host::ask(HttpMethod::Get, &path, None, move |status, text| {
            let more = (200..300)
                .contains(&status)
                .then(|| serde_json::from_str::<More>(&text).ok())
                .flatten();
            if let Some(more) = &more
                && !more.tiles.is_empty()
            {
                self.parts.update(|parts| {
                    parts
                        .entry(id.clone())
                        .or_default()
                        .extend(more.tiles.clone());
                });
            }
            // As with a page: what did not arrive is not asked for twice.
            self.reading.update(|reading| {
                reading.insert(
                    id.clone(),
                    Reading {
                        cursor: more.and_then(|more| more.cursor),
                        asking: false,
                    },
                );
            });
        });
    }
}

#[component]
fn Bell() -> impl IntoView {
    let shared = expect_context::<Shared>();
    let said = move || notifications::bell_says(shared, shared.unread.get());
    view! {
        <button class="link bell" aria-label=said on:click=move |_| host::go(address::NOTIFICATIONS)>
            // The bell is a picture with no words in it, so the control above carries them and this
            // is a decoration (`browse/Header.kt:70-71` and `:110` say the same of the phone's).
            <svg viewBox="0 0 24 24" aria-hidden="true" focusable="false">
                <path
                    fill="currentColor"
                    d="M12 2a6 6 0 0 0-6 6v3.6l-1.7 3.1a1 1 0 0 0 .9 1.5h13.6a1 1 0 0 0 .9-1.5L18 11.6V8a6 6 0 0 0-6-6Zm0 19a3 3 0 0 0 3-3H9a3 3 0 0 0 3 3Z"
                />
            </svg>
            {move || (shared.unread.get() > 0).then(|| view! {
                <span class="count" aria-hidden="true">
                    {shared.digits.with(|digits| notifications::bell_count(digits, shared.unread.get()))}
                </span>
            })}
        </button>
    }
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
    // The page still under a title's modal: the one the nav marks while the modal is open.
    let behind = Memo::new(move |_| shared.screen.with(|screen| screen.as_ref().map(|s| s.name)));
    view! {
        <header class="top" class:solid-top=scrolled>
            <Logo/>
            // The menu is a list of links to pages, which is what a reader is told it is, and the
            // page in view is marked with `aria-current` as well as by weight.
            <nav aria-label=move || label_text(shared, "nav.home")>
                {move || shared.nav.get().into_iter().map(|item| {
                    let address = address_of(&item);
                    let behind = behind.get();
                    let on_it = shared.route.with(|route| {
                        route.as_ref().is_some_and(|route| {
                            here(route.target, route.screen, route.genre.as_deref(), behind, &item)
                        })
                    });
                    view! {
                        <button
                            class="link"
                            class:here=on_it
                            aria-current=on_it.then_some("page")
                            on:click=move |_| host::go(&address)
                        >
                            {item.label}
                        </button>
                    }
                }).collect_view()}
            </nav>
            <div class="top-end">
                <button class="link" on:click=move |_| host::go(address::SEARCH)>
                    {move || label_text(shared, "nav.search")}
                </button>
                {move || if signed_in() {
                    view! {
                        <Bell/>
                        <button class="link" on:click=move |_| host::go(address::PROFILES)>
                            {label_text(shared, "menu.switch")}
                        </button>
                        <button class="link" on:click=move |_| host::act(Action::SignOut { everywhere: false })>
                            {label_text(shared, "menu.sign_out")}
                        </button>
                    }.into_any()
                } else {
                    view! {
                        <button class="link" on:click=move |_| host::go(address::SIGN_IN)>
                            {label_text(shared, "door.sign_in")}
                        </button>
                    }.into_any()
                }}
            </div>
        </header>
    }
}

/// What one tile says: its name, and the place or the position drawn over it.
///
/// A tile draws a picture and sometimes a mark over it, and no words of its own, so the name is the
/// one thing a reader has to hear and it comes from the answer rather than from a key
/// (`browse/Rows.kt:409-412`). Where a place or a progress bar is drawn, the words for it are read
/// from `strings/`, because neither is words.
fn tile_says(shared: Shared, tile: &Tile, rank: Option<u32>) -> String {
    if let Some(rank) = rank
        && let Some(text) = maybe_text(shared, "say.ranked")
    {
        return fill(
            &fill(&text, "rank", &number(shared, rank)),
            "title",
            &tile.name,
        );
    }
    if let Some(percent) = tile.percent
        && let Some(text) = filled(shared, "say.watched", "title", &tile.name)
    {
        return fill(&text, "percent", &number(shared, percent));
    }
    tile.name.clone()
}

#[component]
fn Card(tile: Tile, #[prop(default = None)] rank: Option<u32>) -> impl IntoView {
    let shared = expect_context::<Shared>();
    let picture = image(
        &tile.images,
        &[ImageRole::Backdrop, ImageRole::Still, ImageRole::Poster],
    );
    let address = title_address(tile.title);
    let percent = tile.percent;
    let name = tile.name.clone();
    let new = tile.badges.iter().any(|badge| badge.kind == BadgeKind::New);
    let style = picture
        .as_ref()
        .map(|url| format!("background-image:url('{url}')"));
    let says = tile_says(shared, &tile, rank);
    view! {
        <div class="tile">
            <button class="card" style=style on:click=move |_| host::go(&address) aria-label=says>
                {picture.is_none().then(|| view! { <span class="card-name">{name.clone()}</span> })}
                {percent.map(|percent| view! {
                    <span class="progress" aria-hidden="true">
                        <span style=format!("width:{percent}%")></span>
                    </span>
                })}
            </button>
            // Decorative overlays sit beside `<button>` rather than inside it: the rank numeral
            // and the "New" badge are already visual decorations (and rank is already in `says`),
            // and WCAG 2.5.3 / Axe visible-text extraction includes `aria-hidden` descendants.
            {rank.map(|rank| view! {
                <span class="rank" aria-hidden="true">{number(shared, rank)}</span>
            })}
            {new.then(|| view! {
                <span class="badge" aria-hidden="true">{label_text(shared, "badge.new")}</span>
            })}
        </div>
    }
}

/// A Continue Watching tile: the art with its progress, and the two controls the phone draws under
/// it. `browse/Rows.kt:306-410`.
#[component]
fn ContinueCard(tile: Tile) -> impl IntoView {
    let shared = expect_context::<Shared>();
    let menu = RwSignal::new(false);
    let gone = RwSignal::new(false);
    let keyed = tile.title;
    let details = title_address(keyed);
    let remove = move |_| {
        menu.set(false);
        gone.set(true);
        host::act(Action::Command {
            command: Command::Hide { title: keyed },
        });
    };
    view! {
        {move || (!gone.get()).then(|| { let tile = tile.clone(); let details = details.clone(); view! {
            <div class="going">
                <Card tile=tile/>
                <div class="going-row">
                    <button
                        class="plain"
                        aria-label=move || label_text(shared, "common.more_info")
                        on:click=move |_| host::go(&details)
                    >
                        {move || label_text(shared, "common.more_info")}
                    </button>
                    <button
                        class="plain"
                        aria-label=move || label_text(shared, "say.more_options")
                        aria-expanded=move || if menu.get() { "true" } else { "false" }
                        on:click=move |_| menu.update(|open| *open = !*open)
                    >
                        "⋯"
                    </button>
                </div>
                {move || menu.get().then(|| view! {
                    <div class="going-menu" role="menu">
                        <button class="plain" role="menuitem" on:click=remove>
                            {label_text(shared, "row.continue.remove")}
                        </button>
                    </div>
                })}
            </div>
        }})}
    }
}

#[component]
fn Hero(section: Section) -> impl IntoView {
    let shared = expect_context::<Shared>();
    let Some(tile) = section.tiles.first().cloned() else {
        return ().into_any();
    };
    let picture = image(
        &tile.images,
        &[ImageRole::Backdrop, ImageRole::Still, ImageRole::Poster],
    );
    let style = picture.map(|url| format!("background-image:url('{url}')"));
    let watch = watch_address(tile.title, None);
    let details = title_address(tile.title);
    view! {
        <section class="hero" style=style>
            <div class="hero-text">
                <h1>{tile.name.clone()}</h1>
                <p class="tags">{tile.tags.join(" · ")}</p>
                <div class="row-of-buttons">
                    <button class="solid" on:click=move |_| host::go(&watch)>
                        {move || label_text(shared, "common.play")}
                    </button>
                    <button class="glass" on:click=move |_| host::go(&details)>
                        {move || label_text(shared, "common.more_info")}
                    </button>
                </div>
            </div>
        </section>
    }
    .into_any()
}

/// The whole of one row, as a page of its own. `browse/Rows.kt:487-583`.
///
/// Opened from the row's name when the row holds more than it shows, and it reads the rest of the
/// row through the same cursor the row itself uses, so it reaches the whole row and not only the
/// part the page arrived with.
#[component]
fn WholeRow(section: Section, depth: Depth, close: Callback<()>) -> impl IntoView {
    let shared = expect_context::<Shared>();
    let name = section.title.clone().unwrap_or_default();
    let watched = section.clone();
    let _keys = window_event_listener(leptos::ev::keydown, move |event| {
        if event.key() == "Escape" {
            close.run(());
        }
    });
    view! {
        <div class="sheet whole-row" role="dialog" aria-modal="true" aria-label=name.clone()>
            <header class="sheet-top">
                <button
                    class="plain"
                    aria-label=move || label_text(shared, "common.back")
                    on:click=move |_| close.run(())
                >
                    {move || label_text(shared, "common.back")}
                </button>
                <h1>{name.clone()}</h1>
            </header>
            <div class="grid">
                {move || depth.tiles(&watched).into_iter().map(|tile| view! {
                    <Card tile=tile/>
                }).collect_view()}
            </div>
            <button class="whole-row-more plain" on:click=move |_| depth.reach(&section)>
                {move || label_text(shared, "common.next")}
            </button>
        </div>
    }
}

/// How far one press of a row's control moves it: a whole page of tiles.
///
/// `reference/netflix-on-the-web.md` measures the distance at two widths and states the same
/// derivation for both - 1808 px at 1920, which is "one full page of 7 tiles", and 350 at 390,
/// "the content width 342 plus one gutter, and also exactly 2 x the 175 pitch". The second form is
/// the one written here, because this surface already knows both of its numbers: the count across
/// is `empty::tiles_across`, which the style sheet draws the row with, and the pitch is the row's
/// own length over the tiles in it. The first form would need the page margin, which this crate
/// deliberately does not carry at run time (`sizes.rs`).
///
/// The two forms are the same number only because a row's tiles fill its content box exactly,
/// which is the one rule that produces a row; the test below holds this to both of the
/// reference's readings.
fn page_step(across: usize, pitch: f64) -> f64 {
    across as f64 * pitch
}

/// Which of a row's two controls stand, from where the row is scrolled to: (the one that goes
/// back, the one that goes on).
///
/// The reference states the first half - "the left arrow has zero width on page 1" - and measures
/// no row that runs out, because the rows it measured never do. This surface's do, so the control
/// that goes on stands only while there is somewhere to go: an arrow that moves nothing is a
/// control that lies about the row.
fn controls(scrolled: f64, seen: f64, whole: f64) -> (bool, bool) {
    // A pixel of slack at the far end: a scroller's own arithmetic lands fractionally short of
    // its length, and an arrow left standing on a row with nothing behind it is the defect above.
    (scrolled > 0.0, scrolled + seen < whole - 1.0)
}

#[component]
fn Row(section: Section, depth: Depth) -> impl IntoView {
    let shared = expect_context::<Shared>();
    let ranked = section.kind == SectionKind::Ranked;
    let going = section.kind == SectionKind::Continue;
    let whole = RwSignal::new(false);
    let more = section.cursor.is_some();
    let name = section.title.clone().unwrap_or_default();
    let read = section.clone();
    let opened = section.clone();
    let scrolled = section.clone();
    let strip = NodeRef::<leptos::html::Div>::new();
    // Nothing is scrolled yet and a row almost always holds more than it shows, so the control
    // that goes on stands from the start and the one that goes back does not. A row too short to
    // scroll corrects the second of these on its first scroll event, which is also the only way
    // a viewer can learn it is short.
    let stand = RwSignal::new((false, true));
    let page = move |back: bool| {
        let Some(strip) = strip.get() else {
            return;
        };
        // The tiles in the document, counted off the row itself rather than off the answer it
        // was built from: the control moves what is drawn, and the two are the same count only
        // while the row is not mid-read. The reference counts them the same way, before a press
        // and after it.
        let held = f64::from(strip.child_element_count());
        let length = f64::from(strip.scroll_width());
        if held <= 0.0 || length <= 0.0 {
            return;
        }
        // The strip spans the page, so its own box is the window the style sheet's queries are
        // matched against, and the count it draws is the count this moves by.
        let across = super::empty::tiles_across(f64::from(strip.client_width()));
        let step = page_step(across, length / held);
        strip.scroll_by_with_x_and_y(if back { -step } else { step }, 0.0);
    };
    let heading = move || {
        let name = name.clone();
        if !more {
            return view! { <h2>{name}</h2> }.into_any();
        }
        let says = filled(shared, "say.see_all", "row", &name).unwrap_or_else(|| name.clone());
        view! {
            <h2>
                <button class="see-all" aria-label=says on:click=move |_| whole.set(true)>
                    {name.clone()}
                    // The chevron says the name is a control. It is a mark and not words, so it is
                    // a decoration and the words above carry where it goes
                    // (`browse/Rows.kt:146-148`).
                    <span aria-hidden="true">"›"</span>
                </button>
            </h2>
        }
        .into_any()
    };
    view! {
        <section class="row">
            {heading}
            <div class="strip-page">
            <div
                class="strip"
                node_ref=strip
                on:scroll=move |event| {
                    let Some(strip) = event.target().and_then(|target| {
                        wasm_bindgen::JsCast::dyn_into::<web_sys::Element>(target).ok()
                    }) else {
                        return;
                    };
                    // In tiles, which is the unit the phone states this trigger in: the row's
                    // whole length over how many tiles are in it is one tile's pitch, and the
                    // tiles left to travel are read against `tiles_ahead` of what is on screen.
                    let held = depth.held(&scrolled) as f64;
                    let whole = f64::from(strip.scroll_width());
                    if held <= 0.0 || whole <= 0.0 {
                        return;
                    }
                    let pitch = whole / held;
                    let seen = f64::from(strip.client_width());
                    let at = f64::from(strip.scroll_left());
                    stand.set(controls(at, seen, whole));
                    let left = (whole - at - seen) / pitch;
                    if left <= tiles_ahead(seen / pitch) as f64 {
                        depth.reach(&scrolled);
                    }
                }
            >
                {move || depth.tiles(&read).into_iter().enumerate().map(|(place, tile)| {
                    let rank = ranked.then(|| {
                        tile.badges
                            .iter()
                            .find_map(|badge| badge.rank)
                            .unwrap_or(place as u32 + 1)
                    });
                    if going {
                        view! { <ContinueCard tile=tile/> }.into_any()
                    } else {
                        view! { <Card tile=tile rank=rank/> }.into_any()
                    }
                }).collect_view()}
            </div>
            // The two controls stand over the page margin, outside the scroller, so they do not
            // travel with the tiles. The mark inside each is a decoration: the words that say
            // where it goes are its own label.
            {move || stand.get().0.then(|| view! {
                <button
                    class="strip-step strip-back"
                    aria-label=move || label_text(shared, "common.back")
                    on:click=move |_| page(true)
                >
                    <span aria-hidden="true">"‹"</span>
                </button>
            })}
            {move || stand.get().1.then(|| view! {
                <button
                    class="strip-step strip-on"
                    aria-label=move || label_text(shared, "common.next")
                    on:click=move |_| page(false)
                >
                    <span aria-hidden="true">"›"</span>
                </button>
            })}
            </div>
            {move || whole.get().then(|| view! {
                <WholeRow
                    section=opened.clone()
                    depth=depth
                    close=Callback::new(move |()| whole.set(false))
                />
            })}
        </section>
    }
}

#[component]
fn Grid(section: Section, depth: Depth) -> impl IntoView {
    let read = section.clone();
    view! {
        <section class="row">
            {section.title.clone().map(|title| view! { <h2>{title}</h2> })}
            <div class="grid">
                {move || depth.tiles(&read).into_iter().map(|tile| view! { <Card tile=tile/> }).collect_view()}
            </div>
        </section>
    }
}

/// True for a section this page draws. A row with no titles is nothing at all, so it is left out
/// rather than drawn as an empty strip (`browse/Rows.kt:122` returns from the row for the same
/// reason, and `browse/Home.kt:71` filters them before that); a notice is words and carries no
/// tiles by design.
fn drawable(section: &Section) -> bool {
    match section.kind.drawn_as() {
        SectionKind::Notice => section.notice.is_some(),
        SectionKind::Hero => !section.tiles.is_empty(),
        kind => ROW_KINDS.contains(&kind) && !section.tiles.is_empty(),
    }
}

fn draw(section: Section, depth: Depth) -> AnyView {
    match section.kind.drawn_as() {
        SectionKind::Hero => view! { <Hero section=section/> }.into_any(),
        SectionKind::Grid => view! { <Grid section=section depth=depth/> }.into_any(),
        SectionKind::Notice => section
            .notice
            .map(|notice| view! { <p class="page-notice">{notice.text}</p> })
            .into_any(),
        _ => view! { <Row section=section depth=depth/> }.into_any(),
    }
}

/// How far down a page has been read: the rows the answer carried, plus every part read since.
///
/// The port of `browse/Paging.kt:71-102`. Held here and not written back into `shared.screen`,
/// which is what this surface used to do, and which was a defect with two halves: the answer and
/// the page became the same value, so nothing could tell a page grown by scrolling from a page
/// rebuilt by the server - and every write of a new row reset every row's own tile depth, because
/// the depths are keyed on the answer.
#[derive(Clone, Copy)]
struct Page {
    parts: RwSignal<Vec<Section>>,
    reading: RwSignal<Reading>,
}

impl Page {
    fn new() -> Self {
        Self {
            parts: RwSignal::new(Vec::new()),
            reading: RwSignal::new(Reading::default()),
        }
    }

    /// The rows of this page as the viewer can reach them.
    fn sections(self, screen: &Screen) -> Vec<Section> {
        self.parts
            .with(|parts| joined(&screen.sections, parts, |section| section.id.clone()))
    }

    /// Start this page again on the answer it arrived with.
    fn start(self, cursor: Option<Cursor>) {
        self.parts.set(Vec::new());
        self.reading.set(Reading {
            cursor,
            asking: false,
        });
    }

    /// Read the next part of this page, unless one is already on its way or there is none.
    fn more(self) {
        let next = self.reading.with_untracked(|reading| {
            if reading.asking {
                None
            } else {
                reading.cursor.clone()
            }
        });
        let Some(next) = next else { return };
        self.reading.set(Reading {
            cursor: Some(next.clone()),
            asking: true,
        });
        let path = format!("/v1/more/{}", next.0);
        host::ask(HttpMethod::Get, &path, None, move |status, text| {
            let more = (200..300)
                .contains(&status)
                .then(|| serde_json::from_str::<More>(&text).ok())
                .flatten();
            if let Some(more) = &more
                && !more.sections.is_empty()
            {
                self.parts
                    .update(|parts| parts.extend(more.sections.clone()));
            }
            // A part that does not arrive stops this page: asking the same cursor again on the next
            // frame would be a loop against a server that has already said no (`Paging.kt:93-95`).
            self.reading.set(Reading {
                cursor: more.and_then(|more| more.cursor),
                asking: false,
            });
        });
    }
}

/// Which answer this page's read parts belong to: the page it is, the cursor that answer carried
/// and how many rows it carried. A page built again gets a new stamp and starts its reading again.
///
/// Both halves of the reading are keyed on it - the page's own parts and each row's depth - because
/// they have to be thrown away together. Keyed apart, a refreshed page kept every row's old cursor
/// while the rows themselves went back to the titles the answer carries, so a row either stopped
/// growing or asked again for tiles it was already showing (`Paging.kt:59-66`).
fn page_stamp(shared: Shared) -> String {
    let here = shared.route.with(|route| {
        route.as_ref().map_or_else(String::new, |route| {
            format!(
                "{:?}|{}",
                route.target,
                route.genre.clone().unwrap_or_default()
            )
        })
    });
    shared
        .screen
        .with(|screen| stamp_of(&here, screen.as_ref()))
}

/// The stamp itself, apart from where its two halves are read, so it can be held to what it is for.
fn stamp_of(here: &str, screen: Option<&Screen>) -> String {
    match screen {
        Some(screen) => format!(
            "{here}|{:?}|{}|{}",
            screen.name,
            screen
                .cursor
                .as_ref()
                .map_or("", |cursor| cursor.0.as_str()),
            screen.sections.len()
        ),
        None => here.to_owned(),
    }
}

/// What a search with no matches shows: six lines of plain text, measured on Netflix web
/// 2026-11-11 - no illustration, no icon, no heading, no button. The words are the server's, so
/// nothing is drawn until the strings carry `search.empty`, and `strings/en.json` does not: it has
/// only `search.recommended`. So gap 13 of `00-gaps.md` is **not** closed by this - what a viewer
/// gets today is still a blank result area, and the shape is here waiting for the words. The words
/// belong to whoever owns `strings/`; the linter accepts the `{query}` placeholder already
/// (`PLACEHOLDER_RE` in `platform/tools/check_strings.py` is `\{[a-z_]+\}`).
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
    let depth = Depth::new();
    let page = Page::new();
    // One page and one set of depths per answer, both started again on the same stamp.
    let stamp = Memo::new(move |_| page_stamp(shared));
    Effect::new(move |_| {
        stamp.track();
        page.start(
            shared
                .screen
                .with_untracked(|screen| screen.as_ref().and_then(|screen| screen.cursor.clone())),
        );
        depth.start();
    });
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
            page.more();
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
                None => view! { <empty::WaitingRows/> }.into_any(),
                Some(screen) if searching() && nothing_found(&screen) => {
                    view! { <NothingFound asked=asked/> }.into_any()
                }
                Some(screen) => page
                    .sections(&screen)
                    .into_iter()
                    .filter(drawable)
                    .map(|section| draw(section, depth))
                    .collect_view()
                    .into_any(),
            }}
        </main>
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AHEAD_VIEWPORTS, ROW_KINDS, controls, drawable, here, lines_of, near_end, nothing_found,
        page_step, stamp_of, tiles_ahead,
    };
    use crate::surfaces::sizes::WidthClass;
    use crate::surfaces::tests_support::section as reference_section;
    use crate::surfaces::tests_support::{english, flat, numbers, one_number, source};
    use contract::core_msg::Target;
    use contract::screen::{NavItem, Screen, ScreenName, Section, SectionKind};

    const PHONE_PAGING: &str =
        "apps/android/surfaces/phone/src/main/kotlin/streams/phone/browse/Paging.kt";

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
        assert!(here(Target::Home, None, None, None, &home));
        assert!(!here(Target::Home, None, None, None, &series));
        assert!(here(
            Target::Screen,
            Some(ScreenName::Series),
            None,
            None,
            &series
        ));
        assert!(!here(
            Target::Screen,
            Some(ScreenName::Series),
            None,
            None,
            &home
        ));
        assert!(here(
            Target::Screen,
            Some(ScreenName::Genre),
            Some("drama"),
            None,
            &drama
        ));
        // The same screen with another genre is another page.
        assert!(!here(
            Target::Screen,
            Some(ScreenName::Genre),
            Some("comedy"),
            None,
            &drama
        ));
        // A page with no nav item of its own marks nothing.
        assert!(!here(Target::Watch, None, None, None, &home));
    }

    #[test]
    fn a_title_keeps_the_marking_of_the_page_behind_its_modal() {
        let home = item(ScreenName::Home, None);
        let series = item(ScreenName::Series, None);
        let drama = item(ScreenName::Genre, Some("drama"));
        // A title's address carries no screen; the page under it does.
        assert!(here(
            Target::Title,
            None,
            None,
            Some(ScreenName::Series),
            &series
        ));
        assert!(!here(
            Target::Title,
            None,
            None,
            Some(ScreenName::Series),
            &home
        ));
        // A genre page keeps no genre in a title's address, so no genre item is marked.
        assert!(!here(
            Target::Title,
            None,
            None,
            Some(ScreenName::Genre),
            &drama
        ));
        // Nothing loaded behind it marks nothing.
        assert!(!here(Target::Title, None, None, None, &home));
        // The marking still drops on a page that is not browse at all.
        assert!(!here(
            Target::Watch,
            None,
            None,
            Some(ScreenName::Home),
            &home
        ));
    }

    #[test]
    fn more_is_asked_for_two_viewports_from_the_end() {
        // 955 of document left to scroll, one viewport: inside the trigger.
        assert!(near_end(0.0, 955.0, 1910.0));
        // Three viewports of document with the viewer at the top: outside it.
        assert!(!near_end(0.0, 955.0, 3820.0));
        // The same page, scrolled one viewport down: inside it.
        assert!(near_end(955.0, 955.0, 3820.0));
        // A page shorter than the window is always at its end.
        assert!(near_end(0.0, 955.0, 600.0));
    }

    /// The rule that reads more of a row gives the phone's own answer from the phone's own numbers.
    ///
    /// `browse/Paging.kt:43-47` states the whole derivation in prose - a row shows 3.29 tiles, so
    /// two rows' worth of travel is 6.6 tiles, so seven - and then writes seven down as a constant.
    /// Both numbers are in that file, so the rule here can be held to both: a trigger of one
    /// viewport, or one that rounded down, fails this.
    #[test]
    fn the_rule_that_reads_more_of_a_row_gives_the_phone_s_answer() {
        let paging = source(PHONE_PAGING);
        let said = numbers(
            &[
                "A row shows ",
                " tiles, so two rows' worth of travel is ",
                " tiles, so seven.",
            ],
            &paging,
            "the phone's sideways derivation",
        );
        let (visible, travel) = (said[0], said[1]);
        // The comment's own multiplication, to the one digit it carries. Two rows' worth is
        // `AHEAD_VIEWPORTS` of them, which is what makes one constant serve both axes.
        assert_eq!(
            (visible * AHEAD_VIEWPORTS * 10.0).round() / 10.0,
            travel,
            "{visible} tiles over {AHEAD_VIEWPORTS} viewports is not the {travel} stated"
        );
        assert_eq!(tiles_ahead(visible), travel.ceil() as usize);
        assert_eq!(
            tiles_ahead(visible) as f64,
            numbers(&["TILES_AHEAD = ", "\n"], &paging, "the phone's constant")[0]
        );
        // A row with nothing visible still reads ahead by one, rather than never reading at all.
        assert_eq!(tiles_ahead(0.0), 1);
    }

    /// The rows this page draws are the kinds the phone draws.
    #[test]
    fn the_rows_drawn_are_the_kinds_the_phone_draws() {
        let rows =
            source("apps/android/surfaces/phone/src/main/kotlin/streams/phone/browse/Rows.kt");
        let named = rows
            .split_once("val RowKinds = setOf(")
            .expect("the phone still names its row kinds")
            .1
            .split_once(')')
            .expect("the set closes")
            .0;
        for kind in ROW_KINDS {
            // The word is the one the kind goes over the wire as, which is the one the generated
            // Kotlin names its own case by.
            let word = serde_json::to_string(&kind).expect("a kind is one word");
            let there = format!("SectionKind.{}", word.trim_matches('"').to_uppercase());
            assert!(named.contains(&there), "the phone does not draw {there}");
        }
        assert_eq!(
            named.matches("SectionKind.").count(),
            ROW_KINDS.len(),
            "the phone draws a kind this surface does not: {named}"
        );
    }

    fn section(tiles: usize) -> Section {
        named("s", tiles)
    }

    fn named(id: &str, tiles: usize) -> Section {
        Section {
            id: String::from(id),
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

    /// A page grown by scrolling keeps its stamp; a page the server built again does not.
    ///
    /// The defect this separation is against, found while writing it: this surface used to write
    /// every new row back into the one value that holds the answer, so the stamp moved on every
    /// part read and every row's own tile depth was thrown away each time the page grew down.
    #[test]
    fn a_page_grown_by_scrolling_keeps_its_stamp_and_one_built_again_does_not() {
        let answer = screen(vec![named("a", 3), named("b", 3)]);
        let was = stamp_of("Home|", Some(&answer));
        assert_eq!(was, stamp_of("Home|", Some(&answer)), "the same answer");
        // The parts read since are held apart from the answer, so growing the page changes
        // nothing here. If they were written back, this is the line that would fail.
        let grown = page_parts(&answer, vec![named("c", 3)]);
        assert_eq!(
            was,
            stamp_of("Home|", Some(&answer)),
            "{} rows now",
            grown.len()
        );
        assert_eq!(grown.len(), 3);
        // A fresh answer with another row in it is another page.
        let again = screen(vec![named("a", 3), named("b", 3), named("c", 3)]);
        assert_ne!(was, stamp_of("Home|", Some(&again)));
        // So is the same page of another genre, and so is having no answer at all.
        assert_ne!(was, stamp_of("Screen|drama", Some(&answer)));
        assert_ne!(was, stamp_of("Home|", None));
    }

    /// What `Page::sections` does to the answer and the parts, without the signals it holds them in.
    fn page_parts(answer: &Screen, parts: Vec<Section>) -> Vec<Section> {
        super::joined(&answer.sections, &parts, |section| section.id.clone())
    }

    /// The answer and the parts are joined in order, and nothing is in the list twice.
    #[test]
    fn a_thing_the_answer_and_a_part_both_name_is_in_the_list_once() {
        let answer = screen(vec![named("a", 1)]);
        // A part naming a row the answer already carried does not repeat it, and the answer's copy
        // is the one kept: it is the one the viewer is already looking at.
        let joined = page_parts(&answer, vec![named("a", 2), named("b", 1)]);
        assert_eq!(joined.len(), 2);
        assert_eq!(joined[0].tiles.len(), 1, "the answer's copy stays");
        assert_eq!(joined[1].id, "b");
        // Nothing read yet is the answer, unchanged.
        assert_eq!(page_parts(&answer, Vec::new()), answer.sections);
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
    fn a_notice_carries_words_without_tiles_and_is_not_an_empty_answer() {
        let mut notice = section(0);
        notice.kind = SectionKind::Notice;
        assert!(!nothing_found(&screen(vec![notice.clone()])));
        assert!(!nothing_found(&screen(vec![notice, section(0)])));
    }

    /// A row with no titles is not drawn, and a notice with no words is not drawn either.
    #[test]
    fn a_row_with_no_titles_is_not_drawn_and_a_notice_without_words_is_not() {
        assert!(!drawable(&section(0)));
        assert!(drawable(&section(1)));
        let mut notice = section(0);
        notice.kind = SectionKind::Notice;
        assert!(!drawable(&notice), "a notice with nothing in it is nothing");
        notice.notice = Some(
            serde_json::from_str(r#"{"label":"problem-failed","text":"x"}"#).expect("a notice"),
        );
        assert!(drawable(&notice));
        // A kind this build does not know falls back to a row, so it is drawn when it has titles.
        let mut odd = section(1);
        odd.kind = SectionKind::Unknown;
        assert!(drawable(&odd));
    }

    #[test]
    fn the_words_with_no_matches_echo_the_query_and_keep_their_one_list() {
        let text = "Your search for \"{query}\" did not have any matches.\nSuggestions:\n- Try different keywords\n- Try a genre\n";
        let (paragraphs, items) = lines_of(text, "qzxjvwkplmn");
        // The query is isolated where it is filled in, as every filled value is, so a title or a
        // typed word keeps its own order inside a line of either direction.
        assert_eq!(
            paragraphs,
            vec![
                String::from(
                    "Your search for \"\u{2068}qzxjvwkplmn\u{2069}\" did not have any matches."
                ),
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

    /// Every label this page names is one `strings/en.json` carries.
    #[test]
    fn every_label_this_page_names_is_in_the_strings_file() {
        let book = english();
        for label in [
            "badge.new",
            "common.more_info",
            "common.next",
            "common.play",
            "common.back",
            "door.sign_in",
            "menu.sign_out",
            "menu.switch",
            "nav.home",
            "nav.search",
            "row.continue.remove",
            "say.more_options",
            "say.ranked",
            "say.see_all",
            "say.watched",
        ] {
            assert!(book.contains_key(label), "strings/en.json has no {label}");
        }
        // The one key this page asks for that no language has yet, named so the gap cannot outlive
        // itself: filling it fails this line and the test above it starts drawing the words.
        assert!(
            !book.contains_key(super::NOTHING_FOUND),
            "{} is in strings/en.json now: move it to the list above",
            super::NOTHING_FOUND
        );
    }

    /// What this page is drawn to, measured live on 2026-11-13.
    const WEB_REFERENCE: &str = "reference/netflix-on-the-web.md";

    /// The style sheet, which is where every size on this page lives: the browser is what draws
    /// them and nothing in this crate reads one while the page runs.
    const SITE: &str = "apps/web/style.css";

    /// How close a size has to be, as that file sets it for itself.
    fn tolerance(reference: &str) -> f64 {
        one_number(
            &["within **", " px** of the number"],
            reference,
            "the reference's tolerance",
        )
    }

    /// The brand's spacing unit, which is what every gap in the style sheet is written in.
    fn unit() -> f64 {
        one_number(
            &["\"unit\": ", ""],
            &source("brand.json"),
            "the brand's unit",
        )
    }

    /// Every billboard one of the reference's tables states, as (viewport, its width, its
    /// height). Its rows read `| 1920 | 60 | 7 | 250.3 x 140.5 | 258.29 | 1800 x 825.7 |`, and a
    /// row whose last cell is not a width by a height - the phone table's, which states a width
    /// alone - is not one of them.
    fn table_billboards(table: &str) -> Vec<(f64, f64, f64)> {
        let mut read = Vec::new();
        for line in table.lines() {
            let cells: Vec<&str> = line
                .trim()
                .trim_matches('|')
                .split('|')
                .map(str::trim)
                .collect();
            let Some(last) = cells.last() else { continue };
            let (Ok(wide), Some((billboard, tall))) =
                (cells[0].parse::<f64>(), last.split_once(" x "))
            else {
                continue;
            };
            let (Ok(billboard), Ok(tall)) = (billboard.parse::<f64>(), tall.parse::<f64>()) else {
                continue;
            };
            read.push((wide, billboard, tall));
        }
        assert!(!read.is_empty(), "no table row reads as a billboard");
        read
    }

    /// The three ratios the style sheet draws the billboard at, narrowest band first, each with
    /// the width its band starts at: (from, height over width).
    fn billboard_bands() -> [(f64, f64); 3] {
        let css = source(SITE);
        let narrow = numbers(
            &[
                ".hero { position: relative; display: flex; align-items: flex-end; aspect-ratio: ",
                " / ",
                "; margin: 0 var(--edge);",
            ],
            &css,
            "the billboard's ratio under the first step",
        );
        let middle = numbers(
            &[
                "@media (min-width: ",
                "px) { .hero { aspect-ratio: ",
                " / ",
                "; padding-inline: var(--edge); }",
            ],
            &css,
            "the billboard's ratio from the first step",
        );
        let wide = numbers(
            &[
                "@media (min-width: ",
                "px) { .hero { aspect-ratio: ",
                " / ",
                "; } }",
            ],
            &css,
            "the billboard's ratio from the second step",
        );
        // `aspect-ratio` is written width over height and the reference states height over width.
        [
            (0.0, narrow[1] / narrow[0]),
            (middle[0], middle[2] / middle[1]),
            (wide[0], wide[2] / wide[1]),
        ]
    }

    /// The billboard is a ratio of its own width at every width the reference measured, and this
    /// page's three bands give the height that file read at each of them.
    ///
    /// The correction of 2026-11-13 is what this is for. That file first said the height below
    /// 1024 is set by the content, and this page carried that claim as `aspect-ratio: auto` with a
    /// 76vh floor under 720. It is a ratio there too, in two further bands, and both are
    /// reproduced from two readings each that the file quotes - so every number this test holds is
    /// read out of it rather than restated here.
    ///
    /// Fed the reference's own billboard width, not this page's: the margins differ, so what is
    /// under test is the ratio and not a width this page never draws.
    #[test]
    fn the_billboard_keeps_the_reference_s_ratio_at_every_width_it_measured() {
        let reference = source(WEB_REFERENCE);
        let close = tolerance(&reference);
        let bands = billboard_bands();
        // Every row of the wide table, read as a table rather than width by width, so a width
        // measured into it later is held too and none can be quietly skipped.
        let table = reference_section(&reference, "## The one rule that produces a row");
        let mut read = table_billboards(&table);
        assert_eq!(
            read.len(),
            9,
            "that table has stated nine widths since it was written; a tenth is a new \
             measurement and wants reading before it is held to"
        );
        // And the readings the two narrow bands are each derived from, quoted in that file's own
        // "Hero" section as height over width at a stated viewport.
        let heroes = reference_section(&reference, "### Hero");
        let narrow = numbers(
            &[
                "hero height = ",
                " x hero width** at every width from 160 to 599 (",
                "/",
                " at 390, ",
                "/",
                " at 570, ",
                "/",
                " at 320)",
            ],
            &heroes,
            "the narrow band's own readings",
        );
        let middle = numbers(
            &[
                "hero height = ",
                " x hero width** from 600 to 768 (",
                "/",
                " at 600, ",
                "/",
                " at 768)",
            ],
            &heroes,
            "the middle band's own readings",
        );
        for (at, pair) in [(390.0, 1), (570.0, 3), (320.0, 5)] {
            read.push((at, narrow[pair + 1], narrow[pair]));
        }
        for (at, pair) in [(600.0, 1), (768.0, 3)] {
            read.push((at, middle[pair + 1], middle[pair]));
        }
        for (wide, billboard, tall) in read {
            let ratio = bands
                .iter()
                .filter(|(from, _)| wide >= *from)
                .map(|(_, ratio)| *ratio)
                .next_back()
                .expect("a band for every width");
            let drawn = billboard * ratio;
            assert!(
                (drawn - tall).abs() <= close,
                "at {wide} this page draws a {billboard} x {drawn} billboard against the \
                 measured {billboard} x {tall}"
            );
        }
        // And the ratio each narrow band is stated as is the one this page draws, to the four
        // digits that file carries it to.
        for (said, drawn) in [(narrow[0], bands[0].1), (middle[0], bands[1].1)] {
            assert_eq!((drawn * 10_000.0).round() / 10_000.0, said);
        }
    }

    /// Where the billboard's ratio steps: the first is the width the reference bracketed to 1 px,
    /// and the second is the boundary this page already has, inside the bounds that file leaves
    /// open.
    #[test]
    fn the_billboard_steps_where_something_measured_or_where_this_page_already_steps() {
        let reference = source(WEB_REFERENCE);
        let bands = billboard_bands();
        let first = one_number(
            &["### The first step is at exactly ", " Bracketed"],
            &reference,
            "the reference's first step",
        );
        assert_eq!(bands[1].0, first);
        // The second is unbracketed: 1.2048 is measured at 768 and 0.4587 at 1024, so it may sit
        // anywhere in between. It sits where this page's margin changes class.
        let heroes = reference_section(&reference, "### Hero");
        let widest_narrow = one_number(
            &[" x hero width** from 600 to ", " ("],
            &heroes,
            "the widest width measured at the middle ratio",
        );
        let narrowest_wide = one_number(
            &[
                "constant to four decimals at seven widths from ",
                " to 1920",
            ],
            &reference,
            "the narrowest width measured at the wide ratio",
        );
        let step = bands[2].0;
        assert!(
            widest_narrow < step && step <= narrowest_wide,
            "the billboard steps at {step}, outside the {widest_narrow} to {narrowest_wide} the \
             reference leaves open"
        );
        assert_ne!(
            WidthClass::of(step - 1.0),
            WidthClass::of(step),
            "{step} is not a boundary this page has"
        );
        // The three bands rise, and each is flatter than the one below it.
        for pair in bands.windows(2) {
            assert!(pair[0].0 < pair[1].0 && pair[0].1 > pair[1].1);
        }
    }

    /// A row's name is the size the reference read at that width, in all three bands.
    ///
    /// Each size is read out of the section it was measured in: that file states a row's rhythm
    /// twice, at 1920 and at 390, in two sections whose sentences differ only in their numbers.
    #[test]
    fn a_row_s_name_is_the_size_the_reference_read_at_that_width() {
        let reference = source(WEB_REFERENCE);
        let css = source(SITE);
        // Each shape stops before the bold that closes the reading rather than naming it: a piece
        // of a shape is flattened the way a line of source is, so a piece that opened with the
        // `*` of that bold would lose it and match nothing. The empty piece at the end is the
        // "and then anything" `numbers` documents.
        let phone = numbers(
            &["Row title y ", "-", " (**", " tall, ", " px / ", ""],
            &reference_section(&reference, "### Vertical rhythm of the rows, at 390"),
            "the row's name at 390",
        );
        let desk = numbers(
            &["Row title y ", "-", " (**", " tall, ", " px / ", ""],
            &reference_section(&reference, "## Vertical rhythm of the rows, at 1920"),
            "the row's name at 1920",
        );
        let stepped = numbers(
            &[
                "At **600** the page margin is ",
                ", the count across is ",
                ", row titles are ",
                " px / ",
                " and the hero",
            ],
            &reference,
            "the row's name at the first step",
        );
        // Under the first step: the size and the box that file measured, and the weight.
        assert_eq!(
            numbers(
                &[
                    "& h2 { padding: 0 var(--edge); margin-bottom: calc(var(--unit) * 1.5); \
                     font-size: ",
                    "px; line-height: ",
                    "px; font-weight: ",
                    "; }",
                ],
                &css,
                "the row's name under the first step",
            ),
            vec![phone[3], phone[2], phone[4]]
        );
        // From the first step: the size that file read at exactly 600, and no box, because it
        // measured none there.
        assert_eq!(
            numbers(
                &[
                    "@media (min-width: ",
                    "px) { .row h2 { font-size: ",
                    "px; line-height: normal; } }",
                ],
                &css,
                "the row's name from the first step",
            ),
            vec![600.0, stepped[2]]
        );
        // From the second step: the size and the box measured at 1920.
        assert_eq!(
            numbers(
                &[
                    "@media (min-width: ",
                    "px) { .row h2 { font-size: ",
                    "px; line-height: ",
                    "px; } }",
                ],
                &css,
                "the row's name from the second step",
            ),
            vec![840.0, desk[3], desk[2]]
        );
        // One weight at all three widths, which is why it is written once.
        assert_eq!(phone[4], desk[4]);
        assert_eq!(phone[4], stepped[3]);
        // The three sizes rise with the window, and the two the reference brackets to 1 px differ.
        assert!(phone[3] < stepped[2] && stepped[2] < desk[3]);
    }

    /// The billboard's two controls stand one above the other under the first step and side by
    /// side above it, at the gap the reference measured on each side of it.
    #[test]
    fn the_billboard_s_controls_stack_under_the_first_step() {
        let reference = source(WEB_REFERENCE);
        let css = source(SITE);
        let unit = unit();
        let stacked = one_number(
            &["310 x 40**, stacked with an ", " px gap"],
            &reference,
            "the gap between the stacked controls",
        );
        let beside = one_number(
            &["with a white pill 108 wide, a ", " px gap, then a grey"],
            &reference,
            "the gap between the controls side by side",
        );
        let column = one_number(
            &[
                ".hero .row-of-buttons { flex-direction: column; gap: calc(var(--unit) * ",
                "); }",
            ],
            &css,
            "the stacked controls' gap",
        );
        let row = one_number(
            &[
                ".hero .row-of-buttons { flex-direction: row; gap: calc(var(--unit) * ",
                "); }",
            ],
            &css,
            "the side by side controls' gap",
        );
        assert_eq!(column * unit, stacked);
        assert_eq!(row * unit, beside);
        // The row of them turns back at the width the reference bracketed the step to.
        assert!(
            flat(&css).contains(
                "@media (min-width: 600px) { .hero { aspect-ratio: 1 / 1.2048; padding-inline: \
                 var(--edge); } .hero .row-of-buttons { flex-direction: row;"
            ),
            "the controls stand side by side from some other width"
        );
        // Stacked, each is the full content width, which is what a column of flex items is
        // without a width stated: nothing here may state one.
        assert!(
            !flat(&css).contains(
                ".hero .row-of-buttons { flex-direction: column; gap: \
                 calc(var(--unit) * 2); width"
            ),
            "a stacked control states a width of its own"
        );
    }

    /// One press of a row's control moves it the distance the reference measured, at both of the
    /// widths it measured one.
    ///
    /// Both readings are held, and so is the derivation behind them: that file states the 390
    /// distance twice over - as the content width plus a gutter and as the count across times the
    /// pitch - and this surface computes the second, so the test reads the first out of the file
    /// and checks the two agree there before holding this code to either.
    #[test]
    fn a_row_pages_by_the_reference_s_own_distance() {
        let reference = source(WEB_REFERENCE);
        let close = tolerance(&reference);
        // At 390: the distance, the content width, the count across and the pitch, all in one
        // sentence of that file.
        let phone = numbers(
            &[
                "moves the row **",
                " px at 390**, which is the content width ",
                " plus one gutter, and also exactly ",
                " x the ",
                " pitch",
            ],
            &reference,
            "the paging distance at 390",
        );
        let (moved, content, across, pitch) = (phone[0], phone[1], phone[2], phone[3]);
        // The file's own two forms of the same distance: the content box plus one gutter, and a
        // whole page of tiles. The gutter is that file's 8.
        let gutter = one_number(
            &["The gutter is **", " px at every width measured"],
            &reference,
            "the reference's gutter",
        );
        assert_eq!(content + gutter, moved, "the file's two forms disagree");
        assert!((page_step(across as usize, pitch) - moved).abs() <= close);
        // And at 1920, where the file states the page in tiles and the table states the pitch.
        let wide = numbers(
            &["one full page of ", " tiles = ", " px at 1920"],
            &reference,
            "the paging distance at 1920",
        );
        let table = reference_section(&reference, "## The one rule that produces a row");
        let wide_pitch = numbers(
            &["| 1920 | ", " | ", " | ", " x ", " | ", " | "],
            &table,
            "the row at 1920",
        )[4];
        assert!((page_step(wide[0] as usize, wide_pitch) - wide[1]).abs() <= close);
        // The count the file pages by at each width is the count this page draws there, which is
        // what makes `tiles_across` the right number to move by.
        assert_eq!(super::super::empty::tiles_across(390.0) as f64, across);
        assert_eq!(super::super::empty::tiles_across(1920.0) as f64, wide[0]);
        // A row with nothing in it moves nothing, rather than dividing by it.
        assert_eq!(page_step(0, 175.0), 0.0);
    }

    /// The band a row's control stands in is the page margin wide, which is the rule the
    /// reference states, and the arrow in it is the size that file measured.
    #[test]
    fn a_row_s_control_stands_in_a_band_the_page_margin_wide() {
        let reference = source(WEB_REFERENCE);
        let css = source(SITE);
        // The rule, and the two readings it was drawn from.
        let scrims = numbers(
            &[
                "scrim width equals the page margin** at both widths measured (",
                " at 1920, ",
                " at 390)",
            ],
            &reference,
            "the scrim at both widths",
        );
        let margins = reference_section(&reference, "## The one rule that produces a row");
        // Each is that width's own page margin in the table, which is what makes it a rule and
        // not two numbers.
        for (wide, scrim) in [(1920.0, scrims[0]), (390.0, scrims[1])] {
            let table = if wide > 599.0 {
                &margins
            } else {
                &reference_section(&reference, "### The one rule still holds")
            };
            let margin = numbers(
                &[&format!("| {wide} | "), " | "],
                table,
                "the page margin at this width",
            )[0];
            assert_eq!(scrim, margin, "the scrim at {wide} is not the page margin");
        }
        // So this page draws it from the one place its own margin lives.
        assert!(
            flat(&css).contains("width: var(--edge); display: grid; place-items: center;"),
            "the band is not the page margin wide"
        );
        // The arrow: 28 x 56 at both widths that file measured one.
        let arrow = numbers(
            &["The control is an arrow **", " x ", ""],
            &reference,
            "the arrow at 390",
        );
        let wide_arrow = numbers(
            &["The control is an arrow ", " x ", " inside a **"],
            &reference,
            "the arrow at 1920",
        );
        assert_eq!(arrow, wide_arrow, "the arrow is one size at both widths");
        assert_eq!(
            numbers(
                &[
                    "font-size: ",
                    "px; line-height: ",
                    "px; color: var(--text);"
                ],
                &css,
                "the arrow this page draws",
            ),
            arrow
        );
    }

    /// Which of a row's controls stand, and when.
    #[test]
    fn a_row_offers_no_control_that_would_move_nothing() {
        // Page 1: nothing behind, a page ahead. The reference states the first half of this.
        assert_eq!(controls(0.0, 390.0, 1200.0), (false, true));
        // Part way along: both.
        assert_eq!(controls(350.0, 390.0, 1200.0), (true, true));
        // At the end: nothing ahead.
        assert_eq!(controls(810.0, 390.0, 1200.0), (true, false));
        // A scroller that lands a fraction short of its own length is at the end.
        assert_eq!(controls(809.5, 390.0, 1200.0), (true, false));
        // A row shorter than the window has neither.
        assert_eq!(controls(0.0, 390.0, 300.0), (false, false));
    }

    /// The billboard's content sits the reference's own inset inside the card, which is not the
    /// page margin: that file measures 16 inside a 24 px margin at 390 and 40 inside a 60 px one
    /// at 1920.
    ///
    /// The wide half of it is held to `sizes.rs`'s own margin rather than to a second number here:
    /// this page draws 40 there, which is the inset the reference measured at 1920, and it draws
    /// it out of `--edge`.
    #[test]
    fn the_billboard_s_content_sits_the_reference_s_inset_inside_it() {
        let reference = source(WEB_REFERENCE);
        let css = source(SITE);
        let phone = one_number(
            &["Content sits **", " px** inside the card (x 40), against "],
            &reference_section(&reference, "### Hero"),
            "the inset at 390",
        );
        let desk = one_number(
            &["Content sits ", " px inside the card (x 100)"],
            &reference,
            "the inset at 1920",
        );
        let drawn = one_number(
            &[
                "border-radius: var(--radius-panel); padding: 0 calc(var(--unit) * ",
                ") calc(var(--unit) * 16);",
            ],
            &css,
            "the billboard's own inset",
        );
        assert_eq!(drawn * unit(), phone);
        // Above the first step the inset is the page margin, and in the widest class that margin
        // is the number the reference measured at 1920.
        assert!(
            flat(&css).contains(".hero { aspect-ratio: 1 / 1.2048; padding-inline: var(--edge); }"),
            "the billboard's inset stops being the page margin above the first step"
        );
        assert_eq!(f64::from(WidthClass::Expanded.page_edge()), desk);
        assert!(phone < desk, "the inset grows with the window");
    }

    /// Decorative rank and badge spans live outside `<button class="card">` so Axe's visible-text
    /// extraction for WCAG 2.5.3 never prepends the rank numeral or appends the badge to the
    /// button's visible label.
    #[test]
    fn card_button_keeps_decorative_overlays_outside_the_button() {
        let code = source("apps/web/src/surfaces/browse.rs");
        let prod = code.split_once("#[cfg(test)]").expect("tests").0;
        let card = prod.split_once("fn Card(").expect("Card component").1;
        let button = card
            .split_once("<button class=\"card\"")
            .expect("card button")
            .1
            .split_once("</button>")
            .expect("closing button")
            .0;
        assert!(
            !button.contains("class=\"rank\"") && !button.contains("class=\"badge\""),
            "decorative overlays must stay outside <button class=\"card\">: {button}"
        );
        let wrapper = card.split_once("</button>").expect("after button").1;
        assert!(
            wrapper.contains("class=\"rank\"") && wrapper.contains("class=\"badge\""),
            "expected .rank and .badge inside .tile beside <button>"
        );
        let css = flat(&source(SITE));
        assert!(css.contains(".tile {") && css.contains("pointer-events: none;"));
    }
}
