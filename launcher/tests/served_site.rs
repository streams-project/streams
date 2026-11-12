//! What a deployment publishes, and what it stopped publishing.
//!
//! The television page at `apps/tv/page` was served under `/tv/` on every start until 2026-11-12.
//! Work on it stopped on 2026-11-07, because a television gets an app of its own. What `/tv/` is
//! for now is the handful of media files those apps fetch from the deployment, and the first test
//! here holds that list against the surfaces that ask, in both directions: a name an app asks for
//! and nobody publishes is a film that never plays, and a name published that nobody asks for is
//! how a page outlived the decision to stop it.

use std::path::{Path, PathBuf};

use launcher::brand::Brand;
use launcher::site::{TV_MEDIA, publish, tv_media};

/// The surfaces that fetch from the deployment under `/tv/`. Both television families, both the
/// landing and the opening.
const TELEVISION_SURFACES: [&str; 4] = [
    "apps/android/surfaces/tv/src/main/kotlin/streams/tv/door/Door.kt",
    "apps/android/surfaces/tv/src/main/kotlin/streams/tv/app/Opening.kt",
    "apps/apple/Sources/SurfacesTV/door/Door.swift",
    "apps/apple/Sources/SurfacesTV/app/Opening.swift",
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// A folder of this test's own, named after what it is for, so two tests never share one.
fn scratch(what: &str) -> PathBuf {
    let folder =
        std::env::temp_dir().join(format!("launcher-served-{what}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&folder);
    folder
}

/// Every file name a text asks the deployment for under `/tv/`. A name has a dot in it, which is
/// what parts a file from an address like `/watch/tv/7/2/3`.
fn asked_for(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find("/tv/") {
        rest = &rest[at + "/tv/".len()..];
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
            .collect();
        if name.contains('.') {
            found.push(name);
        }
    }
    found
}

#[test]
fn the_television_apps_ask_for_exactly_what_the_deployment_publishes() {
    let root = root();
    let mut asked: Vec<String> = Vec::new();
    for file in TELEVISION_SURFACES {
        let text = std::fs::read_to_string(root.join(file))
            .unwrap_or_else(|why| panic!("{file} is readable: {why}"));
        asked.extend(asked_for(&text));
    }
    assert!(
        asked.len() >= TV_MEDIA.len(),
        "the surfaces were read and name something: {asked:?}"
    );
    for name in &asked {
        assert!(
            TV_MEDIA.contains(&name.as_str()),
            "a television app fetches /tv/{name} and the deployment does not publish it"
        );
    }
    for name in TV_MEDIA {
        assert!(
            asked.iter().any(|one| one == name),
            "the deployment publishes {name} and no television app asks for it"
        );
    }
}

#[test]
fn the_media_makes_its_own_folder_now_the_page_no_longer_does() {
    let home = scratch("media");
    let assets = home.join("assets");
    std::fs::create_dir_all(assets.join("media")).expect("the brand's media folder");
    // The two the page used and the apps do not are in here on purpose: a brand's folder keeps
    // whatever it keeps, and the deployment publishes its own list out of it.
    for name in [
        "showreel.mp4",
        "opening-chime.m4a",
        "opening-swell.m4a",
        "peek.jpg",
        "opening.mp4",
    ] {
        std::fs::write(assets.join("media").join(name), name).expect("a media file");
    }
    let tv = home.join("site/tv");
    assert!(!tv.exists(), "nothing has made the folder yet");
    tv_media(&assets, &tv).expect("the media is published");
    let mut published: Vec<String> = std::fs::read_dir(&tv)
        .expect("the folder was made by the first file published")
        .map(|entry| {
            entry
                .expect("an entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    published.sort();
    let mut wanted: Vec<String> = TV_MEDIA.iter().map(|name| (*name).to_owned()).collect();
    wanted.sort();
    assert_eq!(published, wanted);
    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn a_brand_with_no_media_leaves_no_empty_folder() {
    let home = scratch("nomedia");
    let assets = home.join("assets");
    std::fs::create_dir_all(&assets).expect("the brand's folder");
    let tv = home.join("site/tv");
    tv_media(&assets, &tv).expect("nothing to publish is not a failure");
    assert!(!tv.exists(), "an empty folder was made for nothing");
    let _ = std::fs::remove_dir_all(&home);
}

/// The whole published site, over a made-up root, so what is in it and what is not are both read
/// off one run rather than argued about.
#[test]
fn the_published_site_holds_the_website_the_marks_and_the_media_and_no_television_page() {
    let home = scratch("site");
    let tree = home.join("root");
    let brand = Brand::read(&root().join("brand.json")).expect("the brand file");
    std::fs::create_dir_all(tree.join("apps/web/dist")).expect("a built website");
    std::fs::write(tree.join("apps/web/dist/index.html"), "<!doctype html>").expect("its page");
    std::fs::write(tree.join("apps/web/dist/web.wasm"), "wasm").expect("its core");
    let assets = tree.join(&brand.assets);
    std::fs::create_dir_all(assets.join("media")).expect("the media folder");
    std::fs::create_dir_all(assets.join("avatars")).expect("the avatars folder");
    for name in TV_MEDIA {
        std::fs::write(assets.join("media").join(name), name).expect("a media file");
    }
    std::fs::write(assets.join("media/peek.jpg"), "peek").expect("the page's old background");
    std::fs::write(assets.join("avatars/grey.webp"), "grey").expect("an avatar");
    for name in ["mark.svg", "lockup-on-dark.svg"] {
        std::fs::write(assets.join(name), name).expect("a mark");
    }

    let site = publish(&tree, &home, &brand).expect("the site is published");

    let mut published: Vec<String> = Vec::new();
    let mut folders = vec![site.clone()];
    while let Some(folder) = folders.pop() {
        for entry in std::fs::read_dir(&folder).expect("a published folder") {
            let entry = entry.expect("an entry");
            if entry.file_type().expect("its kind").is_dir() {
                folders.push(entry.path());
            } else {
                published.push(
                    entry
                        .path()
                        .strip_prefix(&site)
                        .expect("under the site")
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }
    }
    published.sort();
    assert_eq!(
        published,
        [
            "avatars/grey.webp",
            "brand.css",
            "index.html",
            "lockup.svg",
            "mark.svg",
            "tv/opening-chime.m4a",
            "tv/opening-swell.m4a",
            "tv/showreel.mp4",
            "web.wasm",
        ]
    );
    // The page that was stopped, named one file at a time, because this is the assertion that
    // would have to be edited to publish it again.
    for gone in [
        "tv/index.html",
        "tv/tv.css",
        "tv/host.js",
        "tv/core.wasm",
        "tv/peek.jpg",
    ] {
        assert!(!site.join(gone).exists(), "{gone} is published again");
    }
    let _ = std::fs::remove_dir_all(&home);
}
