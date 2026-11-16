//! Who is watching.
//!
//! The profiles sit on the page's own grid: the same page margin, the same count across and the
//! same 8 px gutter as a row of tiles, because a viewer who has seen the rows has already learned
//! that grid. A face is a square as tall as a tile, which is the one measurement the reference
//! gives for anything that size, and the name is under it in the row title's type.
//!
//! A profile with a PIN opens the PIN card instead of entering: `Profile.has_pin` is the server's
//! own answer and the only thing this screen reads it for. No PIN is ever held in a view, and the
//! one typed here is drawn as marks and dropped the moment it is sent.

use contract::account::Profile;
use contract::core_msg::Action;

use super::door;
use super::scene::{Corner, Field, Ink, Part, Rect, Scene, Touch};
use super::{Ask, Surface, sizes};

/// The letter that stands in for a picture of a person: the first of their name.
fn letter(name: &str) -> String {
    name.chars().next().map(String::from).unwrap_or_default()
}

/// Where the face of the profile at this place in the list sits.
fn face_at(wide: f64, place: usize) -> Rect {
    let across = sizes::across(wide);
    // A count to a float with nothing lost and no cast: the server caps a list of profiles at
    // `Limits.profiles_max`, and `f64::from` takes a `u32` exactly.
    let place = f64::from(u32::try_from(place).unwrap_or(u32::MAX));
    let column = place % across;
    let row = (place / across).floor();
    let side = sizes::tile(wide) * sizes::TILE_HIGH;
    let (_, name_high) = sizes::row_title(wide);
    let pitch = sizes::pitch(wide);
    Rect::new(
        sizes::margin(wide) + column * pitch + (sizes::tile(wide) - side) / 2.0,
        sizes::BAR_HIGH + sizes::HERO_GAP + row * (side + name_high + sizes::ROW_GAP),
        side,
        side,
    )
}

/// One profile: the face, the name under it, and the mark a child's profile carries.
fn person(surface: &Surface, scene: &mut Scene, place: usize, profile: &Profile) {
    let wide = scene.window.wide;
    let rect = face_at(wide, place);
    let says = if profile.has_pin {
        surface
            .filled("say.profile_locked", "name", &profile.name)
            .unwrap_or_else(|| profile.name.clone())
    } else {
        profile.name.clone()
    };
    let ask = Ask::Act(Action::EnterProfile {
        profile: profile.id.clone(),
        pin: None,
    });
    let ink = if profile.kids { Ink::Join } else { Ink::Accent };
    let on = if profile.kids {
        Ink::JoinText
    } else {
        Ink::AccentText
    };
    scene.add(
        Part::plate(rect, ink, Corner::Card)
            .touched(Touch::Does(ask))
            .saying(says),
    );
    match &profile.avatar {
        Some(url) => scene.add(Part::picture(rect, Corner::Card, url.clone())),
        None => {
            let (size, _) = sizes::row_title(wide);
            scene.add(
                Part::words(rect, on, letter(&profile.name), size, 500)
                    .middle()
                    .displayed(),
            );
        }
    }
    let (size, name_high) = sizes::row_title(wide);
    scene.add(
        Part::words(
            Rect::new(rect.x, rect.bottom(), rect.wide, name_high),
            Ink::Text,
            profile.name.clone(),
            size,
            400,
        )
        .middle(),
    );
    if profile.kids {
        let words = surface.text("profile.badge.kids");
        let high = sizes::body_type(wide).1;
        scene.add(Part::plate(
            Rect::new(rect.x, rect.y, rect.wide, high),
            Ink::Join,
            Corner::Chip,
        ));
        scene.add(
            Part::words(
                Rect::new(rect.x, rect.y, rect.wide, high),
                Ink::JoinText,
                words,
                sizes::body_type(wide).0,
                500,
            )
            .middle(),
        );
    }
}

/// The card that asks for a PIN, over the faces.
fn pin(surface: &Surface, scene: &mut Scene, profile: &str) {
    let wide = scene.window.wide;
    let (_, line_high) = sizes::body_type(wide);
    let (_, head_high) = sizes::row_title(wide);
    let high = head_high
        + line_high
        + (line_high + sizes::pill(wide))
        + sizes::pill(wide)
        + 3.0 * sizes::GUTTER
        + 2.0 * sizes::hero_inset(wide);
    let card = door::card(scene, high);
    let inset = sizes::hero_inset(wide);
    let mut at = Rect::new(
        card.x + inset,
        card.y + inset,
        (card.wide - 2.0 * inset).max(0.0),
        0.0,
    );
    at.y += door::heading(scene, at, &surface.text("say.pin")) + sizes::GUTTER;
    at.y += door::line(scene, at, Ink::Muted, &surface.text("profile.pin.enter")) + sizes::GUTTER;
    at.y += door::field(surface, scene, at, Field::Pin, "say.pin") + sizes::GUTTER;
    door::button(
        scene,
        at,
        Ink::Accent,
        Ink::AccentText,
        &surface.text("common.done"),
        Ask::Act(Action::EnterProfile {
            profile: profile.to_owned(),
            pin: Some(surface.text_in(Field::Pin).to_owned()),
        }),
    );
}

pub(super) fn draw(surface: &Surface, scene: &mut Scene) {
    let wide = scene.window.wide;
    let (size, head_high) = sizes::row_title(wide);
    scene.add(
        Part::words(
            Rect::new(
                sizes::margin(wide),
                sizes::BAR_HIGH,
                sizes::content(wide),
                head_high,
            ),
            Ink::Text,
            surface.text("profiles.choose"),
            size,
            500,
        )
        .displayed(),
    );
    for (place, profile) in surface.profiles.iter().enumerate() {
        person(surface, scene, place, profile);
    }
    if let Some(profile) = &surface.locked {
        pin(surface, scene, profile);
    }
}

#[cfg(test)]
mod tests {
    use super::super::reading::english;
    use super::super::scene::Field;
    use super::super::tests::a_profile;
    use super::super::{Ask, Face, Surface, Window, sizes};
    use super::face_at;
    use crate::host::Turn;
    use contract::account::ProfileList;
    use contract::core_msg::{Action, GateView, View};

    const WINDOWS: [(f64, f64); 4] = [
        (390.0, 844.0),
        (768.0, 899.0),
        (1440.0, 900.0),
        (1920.0, 899.0),
    ];

    fn watching(profiles: Vec<contract::account::Profile>) -> Surface {
        let mut surface = Surface::new();
        let mut turn = Turn::default();
        turn.views.push(View {
            gate: Some(GateView { ask: true }),
            profiles: Some(ProfileList { profiles }),
            ..View::default()
        });
        surface.apply(&turn);
        assert_eq!(surface.face(), Face::Gate);
        surface
    }

    /// One face per profile, each on the page's own grid, each pressable, none outside the window.
    #[test]
    fn every_profile_the_account_has_is_drawn_and_can_be_chosen() {
        let book = english();
        let people = vec![
            a_profile("Ada", false, false),
            a_profile("Grace", false, false),
            a_profile("Bo", true, false),
        ];
        for (wide, high) in WINDOWS {
            let window = Window::new(wide, high);
            let mut surface = watching(people.clone());
            let scene = surface.draw(window);
            assert!(scene.says(&book["profiles.choose"]), "at {wide}");
            for person in &people {
                assert!(
                    scene.says(&person.name),
                    "{} is not drawn at {wide}",
                    person.name
                );
            }
            // The child's profile says so, in the words the strings file carries.
            assert!(scene.says(&book["profile.badge.kids"]), "at {wide}");
            assert!(scene.outside().is_empty(), "a face is outside at {wide}");
            // Pressing a face enters that profile, by its own id.
            let asks = surface.press(window, "Grace");
            assert_eq!(
                asks,
                vec![Ask::Act(Action::EnterProfile {
                    profile: "p-Grace".to_owned(),
                    pin: None,
                })],
                "at {wide}"
            );
        }
    }

    /// The faces sit on the same grid the rows do: the page margin, the row's pitch, and the count
    /// across the reference measured. A grid of its own would be a second layout rule.
    #[test]
    fn the_faces_sit_on_the_page_s_own_grid() {
        for (wide, _) in WINDOWS {
            let across = sizes::across(wide);
            let first = face_at(wide, 0);
            let second = face_at(wide, 1);
            let next_row = face_at(wide, across as usize);
            assert!(
                (second.x - first.x - sizes::pitch(wide)).abs() < 0.01,
                "two faces are not one pitch apart at {wide}"
            );
            assert!(next_row.y > first.y, "the grid does not wrap at {wide}");
            assert!(
                (next_row.x - first.x).abs() < 0.01,
                "the second row does not start on the margin at {wide}"
            );
            assert!(
                first.x >= sizes::margin(wide) - 0.01,
                "a face is inside the page margin at {wide}"
            );
            assert!(
                (first.high - sizes::tile(wide) * sizes::TILE_HIGH).abs() < 0.01,
                "a face is not as tall as a tile at {wide}"
            );
        }
    }

    /// A locked profile asks for its PIN instead of entering, and the PIN is never drawn.
    #[test]
    fn a_locked_profile_asks_for_its_pin_and_never_draws_it() {
        let window = Window::new(1440.0, 900.0);
        let book = english();
        let mut surface = watching(vec![a_profile("Ada", false, true)]);
        let asks = surface.press(window, "Ada");
        assert!(
            asks.is_empty(),
            "a locked profile was entered without a PIN"
        );
        let scene = surface.draw(window);
        assert!(
            scene.says(&book["profile.pin.enter"]),
            "nothing asks for a PIN"
        );
        assert!(scene.says(&book["say.pin"]));
        surface.write(Field::Pin, "1234");
        let scene = surface.draw(window);
        for part in &scene.parts {
            assert!(
                part.text() != Some("1234"),
                "the PIN is drawn where anyone can read it"
            );
        }
        assert!(scene.says("\u{2022}\u{2022}\u{2022}\u{2022}"));
        let asks = surface.press(window, &book["common.done"]);
        assert_eq!(
            asks,
            vec![Ask::Act(Action::EnterProfile {
                profile: "p-Ada".to_owned(),
                pin: Some("1234".to_owned()),
            })]
        );
        // And it is gone from the surface the moment it was sent.
        assert_eq!(surface.text_in(Field::Pin), "");
    }

    /// A profile with no PIN is entered on the press, with no PIN at all rather than an empty one.
    #[test]
    fn a_profile_with_no_pin_is_entered_at_once() {
        let window = Window::new(1440.0, 900.0);
        let mut surface = watching(vec![a_profile("Ada", false, false)]);
        let asks = surface.press(window, "Ada");
        assert_eq!(
            asks,
            vec![Ask::Act(Action::EnterProfile {
                profile: "p-Ada".to_owned(),
                pin: None,
            })]
        );
    }

    #[test]
    fn every_label_this_screen_names_is_in_the_strings_file() {
        let book = english();
        for label in [
            "common.done",
            "profile.badge.kids",
            "profile.pin.enter",
            "profiles.choose",
            "say.pin",
            "say.profile_locked",
        ] {
            assert!(book.contains_key(label), "strings/en.json has no {label}");
        }
    }
}
