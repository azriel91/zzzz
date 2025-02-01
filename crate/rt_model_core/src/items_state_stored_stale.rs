use std::ops::{Deref, DerefMut};

use indexmap::IndexMap;
use peace_item_model::ItemId;

use crate::StateStoredAndDiscovered;

/// Items whose stored and discovered state are not equal.
///
/// `IndexMap<ItemId, StateStoredAndDiscovered>` newtype.
///
/// This can be used for either current state or goal state.
#[derive(Clone, Debug, Default)]
pub struct ItemsStateStoredStale(IndexMap<ItemId, StateStoredAndDiscovered>);

impl ItemsStateStoredStale {
    /// Returns a new `ItemsStateStoredStale` map.
    pub fn new() -> Self {
        Self(IndexMap::new())
    }

    /// Returns a new `ItemsStateStoredStale` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(IndexMap::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> IndexMap<ItemId, StateStoredAndDiscovered> {
        self.0
    }

    /// Returns `true` if there is at least one stale stored state.
    pub fn stale(&self) -> bool {
        !self.0.is_empty()
    }
}

impl Deref for ItemsStateStoredStale {
    type Target = IndexMap<ItemId, StateStoredAndDiscovered>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ItemsStateStoredStale {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromIterator<(ItemId, StateStoredAndDiscovered)> for ItemsStateStoredStale {
    fn from_iter<I: IntoIterator<Item = (ItemId, StateStoredAndDiscovered)>>(iter: I) -> Self {
        Self(IndexMap::from_iter(iter))
    }
}
