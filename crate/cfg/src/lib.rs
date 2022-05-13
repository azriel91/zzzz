//! Configuration model for the zzzz automation library.
//!
//! This crate defines the API for logic and data to be used in the `zzzz`
//! framework.

pub use crate::{
    op_check_status::OpCheckStatus, op_spec::OpSpec, op_spec_dry::OpSpecDry,
    progress_limit::ProgressLimit, work_spec::WorkSpec,
};

mod op_check_status;
mod op_spec;
mod op_spec_dry;
mod progress_limit;
mod work_spec;
