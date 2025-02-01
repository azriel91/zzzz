use serde::{Deserialize, Serialize};

use crate::{ItemInteractionsCurrent, ItemInteractionsExample};

/// [`ItemInteraction`]s constructed from parameters derived from at least some
/// example state.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum ItemInteractionsCurrentOrExample {
    /// [`ItemInteraction`]s constructed from parameters derived from fully
    /// known state.
    Current(ItemInteractionsCurrent),
    /// [`ItemInteraction`]s constructed from parameters derived from at least
    /// some example state.
    Example(ItemInteractionsExample),
}

impl From<ItemInteractionsCurrent> for ItemInteractionsCurrentOrExample {
    fn from(item_interactions_current: ItemInteractionsCurrent) -> Self {
        Self::Current(item_interactions_current)
    }
}

impl From<ItemInteractionsExample> for ItemInteractionsCurrentOrExample {
    fn from(item_interactions_example: ItemInteractionsExample) -> Self {
        Self::Example(item_interactions_example)
    }
}
