//! The language port: which language this viewer reads, and how the page says which way it runs.
//!
//! The phone asks the system (`java.util.Locale.getDefault().language`,
//! `apps/android/core/src/main/kotlin/streams/core/Host.kt:346`) and a browser is asked the same
//! question a different way. A page also has to be told its own language and direction, because the
//! browser itself lays out text, picks a font and reads the page aloud from those two attributes —
//! nothing a surface draws can stand in for them.

use crate::host::window;

/// The language a run starts in when nothing else is known. The phone's `DEFAULT_LANGUAGE`.
pub const DEFAULT: &str = "en";

/// The language this viewer reads, as a bare code: `en`, `ar`, `pt`.
///
/// `navigator.languages` is the viewer's own order of preference and is what a browser is meant to
/// be asked; `navigator.language` is the one the browser itself runs in, and is the fallback for a
/// browser that does not carry the list. Only the language is kept, not the region, because
/// `strings/` is a language a file: `pt.json`, not `pt-BR.json`.
pub fn preferred() -> String {
    let navigator = window().navigator();
    let listed = navigator
        .languages()
        .iter()
        .find_map(|tag| tag.as_string())
        .or_else(|| navigator.language());
    listed
        .as_deref()
        .and_then(|tag| tag.split(['-', '_']).next())
        .map(str::to_ascii_lowercase)
        .filter(|code| !code.is_empty())
        .unwrap_or_else(|| DEFAULT.to_owned())
}

/// Tell the page which language it is in and which way that language runs.
///
/// `lang` is what a screen reader picks its voice from and what the browser hyphenates and sorts
/// by; `dir` is what reverses the whole layout, so every start and end in the style sheet follows
/// the language without a second set of rules. The phone does the same with
/// `LocalLayoutDirection` (`apps/android/surfaces/phone/.../app/Root.kt:47`).
pub fn speak(lang: &str, rtl: bool) {
    let Some(root) = window()
        .document()
        .and_then(|document| document.document_element())
    else {
        return;
    };
    let _ = root.set_attribute("lang", lang);
    let _ = root.set_attribute("dir", if rtl { "rtl" } else { "ltr" });
}

#[cfg(test)]
mod tests {
    use super::DEFAULT;

    /// The default is a language `strings/` really has, and the same one the phone falls back to.
    #[test]
    fn the_default_language_is_one_the_server_has_words_for() {
        let root = crate::surfaces::tests_support::repo_root();
        assert!(
            root.join(format!("strings/{DEFAULT}.json")).is_file(),
            "strings/{DEFAULT}.json is not there"
        );
        let kotlin = std::fs::read_to_string(
            root.join("apps/android/core/src/main/kotlin/streams/core/Host.kt"),
        )
        .expect("the phone's host");
        let said = kotlin
            .split_once("DEFAULT_LANGUAGE = \"")
            .expect("the phone still names a default language")
            .1
            .split_once('"')
            .expect("the phone's default language is quoted")
            .0;
        assert_eq!(said, DEFAULT, "the two families fall back differently");
    }
}
