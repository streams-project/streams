//! The one file that names and styles a deployment. Nothing else in the repository holds a name
//! or a look, so changing this file changes the command, the server, the site and the apps.

use std::path::Path;

use anyhow::{Context, Result, bail};
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
    /// A chip's corners inside a chip row; the row's two outer ends take `chip_end_radius`.
    pub chip_radius: u32,
    pub chip_end_radius: u32,
    pub field_radius: u32,
    pub panel_radius: u32,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spacing {
    pub unit: u32,
    pub gutter: u32,
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

impl Brand {
    /// Read and check the file, so a bad value stops here with its name.
    ///
    /// # Errors
    /// When the file is missing, unreadable, or holds a value it should not.
    pub fn read(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("{} is missing", path.display()))?;
        let brand: Self = serde_json::from_str(&text)
            .with_context(|| format!("{} does not read", path.display()))?;
        brand.check()?;
        Ok(brand)
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
        if let Some(arabic) = &self.fonts.arabic {
            if !font_ok(arabic) {
                bail!("font arabic must be a plain name, found {arabic}");
            }
        }
        if let Some(hebrew) = &self.fonts.hebrew {
            if !font_ok(hebrew) {
                bail!("font hebrew must be a plain name, found {hebrew}");
            }
        }
        Ok(())
    }

    /// The look as a style sheet, loaded before the site's own. The chip corners stay with the apps
    /// until the site has a chip row (the site test refuses a value it never uses).
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
             --radius-button: {}px;\n  --radius-card: {}px;\n  --radius-field: {}px;\n  \
             --radius-panel: {}px;\n  --unit: {}px;\n  --gutter: {}px;\n}}\n",
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
            s.field_radius,
            s.panel_radius,
            g.unit,
            g.gutter,
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
        changed["fonts"]["display"] = "Some Font".into();
        let css = brand(changed).unwrap().css();
        assert!(css.contains("--accent: #123456;"));
        assert!(css.contains("--radius-button: 22px;"));
        assert!(css.contains("--font-display: \"Some Font\";"));
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
}
