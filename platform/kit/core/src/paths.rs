//! Where the kit finds things. Source paths come from the manifest cargo compiled this crate from,
//! and build outputs from the folder cargo is writing this build into, so no path is spelled out
//! and a clone with its own target folder still links and runs what that build wrote.

use std::fs;
use std::path::{Path, PathBuf};

/// The repository root, the folder holding the workspace manifest.
#[must_use]
pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The cargo running this build, so a nested build uses the same one.
#[must_use]
pub fn cargo() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string())
}

/// The folder cargo writes this build into. Cargo gives every binary it runs its own path and marks
/// the folder it owns with a cache tag, so the nearest tagged folder above this binary is that
/// folder, whatever `CARGO_TARGET_DIR`, `--target-dir` or a config file set it to. Without a tag,
/// fall back to the shape cargo lays out, `<target>/<profile>/deps/<binary>`.
#[must_use]
pub fn target_root() -> PathBuf {
    let exe = std::env::current_exe().expect("cargo gives this binary a path");
    if let Some(folder) = exe.ancestors().find(|at| at.join("CACHEDIR.TAG").is_file()) {
        return folder.to_path_buf();
    }
    exe.ancestors()
        .nth(3)
        .expect("a test binary sits in <target>/<profile>/deps")
        .to_path_buf()
}

/// A folder of the kit's own inside that target folder, made if it is not there.
#[must_use]
pub fn target_folder(part: &str) -> PathBuf {
    let path = target_root().join(part);
    fs::create_dir_all(&path).expect("the folder can be made");
    path
}
