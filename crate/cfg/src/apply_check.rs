use serde::{Deserialize, Serialize};

#[cfg(feature = "output_progress")]
use peace_progress_model::ProgressLimit;

/// Whether the `apply` function needs to be executed.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum ApplyCheck {
    /// Item is not in goal state.
    #[cfg(not(feature = "output_progress"))]
    ExecRequired,
    /// Item is not in goal state.
    #[cfg(feature = "output_progress")]
    ExecRequired {
        /// Unit of measurement and limit to indicate progress.
        progress_limit: ProgressLimit,
    },
    /// Item is already in goal state.
    ExecNotRequired,
}
