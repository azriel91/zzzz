use serde::{Deserialize, Serialize};

use crate::{ItemLocation, ItemLocationAncestors};

/// Represents a location-to-location push interaction.
///
/// This can represent a file transfer from one host to another.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ItemInteractionPush {
    /// Where the interaction begins from.
    ///
    /// Example:
    ///
    /// 1. `ItemLocation::localhost()`
    /// 2. `ItemLocation::new("/path/to/file", ItemLocationType::Path)`
    pub location_from: ItemLocationAncestors,
    /// Where the interaction goes to.
    ///
    /// Example:
    ///
    /// 1. `ItemLocation::new("app.domain.com", ItemLocationType::Host)`
    /// 2. `ItemLocation::new("http://app.domain.com/resource",
    ///    ItemLocationType::Path)`
    pub location_to: ItemLocationAncestors,
}

impl ItemInteractionPush {
    /// Returns a new `ItemInteractionPush`.
    pub fn new(location_from: ItemLocationAncestors, location_to: ItemLocationAncestors) -> Self {
        Self {
            location_from,
            location_to,
        }
    }

    /// Returns where the interaction begins from.
    ///
    /// Example:
    ///
    /// 1. `ItemLocation::localhost()`
    /// 2. `ItemLocation::new("/path/to/file", ItemLocationType::Path)`
    pub fn location_from(&self) -> &[ItemLocation] {
        &self.location_from
    }

    /// Returns where the interaction goes to.
    ///
    /// Example:
    ///
    /// 1. `ItemLocation::new("app.domain.com", ItemLocationType::Host)`
    /// 2. `ItemLocation::new("http://app.domain.com/resource",
    ///    ItemLocationType::Path)`
    pub fn location_to(&self) -> &[ItemLocation] {
        &self.location_to
    }
}
