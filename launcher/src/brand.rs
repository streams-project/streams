//! The one file that names and styles a deployment. Nothing else in the repository holds a name
//! or a look, so changing this file changes the command, the server, the site and the apps.

use std::path::Path;

use anyhow::{Result, bail};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Colors {
    pub background: String,
    pub surface: String,
    pub raised: String,
    pub text: String,
    pub muted: String,
    pub accent: String,
    pub accent_text: String,
    pub join: String,
    pub join_text: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fonts {
    pub body: String,
    pub display: String,
    #[serde(default)]
    pub arabic: Option<String>,
    #[serde(default)]
    pub hebrew: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Shape {
    pub button_radius: u32,
    pub card_radius: u32,
    /// A card on a 390 wide page, measured at 12 where `card_radius` is 14
    /// (`reference/netflix-on-the-web.md`, line 273). A surface that draws it reads it from here.
    pub card_radius_compact: u32,
    /// A chip: a badge on its own, or one inside a chip row whose two outer ends take
    /// `chip_end_radius`. Both are drawn from this one value -- `apps/web/style.css`'s `.chip` and
    /// `phone/notifications/Notifications.kt`'s kind badge are the same part on two families.
    pub chip_radius: u32,
    pub chip_end_radius: u32,
    pub field_radius: u32,
    pub panel_radius: u32,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spacing {
    pub unit: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Brand {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub support_email: String,
    /// Folder of `mark.svg`, from the repository root.
    pub assets: String,
    pub colors: Colors,
    pub fonts: Fonts,
    pub shape: Shape,
    pub spacing: Spacing,
}

fn color_ok(text: &str) -> bool {
    let digits = text.strip_prefix('#').unwrap_or("");
    matches!(digits.len(), 3 | 6 | 8) && digits.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn font_ok(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == ' ' || c == '-')
}

fn command_ok(text: &str) -> bool {
    let first = text.chars().next();
    first.is_some_and(|c| c.is_ascii_lowercase())
        && text
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Every key this build reads out of the brand file, by path, and whether the file must hold it.
///
/// `different_ages` compares a file against this list, so a person is told which key moved rather
/// than handed whatever sentence a parser happened to produce. The same reason the rest of this
/// repository matches on what a thing did and not on one supplier's wording: that wording changes
/// under you, and then the test that watched for it quietly watches for nothing.
///
/// `one_key_of_this_table_is_every_key_of_the_struct` holds this list and the types above together:
/// a key added or dropped there and not here fails it.
const KEYS: [(&str, bool); 26] = [
    ("name", true),
    ("command", true),
    ("domain", false),
    ("support_email", false),
    ("assets", true),
    ("colors.background", true),
    ("colors.surface", true),
    ("colors.raised", true),
    ("colors.text", true),
    ("colors.muted", true),
    ("colors.accent", true),
    ("colors.accent_text", true),
    ("colors.join", true),
    ("colors.join_text", true),
    ("fonts.body", true),
    ("fonts.display", true),
    ("fonts.arabic", false),
    ("fonts.hebrew", false),
    ("shape.button_radius", true),
    ("shape.card_radius", true),
    ("shape.card_radius_compact", true),
    ("shape.chip_radius", true),
    ("shape.chip_end_radius", true),
    ("shape.field_radius", true),
    ("shape.panel_radius", true),
    ("spacing.unit", true),
];

/// What a brand file that will not read into this build turns out to be.
#[derive(Clone, Debug)]
pub enum Refusal {
    /// The file and this program were built from different versions of the same schema: the file
    /// names keys this build has never heard of, or it has dropped keys this build still reads.
    /// Neither side is broken. One of them is older.
    Ages {
        /// Keys the file holds that this build does not know.
        file_has: Vec<String>,
        /// Keys this build requires that the file does not hold.
        file_lacks: Vec<String>,
    },
    /// Missing, not JSON at all, or holding a value this build refuses.
    Wrong(String),
}

impl Refusal {
    /// What to print, with the file named, ending in what the reader can do about it.
    #[must_use]
    pub fn sentence(&self, path: &Path) -> String {
        let file = path.display();
        match self {
            Self::Wrong(why) => format!("{file} does not read: {why}"),
            Self::Ages {
                file_has,
                file_lacks,
            } => {
                let mut said = format!("{file} and this program are different ages.");
                if !file_has.is_empty() {
                    said.push_str(&format!(
                        " The file names {}, which this program does not know.",
                        file_has.join(", ")
                    ));
                }
                if !file_lacks.is_empty() {
                    said.push_str(&format!(
                        " The file has no {}, which this program still reads.",
                        file_lacks.join(", ")
                    ));
                }
                said.push_str(
                    " Build the program again from this copy of the repository and install it, or \
                     use the brand file this program was built against.",
                );
                said
            }
        }
    }
}

/// The keys of a brand file that this build does not know, and the ones it needs and did not find.
///
/// Both lists empty means the file's shape is the shape this build reads, whatever else is wrong
/// with it.
fn different_ages(file: &serde_json::Value) -> (Vec<String>, Vec<String>) {
    let mut file_has = Vec::new();
    let mut file_lacks = Vec::new();
    let Some(top) = file.as_object() else {
        return (file_has, file_lacks);
    };
    let known = |path: &str| KEYS.iter().any(|(key, _)| *key == path);
    let group = |name: &str| {
        KEYS.iter()
            .any(|(key, _)| key.starts_with(&format!("{name}.")))
    };
    for (name, value) in top {
        if group(name) {
            // A group this build reads: its own keys are judged one by one. A group that is not an
            // object at all is a wrong file, not an old one, so nothing is said about it here.
            if let Some(inside) = value.as_object() {
                for key in inside.keys() {
                    let path = format!("{name}.{key}");
                    if !known(&path) {
                        file_has.push(path);
                    }
                }
            }
        } else if !known(name) {
            file_has.push(name.clone());
        }
    }
    for (path, must) in KEYS {
        if !must {
            continue;
        }
        let found = match path.split_once('.') {
            Some((group, key)) => top.get(group).and_then(|g| g.get(key)).is_some(),
            None => top.get(path).is_some(),
        };
        if !found {
            file_lacks.push(path.to_owned());
        }
    }
    (file_has, file_lacks)
}

impl Brand {
    /// Read and check the file, so a bad value stops here with its name.
    ///
    /// # Errors
    /// When the file is missing, unreadable, or holds a value it should not.
    pub fn read(path: &Path) -> Result<Self> {
        Self::read_or_say(path).map_err(|refusal| anyhow::anyhow!(refusal.sentence(path)))
    }

    /// The same read, with the refusal classified: a file of another age is told apart from a file
    /// that is wrong.
    ///
    /// This exists because a brand file one key out of step used to take every command down with
    /// it, `doctor` included -- the command a person runs to be told what is wrong was the first
    /// thing a schema change took away. Nothing here accepts a file the strict read refuses: the
    /// shape of the answer changes, never the set of files that are allowed through.
    ///
    /// # Errors
    /// When the file is missing, unreadable, of another age, or holds a value it should not.
    pub fn read_or_say(path: &Path) -> std::result::Result<Self, Refusal> {
        let text = std::fs::read_to_string(path)
            .map_err(|why| Refusal::Wrong(format!("{} is missing: {why}", path.display())))?;
        match serde_json::from_str::<Self>(&text) {
            Ok(brand) => match brand.check() {
                Ok(()) => Ok(brand),
                Err(why) => Err(Refusal::Wrong(format!("{why:#}"))),
            },
            Err(parse) => {
                let Ok(loose) = serde_json::from_str::<serde_json::Value>(&text) else {
                    return Err(Refusal::Wrong(parse.to_string()));
                };
                let (file_has, file_lacks) = different_ages(&loose);
                if file_has.is_empty() && file_lacks.is_empty() {
                    return Err(Refusal::Wrong(parse.to_string()));
                }
                Err(Refusal::Ages {
                    file_has,
                    file_lacks,
                })
            }
        }
    }

    /// The name in a brand file that did not read, when the file still holds a plain one, so a
    /// program with no brand can still say whose install this is.
    #[must_use]
    pub fn name_in(path: &Path) -> Option<String> {
        let text = std::fs::read_to_string(path).ok()?;
        let file: serde_json::Value = serde_json::from_str(&text).ok()?;
        let name = file.get("name")?.as_str()?.trim();
        (!name.is_empty()).then(|| name.to_owned())
    }

    fn check(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            bail!("brand name is empty");
        }
        if !command_ok(&self.command) {
            bail!("brand command must be lower case letters, digits and hyphens");
        }
        let colors = &self.colors;
        for (name, value) in [
            ("background", &colors.background),
            ("surface", &colors.surface),
            ("raised", &colors.raised),
            ("text", &colors.text),
            ("muted", &colors.muted),
            ("accent", &colors.accent),
            ("accent_text", &colors.accent_text),
            ("join", &colors.join),
            ("join_text", &colors.join_text),
        ] {
            if !color_ok(value) {
                bail!("colour {name} must look like #2f6bff, found {value}");
            }
        }
        for (name, value) in [("body", &self.fonts.body), ("display", &self.fonts.display)] {
            if !font_ok(value) {
                bail!("font {name} must be a plain name, found {value}");
            }
        }
        if let Some(arabic) = &self.fonts.arabic
            && !font_ok(arabic)
        {
            bail!("font arabic must be a plain name, found {arabic}");
        }
        if let Some(hebrew) = &self.fonts.hebrew
            && !font_ok(hebrew)
        {
            bail!("font hebrew must be a plain name, found {hebrew}");
        }
        Ok(())
    }

    /// The look as a style sheet, loaded before the site's own.
    ///
    /// Every variable here has a user in `apps/web/style.css`, and `tests/one_name.rs` refuses one
    /// that has none: `--gutter` was emitted for years after its last user went with the television
    /// page, and `gutter` left this file and the brand file on 2026-11-12. `chip_end_radius` is for
    /// that reason not emitted. It is the corner a chip row's two outer ends take, the site draws no
    /// chip row, and the value already reaches the families that draw one (`CHIP_END_RADIUS` through
    /// Android's `BuildConfig`, `Corner.chipEnd` through `apps/apple/write-brand.py`). The day the
    /// site grows a chip row, the other half of that test -- a variable the site uses that the brand
    /// file does not give -- fails on `var(--radius-chip-end)` and names what to add here.
    #[must_use]
    pub fn css(&self) -> String {
        let (c, f, s, g) = (&self.colors, &self.fonts, &self.shape, &self.spacing);
        let arabic = f
            .arabic
            .as_deref()
            .map(|a| format!("  --font-arabic: \"{a}\";\n"))
            .unwrap_or_default();
        let hebrew = f
            .hebrew
            .as_deref()
            .map(|h| format!("  --font-hebrew: \"{h}\";\n"))
            .unwrap_or_default();
        format!(
            ":root {{\n  --background: {};\n  --surface: {};\n  --raised: {};\n  --text: {};\n  \
             --muted: {};\n  --accent: {};\n  --accent-text: {};\n  --join: {};\n  \
             --join-text: {};\n  --font-body: \"{}\";\n  --font-display: \"{}\";\n{arabic}{hebrew}  \
             --radius-button: {}px;\n  --radius-card: {}px;\n  --radius-chip: {}px;\n  \
             --radius-field: {}px;\n  --radius-panel: {}px;\n  --unit: {}px;\n}}\n",
            c.background,
            c.surface,
            c.raised,
            c.text,
            c.muted,
            c.accent,
            c.accent_text,
            c.join,
            c.join_text,
            f.body,
            f.display,
            s.button_radius,
            s.card_radius,
            s.chip_radius,
            s.field_radius,
            s.panel_radius,
            g.unit,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file() -> serde_json::Value {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../brand.json");
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    fn brand(value: serde_json::Value) -> Result<Brand> {
        let brand: Brand = serde_json::from_value(value)?;
        brand.check()?;
        Ok(brand)
    }

    #[test]
    fn the_file_in_this_repository_reads() {
        assert!(brand(file()).is_ok());
    }

    #[test]
    fn every_look_value_reaches_the_style_sheet() {
        let mut changed = file();
        changed["colors"]["accent"] = "#123456".into();
        changed["shape"]["button_radius"] = 22.into();
        changed["shape"]["chip_radius"] = 7.into();
        changed["fonts"]["display"] = "Some Font".into();
        let css = brand(changed).unwrap().css();
        assert!(css.contains("--accent: #123456;"));
        assert!(css.contains("--radius-button: 22px;"));
        assert!(css.contains("--radius-chip: 7px;"));
        assert!(css.contains("--font-display: \"Some Font\";"));
    }

    /// A chip row's outer corner is the one shape value the site is not given, and `css` saying so
    /// in a comment is not the same as a test saying so. The apps have it; `chip_end_radius` is read
    /// by `apps/android/surfaces/shared/build.gradle.kts` and `apps/apple/write-brand.py`.
    #[test]
    fn the_corner_no_page_draws_is_not_in_the_style_sheet() {
        let css = brand(file()).unwrap().css();
        assert!(
            !css.contains("chip-end"),
            "the site has no chip row to use it"
        );
        assert!(
            css.contains("--radius-chip: 12px;"),
            "and the chip itself is given"
        );
    }

    #[test]
    fn a_value_that_could_break_out_of_the_style_sheet_is_refused() {
        let mut bad = file();
        bad["colors"]["accent"] = "red; } body { display: none".into();
        assert!(brand(bad).is_err());
        let mut bad = file();
        bad["fonts"]["body"] = "x\"; } *{".into();
        assert!(brand(bad).is_err());
    }

    #[test]
    fn a_command_must_be_safe_as_a_file_name() {
        for name in ["../evil", "Has Space", "", "UPPER"] {
            let mut bad = file();
            bad["command"] = name.into();
            assert!(brand(bad).is_err(), "{name}");
        }
    }

    #[test]
    fn an_unknown_key_is_refused_so_a_typo_is_never_silent() {
        let mut bad = file();
        bad["colours"] = "#fff".into();
        assert!(brand(bad).is_err());
    }

    /// Write a brand file into a folder of this test's own and read it the way a command does.
    fn reading(
        value: &serde_json::Value,
    ) -> (std::path::PathBuf, std::result::Result<Brand, Refusal>) {
        let folder = std::env::temp_dir().join(format!(
            "brand-{}-{}",
            std::process::id(),
            COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&folder).expect("a folder");
        let path = folder.join("brand.json");
        std::fs::write(&path, value.to_string()).expect("a file");
        let read = Brand::read_or_say(&path);
        (path, read)
    }

    static COUNT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

    fn without(path: &str) -> serde_json::Value {
        let mut value = file();
        match path.split_once('.') {
            Some((group, key)) => {
                value[group].as_object_mut().expect("a group").remove(key);
            }
            None => {
                value.as_object_mut().expect("an object").remove(path);
            }
        }
        value
    }

    /// The table `different_ages` reads and the types above it, held together. Every key the table
    /// names must be one the strict read really knows, required exactly where the table says so,
    /// and a key the table does not name must be refused wherever it is put. A field added to a
    /// type and not to the table, or dropped from a type and left in it, fails here -- which is
    /// what stops the sentence a person reads from naming keys that no longer mean anything.
    #[test]
    fn one_key_of_this_table_is_every_key_of_the_struct() {
        for (path, must) in KEYS {
            let short = brand(without(path));
            assert_eq!(
                short.is_err(),
                must,
                "{path} is marked {}required and the read says otherwise",
                if must { "" } else { "not " }
            );
            // And the key is one this build knows: putting it back reads.
            assert!(brand(file()).is_ok());
        }
        // A key the table does not name, at the top and inside each group.
        let mut places = vec![String::from("x_unknown")];
        for (path, _) in KEYS {
            if let Some((group, _)) = path.split_once('.') {
                let at = format!("{group}.x_unknown");
                if !places.contains(&at) {
                    places.push(at);
                }
            }
        }
        assert_eq!(places.len(), 5, "the top level and four groups");
        for place in places {
            let mut bad = file();
            match place.split_once('.') {
                Some((group, key)) => bad[group][key] = 1.into(),
                None => bad[&place] = 1.into(),
            }
            assert!(brand(bad).is_err(), "{place} was accepted");
        }
    }

    /// The whole reason this classification exists: `spacing.gutter` left the brand file on
    /// 2026-11-12 and every command of the installed program, `doctor` first among them, answered
    /// with a parser's sentence about a missing field and exit 1. A program a key out of step with
    /// its brand file must say which key and that the two are different ages.
    #[test]
    fn a_brand_file_of_another_age_is_said_to_be_one_and_names_the_key() {
        // The file is older: it has dropped a key this build still reads.
        let (path, read) = reading(&without("spacing.unit"));
        let why = read.expect_err("a build that reads spacing.unit cannot read a file without it");
        let Refusal::Ages { file_lacks, .. } = &why else {
            panic!("{why:?} is not an age difference");
        };
        assert_eq!(file_lacks, &["spacing.unit"]);
        let said = why.sentence(&path);
        assert!(said.contains("different ages"), "{said}");
        assert!(said.contains("spacing.unit"), "{said}");

        // The file is newer: it names something this build has never heard of.
        let mut newer = file();
        newer["spacing"]["gutter"] = 16.into();
        let (path, read) = reading(&newer);
        let why = read.expect_err("an unknown key is still refused");
        let Refusal::Ages { file_has, .. } = &why else {
            panic!("{why:?} is not an age difference");
        };
        assert_eq!(file_has, &["spacing.gutter"]);
        assert!(why.sentence(&path).contains("spacing.gutter"));
    }

    /// And the other half: a file of this build's own age that is simply wrong is not excused as
    /// an age difference, because the answer to it is not to rebuild anything.
    #[test]
    fn a_file_this_build_understands_and_refuses_is_not_called_an_age_difference() {
        let mut bad = file();
        bad["colors"]["accent"] = "not a colour".into();
        let (_, read) = reading(&bad);
        assert!(matches!(read, Err(Refusal::Wrong(_))), "a bad value");

        let folder = std::env::temp_dir().join(format!("brand-text-{}", std::process::id()));
        std::fs::create_dir_all(&folder).expect("a folder");
        let path = folder.join("brand.json");
        std::fs::write(&path, "{ not json").expect("a file");
        assert!(matches!(Brand::read_or_say(&path), Err(Refusal::Wrong(_))));
        assert!(matches!(
            Brand::read_or_say(&folder.join("nothing.json")),
            Err(Refusal::Wrong(_))
        ));
    }

    /// A file that does not read can still say whose install this is, which is what `doctor` puts
    /// on its first line when it has no brand to print.
    #[test]
    fn the_name_is_readable_out_of_a_file_that_does_not_read() {
        let (path, read) = reading(&without("spacing.unit"));
        assert!(read.is_err());
        let name = Brand::name_in(&path).expect("the name is plain text in the file");
        assert_eq!(name, file()["name"].as_str().expect("a name"));
    }
}
