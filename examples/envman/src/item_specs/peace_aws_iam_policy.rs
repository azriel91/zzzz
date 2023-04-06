//! Copies a number from one resource to another.

pub use self::{
    iam_policy_apply_op_spec::IamPolicyApplyOpSpec, iam_policy_data::IamPolicyData,
    iam_policy_error::IamPolicyError, iam_policy_item_spec::IamPolicyItemSpec,
    iam_policy_params::IamPolicyParams, iam_policy_state::IamPolicyState,
    iam_policy_state_current_fn::IamPolicyStateCurrentFn,
    iam_policy_state_desired_fn::IamPolicyStateDesiredFn,
    iam_policy_state_diff::IamPolicyStateDiff,
    iam_policy_state_diff_fn_spec::IamPolicyStateDiffFnSpec,
};

pub mod model;

mod iam_policy_apply_op_spec;
mod iam_policy_data;
mod iam_policy_error;
mod iam_policy_item_spec;
mod iam_policy_params;
mod iam_policy_state;
mod iam_policy_state_current_fn;
mod iam_policy_state_desired_fn;
mod iam_policy_state_diff;
mod iam_policy_state_diff_fn_spec;
