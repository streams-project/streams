//! The screens that come before a film: the server address, the door, who is watching, and rows.
//!
//! This family drew nothing until today. `DECISIONS.md` decision 2 said the pointer screens were
//! `apps/web`'s and this crate would add none; the owner has since decided the desktop is its own
//! app and not a web page in a frame, so the screens are here. What that decision does **not**
//! settle is which toolkit paints them, and a toolkit is architecture: it is a dependency the size
//! of the rest of this repository put together and it is hard to reverse. So nothing here paints.
//! These screens lay out into [`scene::Scene`] - rectangles, corners, colours by their brand role,
//! words, pictures and the caret - and a painter of forty lines turns a scene into whatever window
//! the owner picks. The surface is then the same surface under every answer to that question.
//!
//! Three things make that honest rather than a dodge:
//!
//! * Every measurement is the reference's. [`sizes`] reads `reference/netflix-on-the-web.md` in
//!   its own tests, at all 28 widths from 160 to 1920.
//! * Every word is `strings/`. This file holds the English file as its first words - the address
//!   screen is drawn before any server exists, so no server can have sent them - and replaces all
//!   of them the moment `/v1/strings/{lang}` answers. No English is written into this crate.
//! * Every colour, corner and face is a **role**, resolved by the painter against `brand.json`.
//!   Nothing here is a hex colour or a radius, so there is no copy of the brand to drift.
//!
//! The ring holds: a screen reads a [`Turn`] and hands back [`Ask`]s. It reaches no port, no
//! adapter and no address of its own.

mod browse;
mod door;
mod gate;
pub mod scene;
pub mod sizes;

use std::collections::BTreeMap;

use contract::account::Profile;
use contract::core_msg::{Action, RouteTarget, Target, View};
use contract::error::Notice;
use contract::hello::{Hello, Strings};
use contract::screen::{Nav, NavItem, Screen, ScreenName};

use crate::host::Turn;
pub use scene::{Corner, Ink, Letters, Mark, Part, Rect, Touch};
use scene::{Field, Scene};
pub use sizes::Window;

/// The words a screen has before any server has answered.
///
/// The address screen is drawn when there is no server to ask, so the English file travels in the
/// binary. It is the same file `check_strings.py` holds every other language to, read at build
/// time and never copied: `server-pages/src/words.rs` does exactly this for the same reason. Every
/// one of them is replaced by `/v1/strings/{lang}` as soon as a server answers, so a viewer sees
/// English only before they have named a server.
const FIRST_WORDS: &str = include_str!("../../../../strings/en.json");

/// The id this surface gives its own request for the words. A reply carries the id of the event
/// that asked, so the surface names its own and matches the answer by it.
const WORDS: &str = "surface.words";

/// The label a trouble falls back to when the one a refusal named has no words on this server.
const FAILED: &str = "problem-failed";

/// What the address screen says when the server it was given never answered.
const UNREACHED: &str = "error.no_server";

/// Which screen is up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Face {
    /// No server yet: the one screen that is this family's own.
    Address,
    /// The way in, before a name is given.
    Door,
    /// Sign in.
    SignIn,
    /// Who is watching.
    Gate,
    /// A page of rows.
    Browse,
    /// The core has not said yet.
    Waiting,
}

/// What a screen asks the shell for. Nothing here performs anything.
#[derive(Clone, Debug, PartialEq)]
pub enum Ask {
    /// Open the core against this server. Only the address screen asks it, once, and the shell
    /// holds the address from there on: no screen keeps one.
    Server(String),
    /// Hand the core this action, as an event.
    Act(Action),
    /// Open this address, as an event the core routes.
    Go(String),
    /// Send this prepared request and hand the answer back under this id.
    Read { id: String, path: String },
    /// Move a row on by one page of tiles, or back by one. Nothing leaves this surface for it:
    /// where a row is is this surface's own, as the scroll of the page is, and a shell never sees
    /// one of these.
    Page { section: String, on: bool },
}

/// Everything the core and the server have told these screens, and where the viewer is in them.
pub struct Surface {
    opened: bool,
    words: BTreeMap<String, String>,
    hello: Option<Hello>,
    route: Option<RouteTarget>,
    gate: bool,
    profiles: Vec<Profile>,
    nav: Vec<NavItem>,
    screen: Option<Screen>,
    notice: Option<Notice>,
    /// What is typed in each field. A secret field's text is here and never in a scene.
    typed: BTreeMap<Field, String>,
    caret: Option<Field>,
    /// The profile whose PIN is being asked for.
    locked: Option<String>,
    /// A label the screen that is up should say, where the shell or the core has told this
    /// surface something that is not a notice over a page.
    trouble: Option<String>,
    /// How far the page of rows is scrolled, in pixels.
    down: f64,
    /// How many pages each row has been moved along, by its section id.
    paged: BTreeMap<String, u32>,
}

impl Default for Surface {
    fn default() -> Self {
        Self::new()
    }
}

impl Surface {
    /// A surface with the words it has before a server exists, on the address screen.
    #[must_use]
    pub fn new() -> Self {
        Self {
            opened: false,
            words: serde_json::from_str(FIRST_WORDS).unwrap_or_default(),
            hello: None,
            route: None,
            gate: false,
            profiles: Vec::new(),
            nav: Vec::new(),
            screen: None,
            notice: None,
            typed: BTreeMap::new(),
            caret: None,
            locked: None,
            trouble: None,
            down: 0.0,
            paged: BTreeMap::new(),
        }
    }

    /// Which screen is up.
    #[must_use]
    pub fn face(&self) -> Face {
        if !self.opened {
            return Face::Address;
        }
        if self.gate && !self.profiles.is_empty() {
            return Face::Gate;
        }
        match self.route.as_ref().map(|route| route.target) {
            Some(Target::Start) => Face::Door,
            Some(Target::SignIn) => Face::SignIn,
            Some(Target::Profiles) => Face::Gate,
            Some(Target::Home | Target::KidsHome | Target::Screen | Target::Title) => Face::Browse,
            // Everything else is a screen this surface has not drawn yet, and it waits rather
            // than drawing a page of rows that is not the page the core asked for: a film
            // (`Watch`), the device approval screen (`Activate`), a page that is not there
            // (`NotFound`) and a build this server refuses (`Upgrade`).
            _ => Face::Waiting,
        }
    }

    /// The shell could not reach the server this surface asked it to open.
    ///
    /// It goes back to the address screen with the address still in the box, so a mistyped one is
    /// corrected rather than retyped, and with the words for what happened. The shell calls this
    /// when `Host::open` refuses and when the first request to that server never answers; this
    /// surface has no port and cannot tell either by itself.
    pub fn no_server(&mut self) {
        self.opened = false;
        self.trouble = Some(UNREACHED.to_owned());
        self.caret = Some(Field::Server);
    }

    /// Ask for the interface's words in this language. The shell knows which language the machine
    /// is in; this surface does not, and does not guess.
    #[must_use]
    pub fn speak(lang: &str) -> Ask {
        Ask::Read {
            id: WORDS.to_owned(),
            path: format!("/v1/strings/{lang}"),
        }
    }

    /// Take one turn of the core.
    ///
    /// Views are merged in order and a part that is absent did not change, which is the contract's
    /// own rule. Page bodies carry no path - `ScreenBody` holds the id of the request and the body
    /// and nothing else - so each is read as the two shapes this screen draws, a built page and a
    /// menu, and dropped when it is neither. The shapes do not overlap: a page has `name` and
    /// `sections`, a menu has `items`, and neither is optional in the contract.
    pub fn apply(&mut self, turn: &Turn) -> Vec<Ask> {
        for view in &turn.views {
            self.view(view);
        }
        for page in &turn.pages {
            if let Ok(screen) = serde_json::from_str::<Screen>(&page.body) {
                self.screen = Some(screen);
                self.down = 0.0;
                self.paged.clear();
            } else if let Ok(nav) = serde_json::from_str::<Nav>(&page.body) {
                self.nav = nav.items;
            }
        }
        for reply in &turn.replies {
            if reply.request == WORDS
                && (200..300).contains(&reply.status)
                && let Ok(strings) = serde_json::from_str::<Strings>(&reply.body)
            {
                self.words = strings.texts;
            }
        }
        if let Some(notice) = turn.notices.last() {
            self.notice = Some(notice.clone());
        }
        Vec::new()
    }

    fn view(&mut self, view: &View) {
        // `View.session` is read by no screen here. Who is signed in decides which screen the
        // core routes to, and these screens draw the route; a copy of the session nothing read
        // would be state that can go stale without anything noticing.
        if let Some(route) = &view.route {
            self.route = Some(route.clone());
            self.notice = None;
        }
        if let Some(gate) = &view.gate {
            self.gate = gate.ask;
            if !gate.ask {
                self.locked = None;
            }
        }
        if let Some(profiles) = &view.profiles {
            self.profiles.clone_from(&profiles.profiles);
        }
        if let Some(hello) = &view.hello {
            self.hello = Some(hello.clone());
        }
    }

    /// The screen, laid out for a window this size.
    #[must_use]
    pub fn draw(&self, window: Window) -> Scene {
        let mut scene = Scene::new(self.face(), window);
        scene.add(Part::plate(
            Rect::new(0.0, 0.0, window.wide, window.high),
            Ink::Background,
            Corner::Square,
        ));
        match self.face() {
            Face::Address => door::address(self, &mut scene),
            Face::Door => door::landing(self, &mut scene),
            Face::SignIn => door::sign_in(self, &mut scene),
            Face::Gate => gate::draw(self, &mut scene),
            Face::Browse => browse::draw(self, &mut scene),
            Face::Waiting => browse::waiting(self, &mut scene),
        }
        if let Some(notice) = &self.notice {
            self.notice_bar(notice, &mut scene);
        }
        scene
    }

    /// A press at this place in a window this size.
    pub fn click(&mut self, window: Window, x: f64, y: f64) -> Vec<Ask> {
        let Some(touch) = self.draw(window).touch(x, y).cloned() else {
            return Vec::new();
        };
        match touch {
            Touch::Field(field) => {
                self.caret = Some(field);
                Vec::new()
            }
            Touch::Does(ask) => self.does(ask),
        }
    }

    /// What an ask from a part really does. Most are handed to the shell as they are; the ones
    /// that only move this surface are taken here, so a shell never sees them.
    fn does(&mut self, ask: Ask) -> Vec<Ask> {
        match ask {
            Ask::Server(address) => {
                let address = address.trim().to_owned();
                if address.is_empty() {
                    return Vec::new();
                }
                self.opened = true;
                self.trouble = None;
                self.caret = None;
                vec![Ask::Server(address)]
            }
            Ask::Act(Action::EnterProfile { profile, pin: None })
                if self.profile_is_locked(&profile) =>
            {
                self.locked = Some(profile);
                self.caret = Some(Field::Pin);
                Vec::new()
            }
            Ask::Page { section, on } => {
                let at = self.paged.entry(section).or_insert(0);
                *at = if on { *at + 1 } else { at.saturating_sub(1) };
                Vec::new()
            }
            other => {
                // Whatever this was, it leaves the surface, and nothing a viewer typed in secret
                // has any reason to outlive it.
                self.forget_secrets();
                self.caret = None;
                vec![other]
            }
        }
    }

    fn profile_is_locked(&self, id: &str) -> bool {
        self.profiles
            .iter()
            .any(|profile| profile.id == id && profile.has_pin)
    }

    /// One character typed into whichever field the caret is in. A character typed with the caret
    /// nowhere is dropped rather than going into the last field that had it.
    pub fn typed(&mut self, mark: char) {
        if mark.is_control() {
            return;
        }
        if let Some(field) = self.caret {
            self.typed.entry(field).or_default().push(mark);
        }
    }

    /// Backspace.
    pub fn rubbed(&mut self) {
        if let Some(field) = self.caret {
            self.typed.entry(field).or_default().pop();
        }
    }

    /// Enter: what the screen that is up does with what has been typed. It needs no window,
    /// because what a key does is the screen's and not the layout's.
    pub fn submit(&mut self) -> Vec<Ask> {
        let ask = match self.face() {
            Face::Address => Some(Ask::Server(self.text_in(Field::Server).to_owned())),
            Face::SignIn => Some(Ask::Act(Action::SignIn {
                email: self.text_in(Field::Email).to_owned(),
                password: self.text_in(Field::Password).to_owned(),
            })),
            Face::Gate => self.locked.clone().map(|profile| {
                Ask::Act(Action::EnterProfile {
                    profile,
                    pin: Some(self.text_in(Field::Pin).to_owned()),
                })
            }),
            _ => None,
        };
        match ask {
            Some(ask) => self.does(ask),
            None => Vec::new(),
        }
    }

    /// A wheel or a two finger drag on a page of rows.
    pub fn wheel(&mut self, window: Window, by: f64) {
        if self.face() != Face::Browse {
            return;
        }
        let page = browse::page_high(self, window);
        self.down = (self.down + by).clamp(0.0, (page - window.high).max(0.0));
    }

    /// What is typed in a field.
    fn text_in(&self, field: Field) -> &str {
        self.typed.get(&field).map_or("", String::as_str)
    }

    /// A secret stops being held the moment it has been sent. A viewer's password has no reason to
    /// outlive the press that submitted it, and a surface that keeps it keeps it for as long as
    /// the app is open.
    fn forget_secrets(&mut self) {
        self.typed.retain(|field, _| !field.secret());
    }

    /// The text of a label, as this server states it, or nothing where it has no such label. A
    /// screen whose whole point is its words draws none rather than inventing them.
    fn maybe_text(&self, label: &str) -> Option<String> {
        let text = self.words.get(label)?;
        Some(match &self.hello {
            Some(hello) => hello.brand.fill(text),
            None => text.clone(),
        })
    }

    /// The text of a label, or nothing at all. Empty rather than English.
    fn text(&self, label: &str) -> String {
        self.maybe_text(label).unwrap_or_default()
    }

    /// A label's words with one `{name}` filled in, or nothing where this server has no such
    /// label.
    ///
    /// The value is wrapped in U+2068 FIRST STRONG ISOLATE and U+2069 POP DIRECTIONAL ISOLATE, so
    /// a name, a number or a title keeps its own order inside a line that runs the other way.
    /// Nine of the eighteen languages need that and none of them can fix it afterwards. The rule
    /// is the phone's `Theme.kt:66` and the web's `words::fill`, and the test below reads the
    /// web's two characters rather than trusting that these are the same two.
    fn filled(&self, label: &str, name: &str, value: &str) -> Option<String> {
        Some(
            self.maybe_text(label)?
                .replace(&format!("{{{name}}}"), &format!("\u{2068}{value}\u{2069}")),
        )
    }

    /// What the core wants said, over whatever screen is up.
    fn notice_bar(&self, notice: &Notice, scene: &mut Scene) {
        let said = if notice.text.is_empty() {
            self.maybe_text(&notice.label)
                .or_else(|| self.maybe_text(FAILED))
                .unwrap_or_default()
        } else {
            notice.text.clone()
        };
        if said.is_empty() {
            return;
        }
        let wide = scene.window.wide;
        let margin = sizes::margin(wide);
        let rect = Rect::new(margin, margin, (wide - 2.0 * margin).max(0.0), 48.0);
        scene.add(Part::plate(rect, Ink::Surface, Corner::Panel));
        scene.add(Part::words(
            Rect::new(rect.x + 16.0, rect.y + 14.0, rect.wide - 32.0, 20.0),
            Ink::Text,
            said,
            16.0,
            400,
        ));
    }
}

/// The screen a nav item opens, for marking the one the viewer is on. Home and the kids' home
/// carry no screen name in the address, so they are read back off the target, exactly as
/// `apps/web/src/surfaces/browse.rs` reads them.
fn here(route: Option<&RouteTarget>, item: &NavItem) -> bool {
    let Some(route) = route else { return false };
    let screen = match route.target {
        Target::Home => Some(ScreenName::Home),
        Target::KidsHome => Some(ScreenName::Kids),
        _ => route.screen,
    };
    screen == Some(item.screen) && route.genre.as_deref() == item.genre.as_deref()
}

/// What a test may read off this repository: the measurements' own source, the brand file and the
/// words. The model is `apps/web/src/surfaces/mod.rs`'s `tests_support`, and `numbers` is that
/// file's, so a number this surface took from a reference is read back out of the reference.
#[cfg(test)]
pub mod reading {
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

    /// The brand file, as a tree.
    pub fn brand() -> serde_json::Value {
        serde_json::from_str(&source("brand.json")).expect("brand.json reads")
    }

    /// Every word the English strings file has.
    pub fn english() -> BTreeMap<String, String> {
        serde_json::from_str(&source("strings/en.json")).expect("strings/en.json reads")
    }

    /// The same text with every run of whitespace, and the marker a comment repeats at the start
    /// of each line, as one space. `apps/web/src/surfaces/mod.rs:flat`, which exists so a shape
    /// may be written on one line and matched against prose that wrapped over three.
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
        if !text.is_empty() && text.ends_with(char::is_whitespace) {
            out.push(' ');
        }
        out
    }

    /// The numbers a shape names, with one number between each neighbouring pair of pieces. The
    /// shape must occur exactly once: a shape that matches twice means the number has been copied,
    /// which is the drift this is for.
    pub fn numbers(parts: &[&str], text: &str, what: &str) -> Vec<f64> {
        assert!(
            parts.len() >= 2,
            "{what}: a shape needs a piece on each side"
        );
        let flat_text = flat(text);
        let pieces: Vec<String> = parts.iter().map(|part| flat(part)).collect();
        let marks: Vec<char> = flat_text.chars().collect();
        let mut found: Vec<Vec<f64>> = Vec::new();
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
}

#[cfg(test)]
mod tests {
    use super::reading::{english, source};
    use super::{Ask, Face, Field, Surface, Window};
    use crate::host::{Reply, Turn};
    use contract::core_msg::{GateView, Target, View};

    /// A surface that has been given a server, so the screens behind the address one can be
    /// reached without a core.
    pub fn opened() -> Surface {
        let mut surface = Surface::new();
        surface.write(Field::Server, "https://example.test");
        let asks = surface.submit();
        assert!(matches!(asks.first(), Some(Ask::Server(_))), "{asks:?}");
        surface
    }

    impl Surface {
        /// Type into a field, as a viewer does: put the caret in it first.
        pub fn write(&mut self, field: Field, text: &str) {
            self.caret = Some(field);
            for mark in text.chars() {
                self.typed(mark);
            }
        }

        /// Press the part that can be pressed and that says these words: its own words, or what
        /// a reader is told it is. A button's words and its plate are two parts, and it is the
        /// plate that carries the press, so the one that is pressed is found by what it says.
        pub fn press(&mut self, window: Window, words: &str) -> Vec<Ask> {
            let scene = self.draw(window);
            let part = scene
                .parts
                .iter()
                .rev()
                .find(|part| {
                    part.touch.is_some()
                        && (part.text() == Some(words) || part.says.contains(words))
                })
                .unwrap_or_else(|| panic!("no part says {words:?} on {:?}", scene.face));
            let (x, y) = (part.rect.x + 1.0, part.rect.y + 1.0);
            self.click(window, x, y)
        }
    }

    /// Give a surface a route, as the core does.
    pub fn route(surface: &mut Surface, target: Target) {
        let mut turn = Turn::default();
        turn.views.push(View {
            route: Some(contract::core_msg::RouteTarget {
                target,
                screen: None,
                genre: None,
                title: None,
                episode: None,
                next: None,
                code: None,
            }),
            ..View::default()
        });
        surface.apply(&turn);
    }

    #[test]
    fn a_surface_with_no_server_is_on_the_address_screen_and_nowhere_else() {
        let surface = Surface::new();
        assert_eq!(surface.face(), Face::Address);
        // And it stays there whatever the core says, because there is no core until a server is
        // named: a route that arrived first would otherwise draw a page of rows over nothing.
        let mut surface = surface;
        route(&mut surface, Target::Home);
        assert_eq!(surface.face(), Face::Address);
    }

    #[test]
    fn the_core_s_route_decides_which_screen_is_up() {
        let mut surface = opened();
        assert_eq!(surface.face(), Face::Waiting);
        for (target, face) in [
            (Target::Start, Face::Door),
            (Target::SignIn, Face::SignIn),
            (Target::Home, Face::Browse),
            (Target::Screen, Face::Browse),
            (Target::KidsHome, Face::Browse),
            (Target::Profiles, Face::Gate),
            (Target::Wait, Face::Waiting),
        ] {
            route(&mut surface, target);
            assert_eq!(surface.face(), face, "{target:?}");
        }
    }

    /// The gate is over whatever page is up, which is what `GateView` means: the core asks who is
    /// watching without changing the address.
    #[test]
    fn the_gate_shows_over_the_page_the_viewer_was_on() {
        let mut surface = opened();
        route(&mut surface, Target::Home);
        let mut turn = Turn::default();
        turn.views.push(View {
            gate: Some(GateView { ask: true }),
            profiles: Some(contract::account::ProfileList {
                profiles: vec![a_profile("Ada", false, false)],
            }),
            ..View::default()
        });
        surface.apply(&turn);
        assert_eq!(surface.face(), Face::Gate);
        let mut closed = Turn::default();
        closed.views.push(View {
            gate: Some(GateView { ask: false }),
            ..View::default()
        });
        surface.apply(&closed);
        assert_eq!(surface.face(), Face::Browse);
    }

    /// A profile for a test.
    pub fn a_profile(name: &str, kids: bool, pin: bool) -> contract::account::Profile {
        contract::account::Profile {
            id: format!("p-{name}"),
            name: name.to_owned(),
            kids,
            main: false,
            has_pin: pin,
            avatar: None,
            language: None,
        }
    }

    /// The words of the interface are replaced whole by the server's, and the request that asks
    /// for them carries the id this surface matches the answer by.
    #[test]
    fn the_server_s_words_replace_the_ones_the_binary_carries() {
        let mut surface = opened();
        let Ask::Read { id, path } = Surface::speak("fr") else {
            panic!("the words are asked for by a read");
        };
        assert_eq!(path, "/v1/strings/fr");
        let before = surface.text("door.sign_in");
        assert!(!before.is_empty(), "the binary's own words are there");
        let mut turn = Turn::default();
        turn.replies.push(Reply {
            request: id,
            status: 200,
            body: r#"{"lang":"fr","dir":"ltr","texts":{"door.sign_in":"Se connecter"}}"#.to_owned(),
        });
        surface.apply(&turn);
        assert_eq!(surface.text("door.sign_in"), "Se connecter");
        // And a label this server does not carry draws nothing rather than falling back to the
        // English in the binary, which would put two languages on one screen.
        assert_eq!(surface.text("door.join"), "");
    }

    /// The words this surface starts with are the repository's own English file, not a copy of it.
    #[test]
    fn the_first_words_are_the_strings_file_itself() {
        let surface = Surface::new();
        let book = english();
        assert_eq!(surface.words.len(), book.len());
        for (label, text) in &book {
            assert_eq!(surface.words.get(label), Some(text), "{label}");
        }
        // And they are included from that file rather than written out here.
        let me = source("apps/desktop/src/surfaces/mod.rs");
        assert!(
            me.contains(r#"include_str!("../../../../strings/en.json")"#),
            "the first words stopped being the strings file"
        );
    }

    /// A filled value is isolated, with the same two characters the web surface uses, read out of
    /// that file. Without them a name inside an Arabic line is reordered by the paragraph and
    /// reads back wrong.
    #[test]
    fn a_filled_value_keeps_its_own_order_inside_the_line() {
        let mut surface = opened();
        surface
            .words
            .insert("say.test".to_owned(), "{name}, locked".to_owned());
        let filled = surface
            .filled("say.test", "name", "Ada")
            .expect("the label");
        let web = source("apps/web/src/surfaces/words.rs");
        let isolate = |named: &str| {
            let marker = format!("const {named}: char = '\\u{{");
            let hex = web
                .split(&marker)
                .nth(1)
                .and_then(|rest| rest.split('}').next())
                .unwrap_or_else(|| panic!("the web surface no longer names {named}"));
            char::from_u32(u32::from_str_radix(hex, 16).expect("hex digits")).expect("a character")
        };
        assert_eq!(
            filled,
            format!(
                "{}Ada{}, locked",
                isolate("OPEN_ISOLATE"),
                isolate("CLOSE_ISOLATE")
            ),
            "the isolates here are not the web surface's"
        );
        assert!(surface.filled("no.such.label", "name", "Ada").is_none());
    }

    #[test]
    fn a_character_typed_with_the_caret_nowhere_goes_nowhere() {
        let mut surface = opened();
        surface.typed('a');
        assert_eq!(surface.text_in(Field::Email), "");
        surface.rubbed();
        surface.write(Field::Email, "ada@example.test");
        assert_eq!(surface.text_in(Field::Email), "ada@example.test");
        surface.rubbed();
        assert_eq!(surface.text_in(Field::Email), "ada@example.tes");
    }

    /// A notice is drawn over whatever is up, and a notice with no words of its own falls back to
    /// a label this server has rather than to silence.
    #[test]
    fn a_notice_is_drawn_over_the_screen_and_always_says_something() {
        let mut surface = opened();
        route(&mut surface, Target::Home);
        let mut turn = Turn::default();
        turn.notices.push(contract::error::Notice {
            label: "problem-offline".to_owned(),
            text: String::new(),
            action_label: None,
            action_address: None,
        });
        surface.apply(&turn);
        let window = Window::new(1440.0, 900.0);
        let said = english()["problem-offline"].clone();
        assert!(surface.draw(window).says(&said), "the notice is not drawn");
        // A label no server has still says something, because the fallback label is one every
        // language file carries.
        let mut unknown = Turn::default();
        unknown.notices.push(contract::error::Notice {
            label: "no.such.label".to_owned(),
            text: String::new(),
            action_label: None,
            action_address: None,
        });
        surface.apply(&unknown);
        let failed = english()[super::FAILED].clone();
        assert!(surface.draw(window).says(&failed));
    }
}
