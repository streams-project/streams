//! Opaque cursors. A cursor is position only, bound to the session and profile that asked, and the
//! answer is filtered again at read, so a cursor grants nothing.

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

/// Bytes of the tag kept in a cursor.
const TAG_BYTES: usize = 16;

/// Where a client stopped.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    /// Whose cursor this is: the session and profile, or `visitor`.
    pub owner: String,
    /// The screen, as the text of its key.
    pub screen: String,
    pub genre: Option<String>,
    /// The six hour slot the rows were built in, so a rebuild gives the same rows.
    pub slot: u64,
    /// The row being paged. None pages the sections of the screen.
    pub section: Option<String>,
    pub offset: u32,
}

/// Why a cursor was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorError {
    /// Not one this server wrote.
    Forged,
    /// Written for another session or profile.
    NotYours,
}

/// The key cursors are signed with. Made at start, so old cursors die with the process and a
/// client reads the screen again.
#[derive(Clone)]
pub struct CursorKey([u8; 32]);

impl std::fmt::Debug for CursorKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CursorKey(hidden)")
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn unhex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) || !text.is_ascii() {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).ok())
        .collect()
}

impl CursorKey {
    /// A key from bytes.
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// A random key.
    #[must_use]
    pub fn random() -> Self {
        let bytes: [u8; 32] = rand::random();
        Self(bytes)
    }

    fn tag(&self, body: &[u8]) -> Vec<u8> {
        let mut mac = Hmac::<Sha256>::new_from_slice(&self.0).unwrap_or_else(|_| unreachable!());
        mac.update(body);
        mac.finalize().into_bytes()[..TAG_BYTES].to_vec()
    }

    /// Write a cursor.
    #[must_use]
    pub fn write(&self, position: &Position) -> String {
        let body = serde_json::to_vec(position).unwrap_or_default();
        format!("{}.{}", hex(&body), hex(&self.tag(&body)))
    }

    /// Read a cursor for a caller.
    ///
    /// # Errors
    /// `Forged` for anything this key did not write, `NotYours` for another caller's cursor.
    pub fn read(&self, text: &str, owner: &str) -> Result<Position, CursorError> {
        let (body, tag) = text.split_once('.').ok_or(CursorError::Forged)?;
        let (body, tag) = (
            unhex(body).ok_or(CursorError::Forged)?,
            unhex(tag).ok_or(CursorError::Forged)?,
        );
        let mut mac = Hmac::<Sha256>::new_from_slice(&self.0).map_err(|_| CursorError::Forged)?;
        mac.update(&body);
        mac.verify_truncated_left(&tag)
            .map_err(|_| CursorError::Forged)?;
        if tag.len() != TAG_BYTES {
            return Err(CursorError::Forged);
        }
        let position: Position = serde_json::from_slice(&body).map_err(|_| CursorError::Forged)?;
        if position.owner != owner {
            return Err(CursorError::NotYours);
        }
        Ok(position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn position() -> Position {
        Position {
            owner: String::from("session:profile"),
            screen: String::from("home"),
            genre: None,
            slot: 1,
            section: Some(String::from("row-3")),
            offset: 24,
        }
    }

    #[test]
    fn h29_h92_a_cursor_reads_back() {
        let key = CursorKey::new([7; 32]);
        let cursor = key.write(&position());
        assert_eq!(key.read(&cursor, "session:profile").unwrap(), position());
    }

    #[test]
    fn tmk5_a_forged_or_edited_cursor_is_refused() {
        let key = CursorKey::new([7; 32]);
        assert_eq!(key.read("nonsense", "x"), Err(CursorError::Forged));
        let cursor = key.write(&position());
        let (body, tag) = cursor.split_once('.').unwrap();
        let other = hex(&serde_json::to_vec(&Position {
            offset: 0,
            ..position()
        })
        .unwrap());
        assert_eq!(
            key.read(&format!("{other}.{tag}"), "session:profile"),
            Err(CursorError::Forged)
        );
        assert_eq!(
            key.read(&format!("{body}.00"), "session:profile"),
            Err(CursorError::Forged)
        );
        let foreign = CursorKey::new([8; 32]);
        assert_eq!(
            foreign.read(&cursor, "session:profile"),
            Err(CursorError::Forged)
        );
    }

    #[test]
    fn tmk5_a_cursor_from_another_session_or_profile_is_refused() {
        let key = CursorKey::new([7; 32]);
        let cursor = key.write(&position());
        assert_eq!(
            key.read(&cursor, "session:kids"),
            Err(CursorError::NotYours)
        );
    }
}
