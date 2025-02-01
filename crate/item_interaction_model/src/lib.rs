//! Data types for resource interactions for the Peace framework.

// Re-exports
pub use url::{self, Host, Url};

// TODO: Remove this when we have refactored where progress types live.
#[cfg(feature = "output_progress")]
pub use peace_progress_model::ItemLocationState;

pub use crate::{
    item_interaction::{
        ItemInteraction, ItemInteractionPull, ItemInteractionPush, ItemInteractionWithin,
    },
    item_interactions_current::ItemInteractionsCurrent,
    item_interactions_current_or_example::ItemInteractionsCurrentOrExample,
    item_interactions_example::ItemInteractionsExample,
    item_location::ItemLocation,
    item_location_ancestors::ItemLocationAncestors,
    item_location_tree::ItemLocationTree,
    item_location_type::ItemLocationType,
    item_locations_combined::ItemLocationsCombined,
};

#[cfg(feature = "item_locations_and_interactions")]
pub use crate::item_locations_and_interactions::ItemLocationsAndInteractions;

#[cfg(feature = "output_progress")]
pub use crate::{
    // TODO: uncomment when we have refactored where progress types live.
    // item_location_state::ItemLocationState,
    item_location_state_in_progress::ItemLocationStateInProgress,
};

mod item_interaction;
mod item_interactions_current;
mod item_interactions_current_or_example;
mod item_interactions_example;
mod item_location;
mod item_location_ancestors;
mod item_location_tree;
mod item_location_type;
mod item_locations_combined;

// TODO: uncomment when we have refactored where progress types live.
// #[cfg(feature = "output_progress")]
// mod item_location_state;
#[cfg(feature = "output_progress")]
mod item_location_state_in_progress;

#[cfg(feature = "item_locations_and_interactions")]
mod item_locations_and_interactions;
