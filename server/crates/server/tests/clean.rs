//! No brand word, third party word, default address or owner value in any file of this
//! repository. The banned words are spelled in parts, so this file passes its own check.

use std::path::{Path, PathBuf};

fn walk(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("folder") {
        let path = entry.expect("entry").path();
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        if matches!(
            name.as_str(),
            "target" | ".git" | "Cargo.lock" | "LICENSE-APACHE" | "LICENSE-MIT" | "LICENSES"
        ) {
            continue;
        }
        if path.is_dir() {
            walk(&path, found);
        } else {
            found.push(path);
        }
    }
}

fn banned() -> Vec<String> {
    [
        ["net", "flix"],
        ["strea", "ms.watch"],
        ["strea", "ms-app"],
        ["pstr", "eam"],
        ["ib", "rahim"],
        ["supa", "base.co"],
        ["cloud", "flare"],
        ["tm", "db"],
        ["sub", "dl"],
        ["you", "tube"],
        ["hugging", "face"],
        ["co", "lab"],
    ]
    .iter()
    .map(|parts| parts.concat())
    .collect()
}

#[test]
fn h125_h127_no_brand_word_or_owner_value_in_any_file() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = Vec::new();
    walk(&root, &mut files);
    assert!(files.len() > 50, "the walk found the repository");
    let words = banned();
    for file in files {
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        let text = text.to_lowercase();
        for word in &words {
            assert!(!text.contains(word), "{} holds {word}", file.display());
        }
    }
}

#[test]
fn h126_no_address_is_built_into_the_source() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut files = Vec::new();
    walk(&root, &mut files);
    for file in files {
        let shown = file.to_string_lossy().into_owned();
        if !shown.ends_with(".rs") || shown.contains("/tests/") {
            continue;
        }
        let text = std::fs::read_to_string(&file).unwrap_or_default();
        for (n, line) in text.lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") || line.contains("#[test]") {
                continue;
            }
            let has_host = |scheme: &str| {
                code.match_indices(scheme)
                    .any(|(at, _)| code[at + scheme.len()..].starts_with(|c: char| c != '"'))
            };
            let address = has_host("\"http://") || has_host("\"https://");
            let in_test_module = text[..text.lines().take(n).map(|l| l.len() + 1).sum::<usize>()]
                .contains("#[cfg(test)]");
            assert!(!address || in_test_module, "{shown}:{}: {code}", n + 1);
        }
    }
}
