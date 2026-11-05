//! Opaque tokens. Only hashes are kept; the tokens are long random text, so one SHA-256 is enough.

use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::{Digest, Sha256};

/// The stored form of a token.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TokenHash(pub [u8; 32]);

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut text, byte| {
            let _ = write!(text, "{byte:02x}");
            text
        })
}

/// A new token of 256 random bits.
#[must_use]
pub fn new_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    hex(&bytes)
}

/// The hash kept for a token.
#[must_use]
pub fn hash_token(token: &str) -> TokenHash {
    TokenHash(Sha256::digest(token.as_bytes()).into())
}

/// The next refresh token follows from the current one, so a resend after a lost reply gets the
/// same answer while only hashes are stored.
pub(crate) fn derive_next(current: &str, salt: &str) -> String {
    let mut mac =
        Hmac::<Sha256>::new_from_slice(current.as_bytes()).expect("HMAC takes a key of any length");
    mac.update(salt.as_bytes());
    hex(&mac.finalize().into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_differ_and_hashes_are_stable() {
        let (a, b) = (new_token(), new_token());
        assert_ne!(a, b);
        assert_eq!(a.len(), 64);
        assert_eq!(hash_token(&a), hash_token(&a));
        assert_ne!(hash_token(&a), hash_token(&b));
    }

    #[test]
    fn the_next_token_repeats_for_one_current_token() {
        assert_eq!(derive_next("t", "s"), derive_next("t", "s"));
        assert_ne!(derive_next("t", "s"), derive_next("t", "other"));
    }
}
