use serde::{Deserialize, Serialize};

cfg_if::cfg_if! {
    if #[cfg(feature = "output_progress")] {
        use std::collections::HashMap;

        use peace_item_model::ItemId;
        use peace_item_interaction_model::ItemLocationState;
        use peace_progress_model::{CmdBlockItemInteractionType, ProgressStatus};
    }
}

/// How to style the outcome `InfoGraph`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutcomeInfoGraphVariant {
    /// Example `InfoGraph` diagram with no special styling.
    Example,
    /// Current `InfoGraph` diagram that shows execution progress.
    Current {
        /// Type of interactions that a `CmdBlock`s has with `ItemLocation`s.
        #[cfg(feature = "output_progress")]
        cmd_block_item_interaction_type: CmdBlockItemInteractionType,
        /// `ItemLocationState`s of each item.
        ///
        /// This is used in the calculation for styling each node.
        #[cfg(feature = "output_progress")]
        item_location_states: HashMap<ItemId, ItemLocationState>,
        /// Execution progress status of each item.
        ///
        /// This is used in the calculation for styling each edge.
        #[cfg(feature = "output_progress")]
        item_progress_statuses: HashMap<ItemId, ProgressStatus>,
    },
}
