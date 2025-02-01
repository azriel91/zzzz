use peace_item_model::ItemId;
use serde::{Deserialize, Serialize};

/// Serializable representation of values used for / produced by an [`Item`].
///
/// [`Item`]: https://docs.rs/peace_cfg/latest/peace_cfg/trait.Item.html
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ItemInfo {
    /// ID of the `Item`.
    pub item_id: ItemId,
}

impl ItemInfo {
    /// Returns a new `ItemInfo`.
    pub fn new(item_id: ItemId) -> Self {
        Self { item_id }
    }
}
