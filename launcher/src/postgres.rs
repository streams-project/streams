//! One pinned Postgres, fetched on first use and checked against the hashes kept here, so a fresh
//! copy needs no database installed. It listens on this machine only, behind a password.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::{Context, Result, bail};
use flate2::read::GzDecoder;
use rand::Rng;
use sha2::{Digest, Sha256};

const VERSION: &str = "18.6.1";
/// The most this will hold in memory for one archive. The macOS x86-64 build of this version is
/// 92,085,423 bytes (its release `Content-Length`, read 2026-11-11), so this is a ceiling with
/// room for a larger build of another machine and not a second pin on the body.
///
/// The hash below is what makes a substituted archive useless, and it still is: this cap is not
/// there to catch a wrong body but to stop an unbounded one. Reading a body whole before anything
/// judges it was the only read in this program with no size at all, so what a server that answered
/// for ever cost was this process's memory, before the hash ever got a look.
const ARCHIVE_MAX_BYTES: usize = 256 * 1024 * 1024;
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

/// Counts unpacking folders within one run.
static UNPACKING: AtomicU32 = AtomicU32::new(0);

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
    let archive = fetch_capped(&url, ARCHIVE_MAX_BYTES).await?;
    let found = hex(&Sha256::digest(&archive));
    if found != expected {
        bail!("the database download has hash {found}, expected {expected}");
    }
    std::fs::create_dir_all(cache)?;
    // Each start unpacks in its own folder, so two at once cannot empty each other's.
    let turn = UNPACKING.fetch_add(1, Ordering::Relaxed);
    let unpacking = cache.join(format!(".unpacking-{}-{turn}", std::process::id()));
    let _ = std::fs::remove_dir_all(&unpacking);
    std::fs::create_dir_all(&unpacking)?;
    tar::Archive::new(GzDecoder::new(archive.as_slice()))
        .unpack(&unpacking)
        .context("the database archive did not unpack")?;
    let placed = std::fs::rename(unpacking.join(&name), cache.join(&name));
    let _ = std::fs::remove_dir_all(&unpacking);
    // Another start can finish first, and then its copy is the one in place.
    if let Err(error) = placed
        && !bin.join("postgres").exists()
    {
        return Err(error.into());
    }
    Ok(bin)
}

/// Read a body, stopping at the cap rather than holding whatever arrives.
///
/// A declared length over the cap is refused before a byte is read, and a body that declares
/// nothing is stopped at the cap as it arrives, so neither a wrong header nor a missing one buys
/// more than `cap` bytes of memory.
///
/// # Errors
/// The call failed, the service answered with an error, or the body passed the cap.
async fn fetch_capped(url: &str, cap: usize) -> Result<Vec<u8>> {
    let mut answer = reqwest::get(url).await?.error_for_status()?;
    if let Some(declared) = answer.content_length()
        && declared > cap as u64
    {
        bail!("the database download declares {declared} bytes, over the {cap} this reads");
    }
    let mut body: Vec<u8> = Vec::new();
    while let Some(chunk) = answer.chunk().await? {
        if body.len() + chunk.len() > cap {
            bail!("the database download passed {cap} bytes, which is more than this reads");
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
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

/// Write a file only this user can read.
///
/// # Errors
/// When the file cannot be written.
pub fn write_private(path: &Path, text: &str) -> Result<()> {
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

    /// One HTTP answer of this many bytes, with or without a length header, over raw TCP: what is
    /// proved here is what this program does with a body, not what any library does with a URL.
    async fn serving(bytes: usize, declare: bool) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("a port");
        let url = format!(
            "http://{}/x.tar.gz",
            listener.local_addr().expect("address")
        );
        tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                // Read the request first: a server that answers and closes without reading it
                // resets the connection before the client is done sending.
                let mut asked = [0u8; 2048];
                let _ = stream.read(&mut asked).await;
                let head = if declare {
                    format!("HTTP/1.1 200 OK\r\nContent-Length: {bytes}\r\n\r\n")
                } else {
                    // No length and no chunking: the body is whatever arrives until the close,
                    // which is the shape a cap on the header alone would not catch.
                    String::from("HTTP/1.0 200 OK\r\n\r\n")
                };
                let _ = stream.write_all(head.as_bytes()).await;
                let _ = stream.write_all(&vec![b'z'; bytes]).await;
                let _ = stream.shutdown().await;
            }
        });
        url
    }

    /// The archive used to be read whole before the hash judged it, so a body of any size cost
    /// this process's memory first. Both an honest length and none at all now stop at the cap.
    #[tokio::test]
    async fn a_download_over_the_cap_is_refused_however_it_declares_itself() {
        for declare in [true, false] {
            let url = serving(4096, declare).await;
            let why = fetch_capped(&url, 1024)
                .await
                .expect_err("read over the cap")
                .to_string();
            assert!(why.contains("1024"), "{why}");
        }
        // And a body within it arrives whole, so the cap is not in the way of the real archive.
        let url = serving(4096, true).await;
        assert_eq!(fetch_capped(&url, 4096).await.expect("read").len(), 4096);
    }
}
