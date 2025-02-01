use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::ItemInteraction;

/// [`ItemInteraction`]s constructed from parameters derived from fully known
/// state.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ItemInteractionsCurrent(Vec<ItemInteraction>);

impl ItemInteractionsCurrent {
    /// Returns a new `ItemInteractionsCurrent` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `ItemInteractionsCurrent` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Vec<ItemInteraction> {
        self.0
    }
}

impl Deref for ItemInteractionsCurrent {
    type Target = Vec<ItemInteraction>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ItemInteractionsCurrent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<ItemInteraction>> for ItemInteractionsCurrent {
    fn from(inner: Vec<ItemInteraction>) -> Self {
        Self(inner)
    }
}

impl FromIterator<ItemInteraction> for ItemInteractionsCurrent {
    fn from_iter<I: IntoIterator<Item = ItemInteraction>>(iter: I) -> Self {
        Self(Vec::from_iter(iter))
    }
}
