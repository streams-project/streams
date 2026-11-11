//! How this host does what a port asks, with what a desktop has.

mod network;
mod player;
mod reel;
mod secrets;
mod store;

pub use network::HttpNetwork;
pub use player::NoPlayer;
pub use reel::NoReel;
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
    let written = std::fs::File::create(&temporary).and_then(|mut file| {
        file.write_all(value.as_bytes())?;
        own_only(&file)?;
        file.sync_all()
    });
    if written.is_err() || std::fs::rename(&temporary, path).is_err() {
        let _ = std::fs::remove_file(&temporary);
        return false;
    }
    true
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

    #[test]
    fn a_key_with_a_slash_or_a_space_comes_back_as_it_went_in() {
        for key in [
            "q/0000000000000001",
            "index",
            "b/0000000000000002",
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
