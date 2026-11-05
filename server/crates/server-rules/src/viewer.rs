//! The proof types. `TierVisible` may be cached per surface, genre, tier and class. `Visible` is
//! minted last, per request, after the profile's blocked names.

use std::collections::HashSet;

use server_accounts::ProfileScope;
use server_catalog::DataSet;
use server_catalog::catalog::CatalogItem;

use crate::block::normalise_name;
use crate::eligibility::eligible;
use crate::tier::{RuleError, Tier};

/// Proof that a title passed the tier check. Only a [`Viewer`] makes one, and only a viewer of
/// the same tier can turn it into [`Visible`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TierVisible {
    key: u32,
    tier: Tier,
}

impl TierVisible {
    /// The item key of the title.
    #[must_use]
    pub const fn key(self) -> u32 {
        self.key
    }
}

/// Proof that this viewer may see this title now. Fields are private, so response code cannot
/// make one for a title that was not checked.
#[derive(Clone, Copy, Debug)]
pub struct Visible<'data> {
    item: &'data CatalogItem,
}

impl<'data> Visible<'data> {
    /// The title facts.
    #[must_use]
    pub const fn item(self) -> &'data CatalogItem {
        self.item
    }
}

/// Who a response is for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Viewer {
    tier: Tier,
    blocked: HashSet<String>,
}

impl Viewer {
    /// The viewer of a door scope.
    ///
    /// # Errors
    /// The profile's limit cannot be read.
    pub fn of(scope: &ProfileScope) -> Result<Self, RuleError> {
        Ok(Self {
            tier: Tier::of_scope(scope)?,
            blocked: scope
                .blocked_names()
                .iter()
                .map(|name| normalise_name(name))
                .filter(|name| !name.is_empty())
                .collect(),
        })
    }

    /// A signed out visitor at the tier the deployer stated.
    #[must_use]
    pub fn visitor(tier: Tier) -> Self {
        Self {
            tier,
            blocked: HashSet::new(),
        }
    }

    /// The tier of this viewer.
    #[must_use]
    pub const fn tier(&self) -> Tier {
        self.tier
    }

    fn blocks(&self, item: &CatalogItem) -> bool {
        !self.blocked.is_empty()
            && std::iter::once(&item.title)
                .chain(&item.other_names)
                .any(|name| self.blocked.contains(&normalise_name(name)))
    }

    /// The tier check alone, for lists cached per tier.
    #[must_use]
    pub fn tier_check(&self, data: &DataSet, key: u32) -> Option<TierVisible> {
        let item = data.catalog.get(&key)?;
        eligible(self.tier, item, &data.profiles).then_some(TierVisible {
            key,
            tier: self.tier,
        })
    }

    /// Mint `Visible` from a cached tier proof. A proof of another tier is refused.
    #[must_use]
    pub fn visible<'data>(
        &self,
        data: &'data DataSet,
        proof: TierVisible,
    ) -> Option<Visible<'data>> {
        if proof.tier != self.tier {
            return None;
        }
        let item = data.catalog.get(&proof.key)?;
        (!self.blocks(item)).then_some(Visible { item })
    }

    /// Both checks for one title.
    #[must_use]
    pub fn see<'data>(&self, data: &'data DataSet, key: u32) -> Option<Visible<'data>> {
        let proof = self.tier_check(data, key)?;
        self.visible(data, proof)
    }
}
