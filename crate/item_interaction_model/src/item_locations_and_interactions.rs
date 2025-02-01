use indexmap::IndexMap;
use peace_item_model::ItemId;
use serde::{Deserialize, Serialize};

use crate::{ItemInteraction, ItemLocationTree};

#[cfg(feature = "output_progress")]
use std::collections::{HashMap, HashSet};

#[cfg(feature = "output_progress")]
use crate::ItemLocation;

/// Merged [`ItemLocation`]s and [`ItemInteraction`]s from all items.
///
/// [`ItemLocation`]: crate::ItemLocation
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ItemLocationsAndInteractions {
    /// Hierarchical storage of [`ItemLocation`]s.
    ///
    /// [`ItemLocation`]: crate::ItemLocation
    pub item_location_trees: Vec<ItemLocationTree>,
    /// The [`ItemInteraction`]s from each item.
    pub item_to_item_interactions: IndexMap<ItemId, Vec<ItemInteraction>>,
    /// Number of `ItemLocation`s from all merged [`ItemInteraction`]s.
    ///
    /// [`ItemLocation`]: crate::ItemLocation
    pub item_location_count: usize,
    /// Map that tracks the items that referred to each item location.
    #[cfg(feature = "output_progress")]
    pub item_location_to_item_id_sets: HashMap<ItemLocation, HashSet<ItemId>>,
}

impl ItemLocationsAndInteractions {
    /// Returns a new `ItemLocationsAndInteractions` container.
    pub fn new(
        item_location_trees: Vec<ItemLocationTree>,
        item_to_item_interactions: IndexMap<ItemId, Vec<ItemInteraction>>,
        item_location_count: usize,
        #[cfg(feature = "output_progress")] item_location_to_item_id_sets: HashMap<
            ItemLocation,
            HashSet<ItemId>,
        >,
    ) -> Self {
        Self {
            item_location_trees,
            item_to_item_interactions,
            item_location_count,
            #[cfg(feature = "output_progress")]
            item_location_to_item_id_sets,
        }
    }

    /// Returns the hierarchical storage of [`ItemLocation`]s.
    ///
    /// [`ItemLocation`]: crate::ItemLocation
    pub fn item_location_trees(&self) -> &[ItemLocationTree] {
        &self.item_location_trees
    }

    /// Returns the [`ItemInteraction`]s from each item.
    pub fn item_to_item_interactions(&self) -> &IndexMap<ItemId, Vec<ItemInteraction>> {
        &self.item_to_item_interactions
    }

    /// Returns the number of `ItemLocation`s from all merged
    /// [`ItemInteraction`]s.
    ///
    /// [`ItemLocation`]: crate::ItemLocation
    pub fn item_location_count(&self) -> usize {
        self.item_location_count
    }

    /// Returns the map that tracks the items that referred to each item
    /// location.
    #[cfg(feature = "output_progress")]
    pub fn item_location_to_item_id_sets(&self) -> &HashMap<ItemLocation, HashSet<ItemId>> {
        &self.item_location_to_item_id_sets
    }
}
