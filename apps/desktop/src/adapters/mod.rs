//! How this host does what a port asks, with what a desktop has.

mod network;
mod player;
mod secrets;
mod store;

pub use network::HttpNetwork;
pub use player::NoPlayer;
pub use secrets::FileSecrets;
pub use store::FileStore;

/// A key as one file name. Store keys hold `/` (`q/0000000000000001`) and a secret is named by the
/// core, so neither can be a file name as it stands. Hex is written here rather than escaped:
/// every key comes back exactly, on a case insensitive file system as well, and no key can reach
/// out of its folder.
fn file_name(key: &str) -> String {
    let mut name = String::with_capacity(key.len() * 2);
    for byte in key.as_bytes() {
        name.push(char::from_digit(u32::from(byte >> 4), 16).unwrap_or('0'));
        name.push(char::from_digit(u32::from(byte & 15), 16).unwrap_or('0'));
    }
    name
}

/// The key a file name holds, or `None` when the name was not written by `file_name`.
fn key_of(name: &str) -> Option<String> {
    let digits = name.as_bytes();
    if !digits.len().is_multiple_of(2) {
        return None;
    }
    let mut bytes = Vec::with_capacity(digits.len() / 2);
    for pair in digits.chunks(2) {
        let high = char::from(pair[0]).to_digit(16)?;
        let low = char::from(pair[1]).to_digit(16)?;
        bytes.push(u8::try_from(high * 16 + low).ok()?);
    }
    String::from_utf8(bytes).ok()
}

/// Write a whole file, or say it was refused. The bytes land under a temporary name and are
/// renamed over the old file, so a run that stops halfway leaves the last good copy and never a
/// half written session.
fn write_whole(path: &std::path::Path, value: &str) -> bool {
    use std::io::Write as _;

    let Some(folder) = path.parent() else {
        return false;
    };
    if std::fs::create_dir_all(folder).is_err() {
        return false;
    }
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(".writing");
    let temporary = std::path::PathBuf::from(temporary);
    let written = create_own_only(&temporary).and_then(|mut file| {
        // A temporary left by a run that stopped halfway may carry another mode, so this says it
        // again on a file it did not create.
        own_only(&file)?;
        file.write_all(value.as_bytes())?;
        file.sync_all()
    });
    if written.is_err() || std::fs::rename(&temporary, path).is_err() {
        let _ = std::fs::remove_file(&temporary);
        return false;
    }
    true
}

/// Make a file nobody else can read, from the moment it exists. The mode goes on at creation
/// rather than after the write: a session token written into a `0644` file is readable by every
/// account on the machine for as long as the write takes, and a test on the finished file cannot
/// see that window.
#[cfg(unix)]
fn create_own_only(path: &std::path::Path) -> std::io::Result<std::fs::File> {
    use std::os::unix::fs::OpenOptionsExt as _;

    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
}

#[cfg(not(unix))]
fn create_own_only(path: &std::path::Path) -> std::io::Result<std::fs::File> {
    std::fs::File::create(path)
}

/// Keep a file to its owner. Windows has no such mode, so there the file carries the folder's
/// rights and `DECISIONS.md` says so out loud rather than leaving it to be assumed.
#[cfg(unix)]
fn own_only(file: &std::fs::File) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt as _;

    file.set_permissions(std::fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn own_only(_file: &std::fs::File) -> std::io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{file_name, key_of};

    /// The mode has to be on the file from the moment it exists. `tests/ports.rs` reads the
    /// finished secret and cannot see the moment between a `0644` creation and a later `chmod`,
    /// which is the moment a session token would be readable by every account on the machine.
    #[cfg(unix)]
    #[test]
    fn a_file_this_host_creates_is_the_owners_from_the_first_byte() {
        use std::os::unix::fs::PermissionsExt as _;

        let at = std::env::temp_dir().join(format!("desktop-create-mode-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("a folder for this test");
        let path = at.join("fresh");
        let file = super::create_own_only(&path).expect("a file");
        let mode = file.metadata().expect("its mode").permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "a token file was readable by someone else");
        drop(file);
        let _ = std::fs::remove_dir_all(&at);
    }

    #[test]
    fn a_key_with_a_slash_or_a_space_comes_back_as_it_went_in() {
        for key in [
            "q/0000000000000001",
            "index",
            "c/0000000000000002",
            "a b",
            "ليل",
        ] {
            let name = file_name(key);
            assert!(name.chars().all(|digit| digit.is_ascii_hexdigit()));
            assert_eq!(key_of(&name).as_deref(), Some(key));
        }
    }

    #[test]
    fn a_file_nobody_here_wrote_is_not_read_as_a_key() {
        assert_eq!(key_of("abc"), None);
        assert_eq!(key_of(".DS_Store"), None);
        assert_eq!(key_of("ffff"), None);
    }
}
