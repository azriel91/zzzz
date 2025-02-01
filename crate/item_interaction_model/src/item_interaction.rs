use serde::{Deserialize, Serialize};

mod item_interaction_pull;
mod item_interaction_push;
mod item_interaction_within;

pub use self::{
    item_interaction_pull::ItemInteractionPull, item_interaction_push::ItemInteractionPush,
    item_interaction_within::ItemInteractionWithin,
};

/// Represents the resources that are read from / written to.
///
/// This is used on an outcome diagram to highlight the resources that are being
/// accessed. For example, a file is read from the user's computer, and uploaded
/// / written to a file server.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum ItemInteraction {
    /// Represents a location-to-location push interaction.
    ///
    /// This can represent a file transfer from one host to another.
    Push(ItemInteractionPush),
    /// Represents a location-to-location pull interaction.
    ///
    /// This can represent a file download from a server.
    Pull(ItemInteractionPull),
    /// Represents a resource interaction that happens within a location.
    ///
    /// This can represent application installation / startup happening on a
    /// server.
    Within(ItemInteractionWithin),
}

impl From<ItemInteractionPush> for ItemInteraction {
    fn from(item_interaction_push: ItemInteractionPush) -> Self {
        Self::Push(item_interaction_push)
    }
}

impl From<ItemInteractionPull> for ItemInteraction {
    fn from(item_interaction_pull: ItemInteractionPull) -> Self {
        Self::Pull(item_interaction_pull)
    }
}

impl From<ItemInteractionWithin> for ItemInteraction {
    fn from(item_interaction_within: ItemInteractionWithin) -> Self {
        Self::Within(item_interaction_within)
    }
}
