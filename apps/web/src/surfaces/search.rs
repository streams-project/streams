//! The search field, and the suggestions under it.
//!
//! `/v1/search/suggest` has been on the server since the contract carried it and no surface has
//! ever called it: not this page, not the phone, not the television. This file is its first
//! caller. What it answers is a list of plain names - `contract::screen::Suggestions` is
//! `{ items: Vec<String> }`, with no key, no kind and no score - so a chosen suggestion can only be
//! handed back as text, which is what a search field wants anyway. A blank `q` is answered with the
//! recommended mix rather than refused, which is why the field asks the moment it is entered; a
//! `q` that matches nothing is answered with an empty list and a 200, which is why an empty list
//! closes the list rather than drawing a box with nothing in it.

use contract::core_msg::HttpMethod;
use contract::screen::{Screen, Suggestions};
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;

use super::label_text;
use crate::host;
use crate::state::Shared;

/// How long after the last keystroke the field asks, in milliseconds.
///
/// The television's, read out of `tv/search/Search.kt` by the test below. Its search has settled
/// 350 ms after the last press since it was written, Apple's television surface carries the same
/// 350 (`apps/apple/Sources/SurfacesTV/search/Search.swift:6`) and so does Harmony's touch search
/// (`apps/harmony/surfaces/touch/Sizes.ets:316`, and that surface's README says in so many words
/// that it took the behaviour from the television). Three copies and nothing held any of them to
/// any other; this is the fourth and the first that fails when the television's number moves.
///
/// The television's 350 carries no measurement citation anywhere in this repository - it is a
/// behaviour this project chose, not a number read off the reference, and `reference/` measures no
/// typing at all. So it is carried across unchanged rather than reasoned about, the way the page
/// margin's 40 is in `sizes.rs`. What does hold it from above is the server: the guard meters every
/// non-start route at 600 calls per address per minute, shared with every other call this page
/// makes, and a field that asked on each keystroke would spend that budget on one viewer typing. At
/// one ask per settle the worst case is 60000/350 asks a minute, which the second test below holds
/// against the server's own number.
const SETTLE_MS: i32 = 350;

/// The id of the list, so the field can name it as the thing it controls.
const LIST_ID: &str = "search-suggestions";

/// The id of one option, so the field can name the one it is on.
fn option_id(at: usize) -> String {
    format!("{LIST_ID}-{at}")
}

/// The guard against an older answer landing on a newer one.
///
/// Every ask is numbered, and the number goes with the reply. A reply is applied only while its
/// number is still the newest issued: the moment the viewer types again the number moves, and every
/// answer still in flight - including the one for what they typed a second ago - is dropped on
/// arrival. So a slow answer for "t" cannot replace a fast answer for "twin", and it cannot land on
/// an empty list and sit there either, because by then the ask for "twin" owns the list.
///
/// `host::ask` is fire and forget: a request already sent cannot be called back, and the one
/// `AbortController` in this crate belongs to the player. The number is therefore the whole of the
/// guarantee, which is why it is a type with a test and not a comparison written inline.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Race {
    sent: u32,
}

impl Race {
    /// Number the next ask.
    fn next(&mut self) -> u32 {
        self.sent = self.sent.wrapping_add(1);
        self.sent
    }

    /// True while this reply is the one the field is still waiting for.
    const fn wanted(self, reply: u32) -> bool {
        reply == self.sent
    }
}

/// Where Down and Up put the highlight in a list of `count`, from wherever it is now.
///
/// Nothing highlighted and Down takes the first; nothing highlighted and Up takes the last; both
/// wrap at the ends. A list with nothing in it has nowhere to go.
fn walked(at: Option<usize>, down: bool, count: usize) -> Option<usize> {
    if count == 0 {
        return None;
    }
    Some(match (at, down) {
        (None, true) => 0,
        (None, false) => count - 1,
        (Some(at), true) => (at + 1) % count,
        (Some(at), false) => (at + count - 1) % count,
    })
}

/// What Enter searches for: the highlighted suggestion, or what the viewer typed.
fn taken<'a>(at: Option<usize>, items: &'a [String], typed: &'a str) -> &'a str {
    at.and_then(|at| items.get(at))
        .map_or(typed, String::as_str)
}

/// Whether the list is drawn. It says nothing when it has nothing: an answer with no items closes
/// it rather than leaving an empty box under the field.
const fn drawn(open: bool, count: usize) -> bool {
    open && count > 0
}

/// What the field holds while the viewer types.
#[derive(Clone, Copy)]
struct Ahead {
    /// What arrived for the newest ask. A keystroke does not clear it: the drawn list is replaced
    /// when an answer arrives and at no other moment, so nothing flashes between two answers.
    items: RwSignal<Vec<String>>,
    /// False once Escape closes the list, or a search runs. A keystroke opens it again.
    open: RwSignal<bool>,
    /// Which suggestion the keyboard is on.
    at: RwSignal<Option<usize>>,
    /// The race guard.
    race: RwSignal<Race>,
    /// The timeout waiting out the settle, so the next keystroke can cancel it.
    waiting: RwSignal<Option<i32>>,
}

impl Ahead {
    fn new() -> Self {
        Self {
            items: RwSignal::new(Vec::new()),
            open: RwSignal::new(false),
            at: RwSignal::new(None),
            race: RwSignal::new(Race::default()),
            waiting: RwSignal::new(None),
        }
    }

    /// Stop waiting, so a keystroke that arrives inside the settle cancels the ask it would have
    /// made and the field asks once for what was finally typed.
    fn stop(self) {
        if let Some(old) = self.waiting.get_untracked() {
            host::window().clear_timeout_with_handle(old);
        }
        self.waiting.set(None);
    }

    /// Close the list and forget where the keyboard was, leaving what was typed alone.
    ///
    /// The number moves as well, which is what makes Escape stick. Without it a list closed while
    /// an ask was in flight opened again the moment that answer landed, a second or so later, with
    /// nothing the viewer did in between - the one deliberate act on this list undone by the
    /// network. Closing is therefore the same thing to an answer in flight as typing again is: it
    /// is no longer wanted.
    fn shut(self) {
        self.stop();
        self.race.update(|race| {
            race.next();
        });
        self.open.set(false);
        self.at.set(None);
    }

    /// Number the next ask, so its answer can be told from the answer to any other.
    fn numbered(self) -> u32 {
        self.race.try_update(Race::next).unwrap_or_default()
    }

    /// An answer landing. The only thing in this file that writes the drawn list, which is what
    /// makes the guard on its first line the whole of the guarantee rather than one of several.
    fn landed(self, mine: u32, items: Vec<String>) {
        if !self.race.get_untracked().wanted(mine) {
            return;
        }
        self.at.set(None);
        self.open.set(!items.is_empty());
        self.items.set(items);
    }

    /// Ask for the suggestions of `words`, now.
    fn ask(self, words: String) {
        let mine = self.numbered();
        let query: String = js_sys::encode_uri_component(words.trim()).into();
        let path = format!("/v1/search/suggest?q={query}");
        host::ask(HttpMethod::Get, &path, None, move |status, body| {
            let answer = (200..300)
                .contains(&status)
                .then(|| serde_json::from_str::<Suggestions>(&body).ok())
                .flatten();
            self.landed(mine, answer.map(|answer| answer.items).unwrap_or_default());
        });
    }

    /// Ask for whatever `text` holds once the field has been still for the settle.
    ///
    /// The field is read when the timer fires and not when it is set, so this does not rest on
    /// which of two listeners on the same `input` event runs first: by the time the settle is over
    /// the field holds the keystroke whatever order they went on in.
    fn ask_soon(self, text: RwSignal<String>) {
        self.stop();
        let fire = Closure::once_into_js(move || {
            self.waiting.set(None);
            self.ask(text.get_untracked());
        });
        let handle = host::window()
            .set_timeout_with_callback_and_timeout_and_arguments_0(fire.unchecked_ref(), SETTLE_MS)
            .ok();
        self.waiting.set(handle);
    }
}

/// The search field, with the suggestions under it.
///
/// The field is a combobox: it owns the list, says whether the list is open, and names the option
/// the keyboard is on. That is how a reader is told. The options are list items and never buttons,
/// and the pointer's press on one is refused its default, so focus stays in the field through every
/// one of Down, Up, Enter, Escape and a click - which is the point: a list a reader can hear but
/// has to leave the field to reach is no better than one that is never announced.
#[component]
pub fn SearchBox(asked: RwSignal<String>) -> impl IntoView {
    let shared = expect_context::<Shared>();
    let text = RwSignal::new(String::new());
    let ahead = Ahead::new();
    let search = move |words: String| {
        ahead.shut();
        text.set(words.clone());
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
    let run = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        search(text.get());
    };
    // The field is entered with nothing typed, and a blank `q` is what the route answers with the
    // recommended mix, so the first ask is the one this page makes on arriving. In an effect and
    // not in the body: the body runs while the view is being built, and the host this asks through
    // is started by an effect of its own one level up.
    Effect::new(move |_| ahead.ask(text.get_untracked()));
    let typed = move |_| {
        ahead.at.set(None);
        ahead.ask_soon(text);
    };
    let pressed = move |event: leptos::ev::KeyboardEvent| {
        let count = ahead.items.with_untracked(Vec::len);
        let shown = drawn(ahead.open.get_untracked(), count);
        match event.key().as_str() {
            // Down and Up walk the list. They never move the caret while the list is open, and
            // Down opens a list that Escape closed rather than doing nothing.
            key @ ("ArrowDown" | "ArrowUp") if count > 0 => {
                event.prevent_default();
                if shown {
                    ahead
                        .at
                        .set(walked(ahead.at.get_untracked(), key == "ArrowDown", count));
                } else {
                    ahead.open.set(true);
                    ahead.at.set(None);
                }
            }
            // Enter takes the highlighted suggestion. With nothing highlighted it is the form's
            // own submit, left to run, which searches for what was typed.
            "Enter" if shown && ahead.at.get_untracked().is_some() => {
                event.prevent_default();
                let words = ahead
                    .items
                    .with_untracked(|items| taken(ahead.at.get_untracked(), items, "").to_owned());
                search(words);
            }
            // Escape closes the list and leaves what was typed where it is.
            "Escape" if shown => {
                event.prevent_default();
                ahead.shut();
            }
            _ => {}
        }
    };
    // The field is labelled rather than given a placeholder. A placeholder is not a label - it is
    // gone the moment anything is typed, and a reader is not promised it - and the one that was
    // here, "Titles, people, genres", was English written into this file. `strings/` has no key for
    // those words; it has the name of the page, which is what the field is for.
    view! {
        <form class="search" on:submit=run>
            <label class="said" for="search-field">{move || label_text(shared, "nav.search")}</label>
            <div class="search-field">
                <input
                    id="search-field"
                    type="search"
                    autofocus
                    autocomplete="off"
                    role="combobox"
                    aria-autocomplete="list"
                    aria-controls=LIST_ID
                    aria-expanded=move || {
                        drawn(ahead.open.get(), ahead.items.with(Vec::len)).to_string()
                    }
                    aria-activedescendant=move || ahead.at.get().map(option_id)
                    bind:value=text
                    on:input=typed
                    on:keydown=pressed
                />
                {move || {
                    let items = ahead.items.get();
                    drawn(ahead.open.get(), items.len()).then(|| view! {
                        <ul
                            id=LIST_ID
                            class="suggestions"
                            role="listbox"
                            aria-label=move || label_text(shared, "search.recommended")
                        >
                            {items.into_iter().enumerate().map(|(place, name)| {
                                let chosen = move || ahead.at.get() == Some(place);
                                let words = name.clone();
                                view! {
                                    <li
                                        id=option_id(place)
                                        class="suggestion"
                                        class:here=chosen
                                        role="option"
                                        aria-selected=move || chosen().to_string()
                                        // The pointer's press is refused its default so the field
                                        // never loses focus to the list it owns.
                                        on:mousedown=|event: leptos::ev::MouseEvent| {
                                            event.prevent_default();
                                        }
                                        on:click=move |_| search(words.clone())
                                    >
                                        {name}
                                    </li>
                                }
                            }).collect_view()}
                        </ul>
                    })
                }}
            </div>
            <button class="solid" type="submit">{move || label_text(shared, "nav.search")}</button>
        </form>
    }
}

#[cfg(test)]
mod tests {
    use leptos::prelude::{GetUntracked, Owner, Set};

    use super::{Ahead, LIST_ID, SETTLE_MS, drawn, option_id, taken, walked};
    use crate::surfaces::tests_support::{english, one_number, source};

    const TV_SEARCH: &str = "apps/android/surfaces/tv/src/main/kotlin/streams/tv/search/Search.kt";
    const SERVER_STATE: &str = "server/crates/server-api/src/state.rs";
    const THIS_FILE: &str = "apps/web/src/surfaces/search.rs";

    /// The field's own opening tag, out of this file's source.
    ///
    /// Found by the id its label names rather than by the tag's name: a test that wrote the tag's
    /// name as a literal would be read as a field of this surface by
    /// `surfaces::tests::every_field_and_menu_on_this_surface_has_words`, which scans the whole of
    /// every file here, tests included, and would then report this test's own text as an unlabelled
    /// field. Watched happening on 2026-11-13.
    fn field_of(body: &str) -> &str {
        body.split_once("id=\"search-field\"")
            .expect("the field")
            .1
            .split_once("/>")
            .expect("the end of the field")
            .0
    }

    /// An answer to an older query never replaces a newer one, whichever order the two land in.
    ///
    /// The field cannot cancel a request - `host::ask` takes a reply and hands back no handle to
    /// cancel with - so the number is the whole of the guarantee, and this exercises the field's
    /// own state through the field's own two calls rather than a model of them. `Ahead` is signals
    /// and nothing else until something asks, so it runs here under a bare owner with no document.
    #[test]
    fn a_slower_answer_for_an_older_query_never_lands_on_a_newer_one() {
        Owner::new().with(|| {
            let names = |items: &[&str]| -> Vec<String> {
                items.iter().map(|name| (*name).to_owned()).collect()
            };
            let ahead = Ahead::new();
            // "t" is asked for, then "tw", then "twi": three in flight at once.
            let (first, second, third) = (ahead.numbered(), ahead.numbered(), ahead.numbered());
            assert_eq!((first, second, third), (1, 2, 3));
            // The newest answers first, which is the point: it is the one the viewer is owed.
            ahead.landed(third, names(&["Twin Film"]));
            assert_eq!(ahead.items.get_untracked(), names(&["Twin Film"]));
            assert!(ahead.open.get_untracked());
            // Now the two slow ones land, in either order. Neither may touch the list.
            ahead.landed(first, names(&["T Film"]));
            ahead.landed(second, names(&["Tw Film"]));
            assert_eq!(
                ahead.items.get_untracked(),
                names(&["Twin Film"]),
                "an answer for what was typed two keystrokes ago replaced the newest answer"
            );
        });
        // An older answer that lands before the newest one is dropped too, rather than drawn and
        // then swapped: the field waits on the answer it asked for last and draws no other.
        Owner::new().with(|| {
            let ahead = Ahead::new();
            let older = ahead.numbered();
            let newer = ahead.numbered();
            ahead.landed(older, vec![String::from("T Film")]);
            assert!(
                ahead.items.get_untracked().is_empty() && !ahead.open.get_untracked(),
                "an answer already overtaken was drawn"
            );
            ahead.landed(newer, vec![String::from("Twin Film")]);
            assert_eq!(ahead.items.get_untracked(), vec![String::from("Twin Film")]);
        });
        // And Escape moves the number, which is what makes Escape stick: the ask that was in
        // flight when the list closed lands on a number nobody holds and is dropped, instead of
        // opening the list again a second later with nothing the viewer did in between.
        Owner::new().with(|| {
            let ahead = Ahead::new();
            let sent = ahead.numbered();
            ahead.landed(sent, vec![String::from("Twin Film")]);
            assert!(ahead.open.get_untracked());
            let again = ahead.numbered();
            ahead.shut();
            ahead.landed(again, vec![String::from("Tw Film")]);
            assert!(
                !ahead.open.get_untracked(),
                "the answer to an ask the viewer has since closed re-opened the list"
            );
            assert_eq!(
                ahead.items.get_untracked(),
                vec![String::from("Twin Film")],
                "the answer to an ask the viewer has since closed was drawn"
            );
        });
    }

    /// The settle is the television's own number, read out of the television's own file.
    #[test]
    fn the_settle_after_a_keystroke_is_the_television_s() {
        let kotlin = source(TV_SEARCH);
        assert_eq!(
            one_number(
                &["private const val SETTLE_MS = ", "L"],
                &kotlin,
                "the settle"
            ),
            f64::from(SETTLE_MS)
        );
        // And the two surfaces that took the same behaviour from the television still carry it, so
        // this file is the fourth copy of one number and not the second of two.
        assert_eq!(
            one_number(
                &["private let settle: UInt64 = ", "\n"],
                &source("apps/apple/Sources/SurfacesTV/search/Search.swift"),
                "Apple's settle",
            ),
            f64::from(SETTLE_MS)
        );
        assert_eq!(
            one_number(
                &["export const SETTLE_MS: number = ", ";"],
                &source("apps/harmony/surfaces/touch/Sizes.ets"),
                "Harmony's settle",
            ),
            f64::from(SETTLE_MS)
        );
    }

    /// The share of the server's per-address budget the type-ahead is allowed to want.
    ///
    /// Stated here because it is a rule this file keeps and not a measurement of anything: one
    /// caller on one screen may ask for at most a third of what the whole page is metered at, so
    /// two thirds are still there for the screens, the pages of rows, the commands and the player
    /// while a viewer types. Written down and checked rather than left as "well under".
    const BUDGET_SHARE: f64 = 3.0;

    /// Asking once per settle stays inside the budget the server meters this page by.
    ///
    /// The second half of why the field waits at all, and the half that does not rest on another
    /// surface's choice. A field asking on each keystroke has no bound at all - it asks as fast as
    /// a person types, and the guard layer that meters it is shared with every other call this page
    /// makes. One ask per settle has a bound, `60000 / SETTLE_MS`, and the bound is what this holds
    /// against the server's own number.
    #[test]
    fn a_field_that_waits_this_long_stays_inside_the_server_s_own_budget() {
        let per_minute = one_number(
            &["const CALLS_PER_MINUTE: u32 = ", ";"],
            &source(SERVER_STATE),
            "the server's budget per address",
        );
        let worst = 60_000.0 / f64::from(SETTLE_MS);
        assert!(
            worst * BUDGET_SHARE <= per_minute,
            "typing without pause asks {worst} times a minute against a budget of {per_minute}"
        );
        // The bound is only a bound if a keystroke cancels the ask the one before it set going.
        // Without that each keystroke leaves a timer behind and they all fire, which is the
        // per-keystroke field this settle exists to not be - at one settle's delay, and still
        // counted against the number above. Nothing in these tests can run a browser's timer, so
        // what is held is that the one function that sets a timer clears the old one first.
        let body = source(THIS_FILE)
            .split_once("#[cfg(test)]")
            .expect("the tests")
            .0
            .to_owned();
        let soon = body
            .split_once("fn ask_soon(")
            .expect("the settle")
            .1
            .split_once("\n    }")
            .expect("the end of it")
            .0;
        assert!(
            soon.trim_start()
                .starts_with("self, text: RwSignal<String>) {\n        self.stop();"),
            "a keystroke inside the settle leaves the ask before it running: {soon}"
        );
    }

    /// Down and Up walk the list, from nothing and around the ends.
    #[test]
    fn down_and_up_walk_the_suggestions_and_wrap() {
        // Down from nothing takes the first, Up from nothing takes the last.
        assert_eq!(walked(None, true, 3), Some(0));
        assert_eq!(walked(None, false, 3), Some(2));
        // And then one at a time.
        assert_eq!(walked(Some(0), true, 3), Some(1));
        assert_eq!(walked(Some(1), true, 3), Some(2));
        assert_eq!(walked(Some(2), false, 3), Some(1));
        // Both ends wrap, so the list has no dead end in either direction.
        assert_eq!(walked(Some(2), true, 3), Some(0));
        assert_eq!(walked(Some(0), false, 3), Some(2));
        // One suggestion stays where it is; a list with nothing in it has nowhere to go.
        assert_eq!(walked(Some(0), true, 1), Some(0));
        assert_eq!(walked(None, true, 0), None);
        assert_eq!(walked(Some(0), false, 0), None);
    }

    /// Enter takes the highlighted suggestion, and what was typed when none is highlighted.
    #[test]
    fn enter_takes_the_highlighted_suggestion_or_what_was_typed() {
        let items = vec![String::from("Twin Film"), String::from("Twin Series")];
        assert_eq!(taken(Some(0), &items, "twi"), "Twin Film");
        assert_eq!(taken(Some(1), &items, "twi"), "Twin Series");
        assert_eq!(taken(None, &items, "twi"), "twi");
        // A place the list no longer has falls back to what was typed rather than to a neighbour.
        assert_eq!(taken(Some(9), &items, "twi"), "twi");
        assert_eq!(taken(Some(0), &[], "twi"), "twi");
    }

    /// The list says nothing when it has nothing, and nothing flashes between two answers.
    ///
    /// Both halves, because the first on its own is a rule about a pure function that the field
    /// need never obey: `drawn` can be right while `landed` opens the list on an empty answer, and
    /// a test of `drawn` alone cannot see that. So the second half drives the field.
    #[test]
    fn an_answer_with_nothing_in_it_draws_no_box() {
        assert!(drawn(true, 1));
        assert!(
            !drawn(true, 0),
            "an open list with no items is an empty box"
        );
        assert!(!drawn(false, 3));
        Owner::new().with(|| {
            let ahead = Ahead::new();
            // A query that matches nothing is a 200 with an empty list, which closes the list.
            let nothing = ahead.numbered();
            ahead.landed(nothing, Vec::new());
            assert!(
                !ahead.open.get_untracked(),
                "an answer with nothing in it opened the list on an empty box"
            );
            // A list that is up stays up while the next answer is in flight - a keystroke writes
            // nothing here - and is replaced whole when that answer lands.
            let some = ahead.numbered();
            ahead.landed(some, vec![String::from("Twin Film")]);
            let next = ahead.numbered();
            assert_eq!(
                ahead.items.get_untracked(),
                vec![String::from("Twin Film")],
                "the list was emptied while the next answer was still in flight"
            );
            assert!(ahead.open.get_untracked());
            // And the highlight goes back to nothing when a new list arrives, so the mark is never
            // left on a row of a list that has been replaced under it.
            ahead.at.set(Some(0));
            ahead.landed(next, vec![String::from("Tw Film")]);
            assert_eq!(
                ahead.at.get_untracked(),
                None,
                "the highlight stayed on a place in a list that has been replaced"
            );
        });
    }

    /// Every key this list answers is answered where the field has focus.
    ///
    /// The four keys are named in one `match` on the field's own `keydown`, so a list that only a
    /// pointer can reach cannot pass. Read out of the source because there is no browser in these
    /// tests: what is checked is that the field, and not the document or an option, is the thing
    /// listening, which is also what keeps focus where it is.
    #[test]
    fn the_field_itself_answers_the_four_keys_the_list_needs() {
        let text = source(THIS_FILE);
        let body = text.split_once("#[cfg(test)]").expect("the tests").0;
        let field = field_of(body);
        assert!(
            field.contains("on:keydown=pressed"),
            "the keys are not answered on the field itself"
        );
        for key in ["ArrowDown", "ArrowUp", "Enter", "Escape"] {
            assert!(
                body.contains(&format!("\"{key}\"")),
                "the list does not answer {key}"
            );
        }
        // The pointer's press on an option is refused its default, which is the one thing that
        // keeps focus in the field when the list is used with a mouse.
        let option = body.split_once("role=\"option\"").expect("an option").1;
        assert!(
            option.contains("on:mousedown") && option.contains("event.prevent_default()"),
            "a press on an option takes focus out of the field"
        );
    }

    /// A reader is told the list is there, which of it they are on, and what it is called.
    ///
    /// The defect this is against: a list that appears under a field and is announced by nothing.
    /// The field is the combobox, so it carries the state; the list is named once and its options
    /// say which is current. Every piece is read off the source, including the one string, which
    /// has to be a key `strings/en.json` really has.
    #[test]
    fn the_field_and_the_list_say_what_they_are_and_where_the_keyboard_is() {
        let text = source(THIS_FILE);
        let body = text.split_once("#[cfg(test)]").expect("the tests").0;
        let field = field_of(body);
        for part in [
            "role=\"combobox\"",
            "aria-autocomplete=\"list\"",
            "aria-controls",
            "aria-expanded",
            "aria-activedescendant",
        ] {
            assert!(field.contains(part), "the field carries no {part}");
        }
        let list = body
            .split_once("role=\"listbox\"")
            .expect("the list")
            .1
            .split_once("</ul>")
            .expect("the end of the list")
            .0;
        assert!(list.contains("aria-label"), "the list has no name");
        assert!(
            list.contains("role=\"option\"") && list.contains("aria-selected"),
            "the list's items are not options that say which is current"
        );
        // The field names the list it controls, and an option's id is one the field can name.
        assert!(field.contains("aria-controls=LIST_ID"));
        assert!(option_id(2).starts_with(LIST_ID));
        assert_eq!(option_id(2), format!("{LIST_ID}-2"));
        assert!(
            english().contains_key("search.recommended"),
            "strings/en.json has no search.recommended, which is the list's only word"
        );
    }
}
