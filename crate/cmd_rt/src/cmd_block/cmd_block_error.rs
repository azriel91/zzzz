use std::fmt::Debug;

use fn_graph::StreamOutcome;
use indexmap::IndexMap;
use peace_item_model::ItemId;
use peace_resource_rt::ResourceFetchError;

/// Error while executing a `CmdBlock`.
///
/// # Type Parameters
///
/// * `T`: Execution outcome, mapped from `CmdBlock::OutcomeAcc`.
/// * `E`: Application error type.
#[derive(Debug, thiserror::Error)]
pub enum CmdBlockError<T, E>
where
    T: Debug,
    E: Debug,
{
    /// Error fetching `CmdBlock::InputT` from `resources`.
    ///
    /// If `CmdBlock::InputT` is a tuple, such as `(StatesCurrent, StatesGoal)`,
    /// and `states_current` and `states_goal` are inserted individually in
    /// `Resources`, then `CmdBlock::input_fetch` should be implemented to call
    /// `Resources::remove` for each of them.
    #[error(
        "Failed to fetch `{input_name_short}` from `resource`s.",
        input_name_short = _0.resource_name_short
    )]
    InputFetch(
        #[source]
        #[from]
        ResourceFetchError,
    ),
    /// Error originated from `CmdBlock` exec code.
    #[error("`CmdBlock` block execution or collation logic failed.")]
    Exec(E),
    /// Error originated from at least one item.
    ///
    /// The `CmdBlock::Outcome` is mapped to the `ExecutionOutcome` using
    /// `fn_partial_exec_handler`.
    #[error("`CmdBlock` item logic failed.")]
    ItemError {
        /// The outcome value.
        stream_outcome: StreamOutcome<T>,
        /// Item error(s) from the last command block's execution.
        errors: IndexMap<ItemId, E>,
    },
    /// An interrupt signal was received while the `CmdBlock` was executing.
    #[error("`CmdBlock` item logic failed.")]
    Interrupt {
        /// The stream outcome of the interrupted command block.
        stream_outcome: StreamOutcome<T>,
    },
}
