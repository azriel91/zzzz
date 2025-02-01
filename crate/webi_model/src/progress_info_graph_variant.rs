use indexmap::IndexSet;
use peace_item_model::ItemId;
use serde::{Deserialize, Serialize};

/// How to style the progress `InfoGraph`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgressInfoGraphVariant {
    /// Example `InfoGraph` diagram with no special styling.
    Example,
    /// Current `InfoGraph` diagram that shows execution progress.
    Current {
        /// IDs of items that are currently in progress.
        ///
        /// These items should be styled with animated blue strokes.
        item_ids_in_progress: IndexSet<ItemId>,
        /// IDs of the items that are already done.
        ///
        /// These items should be styled with green strokes.
        item_ids_completed: IndexSet<ItemId>,
        /// Whether the process is interrupted.
        ///
        /// Edges after items in progress should be styled yellow.
        interrupted: bool,
    },
}
