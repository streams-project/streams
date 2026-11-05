//! What reads hand out.

use std::collections::BTreeMap;

/// Settings of a profile. Age rating and blocked titles are not here on purpose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingsView {
    pub autoplay_next: bool,
    pub autoplay_previews: bool,
    pub audio_language: Option<String>,
    pub subtitle_language: Option<String>,
    pub subtitle_style: BTreeMap<String, String>,
    pub ui_language: Option<String>,
    pub handle: Option<String>,
    pub gender: Option<String>,
}

impl Default for SettingsView {
    fn default() -> Self {
        Self {
            autoplay_next: true,
            autoplay_previews: true,
            audio_language: None,
            subtitle_language: None,
            subtitle_style: BTreeMap::new(),
            ui_language: None,
            handle: None,
            gender: None,
        }
    }
}
