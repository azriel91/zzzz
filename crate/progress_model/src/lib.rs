//! Command progress data model for the peace automation framework.
//!
//! # Design Notes
//!
//! Only used when the `"output_progress"` feature is enabled in Peace.

pub use crate::{
    cmd_block_item_interaction_type::CmdBlockItemInteractionType,
    cmd_progress_update::CmdProgressUpdate, item_location_state::ItemLocationState,
    progress_complete::ProgressComplete, progress_delta::ProgressDelta,
    progress_limit::ProgressLimit, progress_msg_update::ProgressMsgUpdate,
    progress_sender::ProgressSender, progress_status::ProgressStatus,
    progress_tracker::ProgressTracker, progress_update::ProgressUpdate,
    progress_update_and_id::ProgressUpdateAndId,
};

mod cmd_block_item_interaction_type;
mod cmd_progress_update;
mod item_location_state;
mod progress_complete;
mod progress_delta;
mod progress_limit;
mod progress_msg_update;
mod progress_sender;
mod progress_status;
mod progress_tracker;
mod progress_update;
mod progress_update_and_id;
