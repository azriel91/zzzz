use std::fmt::Debug;

use async_trait::async_trait;
use peace_fmt::Presentable;

cfg_if::cfg_if! {
    if #[cfg(feature = "output_progress")] {
        use peace_progress_model::{
            CmdBlockItemInteractionType,
            ProgressTracker,
            ProgressUpdateAndId,
        };

        use crate::CmdProgressTracker;
    }
}

/// Transforms return values or errors into a suitable output format.
///
/// # Use cases
///
/// * A CLI implementation transforms the values into text to be printed.
/// * A REST implementation transforms the values into the response.
/// * A frontend implementation transforms the values into HTML elements.
///
/// # Design
///
/// The write functions currently take `&mut self`. From an API implementor
/// perspective, this should not be difficult to use as the return value / error
/// value is intended to be returned at the end of a command.
///
/// Progress updates sent during `ApplyFns::exec` and `CleanOpSpec::exec`.
#[async_trait(?Send)]
pub trait OutputWrite<E>: Debug + Unpin {
    /// Prepares this `OutputWrite` implementation for rendering progress.
    ///
    /// # Implementors
    ///
    /// This is called at the beginning of command execution, before any
    /// potential calls to `OutputWrite::progress_update`.
    ///
    /// At the end of command execution, `OutputWrite::progress_end` is called.
    #[cfg(feature = "output_progress")]
    async fn progress_begin(&mut self, cmd_progress_tracker: &CmdProgressTracker);

    /// Indicates a particular `CmdBlock` has begun.
    ///
    /// # Implementors
    ///
    /// This is called whenever a different `CmdBlock` is started.
    #[cfg(feature = "output_progress")]
    async fn cmd_block_start(
        &mut self,
        cmd_block_item_interaction_type: CmdBlockItemInteractionType,
    );

    /// Signals an update of an `Item`'s `ItemLocationState`.
    ///
    /// # Implementors
    ///
    /// This is called when an `Item`'s current `State` is updated.
    ///
    /// # Maintainers
    ///
    /// The `ItemLocationState` is first constructed in `ItemWrapper`, and this
    /// method is invoked in `Progress`.
    #[cfg(feature = "output_progress")]
    async fn item_location_state(
        &mut self,
        item_id: peace_item_model::ItemId,
        item_location_state: peace_item_interaction_model::ItemLocationState,
    );

    /// Renders progress information, and returns when no more progress
    /// information is available to write.
    ///
    /// This function is infallible as progress information is considered
    /// transient, and loss of progress information is not considered as
    /// something worth stopping a command for.
    ///
    /// # Implementors
    ///
    /// This should render the progress update to the user in a way that is not
    /// overwhelming.
    #[cfg(feature = "output_progress")]
    async fn progress_update(
        &mut self,
        progress_tracker: &ProgressTracker,
        progress_update_and_id: &ProgressUpdateAndId,
    );

    /// Notifies this `OutputWrite` implementation to stop rendering progress.
    ///
    /// # Implementors
    ///
    /// This is called at the end of command execution. After this is called,
    /// there will be no more calls to `OutputWrite::progress_update` until
    /// another call to `OutputWrite::progress_begin`.
    #[cfg(feature = "output_progress")]
    async fn progress_end(&mut self, cmd_progress_tracker: &CmdProgressTracker);

    /// Writes presentable information to the output.
    async fn present<P>(&mut self, presentable: P) -> Result<(), E>
    where
        E: std::error::Error,
        P: Presentable;

    /// Writes an error to the output.
    async fn write_err(&mut self, error: &E) -> Result<(), E>
    where
        E: std::error::Error;
}
