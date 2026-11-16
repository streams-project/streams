//! What a screen leaves behind for a painter: a window's worth of parts, back to front.
//!
//! A part is a rectangle, a corner, a colour **by its role** and one mark - a plate, some words, a
//! picture or the caret. That vocabulary is the whole of what any painter has to implement, and it
//! is deliberately the part that needs no window system at all: integers and floats over measured
//! numbers, so the screens are built and checked on whatever machine builds the tree. The model is
//! `apps/xbox/surfaces/remote/paint.h`, which does the same thing for a surface whose toolkit is
//! not on this machine either.
//!
//! No colour, no corner and no font family is written here. A part names the **role**
//! (`Ink::Accent`, `Corner::Card`, `Letters::Body`) and the painter resolves it against
//! `brand.json`, so this surface has no second copy of the brand to drift from it. The tests in
//! `super` read `brand.json` and hold every role this file names to a key that file really has.

use super::Ask;

/// A rectangle in the window, in the window's own pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub wide: f64,
    pub high: f64,
}

impl Rect {
    #[must_use]
    pub const fn new(x: f64, y: f64, wide: f64, high: f64) -> Self {
        Self { x, y, wide, high }
    }

    #[must_use]
    pub fn right(self) -> f64 {
        self.x + self.wide
    }

    #[must_use]
    pub fn bottom(self) -> f64 {
        self.y + self.high
    }

    /// Whether a pointer at this place is on this rectangle. The right and bottom edges belong to
    /// the next part, so two parts that share an edge never both answer.
    #[must_use]
    pub fn holds(self, x: f64, y: f64) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }
}

/// A colour, by the role `brand.json` gives it. Never a colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ink {
    Background,
    Surface,
    Raised,
    Text,
    Muted,
    Accent,
    AccentText,
    Join,
    JoinText,
}

impl Ink {
    /// The key under `colors` in `brand.json` that a painter resolves this against.
    #[must_use]
    pub const fn role(self) -> &'static str {
        match self {
            Self::Background => "background",
            Self::Surface => "surface",
            Self::Raised => "raised",
            Self::Text => "text",
            Self::Muted => "muted",
            Self::Accent => "accent",
            Self::AccentText => "accent_text",
            Self::Join => "join",
            Self::JoinText => "join_text",
        }
    }
}

/// A corner, by the role `brand.json` gives it. Never a radius.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Corner {
    Square,
    Button,
    Card,
    Chip,
    Field,
    Panel,
}

impl Corner {
    /// The key under `shape` in `brand.json`, or nothing for a square corner.
    #[must_use]
    pub const fn role(self) -> Option<&'static str> {
        match self {
            Self::Square => None,
            Self::Button => Some("button_radius"),
            Self::Card => Some("card_radius"),
            Self::Chip => Some("chip_radius"),
            Self::Field => Some("field_radius"),
            Self::Panel => Some("panel_radius"),
        }
    }
}

/// A typeface, by the role `brand.json` gives it. Never a family name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Letters {
    Body,
    Display,
}

impl Letters {
    /// The key under `fonts` in `brand.json`. A language with its own face is the painter's to
    /// choose between `arabic`, `hebrew` and this one, from the language the words arrived in.
    #[must_use]
    pub const fn role(self) -> &'static str {
        match self {
            Self::Body => "body",
            Self::Display => "display",
        }
    }
}

/// Where words sit in the box they were given.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    /// On the leading edge, which is the right in a language that runs that way.
    Start,
    /// In the middle of the box, both ways.
    Middle,
}

/// The one thing a part draws.
#[derive(Clone, Debug, PartialEq)]
pub enum Mark {
    /// A filled rectangle with this part's corner.
    Plate,
    /// Words, at a measured size and weight, in this part's ink.
    Words {
        text: String,
        size: f64,
        weight: u16,
        letters: Letters,
        align: Align,
    },
    /// A picture from this address, filling the part and clipped to its corner. The plate under it
    /// is a part of its own, so a picture that has not arrived leaves the plate showing.
    Picture { url: String },
    /// Where typing goes.
    Caret,
    /// The brand's lockup, at the size of this part. Which file that is comes from `brand.json`'s
    /// own `assets`, so the mark names no file and no brand.
    Lockup,
}

/// What a press on a part asks for.
#[derive(Clone, Debug, PartialEq)]
pub enum Touch {
    /// Put the caret in this field.
    Field(Field),
    /// Do this.
    Does(Ask),
}

/// A field a viewer types into. The surface holds what is typed; the scene draws it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Field {
    Email,
    Password,
    Pin,
}

impl Field {
    /// Whether what is typed is drawn as itself. A password is not, so the characters a viewer
    /// typed never reach the scene at all, let alone a painter or a screen capture.
    #[must_use]
    pub const fn secret(self) -> bool {
        matches!(self, Self::Password | Self::Pin)
    }
}

/// One part of a screen.
#[derive(Clone, Debug, PartialEq)]
pub struct Part {
    pub rect: Rect,
    pub ink: Ink,
    pub corner: Corner,
    pub mark: Mark,
    pub touch: Option<Touch>,
    /// What a reader is told this part is, already in the viewer's own language. Empty where the
    /// mark says it: words say themselves.
    pub says: String,
}

impl Part {
    #[must_use]
    pub fn plate(rect: Rect, ink: Ink, corner: Corner) -> Self {
        Self {
            rect,
            ink,
            corner,
            mark: Mark::Plate,
            touch: None,
            says: String::new(),
        }
    }

    #[must_use]
    pub fn words(rect: Rect, ink: Ink, text: impl Into<String>, size: f64, weight: u16) -> Self {
        Self {
            rect,
            ink,
            corner: Corner::Square,
            mark: Mark::Words {
                text: text.into(),
                size,
                weight,
                letters: Letters::Body,
                align: Align::Start,
            },
            touch: None,
            says: String::new(),
        }
    }

    /// The same words, in the middle of their box: what a button's label does.
    #[must_use]
    pub fn middle(mut self) -> Self {
        if let Mark::Words { align, .. } = &mut self.mark {
            *align = Align::Middle;
        }
        self
    }

    #[must_use]
    pub fn picture(rect: Rect, corner: Corner, url: impl Into<String>) -> Self {
        Self {
            rect,
            ink: Ink::Raised,
            corner,
            mark: Mark::Picture { url: url.into() },
            touch: None,
            says: String::new(),
        }
    }

    #[must_use]
    pub fn touched(mut self, touch: Touch) -> Self {
        self.touch = Some(touch);
        self
    }

    #[must_use]
    pub fn saying(mut self, says: impl Into<String>) -> Self {
        self.says = says.into();
        self
    }

    #[must_use]
    pub fn displayed(mut self) -> Self {
        if let Mark::Words { letters, .. } = &mut self.mark {
            *letters = Letters::Display;
        }
        self
    }

    /// The words this part draws, or nothing where it draws none.
    #[must_use]
    pub fn text(&self) -> Option<&str> {
        match &self.mark {
            Mark::Words { text, .. } => Some(text),
            _ => None,
        }
    }
}

/// A window's worth of parts, back to front, and what the viewer is on.
#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    pub face: super::Face,
    pub window: super::sizes::Window,
    pub parts: Vec<Part>,
}

impl Scene {
    #[must_use]
    pub fn new(face: super::Face, window: super::sizes::Window) -> Self {
        Self {
            face,
            window,
            parts: Vec::new(),
        }
    }

    pub fn add(&mut self, part: Part) {
        self.parts.push(part);
    }

    /// Every word this scene draws, in the order it draws them.
    #[must_use]
    pub fn words(&self) -> Vec<&str> {
        self.parts.iter().filter_map(Part::text).collect()
    }

    /// Whether this scene draws these words anywhere.
    #[must_use]
    pub fn says(&self, text: &str) -> bool {
        self.parts
            .iter()
            .any(|part| part.text() == Some(text) || part.says == text)
    }

    /// What a press at this place asks for: the front-most part that answers, because the parts
    /// are held back to front and a painter draws them in that order.
    #[must_use]
    pub fn touch(&self, x: f64, y: f64) -> Option<&Touch> {
        self.parts
            .iter()
            .rev()
            .find(|part| part.touch.is_some() && part.rect.holds(x, y))
            .and_then(|part| part.touch.as_ref())
    }

    /// Every part that is at least partly inside the window. A screen that lays a part outside it
    /// has drawn it nowhere, which is what the tests of each screen use this to refuse.
    #[must_use]
    pub fn outside(&self) -> Vec<&Part> {
        self.parts
            .iter()
            .filter(|part| {
                part.rect.x < 0.0
                    || part.rect.y < 0.0
                    || part.rect.right() > self.window.wide + 0.01
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{Corner, Field, Ink, Letters, Part, Rect};
    use crate::surfaces::reading::brand;

    /// Every role this file names is a key `brand.json` really has. This is what makes the surface
    /// brand free rather than brand ignorant: a role that no longer resolves fails here instead of
    /// turning a corner square or a colour black on a viewer's screen, which is what
    /// `apps/web/style.css`'s chip did this week by reading a role that was not its own.
    #[test]
    fn every_role_a_part_can_name_is_a_key_the_brand_file_has() {
        let brand = brand();
        let colors = brand["colors"].as_object().expect("brand colors");
        for ink in [
            Ink::Background,
            Ink::Surface,
            Ink::Raised,
            Ink::Text,
            Ink::Muted,
            Ink::Accent,
            Ink::AccentText,
            Ink::Join,
            Ink::JoinText,
        ] {
            assert!(
                colors.contains_key(ink.role()),
                "brand.json has no colour {}",
                ink.role()
            );
        }
        let shape = brand["shape"].as_object().expect("brand shape");
        for corner in [
            Corner::Button,
            Corner::Card,
            Corner::Chip,
            Corner::Field,
            Corner::Panel,
        ] {
            let role = corner.role().expect("a corner with a role");
            assert!(shape.contains_key(role), "brand.json has no corner {role}");
        }
        assert_eq!(Corner::Square.role(), None);
        let fonts = brand["fonts"].as_object().expect("brand fonts");
        for letters in [Letters::Body, Letters::Display] {
            assert!(
                fonts.contains_key(letters.role()),
                "brand.json has no face {}",
                letters.role()
            );
        }
    }

    /// No part of this surface writes a colour, a radius or a family name down. The check is on
    /// this crate's own surfaces, read as text: a hex colour or a `#` anywhere in them is a value
    /// with a second home, and the brand file stops deciding it.
    #[test]
    fn no_screen_writes_a_colour_of_its_own() {
        for file in [
            "scene.rs",
            "mod.rs",
            "sizes.rs",
            "door.rs",
            "gate.rs",
            "browse.rs",
            "watch.rs",
        ] {
            let whole =
                crate::surfaces::reading::source(&format!("apps/desktop/src/surfaces/{file}"));
            // The screens, not the tests of them: a test has to be able to write down the thing
            // it refuses, which is what this very test does three lines below.
            let text = whole.split("#[cfg(test)]").next().unwrap_or("").to_owned();
            for (number, line) in text.lines().enumerate() {
                // A doc comment may quote the reference, which names `rgb(20, 20, 20)` and
                // `rgba(128,128,128,0.4)`: prose about what Netflix drew is not this surface
                // drawing one. Code may not.
                let code = line.split("//").next().unwrap_or("");
                assert!(
                    !code.contains("#0")
                        && !code.contains("rgb(")
                        && !code.contains("rgba(")
                        && !code.contains("Inter"),
                    "{file}:{} writes a look value: {line}",
                    number + 1
                );
            }
        }
    }

    #[test]
    fn a_rectangle_owns_its_top_left_corner_and_not_its_bottom_right() {
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert!(rect.holds(10.0, 20.0));
        assert!(rect.holds(109.9, 69.9));
        // The far edges belong to whatever is next, so two parts sharing an edge never both
        // answer a press.
        assert!(!rect.holds(110.0, 40.0));
        assert!(!rect.holds(40.0, 70.0));
        assert!(!rect.holds(9.9, 40.0));
        assert_eq!((rect.right(), rect.bottom()), (110.0, 70.0));
    }

    #[test]
    fn a_password_field_is_the_one_that_never_draws_what_was_typed() {
        assert!(Field::Password.secret());
        assert!(Field::Pin.secret());
        assert!(!Field::Email.secret());
    }

    #[test]
    fn a_part_says_what_it_draws() {
        let part = Part::words(
            Rect::new(0.0, 0.0, 10.0, 10.0),
            Ink::Text,
            "hello",
            16.0,
            400,
        );
        assert_eq!(part.text(), Some("hello"));
        assert_eq!(
            Part::plate(Rect::new(0.0, 0.0, 1.0, 1.0), Ink::Raised, Corner::Card).text(),
            None
        );
    }
}
