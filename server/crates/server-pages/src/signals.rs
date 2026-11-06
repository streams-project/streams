//! What a page reads about a profile: titles it saved, watched and liked, as item keys. These come
//! from the library on the server, never from an address.

/// A profile's own titles.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Signals {
    pub saved: Vec<u32>,
    pub recent: Vec<u32>,
    pub liked: Vec<u32>,
    /// True with consent on. Off, only the profile's own lists are drawn, and nothing is
    /// recommended from what it did.
    pub personal: bool,
}

impl Signals {
    /// True for a profile with nothing saved, watched or liked.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.saved.is_empty() && self.recent.is_empty() && self.liked.is_empty()
    }
}
