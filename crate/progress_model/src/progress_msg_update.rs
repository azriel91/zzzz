use serde::{Deserialize, Serialize};

/// Whether to change the progress message.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum ProgressMsgUpdate {
    /// Clears the progress message.
    Clear,
    /// Does not change the progress message.
    NoChange,
    /// Sets the progress message.
    Set(String),
}
