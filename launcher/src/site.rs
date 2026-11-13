//! What a deployment serves: the built website with this brand's look over it, the brand's marks
//! and profile pictures, and the few media files an app fetches from the deployment instead of
//! carrying.
//!
//! Until 2026-11-12 this also published the television page from `apps/tv/page` under `/tv/`.
//! Work on that page stopped on 2026-11-07, because a television gets an app of its own and not a
//! page served by the server (`README.md`). A deployment was still copying it
//! on every start, so every address had a surface nothing opens. Its media stayed, because the
//! television apps read it from here.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::brand::Brand;

/// What `/tv/` holds, and the whole of it.
///
/// These are title art and the opening's two sounds, so they are kept out of the repository: a
/// brand puts its own in its `media/` folder, and the apps run without them. The opening itself is
/// drawn by each app.
///
/// Every name here is one a television app asks the deployment for, and the surfaces that ask are
/// `apps/android/surfaces/tv/src/main/kotlin/streams/tv/door/Door.kt` and `.../app/Opening.kt`,
/// `apps/apple/Sources/SurfacesTV/door/Door.swift` and `.../app/Opening.swift`.
/// `tests/served_site.rs` reads those four files and holds this list against them in both
/// directions, so neither a name an app asks for nor a name nothing asks for can sit here unseen.
pub const TV_MEDIA: [&str; 3] = ["showreel.mp4", "opening-chime.m4a", "opening-swell.m4a"];

/// The brand's marks, under the names the website asks for.
const MARKS: [(&str, &str); 2] = [
    ("mark.svg", "mark.svg"),
    ("lockup-on-dark.svg", "lockup.svg"),
];

fn copy_folder(from: &Path, to: &Path) -> Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_folder(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

/// The media the television apps fetch from the deployment.
///
/// The page copy used to make this folder on its way past. Now the first file published makes it,
/// so a brand with no media of its own leaves no empty folder behind.
pub fn tv_media(assets: &Path, tv: &Path) -> Result<()> {
    for name in TV_MEDIA {
        let from = assets.join("media").join(name);
        if from.exists() {
            std::fs::create_dir_all(tv)?;
            std::fs::copy(from, tv.join(name))?;
        }
    }
    Ok(())
}

/// The built site, with this brand's look and mark laid over it. Answers where it was put.
pub fn publish(root: &Path, home: &Path, brand: &Brand) -> Result<PathBuf> {
    let built = root.join("apps/web/dist");
    if !built.join("index.html").exists() {
        bail!("the website is not built, run ./up from the repository root");
    }
    let site = home.join("site");
    let _ = std::fs::remove_dir_all(&site);
    copy_folder(&built, &site)?;
    std::fs::write(site.join("brand.css"), brand.css())?;
    let assets = root.join(&brand.assets);
    tv_media(&assets, &site.join("tv"))?;
    // A brand's profile pictures. Optional: with none, profiles show a letter.
    let avatars = assets.join("avatars");
    if avatars.is_dir() {
        copy_folder(&avatars, &site.join("avatars"))?;
    }
    for (from, to) in MARKS {
        std::fs::copy(assets.join(from), site.join(to))
            .with_context(|| format!("{from} is missing from {}", assets.display()))?;
    }
    Ok(site)
}
