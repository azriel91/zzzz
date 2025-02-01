//! Core runtime traits for the peace automation framework.
//!
//! These types are in this crate so that the `rt_model_native` and
//! `rt_model_web` crates are able to reference them and either use or provide
//! default implementations.

// Re-exports
pub use async_trait::async_trait;
pub use indexmap::IndexMap;
pub use indicatif;

pub mod output;
pub mod params;

pub use crate::{
    error::{ApplyCmdError, Error, StateDowncastError},
    items_state_stored_stale::ItemsStateStoredStale,
    state_stored_and_discovered::StateStoredAndDiscovered,
};

mod error;
mod items_state_stored_stale;
mod state_stored_and_discovered;

cfg_if::cfg_if! {
    if #[cfg(feature = "output_progress")] {
        pub use peace_progress_model::ProgressUpdate;

        pub use crate::cmd_progress_tracker::CmdProgressTracker;

        mod cmd_progress_tracker;
    }
}

cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        pub use crate::error::NativeError;
    } else {
        pub use crate::error::WebError;
    }
}
