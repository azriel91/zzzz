use std::ops::{Deref, DerefMut};

use peace_data::fn_graph::FnId;

/// Runtime identifier for an [`Item`]. [`FnId`] newtype.
///
/// This is a cheap identifier to copy around, instead of cloning
/// [`ItemId`].
///
/// [`ItemId`]: peace_item_model::ItemId
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemRtId(FnId);

impl ItemRtId {
    /// Returns a new `ItemRtId`.
    pub fn new(fn_id: FnId) -> Self {
        Self(fn_id)
    }

    /// Returns the inner [`FnId`].
    pub fn into_inner(self) -> FnId {
        self.0
    }
}

impl Deref for ItemRtId {
    type Target = FnId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ItemRtId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<usize> for ItemRtId {
    fn from(index: usize) -> Self {
        Self(FnId::new(index))
    }
}

impl From<FnId> for ItemRtId {
    fn from(fn_id: FnId) -> Self {
        Self(fn_id)
    }
}
