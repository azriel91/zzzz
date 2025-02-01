use fn_graph::StreamOutcome;
use indexmap::IndexMap;
use peace_item_model::ItemId;

/// `CmdBlock` stream outcome and item wise errors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StreamOutcomeAndErrors<T, E> {
    /// The `CmdBlock` stream outcome.
    pub stream_outcome: StreamOutcome<T>,
    /// The errors during processing,
    pub errors: IndexMap<ItemId, E>,
}
