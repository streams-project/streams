//! The way in: the server address, the landing, and sign in.
//!
//! The first of those three is this family's own and belongs to no other surface in the project. A
//! browser is already at a server when it loads the page and a television is handed one by its
//! shell; a desktop app is opened from a dock with nothing, so the first thing it can honestly
//! draw is a box to put a server in. Its words are `settings.server` and `common.next`, which are
//! the keys `strings/` already has for exactly that; no key is invented and no English is written
//! here.
//!
//! What is **not** here, named rather than left to be noticed: joining. `door::Join` on the web is
//! a form of five fields that creates an account, and the keys for it are not all in `strings/`
//! either (`apps/web/src/surfaces/door.rs` says which two are missing). This surface draws the way
//! in for someone who has an account; the way to make one arrives with those keys.

use contract::address;
use contract::core_msg::Action;

use super::scene::{Corner, Field, Ink, Mark, Part, Rect, Scene, Touch};
use super::{Ask, Surface, sizes};

/// How many gaps of running text a card holds between its parts: the gutter the reference
/// measured between two stacked pills.
const GAP: f64 = sizes::GUTTER;

/// A card of fields in the middle of the window: where it starts, and how wide it is.
///
/// The card is as wide as the hero's own reading column and no wider, and it is centred both ways
/// over the whole window, so a door is the same shape on a window of any size the reference
/// measured.
pub(super) fn card(scene: &mut Scene, high: f64) -> Rect {
    let window = scene.window;
    let wide = sizes::card(window.wide);
    let rect = Rect::new(
        ((window.wide - wide) / 2.0).max(0.0),
        ((window.high - high) / 2.0).max(sizes::BAR_HIGH),
        wide,
        high,
    );
    scene.add(Part::plate(rect, Ink::Surface, Corner::Panel));
    rect
}

/// The heading of a card, in the row title's own measured type.
pub(super) fn heading(scene: &mut Scene, rect: Rect, text: &str) -> f64 {
    let (size, box_high) = sizes::row_title(scene.window.wide);
    scene.add(
        Part::words(
            Rect::new(rect.x, rect.y, rect.wide, box_high),
            Ink::Text,
            text,
            size,
            500,
        )
        .displayed(),
    );
    box_high
}

/// A line of running text under a heading, in the hero synopsis's own measured type.
pub(super) fn line(scene: &mut Scene, rect: Rect, ink: Ink, text: &str) -> f64 {
    let (size, line_high) = sizes::body_type(scene.window.wide);
    scene.add(Part::words(
        Rect::new(rect.x, rect.y, rect.wide, line_high),
        ink,
        text,
        size,
        400,
    ));
    line_high
}

/// A box a viewer types into: its name above it, the plate, and what is in it.
///
/// A secret field never puts what was typed into the scene. It draws one mark per character, so a
/// painter, a screen capture and a test all see the same thing a shoulder does.
pub(super) fn field(
    surface: &Surface,
    scene: &mut Scene,
    rect: Rect,
    which: Field,
    label: &str,
) -> f64 {
    let wide = scene.window.wide;
    let (size, line_high) = sizes::body_type(wide);
    let high = sizes::pill(wide);
    scene.add(Part::words(
        Rect::new(rect.x, rect.y, rect.wide, line_high),
        Ink::Muted,
        surface.text(label),
        size,
        400,
    ));
    let box_at = Rect::new(rect.x, rect.y + line_high, rect.wide, high);
    scene.add(
        Part::plate(box_at, Ink::Raised, Corner::Field)
            .touched(Touch::Field(which))
            .saying(surface.text(label)),
    );
    let typed = surface.text_in(which);
    let shown = if which.secret() {
        "\u{2022}".repeat(typed.chars().count())
    } else {
        typed.to_owned()
    };
    let inside = Rect::new(
        box_at.x + GAP,
        box_at.y + (high - line_high) / 2.0,
        (box_at.wide - 2.0 * GAP).max(0.0),
        line_high,
    );
    if !shown.is_empty() {
        scene.add(Part::words(inside, Ink::Text, shown, size, 400));
    }
    if surface.caret == Some(which) {
        scene.add(Part {
            rect: Rect::new(inside.x, inside.y, 2.0, line_high),
            ink: Ink::Accent,
            corner: Corner::Square,
            mark: Mark::Caret,
            touch: None,
            says: String::new(),
        });
    }
    line_high + high
}

/// A pill a viewer presses.
pub(super) fn button(
    scene: &mut Scene,
    rect: Rect,
    ink: Ink,
    on: Ink,
    text: &str,
    ask: Ask,
) -> f64 {
    let high = sizes::pill(scene.window.wide);
    let at = Rect::new(rect.x, rect.y, rect.wide, high);
    scene.add(
        Part::plate(at, ink, Corner::Button)
            .touched(Touch::Does(ask))
            .saying(text),
    );
    let (size, line_high) = sizes::body_type(scene.window.wide);
    scene.add(
        Part::words(
            Rect::new(at.x, at.y + (high - line_high) / 2.0, at.wide, line_high),
            on,
            text,
            size,
            500,
        )
        .middle(),
    );
    high
}

/// How tall a card of this many lines, fields and buttons is.
fn stack(wide: f64, headings: f64, lines: f64, fields: f64, buttons: f64) -> f64 {
    let (_, line_high) = sizes::body_type(wide);
    let (_, head_high) = sizes::row_title(wide);
    let parts = headings + lines + fields + buttons;
    headings * head_high
        + lines * line_high
        + fields * (line_high + sizes::pill(wide))
        + buttons * sizes::pill(wide)
        + (parts - 1.0).max(0.0) * GAP
        + 2.0 * sizes::hero_inset(wide)
}

/// The server address: the one screen this family draws that no other surface has.
pub(super) fn address(surface: &Surface, scene: &mut Scene) {
    let wide = scene.window.wide;
    let trouble = surface.trouble.clone();
    let lines = if trouble.is_some() { 1.0 } else { 0.0 };
    let high = stack(wide, 1.0, lines, 1.0, 1.0);
    let card = card(scene, high);
    let inset = sizes::hero_inset(wide);
    let mut at = Rect::new(
        card.x + inset,
        card.y + inset,
        (card.wide - 2.0 * inset).max(0.0),
        0.0,
    );
    at.y += heading(scene, at, &surface.text("settings.server")) + GAP;
    if let Some(label) = &trouble {
        at.y += line(scene, at, Ink::Join, &surface.text(label)) + GAP;
    }
    at.y += field(surface, scene, at, Field::Server, "settings.server") + GAP;
    button(
        scene,
        at,
        Ink::Accent,
        Ink::AccentText,
        &surface.text("common.next"),
        Ask::Server(surface.text_in(Field::Server).to_owned()),
    );
}

/// The landing: the brand's name and a way in, which is all the phone's door has.
pub(super) fn landing(surface: &Surface, scene: &mut Scene) {
    let wide = scene.window.wide;
    let open = surface
        .hello
        .as_ref()
        .is_some_and(|hello| !hello.features.accounts_required);
    let buttons = if open { 2.0 } else { 1.0 };
    let high = stack(wide, 1.0, 0.0, 0.0, buttons);
    let card = card(scene, high);
    let inset = sizes::hero_inset(wide);
    let mut at = Rect::new(
        card.x + inset,
        card.y + inset,
        (card.wide - 2.0 * inset).max(0.0),
        0.0,
    );
    at.y += heading(scene, at, &surface.text("door.watch")) + GAP;
    at.y += button(
        scene,
        at,
        Ink::Accent,
        Ink::AccentText,
        &surface.text("door.sign_in"),
        Ask::Go(address::SIGN_IN.to_owned()),
    ) + GAP;
    // Only where this server lets a visitor look before joining, which is the server's own answer
    // and never this screen's guess.
    if open {
        button(
            scene,
            at,
            Ink::Surface,
            Ink::Text,
            &surface.text("nav.home"),
            Ask::Go(address::HOME.to_owned()),
        );
    }
}

/// Sign in: an email, a password, and the way in.
pub(super) fn sign_in(surface: &Surface, scene: &mut Scene) {
    let wide = scene.window.wide;
    let high = stack(wide, 1.0, 0.0, 2.0, 1.0);
    let card = card(scene, high);
    let inset = sizes::hero_inset(wide);
    let mut at = Rect::new(
        card.x + inset,
        card.y + inset,
        (card.wide - 2.0 * inset).max(0.0),
        0.0,
    );
    at.y += heading(scene, at, &surface.text("door.sign_in")) + GAP;
    at.y += field(surface, scene, at, Field::Email, "signin.email") + GAP;
    at.y += field(surface, scene, at, Field::Password, "signin.password") + GAP;
    button(
        scene,
        at,
        Ink::Accent,
        Ink::AccentText,
        &surface.text("door.sign_in"),
        Ask::Act(Action::SignIn {
            email: surface.text_in(Field::Email).to_owned(),
            password: surface.text_in(Field::Password).to_owned(),
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::super::reading::english;
    use super::super::scene::{Field, Mark};
    use super::super::tests::{opened, route};
    use super::super::{Ask, Face, Surface, Window};
    use contract::core_msg::{Action, Target};

    /// Every window the reference measured, so a screen is checked at the widths the layout steps
    /// at and not only at a comfortable one.
    const WINDOWS: [(f64, f64); 5] = [
        (390.0, 844.0),
        (768.0, 899.0),
        (1280.0, 899.0),
        (1440.0, 900.0),
        (1920.0, 899.0),
    ];

    /// The address screen draws its three parts, in every window, and nothing it draws is outside
    /// the window. Break any one of them and this fails.
    #[test]
    fn the_address_screen_asks_for_a_server_and_nothing_else() {
        let surface = Surface::new();
        let book = english();
        for (wide, high) in WINDOWS {
            let scene = surface.draw(Window::new(wide, high));
            assert_eq!(scene.face, Face::Address, "at {wide}");
            assert!(
                scene.says(&book["settings.server"]),
                "the heading is not drawn at {wide}"
            );
            assert!(
                scene.says(&book["common.next"]),
                "the way on is not drawn at {wide}"
            );
            let fields: Vec<_> = scene
                .parts
                .iter()
                .filter_map(|part| match &part.touch {
                    Some(super::Touch::Field(which)) => Some(*which),
                    _ => None,
                })
                .collect();
            assert_eq!(fields, vec![Field::Server], "at {wide}");
            assert!(
                scene.outside().is_empty(),
                "{:?} is drawn outside a {wide} window",
                scene.outside().first().map(|part| part.rect)
            );
        }
    }

    /// What is typed is drawn, and pressing the way on hands the shell that server and nothing
    /// else.
    #[test]
    fn typing_a_server_and_pressing_on_opens_that_server() {
        let window = Window::new(1440.0, 900.0);
        let mut surface = Surface::new();
        surface.write(Field::Server, " https://films.example  ");
        assert!(
            surface.draw(window).says(" https://films.example  "),
            "what was typed is not drawn"
        );
        let asks = surface.press(window, &english()["common.next"]);
        assert_eq!(asks, vec![Ask::Server("https://films.example".to_owned())]);
        assert_eq!(surface.face(), Face::Waiting, "the core has not spoken yet");
    }

    /// An empty box asks for nothing: a press with nothing typed leaves the viewer where they are
    /// rather than opening a host on the empty string.
    #[test]
    fn an_empty_box_opens_nothing() {
        let window = Window::new(1440.0, 900.0);
        let mut surface = Surface::new();
        assert!(surface.press(window, &english()["common.next"]).is_empty());
        assert_eq!(surface.face(), Face::Address);
        surface.write(Field::Server, "   ");
        assert!(surface.submit().is_empty());
        assert_eq!(surface.face(), Face::Address);
    }

    /// A server that could not be reached puts the viewer back on the address screen with the
    /// words for it, rather than leaving them on a blank page.
    #[test]
    fn a_server_that_does_not_answer_comes_back_to_the_address_screen() {
        let window = Window::new(1440.0, 900.0);
        let mut surface = opened();
        route(&mut surface, Target::Home);
        surface.no_server();
        assert_eq!(surface.face(), Face::Address);
        let scene = surface.draw(window);
        assert!(
            scene.says(&english()["error.no_server"]),
            "nothing says the server did not answer"
        );
        // And the address is still in the box, so it can be corrected instead of retyped.
        assert!(scene.says("https://example.test"));
    }

    #[test]
    fn the_landing_offers_the_way_in_and_offers_looking_round_only_where_the_server_does() {
        let window = Window::new(1440.0, 900.0);
        let book = english();
        let mut surface = opened();
        route(&mut surface, Target::Start);
        let scene = surface.draw(window);
        assert_eq!(scene.face, Face::Door);
        assert!(scene.says(&book["door.watch"]));
        assert!(scene.says(&book["door.sign_in"]));
        assert!(
            !scene.says(&book["nav.home"]),
            "a server that requires an account offered a look round"
        );
        let asks = surface.press(window, &book["door.sign_in"]);
        assert_eq!(asks, vec![Ask::Go(contract::address::SIGN_IN.to_owned())]);
    }

    #[test]
    fn signing_in_sends_what_was_typed_and_never_draws_the_password() {
        let book = english();
        let secret = "correct horse battery";
        for (wide, high) in WINDOWS {
            let window = Window::new(wide, high);
            let mut surface = opened();
            route(&mut surface, Target::SignIn);
            surface.write(Field::Email, "ada@example.test");
            surface.write(Field::Password, secret);
            let scene = surface.draw(window);
            assert!(scene.says(&book["signin.email"]), "at {wide}");
            assert!(scene.says(&book["signin.password"]), "at {wide}");
            assert!(scene.says("ada@example.test"), "at {wide}");
            // The whole point of a secret field: no part of the scene carries the password, and
            // one mark is drawn per character instead.
            for part in &scene.parts {
                if let Mark::Words { text, .. } = &part.mark {
                    assert!(!text.contains(secret), "the password is drawn at {wide}");
                    assert!(!text.contains("battery"), "part of it is drawn at {wide}");
                }
            }
            assert!(
                scene.says(&"\u{2022}".repeat(secret.chars().count())),
                "the password is not marked at {wide}"
            );
            assert!(scene.outside().is_empty(), "a part is outside at {wide}");
            let asks = surface.press(window, &book["door.sign_in"]);
            assert_eq!(
                asks,
                vec![Ask::Act(Action::SignIn {
                    email: "ada@example.test".to_owned(),
                    password: secret.to_owned(),
                })],
                "at {wide}"
            );
        }
    }

    /// A password stops being held the moment it has been sent.
    #[test]
    fn a_password_does_not_outlive_the_press_that_sent_it() {
        let mut surface = opened();
        route(&mut surface, Target::SignIn);
        surface.write(Field::Email, "ada@example.test");
        surface.write(Field::Password, "hunter2");
        surface.submit();
        assert_eq!(surface.text_in(Field::Password), "");
        // The email stays: it is not a secret, and a refused sign in should not make a viewer
        // type their address again.
        assert_eq!(surface.text_in(Field::Email), "ada@example.test");
    }

    /// Every label these three screens name is one `strings/en.json` really carries. A screen that
    /// started naming a key no server has would draw a blank control, which is what this refuses.
    #[test]
    fn every_label_the_door_names_is_in_the_strings_file() {
        let book = english();
        for label in [
            "common.next",
            "door.sign_in",
            "door.watch",
            "error.no_server",
            "nav.home",
            "settings.server",
            "signin.email",
            "signin.password",
        ] {
            assert!(book.contains_key(label), "strings/en.json has no {label}");
        }
    }
}
