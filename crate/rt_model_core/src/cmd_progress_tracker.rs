use indexmap::IndexMap;
use indicatif::MultiProgress;
use peace_item_model::ItemId;
use peace_progress_model::ProgressTracker;

/// Tracks command execution progress for all items.
///
/// The Peace framework initializes the `multi_progress` and `progress_trackers`
/// and manages updating the `ProgressBar` values.
///
/// By default, the `MultiProgress` will use [`ProgressDrawTarget::hidden()`].
/// However, by default [`CliOutput`] sets the draw target to `stderr` if an
/// executable built using Peace is run interactively.
///
/// [`ProgressDrawTarget::hidden()`]: indicatif::ProgressDrawTarget::hidden
/// [`CliOutput`]: https://docs.rs/peace_rt_model_native/latest/peace_rt_model_native/struct.CliOutput.html
#[derive(Debug)]
pub struct CmdProgressTracker {
    /// `MultiProgress` that tracks the remaining progress bars.
    pub multi_progress: MultiProgress,
    /// Tracks progress for each item.
    pub progress_trackers: IndexMap<ItemId, ProgressTracker>,
}

impl CmdProgressTracker {
    /// Returns a new `CmdProgressTracker`.
    pub fn new(
        multi_progress: MultiProgress,
        progress_trackers: IndexMap<ItemId, ProgressTracker>,
    ) -> Self {
        Self {
            multi_progress,
            progress_trackers,
        }
    }

    /// Returns the `MultiProgress` that tracks the remaining progress bars.
    pub fn multi_progress(&self) -> &MultiProgress {
        &self.multi_progress
    }

    /// Returns a mutable reference to the `MultiProgress` that tracks the
    /// remaining progress bars.
    pub fn multi_progress_mut(&mut self) -> &mut MultiProgress {
        &mut self.multi_progress
    }

    /// Returns the `ProgressTracker`s for each item.
    pub fn progress_trackers(&self) -> &IndexMap<ItemId, ProgressTracker> {
        &self.progress_trackers
    }

    /// Returns a mutable reference to the `ProgressTracker`s for each item
    /// spec.
    pub fn progress_trackers_mut(&mut self) -> &mut IndexMap<ItemId, ProgressTracker> {
        &mut self.progress_trackers
    }
}
