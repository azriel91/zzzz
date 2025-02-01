use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::ItemInteraction;

/// [`ItemInteraction`]s constructed from parameters derived from at least some
/// example state.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ItemInteractionsExample(Vec<ItemInteraction>);

impl ItemInteractionsExample {
    /// Returns a new `ItemInteractionsExample` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `ItemInteractionsExample` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Vec<ItemInteraction> {
        self.0
    }
}

impl Deref for ItemInteractionsExample {
    type Target = Vec<ItemInteraction>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ItemInteractionsExample {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<ItemInteraction>> for ItemInteractionsExample {
    fn from(inner: Vec<ItemInteraction>) -> Self {
        Self(inner)
    }
}

impl FromIterator<ItemInteraction> for ItemInteractionsExample {
    fn from_iter<I: IntoIterator<Item = ItemInteraction>>(iter: I) -> Self {
        Self(Vec::from_iter(iter))
    }
}
