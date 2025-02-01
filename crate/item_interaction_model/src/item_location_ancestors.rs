use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::ItemLocation;

/// An [`ItemLocation`] and its ancestors.
///
/// The list of [`ItemLocation`]s within this container starts with the
/// outermost known ancestor, gradually moving closer to the innermost
/// `ItemLocation`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ItemLocationAncestors(Vec<ItemLocation>);

impl ItemLocationAncestors {
    /// Returns a new [`ItemLocationAncestors`] with the given ancestry.
    pub fn new(ancestors: Vec<ItemLocation>) -> Self {
        Self(ancestors)
    }

    /// Returns the underlying `Vec<ItemLocation>`.
    pub fn into_inner(self) -> Vec<ItemLocation> {
        self.0
    }
}

impl Deref for ItemLocationAncestors {
    type Target = Vec<ItemLocation>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ItemLocationAncestors {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<ItemLocation> for ItemLocationAncestors {
    fn from(item_location: ItemLocation) -> Self {
        Self(vec![item_location])
    }
}

impl From<Vec<ItemLocation>> for ItemLocationAncestors {
    fn from(ancestors: Vec<ItemLocation>) -> Self {
        Self(ancestors)
    }
}
