use std::ops::ControlFlow;

use futures::stream::{self, StreamExt};
use peace_item_model::ItemId;
use peace_progress_model::{
    CmdBlockItemInteractionType, CmdProgressUpdate, ItemLocationState, ProgressDelta,
    ProgressMsgUpdate, ProgressStatus, ProgressTracker, ProgressUpdate, ProgressUpdateAndId,
};
use peace_rt_model::{output::OutputWrite, IndexMap};
use tokio::sync::mpsc::Receiver;

pub struct Progress;

impl Progress {
    /// Receives progress updates and updates `output` to render it.
    // TODO: write test for this
    pub async fn progress_render<E, O>(
        output: &mut O,
        progress_trackers: &mut IndexMap<ItemId, ProgressTracker>,
        mut cmd_progress_rx: Receiver<CmdProgressUpdate>,
    ) where
        O: OutputWrite<E>,
    {
        while let Some(cmd_progress_update) = cmd_progress_rx.recv().await {
            let _control_flow =
                Self::handle_cmd_progress_update(output, progress_trackers, cmd_progress_update)
                    .await;
        }
    }

    async fn handle_cmd_progress_update<E, O>(
        output: &mut O,
        progress_trackers: &mut IndexMap<ItemId, ProgressTracker>,
        cmd_progress_update: CmdProgressUpdate,
    ) -> ControlFlow<()>
    where
        O: OutputWrite<E>,
    {
        match cmd_progress_update {
            CmdProgressUpdate::CmdBlockStart {
                cmd_block_item_interaction_type,
            } => {
                Self::handle_cmd_block_start(output, cmd_block_item_interaction_type).await;

                ControlFlow::Continue(())
            }
            CmdProgressUpdate::ItemProgress {
                progress_update_and_id,
            } => {
                Self::handle_progress_update_and_id(
                    output,
                    progress_trackers,
                    progress_update_and_id,
                )
                .await;

                ControlFlow::Continue(())
            }
            CmdProgressUpdate::ItemLocationState {
                item_id,
                item_location_state,
            } => {
                Self::handle_item_location_state(output, item_id, item_location_state).await;

                ControlFlow::Continue(())
            }
            CmdProgressUpdate::Interrupt => {
                stream::iter(progress_trackers.iter_mut())
                    .fold(output, |output, (item_id, progress_tracker)| async move {
                        let item_id = item_id.clone();
                        let progress_update_and_id = ProgressUpdateAndId {
                            item_id,
                            progress_update: ProgressUpdate::Interrupt,
                            msg_update: ProgressMsgUpdate::NoChange,
                        };

                        Self::handle_progress_tracker_progress_update(
                            output,
                            progress_tracker,
                            progress_update_and_id,
                        )
                        .await;

                        output
                    })
                    .await;

                ControlFlow::Break(())
            }
            CmdProgressUpdate::ResetToPending => {
                stream::iter(progress_trackers.iter_mut())
                    .fold(output, |output, (item_id, progress_tracker)| async move {
                        let item_id = item_id.clone();
                        let progress_update_and_id = ProgressUpdateAndId {
                            item_id,
                            progress_update: ProgressUpdate::ResetToPending,
                            msg_update: ProgressMsgUpdate::Clear,
                        };

                        Self::handle_progress_tracker_progress_update(
                            output,
                            progress_tracker,
                            progress_update_and_id,
                        )
                        .await;

                        output
                    })
                    .await;

                ControlFlow::Continue(())
            }
        }
    }

    async fn handle_cmd_block_start<E, O>(
        output: &mut O,
        cmd_block_item_interaction_type: CmdBlockItemInteractionType,
    ) where
        O: OutputWrite<E>,
    {
        output
            .cmd_block_start(cmd_block_item_interaction_type)
            .await;
    }

    async fn handle_item_location_state<E, O>(
        output: &mut O,
        item_id: ItemId,
        item_location_state: ItemLocationState,
    ) where
        O: OutputWrite<E>,
    {
        output
            .item_location_state(item_id, item_location_state)
            .await;
    }

    async fn handle_progress_update_and_id<E, O>(
        output: &mut O,
        progress_trackers: &mut IndexMap<ItemId, ProgressTracker>,
        progress_update_and_id: ProgressUpdateAndId,
    ) where
        O: OutputWrite<E>,
    {
        let item_id = &progress_update_and_id.item_id;
        let Some(progress_tracker) = progress_trackers.get_mut(item_id) else {
            panic!("Expected `progress_tracker` to exist for item: `{item_id}`.");
        };

        Self::handle_progress_tracker_progress_update(
            output,
            progress_tracker,
            progress_update_and_id,
        )
        .await;
    }

    async fn handle_progress_tracker_progress_update<E, O>(
        output: &mut O,
        progress_tracker: &mut ProgressTracker,
        progress_update_and_id: ProgressUpdateAndId,
    ) where
        O: OutputWrite<E>,
    {
        let ProgressUpdateAndId {
            item_id: _,
            progress_update,
            msg_update,
        } = &progress_update_and_id;
        match progress_update {
            ProgressUpdate::Reset => progress_tracker.reset(),
            ProgressUpdate::ResetToPending => progress_tracker.reset_to_pending(),
            ProgressUpdate::Queued => progress_tracker.set_progress_status(ProgressStatus::Queued),
            ProgressUpdate::Interrupt => progress_tracker.interrupt(),
            ProgressUpdate::Limit(progress_limit) => {
                progress_tracker.set_progress_limit(*progress_limit);
                progress_tracker.set_progress_status(ProgressStatus::ExecPending);
            }
            ProgressUpdate::Delta(delta) => {
                match delta {
                    ProgressDelta::Tick => progress_tracker.tick(),
                    ProgressDelta::Inc(unit_count) => progress_tracker.inc(*unit_count),
                }
                progress_tracker.set_progress_status(ProgressStatus::Running);
            }
            ProgressUpdate::Complete(progress_complete) => {
                progress_tracker
                    .set_progress_status(ProgressStatus::Complete(progress_complete.clone()));
            }
        }

        match msg_update {
            ProgressMsgUpdate::Clear => progress_tracker.set_message(None),
            ProgressMsgUpdate::NoChange => {}
            ProgressMsgUpdate::Set(message) => progress_tracker.set_message(Some(message.clone())),
        }

        output
            .progress_update(progress_tracker, &progress_update_and_id)
            .await;
    }
}
