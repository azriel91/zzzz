use peace_item_model::ItemId;
use tokio::sync::mpsc::Sender;

use crate::{
    CmdProgressUpdate, ItemLocationState, ProgressDelta, ProgressMsgUpdate, ProgressUpdate,
    ProgressUpdateAndId,
};

/// Submits progress for an item's `ApplyFns::exec` method.
#[derive(Clone, Copy, Debug)]
pub struct ProgressSender<'exec> {
    /// ID of the item this belongs to.
    item_id: &'exec ItemId,
    /// Channel sender to send progress updates to.
    progress_tx: &'exec Sender<CmdProgressUpdate>,
}

impl<'exec> ProgressSender<'exec> {
    /// Returns a new `ProgressSender`.
    pub fn new(item_id: &'exec ItemId, progress_tx: &'exec Sender<CmdProgressUpdate>) -> Self {
        Self {
            item_id,
            progress_tx,
        }
    }

    /// Increments the progress by the given unit count.
    pub fn inc(&self, unit_count: u64, msg_update: ProgressMsgUpdate) {
        let _progress_send_unused = self.progress_tx.try_send(
            ProgressUpdateAndId {
                item_id: self.item_id.clone(),
                progress_update: ProgressUpdate::Delta(ProgressDelta::Inc(unit_count)),
                msg_update,
            }
            .into(),
        );
    }

    /// Ticks the tracker without incrementing its progress.
    ///
    /// This is useful for spinners -- progress trackers where there is an
    /// unknown.
    ///
    /// Note, this also updates the `last_update_dt`, so in the case of a
    /// spinner, this should only be called when there is actually a detected
    /// change.
    pub fn tick(&self, msg_update: ProgressMsgUpdate) {
        let _progress_send_unused = self.progress_tx.try_send(
            ProgressUpdateAndId {
                item_id: self.item_id.clone(),
                progress_update: ProgressUpdate::Delta(ProgressDelta::Tick),
                msg_update,
            }
            .into(),
        );
    }

    /// Resets the progress tracker to a clean state.
    pub fn reset(&self) {
        let _progress_send_unused = self.progress_tx.try_send(
            ProgressUpdateAndId {
                item_id: self.item_id.clone(),
                progress_update: ProgressUpdate::Reset,
                msg_update: ProgressMsgUpdate::Clear,
            }
            .into(),
        );
    }

    /// Resets the progress tracker to a clean state.
    pub fn reset_to_pending(&self) {
        let _progress_send_unused = self.progress_tx.try_send(
            ProgressUpdateAndId {
                item_id: self.item_id.clone(),
                progress_update: ProgressUpdate::ResetToPending,
                msg_update: ProgressMsgUpdate::Clear,
            }
            .into(),
        );
    }

    /// Sends an `ItemLocationState` update.
    ///
    /// # Implementors
    ///
    /// This is only intended for use by the Peace framework for rendering.
    ///
    /// # Maintainers
    ///
    /// This is used in `ItemWrapper`.
    pub fn item_location_state_send(&self, item_location_state: ItemLocationState) {
        let _progress_send_unused =
            self.progress_tx
                .try_send(CmdProgressUpdate::ItemLocationState {
                    item_id: self.item_id.clone(),
                    item_location_state,
                });
    }
}
