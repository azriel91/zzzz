use serde::{Deserialize, Serialize};

use crate::{ItemLocation, ItemLocationAncestors};

/// Represents a resource interaction that happens within a location.
///
/// This can represent application installation / startup happening on a
/// server.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ItemInteractionWithin {
    /// Where the interaction is happening.
    ///
    /// Example:
    ///
    /// 1. `ItemLocation::Server { address, port: None }`
    pub location: ItemLocationAncestors,
}

impl ItemInteractionWithin {
    /// Returns a new `ItemInteractionWithin`.
    pub fn new(location: ItemLocationAncestors) -> Self {
        Self { location }
    }

    /// Returns where the interaction is happening.
    pub fn location(&self) -> &[ItemLocation] {
        &self.location
    }
}
