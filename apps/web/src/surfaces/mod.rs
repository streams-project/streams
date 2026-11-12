//! The screens. The core says which one shows; each draws what `Shared` holds.

mod browse;
mod door;
mod empty;
mod notifications;
pub mod sizes;
mod title;
mod watch;
pub mod words;

use contract::address;
use contract::core_msg::Target;
use contract::screen::{Image, ImageRole, ScreenName};
use contract::{EpisodeRef, TitleKey};
use leptos::prelude::*;

use crate::host;
use crate::state::Shared;

/// The address of a title's page. Film and series ids overlap, so the kind stays in the address
/// (`contract::address::TITLE`). The phone's `streams.shared.titleAddress`.
pub fn title_address(title: TitleKey) -> String {
    format!("{}{}-{}", address::TITLE, title.kind().word(), title.id())
}

/// The address that plays a title, and an episode of it where one is named: an episode has no id of
/// its own, so it is the season and the number (`contract::address::WATCH`).
pub fn watch_address(title: TitleKey, episode: Option<&EpisodeRef>) -> String {
    let (kind, id) = (title.kind().word(), title.id());
    match episode {
        Some(episode) => format!(
            "{}{kind}/{id}/{}/{}",
            address::WATCH,
            episode.season,
            episode.episode
        ),
        None => format!("{}{kind}/{id}", address::WATCH),
    }
}

/// The first image of the first role that has one.
pub fn image(images: &[Image], roles: &[ImageRole]) -> Option<String> {
    roles.iter().find_map(|role| {
        images
            .iter()
            .find(|image| image.role == *role)
            .map(|image| image.url.clone())
    })
}

/// The brand's name, as the server states it.
pub fn brand_name(shared: Shared) -> String {
    shared
        .hello
        .with(|hello| hello.as_ref().map(|hello| hello.brand.name.clone()))
        .unwrap_or_default()
}

/// The label a notice falls back to when the one it names has no words on this server. It is a
/// label and not a sentence: this file held an English sentence until 2026-11-12, which was the one
/// place in the surfaces where the product spoke for itself instead of reading `strings/`.
const FAILED: &str = "problem-failed";

/// The text of a label, as the server states it, or `None` where this server has no such label.
/// A screen whose whole point is the words - the search with no matches - draws nothing rather
/// than invent them.
pub fn maybe_text(shared: Shared, label: &str) -> Option<String> {
    let text = shared.strings.with(|strings| strings.get(label).cloned())?;
    Some(
        shared
            .hello
            .with(|hello| hello.as_ref().map(|hello| hello.brand.fill(&text)))
            .unwrap_or(text),
    )
}

/// The text of a label, as the server states it, or nothing at all.
///
/// Empty rather than English: a control with no words is reached by its own `aria-label`, and a
/// line of text with no words draws nothing, which is what every caller here does with it.
pub fn label_text(shared: Shared, label: &str) -> String {
    maybe_text(shared, label).unwrap_or_default()
}

/// A page the core named but this surface has nothing to draw for: its heading, the line under it,
/// and the way home.
#[component]
fn Message(label: &'static str) -> impl IntoView {
    let shared = expect_context::<Shared>();
    let help = format!("{label}.help");
    view! {
        <div class="message">
            <h1>{move || label_text(shared, label)}</h1>
            <p>{move || label_text(shared, &help)}</p>
            <a class="glass" href=address::HOME>{move || label_text(shared, "nav.home")}</a>
        </div>
    }
}

/// What the core wants said, over whatever page is up.
///
/// `role="status"` with `aria-live="polite"` so a reader is told without being interrupted: the
/// notice arrives on its own, not from anything the viewer just pressed, and a bar that appears
/// silently is a bar a person who cannot see it never learns about.
#[component]
fn NoticeBar() -> impl IntoView {
    let shared = expect_context::<Shared>();
    move || {
        let notice = shared.notice.get()?;
        let text = if notice.text.is_empty() {
            let named = maybe_text(shared, &notice.label);
            named.or_else(|| maybe_text(shared, FAILED))?
        } else {
            notice.text
        };
        Some(view! {
            <div class="notice" role="status" aria-live="polite">
                <span>{text}</span>
                <button
                    class="plain"
                    aria-label=move || label_text(shared, "say.close")
                    on:click=move |_| shared.notice.set(None)
                >
                    {move || label_text(shared, "say.close")}
                </button>
            </div>
        })
    }
}

/// What a page draws while it is waiting: the reference's placeholder rows, as the phone draws
/// them. `super::empty` carries the measurements.
#[component]
fn Waiting() -> impl IntoView {
    view! { <main class="page"><empty::WaitingRows/></main> }
}

/// The page.
#[component]
pub fn App() -> impl IntoView {
    let shared = Shared::new();
    provide_context(shared);
    Effect::new(move |_| host::start(shared));
    let route = Memo::new(move |_| {
        shared
            .route
            .with(|route| route.as_ref().map(|route| (route.target, route.screen)))
    });
    let screen = move || match route.get() {
        None | Some((Target::Wait, _)) => view! { <Waiting/> }.into_any(),
        Some((Target::Start, _)) => view! { <door::Landing/> }.into_any(),
        Some((Target::SignIn, _)) => view! { <door::SignIn/> }.into_any(),
        Some((Target::Profiles, _)) => view! { <door::Profiles/> }.into_any(),
        Some((Target::Activate, _)) => view! { <door::Activate/> }.into_any(),
        Some((Target::Watch, _)) => view! { <watch::Watch/> }.into_any(),
        Some((Target::Title, _)) => view! { <browse::Browse/><title::Details/> }.into_any(),
        // The notices page is a page of its own and not a page of rows: it reads
        // `/v1/notifications`, which no screen route builds. The phone picks it out of
        // `Target.SCREEN` the same way (`apps/android/surfaces/phone/.../app/Root.kt:69-70`).
        Some((Target::Screen, Some(ScreenName::Notifications))) => {
            view! { <notifications::Notices/> }.into_any()
        }
        Some((Target::NotFound, _)) => view! { <Message label="problem.not_found"/> }.into_any(),
        Some((Target::Upgrade, _)) => view! { <Message label="problem.upgrade"/> }.into_any(),
        Some(_) => view! { <browse::Browse/> }.into_any(),
    };
    view! {
        <NoticeBar/>
        {screen}
        {move || shared.gate.get().then(|| view! { <door::Profiles/> })}
    }
}

/// What a test may read off this repository, which is the only place a measured number's own
/// derivation is written down.
///
/// The model is `apps/android/surfaces/phone/src/test/kotlin/streams/phone/Sources.kt`, and it says
/// what this costs: a test coupled to the written form of another family's source. That is the
/// point. A number this surface took from the phone is checked against the phone, so the day one
/// of them moves the other fails, which is the only kind of check worth having on a number copied
/// between two platforms.
#[cfg(test)]
pub mod tests_support {
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    /// The repository, found by walking up: a test's working folder is not promised.
    pub fn repo_root() -> PathBuf {
        let mut here: Option<&Path> = Some(Path::new(env!("CARGO_MANIFEST_DIR")));
        while let Some(folder) = here {
            if folder.join("strings/en.json").is_file() {
                return folder.to_path_buf();
            }
            here = folder.parent();
        }
        panic!("no strings/en.json above {}", env!("CARGO_MANIFEST_DIR"));
    }

    /// One file of this repository, as text.
    pub fn source(relative: &str) -> String {
        let path = repo_root().join(relative);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("{} is not readable", path.display()))
    }

    /// Every word the English strings file has.
    pub fn english() -> BTreeMap<String, String> {
        serde_json::from_str(&source("strings/en.json")).expect("strings/en.json reads")
    }

    /// The same text with every run of whitespace, and every `*` a doc comment puts at the start of
    /// a line, as one space.
    ///
    /// So a shape may be written as one line here and matched against a sentence that another
    /// file's doc comment wrapped over three. `Sources.kt:withoutComments` exists for the same
    /// reason on the phone: a scan that does not do this reads half a chain and reports the rest
    /// missing, which it did, on fourteen of them.
    /// A gap at the start or the end counts, and stays one space: the pieces of a shape are
    /// flattened by this same function, and a piece that ended in a space and lost it would stop
    /// matching the text it was written against.
    ///
    /// The marker a comment repeats at the start of each of its lines goes too - `//` for a run of
    /// line comments and `*` for a block - so a derivation written as prose over three lines is one
    /// sentence here. Only at the start of a line, so the `*` of a CSS multiplication stays where
    /// it is and a shape may name it.
    #[must_use]
    pub fn flat(text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut gap = text.starts_with(char::is_whitespace);
        for line in text.lines() {
            let bare = line.trim_start();
            let body = bare
                .strip_prefix("//")
                .or_else(|| bare.strip_prefix('*'))
                .unwrap_or(bare);
            for mark in body.chars() {
                if mark.is_whitespace() {
                    gap = true;
                    continue;
                }
                if gap {
                    out.push(' ');
                }
                gap = false;
                out.push(mark);
            }
            gap = true;
        }
        // An empty piece stays empty, so a shape may end with one and mean "and then anything".
        if !text.is_empty() && text.ends_with(char::is_whitespace) {
            out.push(' ');
        }
        out
    }

    /// The numbers a shape names: `parts` are its literal pieces, in order, with one number between
    /// each neighbouring pair, so `n` numbers need `n + 1` parts. Both sides are flattened first.
    ///
    /// The shape must occur exactly once. Every number these tests hold is written once, so a shape
    /// that matches twice means it has been copied, which is the drift the test is for: finding two
    /// is a failure and not a choice of which to read. The model, and the reason for the shape
    /// rather than a bare marker, is `Sources.kt:theOne` and the patterns in `NumbersTest.kt`,
    /// which carry the text between two numbers for exactly this.
    pub fn numbers(parts: &[&str], text: &str, what: &str) -> Vec<f64> {
        assert!(
            parts.len() >= 2,
            "{what}: a shape needs a piece on each side"
        );
        let flat_text = flat(text);
        let pieces: Vec<String> = parts.iter().map(|part| flat(part)).collect();
        let mut found: Vec<Vec<f64>> = Vec::new();
        let marks: Vec<char> = flat_text.chars().collect();
        for start in 0..marks.len() {
            if let Some(read) = read_shape(&pieces, &marks, start) {
                found.push(read);
            }
        }
        assert_eq!(
            found.len(),
            1,
            "{what}: expected one {pieces:?}, found {}",
            found.len()
        );
        found.into_iter().next().expect("one match")
    }

    fn read_shape(pieces: &[String], marks: &[char], start: usize) -> Option<Vec<f64>> {
        let mut at = start;
        let mut read = Vec::new();
        for (place, piece) in pieces.iter().enumerate() {
            let wanted: Vec<char> = piece.chars().collect();
            if marks.len() < at + wanted.len() || marks[at..at + wanted.len()] != wanted[..] {
                return None;
            }
            at += wanted.len();
            if place + 1 == pieces.len() {
                break;
            }
            // A number is a run of digits with at most one dot inside it, and an underscore
            // between digits is a separator both Kotlin and Rust allow: `10_000L` is one number.
            let from = at;
            let mut dotted = false;
            while at < marks.len() {
                let mark = marks[at];
                let joins = (mark == '.' && !dotted) || mark == '_';
                if mark.is_ascii_digit() {
                    at += 1;
                    continue;
                }
                if joins && at > from && marks.get(at + 1).is_some_and(char::is_ascii_digit) {
                    dotted |= mark == '.';
                    at += 1;
                    continue;
                }
                break;
            }
            if at == from {
                return None;
            }
            read.push(
                marks[from..at]
                    .iter()
                    .filter(|mark| **mark != '_')
                    .collect::<String>()
                    .parse()
                    .expect("digits parse"),
            );
        }
        Some(read)
    }

    /// The one number a shape names.
    pub fn one_number(parts: &[&str], text: &str, what: &str) -> f64 {
        numbers(parts, text, what)[0]
    }
}

#[cfg(test)]
mod tests {
    use super::tests_support::{english, numbers, one_number, repo_root, source};

    /// Every field and every menu on every screen of this surface has words.
    ///
    /// The web half of what `apps/apple/Tests/Heard/Heard.swift` does on Apple and what
    /// `phone/src/test/kotlin/streams/phone/SpokenTest.kt` does on Android: a control a person
    /// cannot hear is a defect whatever else ships. There is nothing in `./check` that notices one
    /// on this surface, so this reads the source.
    ///
    /// A field is labelled when it carries `aria-label`, or `aria-labelledby`, or an `id` some
    /// `label for=` in the same file names. A `placeholder` is none of those and is not accepted: it
    /// is gone the moment anything is typed, no browser is obliged to expose it, and before
    /// 2026-11-12 it was the only thing eight of the nine fields on this surface had.
    #[test]
    fn every_field_and_menu_on_this_surface_has_words() {
        let mut unnamed: Vec<String> = Vec::new();
        for path in super::words::tests::rust_files(&repo_root().join("apps/web/src/surfaces")) {
            let text = std::fs::read_to_string(&path).expect("a surface reads");
            let name = path
                .file_name()
                .expect("a file name")
                .to_string_lossy()
                .to_string();
            for (tag, body) in tags(&text) {
                if !matches!(tag.as_str(), "input" | "select" | "img") {
                    continue;
                }
                let said = body.contains("aria-label")
                    || body.contains("aria-labelledby")
                    // A picture with an empty `alt` is a picture marked as a decoration, which is
                    // what the art inside a tile that already says the title's name is.
                    || (tag == "img" && body.contains("alt="))
                    || id_of(&body).is_some_and(|id| named_by(&text, &id));
                if !said {
                    unnamed.push(format!("{name}: <{tag} {}>", body.trim()));
                }
            }
        }
        assert_eq!(unnamed, Vec::<String>::new());
    }

    /// Every opening tag in a piece of this surface's source, as (its name, the rest of it).
    ///
    /// `view!` is HTML inside a macro, so a tag ends at the `>` that is not inside a brace: an
    /// attribute may carry a whole closure, `aria-label=move || label_text(shared, "x")`, and a
    /// scan that stopped at the first `>` would cut `||` in half.
    fn tags(text: &str) -> Vec<(String, String)> {
        let mut found = Vec::new();
        let marks: Vec<char> = text.chars().collect();
        let mut at = 0;
        while at < marks.len() {
            if marks[at] != '<' || !marks.get(at + 1).is_some_and(char::is_ascii_lowercase) {
                at += 1;
                continue;
            }
            let from = at + 1;
            let mut end = from;
            while end < marks.len() && (marks[end].is_ascii_alphanumeric() || marks[end] == '-') {
                end += 1;
            }
            let tag: String = marks[from..end].iter().collect();
            let mut depth = 0_i32;
            let mut shut = end;
            while shut < marks.len() {
                match marks[shut] {
                    '{' => depth += 1,
                    '}' => depth -= 1,
                    '>' if depth == 0 => break,
                    _ => {}
                }
                shut += 1;
            }
            found.push((tag, marks[end..shut.min(marks.len())].iter().collect()));
            at = shut;
        }
        found
    }

    /// The `id` an opening tag carries, as written, so it can be looked for in a `for=`.
    fn id_of(body: &str) -> Option<String> {
        let at = body.find("id=")?;
        let rest = &body[at + 3..];
        let end = rest
            .find(|mark: char| mark.is_whitespace() || mark == '>' || mark == '/')
            .unwrap_or(rest.len());
        Some(rest[..end].to_owned())
    }

    /// True when some `for=` in this file names this id.
    ///
    /// The `for` has to be an attribute of its own, not the end of another word: a plain substring
    /// read accepted `data-for="search-field"`, and this test went on passing with the one label on
    /// the search page turned into a `div`. Found by breaking it on purpose on 2026-11-12.
    fn named_by(text: &str, id: &str) -> bool {
        let wanted = format!("for={id}");
        text.match_indices(&wanted).any(|(at, _)| {
            at == 0
                || text[..at]
                    .chars()
                    .next_back()
                    .is_some_and(|mark| mark.is_whitespace() || mark == '<')
        })
    }

    #[test]
    fn the_tag_reader_keeps_a_closure_inside_the_tag_it_belongs_to() {
        let text = "<input id=\"a\" aria-label=move || words(1 > 0)/>\n<p>x</p>";
        let read = tags(text);
        assert_eq!(read.len(), 2);
        assert_eq!(read[0].0, "input");
        assert!(read[0].1.contains("aria-label"), "{:?}", read[0].1);
        assert_eq!(read[1].0, "p");
        // The id comes back as written, quotes and all, which is how a `for=` writes it too.
        assert_eq!(id_of(&read[0].1).as_deref(), Some("\"a\""));
        assert_eq!(id_of("type=\"search\""), None);
        // A `for` is an attribute of its own and not the end of another word.
        assert!(named_by("<label for=\"a\">x</label>", "\"a\""));
        assert!(named_by("for=\"a\"", "\"a\""));
        assert!(!named_by("<div data-for=\"a\">x</div>", "\"a\""));
        assert!(!named_by("<label for=\"b\">x</label>", "\"a\""));
    }

    /// Every screen of this surface is reachable without a mouse: the two that cover the page
    /// answer Escape, and the player answers the keys the phone's player answers.
    #[test]
    fn what_covers_the_page_can_be_closed_from_a_keyboard() {
        for (file, what) in [
            ("title.rs", "the details panel"),
            ("browse.rs", "a row opened whole"),
            ("watch.rs", "the player"),
        ] {
            let text = source(&format!("apps/web/src/surfaces/{file}"));
            assert!(
                text.contains("leptos::ev::keydown") && text.contains("\"Escape\""),
                "{what} does not answer Escape"
            );
        }
    }

    #[test]
    fn the_reader_takes_the_numbers_a_shape_of_source_states() {
        let text = "private val BarHigh = 30.8.dp\nprivate const val WAITING_ROWS = 4\n";
        // A number ending on a full stop keeps its digits and loses the stop, which is what
        // `30.8.dp` is: a decimal and then a member.
        assert_eq!(one_number(&["BarHigh = ", ".dp"], text, "the bar"), 30.8);
        assert_eq!(
            one_number(&["WAITING_ROWS = ", "\n"], text, "the rows"),
            4.0
        );
        // A separator between the thousands is part of the number, not the end of it.
        assert_eq!(
            one_number(
                &["SEEK_MS = ", "L"],
                "const val SEEK_MS = 10_000L",
                "a step"
            ),
            10_000.0
        );
        // A sentence of a doc comment, wrapped over two lines with its asterisk between them.
        let wrapped = "the bar runs y 1543-1621, which is 79 px,\n * and x 31-1048";
        assert_eq!(
            numbers(
                &["runs y ", "-", ", which is ", " px, and x ", "-", ""],
                wrapped,
                "the bar's box"
            ),
            vec![1543.0, 1621.0, 79.0, 31.0, 1048.0]
        );
    }

    #[test]
    #[should_panic(expected = "expected one")]
    fn a_number_written_twice_is_a_failure_and_not_a_choice_of_which_to_read() {
        one_number(&["= ", "\n"], "a = 1\nb = 2\n", "two of them");
    }

    /// The pieces between the numbers are what make a shape one thing. Two numbers joined by
    /// something else are not the shape, so a derivation re-worded cannot go on passing.
    #[test]
    #[should_panic(expected = "found 0")]
    fn two_numbers_joined_by_something_else_are_not_the_shape() {
        numbers(
            &["y ", " + ", ""],
            "the bar runs y 1543-1621",
            "a wrong join",
        );
    }

    /// The labels this file names are labels the strings file has. The hole
    /// `phone/src/test/.../WordsTest.kt:12-16` was written against: nothing checked `en.json`
    /// against the code, so a label wired to a key no language carries drew nothing and passed
    /// every check.
    #[test]
    fn every_label_this_surface_names_is_in_the_strings_file() {
        let book = english();
        for label in [
            super::FAILED,
            "nav.home",
            "say.close",
            "problem.not_found",
            "problem.not_found.help",
            "problem.upgrade",
            "problem.upgrade.help",
        ] {
            assert!(book.contains_key(label), "strings/en.json has no {label}");
        }
    }
}
