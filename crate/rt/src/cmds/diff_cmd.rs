use std::{fmt::Debug, marker::PhantomData};

use futures::{StreamExt, TryStreamExt};
use peace_cfg::{ItemId, Profile};
use peace_cmd::{
    ctx::CmdCtx,
    scopes::{MultiProfileSingleFlow, MultiProfileSingleFlowView, SingleProfileSingleFlow},
};
use peace_cmd_model::CmdOutcome;
use peace_cmd_rt::{CmdBlockWrapper, CmdExecution, CmdExecutionBuilder};
use peace_params::ParamsSpecs;
use peace_resources::{
    internal::StateDiffsMut,
    resources::ts::SetUp,
    states::{
        ts::{CurrentStored, GoalStored},
        StateDiffs,
    },
    type_reg::untagged::{BoxDtDisplay, TypeMap},
    Resources,
};
use peace_rt_model::{output::OutputWrite, params::ParamsKeys, Error, Flow};

use crate::cmd_blocks::{
    DiffCmdBlock, DiffCmdBlockStatesTsExt, StatesCurrentReadCmdBlock, StatesDiscoverCmdBlock,
    StatesGoalReadCmdBlock,
};

pub use self::{diff_info_spec::DiffInfoSpec, diff_state_spec::DiffStateSpec};

mod diff_info_spec;
mod diff_state_spec;

pub struct DiffCmd<'cmd, CmdCtxTypeParamsT, Scope>(PhantomData<(E, &'cmd O, PKeys, Scope)>);

impl<'cmd, CmdCtxTypeParamsT, Scope> Debug for DiffCmd<'cmd, CmdCtxTypeParamsT, Scope> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("DiffCmd").field(&self.0).finish()
    }
}

impl<'cmd, CmdCtxTypeParamsT>
    DiffCmd<'cmd, CmdCtxTypeParamsT, SingleProfileSingleFlow<'cmd, CmdCtxTypeParamsT, SetUp>>
where
    E: std::error::Error + From<Error> + Send + Sync + Unpin + 'static,
    O: OutputWrite<E> + 'cmd,
    PKeys: ParamsKeys + 'static,
{
    /// Returns the [`state_diff`]`s between the stored current and goal
    /// states.
    ///
    /// Both current and goal states must have been discovered prior to
    /// running this. See [`StatesDiscoverCmd::current_and_goal`].
    ///
    /// This is equivalent to calling:
    ///
    /// ```rust,ignore
    /// DiffCmd::diff(cmd_ctx, DiffStateSpec::CurrentStored, DiffStateSpec::GoalStored).await?;
    /// ```
    ///
    /// [`state_diff`]: peace_cfg::Item::state_diff
    /// [`StatesDiscoverCmd::current_and_goal`]: crate::cmds::StatesDiscoverCmd::current_and_goal
    pub async fn diff_stored<'ctx>(
        cmd_ctx: &mut CmdCtx<SingleProfileSingleFlow<'ctx, CmdCtxTypeParamsT, SetUp>>,
    ) -> Result<CmdOutcome<StateDiffs, E>, E> {
        Self::diff::<CurrentStored, GoalStored>(cmd_ctx).await
    }

    /// Returns the [`state_diff`]`s between two states.
    ///
    /// For `CurrentStored` and `GoalStored`, states must have been discovered
    /// prior to running this. See [`StatesDiscoverCmd::current_and_goal`].
    ///
    /// For `Current` and `Goal` states, though they are discovered during the
    /// `DiffCmd` execution, they are not serialized.
    ///
    /// [`state_diff`]: peace_cfg::Item::state_diff
    /// [`StatesDiscoverCmd::current_and_goal`]: crate::cmds::StatesDiscoverCmd::current_and_goal
    pub async fn diff<'ctx, StatesTs0, StatesTs1>(
        cmd_ctx: &mut CmdCtx<SingleProfileSingleFlow<'ctx, CmdCtxTypeParamsT, SetUp>>,
    ) -> Result<CmdOutcome<StateDiffs, E>, E>
    where
        StatesTs0: Debug + DiffCmdBlockStatesTsExt + Send + Sync + Unpin + 'static,
        StatesTs1: Debug + DiffCmdBlockStatesTsExt + Send + Sync + Unpin + 'static,
    {
        let mut cmd_execution_builder = CmdExecution::<StateDiffs, _, _>::builder();
        cmd_execution_builder = Self::states_fetch_cmd_block_append(
            cmd_execution_builder,
            StatesTs0::diff_state_spec(),
        );
        cmd_execution_builder = Self::states_fetch_cmd_block_append(
            cmd_execution_builder,
            StatesTs1::diff_state_spec(),
        );

        cmd_execution_builder = cmd_execution_builder.with_cmd_block(CmdBlockWrapper::new(
            DiffCmdBlock::<_, _, StatesTs0, StatesTs1>::new(),
            |_state_diffs_ts0_and_ts1| StateDiffs::new(),
        ));

        #[cfg(feature = "output_progress")]
        let cmd_execution_builder = cmd_execution_builder.with_progress_render_enabled(false);

        cmd_execution_builder.build().exec(cmd_ctx).await
    }

    fn states_fetch_cmd_block_append(
        cmd_execution_builder: CmdExecutionBuilder<StateDiffs, CmdCtxTypeParamsT>,
        diff_state_spec: DiffStateSpec,
    ) -> CmdExecutionBuilder<StateDiffs, CmdCtxTypeParamsT> {
        match diff_state_spec {
            DiffStateSpec::Current => cmd_execution_builder.with_cmd_block(CmdBlockWrapper::new(
                StatesDiscoverCmdBlock::current(),
                |_states_current_mut| StateDiffs::new(),
            )),
            DiffStateSpec::CurrentStored => cmd_execution_builder.with_cmd_block(
                CmdBlockWrapper::new(StatesCurrentReadCmdBlock::new(), |_| StateDiffs::new()),
            ),
            DiffStateSpec::Goal => cmd_execution_builder
                .with_cmd_block(CmdBlockWrapper::new(StatesDiscoverCmdBlock::goal(), |_| {
                    StateDiffs::new()
                })),
            DiffStateSpec::GoalStored => {
                cmd_execution_builder
                    .with_cmd_block(CmdBlockWrapper::new(StatesGoalReadCmdBlock::new(), |_| {
                        StateDiffs::new()
                    }))
            }
        }
    }
}

impl<'cmd, CmdCtxTypeParamsT>
    DiffCmd<'cmd, CmdCtxTypeParamsT, MultiProfileSingleFlow<'cmd, CmdCtxTypeParamsT, SetUp>>
where
    E: std::error::Error + From<Error> + Send + 'static,
    O: OutputWrite<E> + 'cmd,
    PKeys: ParamsKeys + 'static,
{
    /// Returns the [`state_diff`]`s between the stored current states of two
    /// profiles.
    ///
    /// Both profiles' current states must have been discovered prior to
    /// running this. See [`StatesDiscoverCmd::current`].
    ///
    /// [`state_diff`]: peace_cfg::Item::state_diff
    /// [`StatesDiscoverCmd::current`]: crate::cmds::StatesDiscoverCmd::current
    pub async fn diff_current_stored<'ctx>(
        cmd_ctx: &mut CmdCtx<MultiProfileSingleFlow<'ctx, CmdCtxTypeParamsT, SetUp>>,
        profile_a: &Profile,
        profile_b: &Profile,
    ) -> Result<StateDiffs, E> {
        let MultiProfileSingleFlowView {
            flow,
            profiles,
            profile_to_params_specs,
            profile_to_states_current_stored,
            resources,
            ..
        } = cmd_ctx.view();

        let params_specs = profile_to_params_specs
            .get(profile_a)
            .or_else(|| profile_to_params_specs.get(profile_b));
        let params_specs = if let Some(Some(params_specs)) = params_specs {
            params_specs
        } else {
            Err(Error::ParamsSpecsNotDefinedForDiff {
                profile_a: profile_a.clone(),
                profile_b: profile_b.clone(),
            })?
        };
        let states_a = profile_to_states_current_stored
            .get(profile_a)
            .ok_or_else(|| {
                let profile = profile_a.clone();
                let profiles_in_scope = profiles.to_vec();
                Error::ProfileNotInScope {
                    profile,
                    profiles_in_scope,
                }
            })?
            .as_ref()
            .ok_or_else(|| {
                let profile = profile_a.clone();
                Error::ProfileStatesCurrentNotDiscovered { profile }
            })?;
        let states_b = profile_to_states_current_stored
            .get(profile_b)
            .ok_or_else(|| {
                let profile = profile_b.clone();
                let profiles_in_scope = profiles.to_vec();
                Error::ProfileNotInScope {
                    profile,
                    profiles_in_scope,
                }
            })?
            .as_ref()
            .ok_or_else(|| {
                let profile = profile_b.clone();
                Error::ProfileStatesCurrentNotDiscovered { profile }
            })?;

        Self::diff_any(flow, params_specs, resources, states_a, states_b).await
    }
}

impl<'cmd, CmdCtxTypeParamsT, Scope> DiffCmd<'cmd, CmdCtxTypeParamsT, Scope>
where
    E: std::error::Error + From<Error> + Send + 'static,
    O: OutputWrite<E> + 'cmd,
    PKeys: ParamsKeys + 'static,
{
    /// Returns the [`state_diff`]` for each [`Item`].
    ///
    /// This does not take in `CmdCtx` as it may be used by both
    /// `SingleProfileSingleFlow` and `MultiProfileSingleFlow`
    /// commands.
    ///
    /// [`Item`]: peace_cfg::Item
    /// [`state_diff`]: peace_cfg::Item::state_diff
    pub async fn diff_any(
        flow: &Flow<CmdCtxTypeParamsT::AppError>,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        states_a: &TypeMap<ItemId, BoxDtDisplay>,
        states_b: &TypeMap<ItemId, BoxDtDisplay>,
    ) -> Result<StateDiffs, E> {
        let state_diffs = {
            let state_diffs_mut = flow
                .graph()
                .stream()
                .map(Result::<_, E>::Ok)
                .try_filter_map(|item| async move {
                    let state_diff_opt = item
                        .state_diff_exec(params_specs, resources, states_a, states_b)
                        .await?;

                    Ok(state_diff_opt.map(|state_diff| (item.id().clone(), state_diff)))
                })
                .try_collect::<StateDiffsMut>()
                .await?;

            StateDiffs::from(state_diffs_mut)
        };

        Ok(state_diffs)
    }
}

impl<'cmd, CmdCtxTypeParamsT, Scope> Default for DiffCmd<'cmd, CmdCtxTypeParamsT, Scope> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
