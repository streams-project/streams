//! The brand's name lives in `brand.json` and in nowhere that decides how the product behaves, so
//! changing that one file renames everything.

use std::path::Path;
use std::process::Command;

use launcher::brand::Brand;

/// Files that are about the brand itself, or read by people and never by a program.
const ABOUT_THE_BRAND: [&str; 7] = [
    "brand.json",
    "brand/",
    "LICENSES/",
    "REUSE.toml",
    "TRADEMARKS.md",
    "README.md",
    "apps/tv/README.md",
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
