//! The brand's name lives in `brand.json` and in nowhere that decides how the product behaves, so
//! changing that one file renames everything.

use std::path::Path;
use std::process::Command;

use launcher::brand::Brand;

/// Files that are about the brand itself, read by people and never by a program, or someone
/// else's code, where the same word has its plain meaning.
const ABOUT_THE_BRAND: [&str; 7] = [
    "brand.json",
    "brand/",
    "LICENSES/",
    "REUSE.toml",
    "TRADEMARKS.md",
    "README.md",
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

/// The other direction: a look value the site asks for that no brand file gives it.
///
/// `the_site_defines_no_look_of_its_own_so_the_brand_file_decides` catches a variable with no user,
/// which is how `--gutter` was found. It cannot catch the opposite, and the opposite is what a brand
/// value missing from `brand.css` does to whoever needs it: `var(--radius-chip)` resolves to nothing,
/// the corner silently becomes square, and the quiet fix is a hard-coded number in the style sheet
/// or the nearest variable that looks close (the chip on `style.css:262` read `--radius-field` until
/// 2026-11-12, which is a different brand role that happens to carry the same number today).
///
/// So every `var(--name)` the site uses must be given by the brand file or set by the site itself.
/// The site's own are its measurements and its glass, which are not brand values: a page edge read
/// off a capture (`--edge`), a tile floor and a row count (`src/surfaces/sizes.rs` and
/// `src/surfaces/empty.rs` hold those to the site), and two white films.
#[test]
fn the_site_asks_for_no_look_value_the_brand_file_does_not_give() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let brand = Brand::read(&root.join("brand.json")).expect("the brand file");
    let site = std::fs::read_to_string(root.join("apps/web/style.css")).expect("the style sheet");
    let css = brand.css();
    let mut missing = Vec::new();
    for (at, _) in site.match_indices("var(--") {
        let name: String = site[at + 4..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();
        let given = css.contains(&format!("  {name}: "));
        let own = site.contains(&format!("{name}: "));
        if !(given || own) {
            missing.push(name);
        }
    }
    missing.sort_unstable();
    missing.dedup();
    assert!(
        missing.is_empty(),
        "style.css asks for {missing:?}, which neither brand.json nor style.css gives"
    );
}
