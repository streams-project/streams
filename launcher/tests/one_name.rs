//! The brand's name lives in `brand.json` and in nowhere that decides how the product behaves, so
//! changing that one file renames everything.

use std::path::Path;
use std::process::Command;

use launcher::brand::Brand;

/// Files that are about the brand itself, read by people and never by a program, or someone
/// else's code, where the same word has its plain meaning.
const ABOUT_THE_BRAND: [&str; 8] = [
    "brand.json",
    "brand/",
    "LICENSES/",
    "REUSE.toml",
    "TRADEMARKS.md",
    "README.md",
    "pstream/notes/netflix-reference/tv/00-netflix-on-the-tv-measured.md",
    "apps/web/vendor/",
];

#[test]
fn no_program_file_states_the_brand_name() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let brand = Brand::read(&root.join("brand.json")).expect("the brand file");
    let listed = Command::new("git")
        .arg("ls-files")
        .current_dir(&root)
        .output()
        .expect("git runs");
    let listed = String::from_utf8_lossy(&listed.stdout).into_owned();
    assert!(
        listed.lines().count() > 100,
        "the walk found the repository"
    );
    let mut found = Vec::new();
    for file in listed.lines() {
        if ABOUT_THE_BRAND.iter().any(|about| file.starts_with(about)) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(root.join(file)) else {
            continue;
        };
        if text.contains(&brand.name) {
            found.push(file.to_owned());
        }
    }
    assert!(found.is_empty(), "{found:?} state the name {}", brand.name);
}

#[test]
fn the_site_defines_no_look_of_its_own_so_the_brand_file_decides() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let brand = Brand::read(&root.join("brand.json")).expect("the brand file");
    let site = std::fs::read_to_string(root.join("apps/web/style.css")).expect("the style sheet");
    let css = brand.css();
    let names = css.lines().filter_map(|line| line.trim().split(':').next());
    for name in names.filter(|name| name.starts_with("--")) {
        assert!(
            site.contains(&format!("var({name})")),
            "{name} is never used"
        );
        assert!(
            !site.contains(&format!("{name}:")),
            "style.css sets {name}, which would hide the brand file"
        );
    }
}
