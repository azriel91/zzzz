//! Configuration model for the peace automation library.
//!
//! This crate defines the API for logic and data to be used in the `peace`
//! framework.

// Re-exports
pub use async_trait::async_trait;
cfg_if::cfg_if! {
    if #[cfg(feature = "output_progress")] {
        pub use peace_core::progress;
        pub use tokio::sync::mpsc::{
            error::{SendError, TrySendError},
            Sender,
        };
    }
}

pub use peace_core::{
    flow_id, item_spec_id, profile, FlowId, FlowIdInvalidFmt, ItemSpecId, ItemSpecIdInvalidFmt,
    OpCheckStatus, Profile, ProfileInvalidFmt,
};

pub use crate::{
    clean_op_spec::CleanOpSpec, ensure_op_spec::EnsureOpSpec, item_spec::ItemSpec, op_ctx::OpCtx,
    state::State, state_diff_fn_spec::StateDiffFnSpec, try_fn_spec::TryFnSpec,
};

pub mod state;

mod clean_op_spec;
mod ensure_op_spec;
mod item_spec;
mod op_ctx;
mod state_diff_fn_spec;
mod try_fn_spec;
