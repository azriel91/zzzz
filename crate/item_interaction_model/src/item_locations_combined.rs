use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::ItemLocationTree;

/// All [`ItemLocation`]s from all items merged and deduplicated.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ItemLocationsCombined(Vec<ItemLocationTree>);

impl ItemLocationsCombined {
    /// Returns a new `ItemLocationsCombined`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `ItemLocationsCombined` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Vec<ItemLocationTree> {
        self.0
    }
}

impl Deref for ItemLocationsCombined {
    type Target = Vec<ItemLocationTree>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ItemLocationsCombined {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<ItemLocationTree>> for ItemLocationsCombined {
    fn from(inner: Vec<ItemLocationTree>) -> Self {
        Self(inner)
    }
}

impl FromIterator<ItemLocationTree> for ItemLocationsCombined {
    fn from_iter<I: IntoIterator<Item = ItemLocationTree>>(iter: I) -> Self {
        Self(Vec::from_iter(iter))
    }
}
