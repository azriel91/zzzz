use serde::{Deserialize, Serialize};

use crate::ProgressComplete;

/// Status of an item's execution progress.
///
/// # Implementation Notes
///
/// ## Variants
///
/// The following variant is possible conceptually, but not applicable to the
/// Peace framework:
///
/// `Stopped`: Task is not running, but can be started.
///
/// This is not applicable because Peace uses runtime borrowing to manage state,
/// and a stopped task has potentially altered data non-atomically, so locking
/// the data is not useful, and unlocking the data may cause undefined behaviour
/// due to reasoning over inconsistent state.
///
/// For rate limiting tasks, the task in its entirety would be held back.
///
/// ## `!Copy`
///
/// This type isn't `Copy`, because one way wish to include detail about the
/// function to render as part of the progress output, and that detail may not
/// be `Copy` -- not sure yet.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum ProgressStatus {
    /// This item is registered for execution.
    ///
    /// This status is used when we don't know the progress limit.
    Initialized,
    /// The item was waiting for execution, when the command was interrupted.
    Interrupted,
    /// Execution has not yet begun.
    ///
    /// This is waiting on either:
    ///
    /// * The framework to begin executing the logic.
    /// * A predecessor's execution completion.
    ExecPending,
    /// Execution has started for this item, but we haven't received
    /// `ProgressDelta` update from the item exec logic.
    Queued,
    /// Execution is in progress.
    ///
    /// This status is best conveyed alongside additional information:
    ///
    /// * **Units total:** Unknown (spinner) / known (progress bar).
    /// * **Units current**
    /// * **Function:** `Item::{state_current, state_goal, apply}`.
    ///
    ///     Certain functions will not be applicable, e.g. when `StateCurrent`
    ///     is feature gated, then the function won't be available when the
    ///     feature is not enabled.
    Running,
    /// Progress updates have not been received for a given period.
    ///
    /// Item implementations are responsible for sending progress updates,
    /// but if there are no progress updates, or an identical "it's running"
    /// progress update is continuously received, then Peace may determine that
    /// the task may have stalled, and user attention is required.
    ///
    /// Peace may also provide a hook for implementors to output a suggestion to
    /// the user.
    RunningStalled,
    /// Task is pending user input.
    UserPending,
    /// Task has completed.
    ///
    /// This status is best conveyed alongside additional information:
    ///
    /// * **Completion Status**: Success, Failed.
    /// * **Function:** `Item::{state_current, state_goal, apply}`.
    Complete(ProgressComplete),
}
