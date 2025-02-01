use std::marker::PhantomData;

use peace_item_model::ItemId;

#[cfg(feature = "output_progress")]
use peace_progress_model::ProgressSender;

/// References to pass information between the Peace framework and an item.
#[derive(Clone, Copy, Debug)]
pub struct FnCtx<'exec> {
    /// ID of the item this belongs to.
    pub item_id: &'exec ItemId,
    /// For items to submit progress updates.
    #[cfg(feature = "output_progress")]
    pub progress_sender: ProgressSender<'exec>,
    /// Marker.
    pub marker: PhantomData<&'exec ()>,
}

impl<'exec> FnCtx<'exec> {
    /// Returns a new `OpCtx`.
    pub fn new(
        item_id: &'exec ItemId,
        #[cfg(feature = "output_progress")] progress_sender: ProgressSender<'exec>,
    ) -> Self {
        Self {
            item_id,
            #[cfg(feature = "output_progress")]
            progress_sender,
            marker: PhantomData,
        }
    }

    /// Returns the `ProgressTracker` for items to send progress to.
    #[cfg(feature = "output_progress")]
    pub fn progress_sender(&self) -> &ProgressSender<'exec> {
        &self.progress_sender
    }
}
