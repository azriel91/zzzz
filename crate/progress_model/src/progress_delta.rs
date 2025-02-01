use serde::{Deserialize, Serialize};

/// The amount that the item progressed by.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum ProgressDelta {
    /// Ticks the progress bar without incrementing its value.
    ///
    /// Generally useful for progress bars with an unknown total.
    Tick,
    /// Increments the progress bar by the specified amount.
    Inc(u64),
}
