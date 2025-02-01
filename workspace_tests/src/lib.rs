#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg(test)]

pub(crate) use crate::{
    fn_invocation::FnInvocation,
    fn_name::fn_name_short,
    fn_tracker_output::FnTrackerOutput,
    fn_tracker_presenter::FnTrackerPresenter,
    no_op_output::NoOpOutput,
    peace_test_error::PeaceTestError,
    vec_copy_item::{
        VecA, VecB, VecCopyDiff, VecCopyError, VecCopyItem, VecCopyItemWrapper, VecCopyState,
    },
};

pub(crate) mod mock_item;

// `peace` test modules
mod cfg;
mod cli;
mod cmd;
mod cmd_model;
mod cmd_rt;
mod data;
mod diff;
mod flow_model;
mod fmt;
#[cfg(feature = "item_interactions")]
mod item_interaction_model;
mod item_model;
mod params;
mod profile_model;
#[cfg(feature = "output_progress")]
mod progress_model;
mod resource_rt;
mod rt;
mod rt_model;
mod state_rt;
#[cfg(feature = "webi")]
mod webi;

// `peace_items` test modules
#[cfg(feature = "items")]
mod items;

// `workspace_tests` support code
mod fn_invocation;
mod fn_name;
mod fn_tracker_output;
mod fn_tracker_presenter;
mod no_op_output;
mod peace_cmd_ctx_types;
mod peace_test_error;
mod test_support;
mod vec_copy_item;
