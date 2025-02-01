use std::marker::PhantomData;

use peace::cfg::{ApplyCheck, FnCtx};
#[cfg(feature = "output_progress")]
use peace::progress_model::ProgressLimit;

use crate::{BlankData, BlankError, BlankParams, BlankState, BlankStateDiff};

/// ApplyFns for the blank state.
#[derive(Debug)]
pub struct BlankApplyFns<Id>(PhantomData<Id>);

impl<Id> BlankApplyFns<Id>
where
    Id: Send + Sync + 'static,
{
    pub async fn apply_check(
        _params: &BlankParams<Id>,
        _data: BlankData<'_, Id>,
        _state_current: &BlankState,
        _state_goal: &BlankState,
        diff: &BlankStateDiff,
    ) -> Result<ApplyCheck, BlankError> {
        let apply_check = match *diff {
            BlankStateDiff::InSync { .. } => ApplyCheck::ExecNotRequired,
            BlankStateDiff::Added { .. } | BlankStateDiff::OutOfSync { .. } => {
                #[cfg(not(feature = "output_progress"))]
                {
                    ApplyCheck::ExecRequired
                }
                #[cfg(feature = "output_progress")]
                {
                    let progress_limit = ProgressLimit::Steps(1);
                    ApplyCheck::ExecRequired { progress_limit }
                }
            }
        };

        Ok(apply_check)
    }

    pub async fn apply_dry(
        _fn_ctx: FnCtx<'_>,
        _params: &BlankParams<Id>,
        _data: BlankData<'_, Id>,
        _state_current: &BlankState,
        state_goal: &BlankState,
        _diff: &BlankStateDiff,
    ) -> Result<BlankState, BlankError> {
        Ok(*state_goal)
    }

    pub async fn apply(
        _fn_ctx: FnCtx<'_>,
        _params: &BlankParams<Id>,
        mut data: BlankData<'_, Id>,
        _state_current: &BlankState,
        state_goal: &BlankState,
        _diff: &BlankStateDiff,
    ) -> Result<BlankState, BlankError> {
        let params = data.params_mut();
        params.dest.0 = Some(params.src.0);

        Ok(*state_goal)
    }
}
