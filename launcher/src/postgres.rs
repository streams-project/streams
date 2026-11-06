//! One pinned Postgres, fetched on first use and checked against the hashes kept here, so a fresh
//! copy needs no database installed. It listens on this machine only, behind a password.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use flate2::read::GzDecoder;
use rand::Rng;
use sha2::{Digest, Sha256};

const VERSION: &str = "18.6.1";
const RELEASES: &str = "https://github.com/theseus-rs/postgresql-binaries/releases/download";
const USER: &str = "postgres";

/// The machines this runs on, and the hash of the one archive each may use.
const BUILDS: [(&str, &str, &str); 4] = [
    (
        "macos",
        "x86_64",
        "4229812543e19c91864ebdf5d8f78883c5816d51d4cc5d065298260c1f64d37e",
    ),
    (
        "macos",
        "aarch64",
        "89e91cc80650a43d22c018b95ca356a0c9e15838a1182fed2880bfb5d7d9c9a6",
    ),
    (
        "linux",
        "x86_64",
        "e84b02dd8d0ce5879fb697330a789231c8b8f8d958dae3ad4d90fecc710b1315",
    ),
    (
        "linux",
        "aarch64",
        "1ab47fa2f06ff70454309ad02e5d89c46488300d43977231481fd6b160b9d556",
    ),
];

fn archive_name(os: &str, arch: &str) -> String {
    let target = if os == "macos" {
        format!("{arch}-apple-darwin")
    } else {
        format!("{arch}-unknown-linux-gnu")
    };
    format!("postgresql-{VERSION}-{target}")
}

fn pinned_hash(os: &str, arch: &str) -> Option<&'static str> {
    BUILDS
        .iter()
        .find(|build| build.0 == os && build.1 == arch)
        .map(|build| build.2)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// The `bin` folder of Postgres: the one named by `DATABASE_BIN` when an installed copy should be
/// used, otherwise the pinned build, fetched into `cache` the first time.
///
/// # Errors
/// When this machine has no build, the download fails, or its hash is wrong.
pub async fn install(cache: &Path) -> Result<PathBuf> {
    if let Some(installed) = std::env::var_os("DATABASE_BIN") {
        return Ok(PathBuf::from(installed));
    }
    let (os, arch) = (std::env::consts::OS, std::env::consts::ARCH);
    let Some(expected) = pinned_hash(os, arch) else {
        bail!("no database build for {os} on {arch}");
    };
    let name = archive_name(os, arch);
    let bin = cache.join(&name).join("bin");
    if bin.join("postgres").exists() {
        return Ok(bin);
    }
    let url = format!("{RELEASES}/{VERSION}/{name}.tar.gz");
    let answer = reqwest::get(&url).await?.error_for_status()?;
    let archive = answer.bytes().await?;
    let found = hex(&Sha256::digest(&archive));
    if found != expected {
        bail!("the database download has hash {found}, expected {expected}");
    }
    std::fs::create_dir_all(cache)?;
    let unpacking = cache.join(format!("{name}.unpacking"));
    let _ = std::fs::remove_dir_all(&unpacking);
    std::fs::create_dir_all(&unpacking)?;
    tar::Archive::new(GzDecoder::new(archive.as_ref()))
        .unpack(&unpacking)
        .context("the database archive did not unpack")?;
    std::fs::rename(unpacking.join(&name), cache.join(&name))?;
    let _ = std::fs::remove_dir_all(&unpacking);
    Ok(bin)
}

/// A running database. Dropping it stops it.
#[derive(Debug)]
pub struct Postgres {
    pub url: String,
    bin: PathBuf,
    data: PathBuf,
}

fn quiet(mut command: Command) -> Result<()> {
    let shown = format!("{command:?}");
    let output = command.stdin(Stdio::null()).output()?;
    if output.status.success() {
        return Ok(());
    }
    bail!(
        "{shown} failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    )
}

fn new_password() -> String {
    let bytes: [u8; 24] = rand::rng().random();
    hex(&bytes)
}

impl Postgres {
    /// Start the database in `data` on `port`, creating it the first time. The password stays in
    /// a private file beside the data.
    ///
    /// # Errors
    /// When a step of creating or starting the database fails.
    pub fn start(bin: &Path, data: &Path, port: u16) -> Result<Self> {
        let parent = data.parent().context("the data folder has no parent")?;
        std::fs::create_dir_all(parent)?;
        let keyfile = parent.join("database.password");
        if !data.join("PG_VERSION").exists() {
            let password = new_password();
            write_private(&keyfile, &password)?;
            let mut init = Command::new(bin.join("initdb"));
            init.arg("-D")
                .arg(data)
                .args(["-U", USER, "-E", "UTF8", "--locale=C"])
                .args(["--auth=scram-sha-256", "--pwfile"])
                .arg(&keyfile);
            quiet(init)?;
        }
        let password = std::fs::read_to_string(&keyfile)?;
        let this = Self {
            url: format!("postgres://{USER}:{password}@127.0.0.1:{port}/{USER}"),
            bin: bin.to_path_buf(),
            data: data.to_path_buf(),
        };
        let mut start = Command::new(bin.join("pg_ctl"));
        start
            .arg("-D")
            .arg(data)
            .arg("-l")
            .arg(parent.join("database.log"))
            .args(["-w", "-t", "60", "-o"])
            .arg(format!(
                "-p {port} -c listen_addresses=127.0.0.1 -c unix_socket_directories=''"
            ))
            .arg("start");
        // pg_ctl leaves the server running with these pipes held, so they must not be captured.
        start
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null());
        if !start.status()?.success() {
            bail!(
                "the database did not start, see {}",
                parent.join("database.log").display()
            );
        }
        Ok(this)
    }

    /// Stop the database, finishing what is in progress quickly.
    pub fn stop(&self) {
        let _ = Command::new(self.bin.join("pg_ctl"))
            .arg("-D")
            .arg(&self.data)
            .args(["-w", "-m", "fast", "stop"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

impl Drop for Postgres {
    fn drop(&mut self) {
        self.stop();
    }
}

fn write_private(path: &Path, text: &str) -> Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(text.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_machine_has_its_own_hash() {
        for (os, arch, hash) in BUILDS {
            assert_eq!(hash.len(), 64, "{os} {arch}");
            assert_eq!(pinned_hash(os, arch), Some(hash));
        }
        assert_eq!(pinned_hash("windows", "x86_64"), None);
    }

    #[test]
    fn the_archive_names_match_the_releases() {
        assert_eq!(
            archive_name("macos", "x86_64"),
            "postgresql-18.6.1-x86_64-apple-darwin"
        );
        assert_eq!(
            archive_name("linux", "aarch64"),
            "postgresql-18.6.1-aarch64-unknown-linux-gnu"
        );
    }
}
