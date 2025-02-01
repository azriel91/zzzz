use serde::{Deserialize, Serialize};

use crate::{ItemLocation, ItemLocationAncestors};

/// Represents a location-to-location pull interaction.
///
/// This can represent a file download from a server.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ItemInteractionPull {
    /// Where the interaction begins from.
    ///
    /// Example:
    ///
    /// 1. `ItemLocation::localhost()`
    /// 2. `ItemLocation::new("/path/to/file", ItemLocationType::Path)`
    pub location_client: ItemLocationAncestors,
    /// Where the interaction goes to.
    ///
    /// Example:
    ///
    /// 1. `ItemLocation::new("app.domain.com", ItemLocationType::Host)`
    /// 2. `ItemLocation::new("http://app.domain.com/resource",
    ///    ItemLocationType::Path)`
    pub location_server: ItemLocationAncestors,
}

impl ItemInteractionPull {
    /// Returns a new `ItemInteractionPull`.
    pub fn new(
        location_client: ItemLocationAncestors,
        location_server: ItemLocationAncestors,
    ) -> Self {
        Self {
            location_client,
            location_server,
        }
    }

    /// Returns where the interaction begins from.
    ///
    /// Example:
    ///
    /// 1. `ItemLocation::localhost()`
    /// 2. `ItemLocation::new("/path/to/file", ItemLocationType::Path)`
    pub fn location_client(&self) -> &[ItemLocation] {
        &self.location_client
    }

    /// Returns where the interaction goes to.
    ///
    /// Example:
    ///
    /// 1. `ItemLocation::new("app.domain.com", ItemLocationType::Host)`
    /// 2. `ItemLocation::new("http://app.domain.com/resource",
    ///    ItemLocationType::Path)`
    pub fn location_server(&self) -> &[ItemLocation] {
        &self.location_server
    }
}
