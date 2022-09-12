//! Data types used by the framework, not part of API.
//!
//! Since this is not API, it is not intended to be used (or useful outside the
//! framework). There may be breakage between releases.

pub use self::{
    op_check_statuses::OpCheckStatuses, state_diffs_mut::StateDiffsMut, states_mut::StatesMut,
    workspace_dirs::WorkspaceDirs,
};

mod op_check_statuses;
mod state_diffs_mut;
mod states_mut;
mod workspace_dirs;
