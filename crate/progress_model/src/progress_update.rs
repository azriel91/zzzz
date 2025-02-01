use serde::{Deserialize, Serialize};

use crate::{ProgressDelta, ProgressLimit};

use super::ProgressComplete;

/// Progress update for a single progress tracker.
///
/// # Potential Future Variants
///
/// * `PendingInput`
/// * `Stall`
///
/// # Implementation Note
///
/// `serde-yaml` 0.9 does not support serializing / deserializing nested enums,
/// and returns errors like the following:
///
/// ```text
/// "deserializing nested enum in ProgressUpdate::Delta from YAML is not supported yet"
/// "serializing nested enums in YAML is not supported yet"
/// ```
///
/// The [`serde_yaml::with::singleton_map`] attributes are necessary for
/// `serde_yaml` to serialize nested enums.
///
/// [`serde_yaml::with::singleton_map`]: https://docs.rs/serde_yaml/latest/serde_yaml/with/singleton_map/index.html
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum ProgressUpdate {
    /// Resets the progress tracker to the `Initialized` state.
    Reset,
    /// Resets the progress tracker to the `ExecPending` state.
    ///
    /// This is similar to `Reset`, but the progress bar is not styled as
    /// disabled.
    ResetToPending,
    /// Sets the progress tracker as `Queued`, meaning it musn't be interrupted
    /// as it is essentially `Running`
    Queued,
    /// `CmdExecution` has been interrupted, we should indicate this on the
    /// progress bar.
    Interrupt,
    /// Progress limit has been discovered.
    #[serde(with = "serde_yaml::with::singleton_map")]
    Limit(ProgressLimit),
    /// Progress units have changed.
    #[serde(with = "serde_yaml::with::singleton_map")]
    Delta(ProgressDelta),
    /// Execution has completed.
    #[serde(with = "serde_yaml::with::singleton_map")]
    Complete(ProgressComplete),
}
