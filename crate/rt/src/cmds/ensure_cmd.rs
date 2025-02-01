use std::{fmt::Debug, marker::PhantomData};

use peace_cmd::{
    ctx::{CmdCtx, CmdCtxTypesConstrained},
    scopes::{SingleProfileSingleFlow, SingleProfileSingleFlowView},
};
use peace_cmd_model::CmdOutcome;
use peace_cmd_rt::{CmdBlockWrapper, CmdExecution};
use peace_flow_rt::ItemGraph;
use peace_resource_rt::{
    paths::{FlowDir, StatesCurrentFile, StatesGoalFile},
    resources::ts::SetUp,
    states::{States, StatesEnsured, StatesEnsuredDry, StatesGoal, StatesPrevious},
    Resources,
};
use peace_rt_model::Storage;

use crate::{
    cmd_blocks::{
        apply_exec_cmd_block::StatesTsApplyExt, ApplyExecCmdBlock, ApplyStateSyncCheckCmdBlock,
        StatesCurrentReadCmdBlock, StatesDiscoverCmdBlock, StatesGoalReadCmdBlock,
    },
    cmds::ApplyStoredStateSync,
};

#[derive(Debug)]
pub struct EnsureCmd<CmdCtxTypesT>(PhantomData<CmdCtxTypesT>);

impl<CmdCtxTypesT> EnsureCmd<CmdCtxTypesT>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    /// Conditionally runs [`Item::apply_exec_dry`] for each [`Item`].
    ///
    /// In practice this runs [`Item::apply_check`], and only runs
    /// [`apply_exec_dry`] if execution is required.
    ///
    /// # Design
    ///
    /// The grouping of item functions run for an `Ensure` execution to
    /// work is as follows:
    ///
    /// 1. For each `Item` run `ItemRt::ensure_prepare`, which runs:
    ///
    ///     1. `Item::state_current`
    ///     2. `Item::state_goal`
    ///     3. `Item::apply_check`
    ///
    /// 2. For `Item`s that return `ApplyCheck::ExecRequired`, run
    ///    `Item::apply_exec_dry`.
    ///
    /// [`apply_exec_dry`]: peace_cfg::Item::apply_exec_dry
    /// [`Item::apply_check`]: peace_cfg::Item::apply_check
    /// [`Item::apply_exec_dry`]: peace_cfg::ItemRt::apply_exec_dry
    /// [`Item`]: peace_cfg::Item
    pub async fn exec_dry<'ctx>(
        cmd_ctx: &mut CmdCtx<SingleProfileSingleFlow<'ctx, CmdCtxTypesT>>,
    ) -> Result<
        CmdOutcome<StatesEnsuredDry, <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError,
    >
    where
        CmdCtxTypesT: 'ctx,
    {
        Self::exec_dry_with(cmd_ctx, ApplyStoredStateSync::Both).await
    }

    /// Conditionally runs [`Item::apply_exec_dry`] for each [`Item`].
    ///
    /// See [`Self::exec_dry`] for full documentation.
    ///
    /// This function exists so that this command can be executed as sub
    /// functionality of another command.
    pub async fn exec_dry_with<'ctx>(
        cmd_ctx: &mut CmdCtx<SingleProfileSingleFlow<'ctx, CmdCtxTypesT>>,
        apply_stored_state_sync: ApplyStoredStateSync,
    ) -> Result<
        CmdOutcome<StatesEnsuredDry, <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError,
    >
    where
        CmdCtxTypesT: 'ctx,
    {
        let cmd_outcome = Self::exec_internal(cmd_ctx, apply_stored_state_sync).await?;

        let cmd_outcome = cmd_outcome.map(|ensure_exec_change| match ensure_exec_change {
            EnsureExecChange::None => Default::default(),
            EnsureExecChange::Some(stateses_boxed) => {
                let (states_previous, states_applied_dry, _states_goal) = *stateses_boxed;
                cmd_ctx
                    .view()
                    .resources
                    .insert::<StatesPrevious>(states_previous);

                states_applied_dry
            }
        });

        Ok(cmd_outcome)
    }

    /// Conditionally runs [`Item::apply_exec`] for each [`Item`].
    ///
    /// In practice this runs [`Item::apply_check`], and only runs
    /// [`apply_exec`] if execution is required.
    ///
    /// # Design
    ///
    /// The grouping of item functions run for an `Ensure` execution to
    /// work is as follows:
    ///
    /// 1. For each `Item` run `ItemRt::ensure_prepare`, which runs:
    ///
    ///     1. `Item::state_current`
    ///     2. `Item::state_goal`
    ///     3. `Item::apply_check`
    ///
    /// 2. For `Item`s that return `ApplyCheck::ExecRequired`, run
    ///    `Item::apply_exec`.
    ///
    /// [`apply_exec`]: peace_cfg::Item::apply_exec
    /// [`Item::apply_check`]: peace_cfg::Item::apply_check
    /// [`Item::apply_exec`]: peace_cfg::ItemRt::apply_exec
    /// [`Item`]: peace_cfg::Item
    pub async fn exec<'ctx>(
        cmd_ctx: &mut CmdCtx<SingleProfileSingleFlow<'ctx, CmdCtxTypesT>>,
    ) -> Result<
        CmdOutcome<StatesEnsured, <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError,
    >
    where
        CmdCtxTypesT: 'ctx,
    {
        Self::exec_with(cmd_ctx, ApplyStoredStateSync::Both).await
    }

    /// Conditionally runs [`Item::apply_exec`] for each [`Item`].
    ///
    /// See [`Self::exec`] for full documentation.
    ///
    /// This function exists so that this command can be executed as sub
    /// functionality of another command.
    pub async fn exec_with<'ctx>(
        cmd_ctx: &mut CmdCtx<SingleProfileSingleFlow<'ctx, CmdCtxTypesT>>,
        apply_stored_state_sync: ApplyStoredStateSync,
    ) -> Result<
        CmdOutcome<StatesEnsured, <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError,
    >
    where
        CmdCtxTypesT: 'ctx,
    {
        let cmd_outcome = Self::exec_internal(cmd_ctx, apply_stored_state_sync).await?;

        let SingleProfileSingleFlowView {
            flow, resources, ..
        } = cmd_ctx.view();
        let (item_graph, resources) = (flow.graph(), resources);

        // We shouldn't serialize current or goal if we returned from an interruption /
        // error handler.
        let cmd_outcome = cmd_outcome
            .map_async(|ensure_exec_change| async move {
                match ensure_exec_change {
                    EnsureExecChange::None => Ok(Default::default()),
                    EnsureExecChange::Some(stateses_boxed) => {
                        let (states_previous, states_applied, states_goal) = *stateses_boxed;
                        Self::serialize_current(item_graph, resources, &states_applied).await?;
                        Self::serialize_goal(item_graph, resources, &states_goal).await?;

                        resources.insert::<StatesPrevious>(states_previous);

                        Ok(states_applied)
                    }
                }
            })
            .await;

        cmd_outcome.transpose()
    }

    /// Conditionally runs [`ApplyFns`]`::`[`exec`] for each [`Item`].
    ///
    /// Same as [`Self::exec`], but does not change the type state, and returns
    /// [`StatesEnsured`].
    ///
    /// [`exec`]: peace_cfg::ApplyFns::exec
    /// [`Item`]: peace_cfg::Item
    /// [`ApplyFns`]: peace_cfg::Item::ApplyFns
    async fn exec_internal<'ctx, StatesTs>(
        cmd_ctx: &mut CmdCtx<SingleProfileSingleFlow<'ctx, CmdCtxTypesT>>,
        apply_stored_state_sync: ApplyStoredStateSync,
    ) -> Result<
        CmdOutcome<EnsureExecChange<StatesTs>, <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError,
    >
    where
        CmdCtxTypesT: 'ctx,
        StatesTs: StatesTsApplyExt + Debug + Send + Sync + Unpin + 'static,
    {
        let mut cmd_execution = {
            let mut cmd_execution_builder =
                CmdExecution::<EnsureExecChange<StatesTs>, _>::builder()
                    .with_cmd_block(CmdBlockWrapper::new(
                        StatesCurrentReadCmdBlock::new(),
                        |_states_current_stored| EnsureExecChange::None,
                    ))
                    .with_cmd_block(CmdBlockWrapper::new(
                        StatesGoalReadCmdBlock::new(),
                        |_states_goal_stored| EnsureExecChange::None,
                    ))
                    // Always discover current and goal states, because they are read whether or not
                    // we are checking for state sync.
                    //
                    // Exception: current states are not used for `ApplyStoredStateSync::None`,
                    // since we have to discover the new current state after every apply.
                    .with_cmd_block(CmdBlockWrapper::new(
                        StatesDiscoverCmdBlock::current_and_goal(),
                        |_states_current_and_goal_mut| EnsureExecChange::None,
                    ));

            cmd_execution_builder = match apply_stored_state_sync {
                ApplyStoredStateSync::None => cmd_execution_builder,
                ApplyStoredStateSync::Current => cmd_execution_builder.with_cmd_block(
                    CmdBlockWrapper::new(ApplyStateSyncCheckCmdBlock::current(), |_| {
                        EnsureExecChange::None
                    }),
                ),
                ApplyStoredStateSync::Goal => cmd_execution_builder.with_cmd_block(
                    CmdBlockWrapper::new(ApplyStateSyncCheckCmdBlock::goal(), |_| {
                        EnsureExecChange::None
                    }),
                ),
                ApplyStoredStateSync::Both => cmd_execution_builder.with_cmd_block(
                    CmdBlockWrapper::new(ApplyStateSyncCheckCmdBlock::current_and_goal(), |_| {
                        EnsureExecChange::None
                    }),
                ),
            };

            cmd_execution_builder
                .with_cmd_block(CmdBlockWrapper::new(
                    ApplyExecCmdBlock::<CmdCtxTypesT, StatesTs>::new(),
                    |(states_previous, states_applied, states_target): (
                        StatesPrevious,
                        States<StatesTs>,
                        States<StatesTs::TsTarget>,
                    )| {
                        EnsureExecChange::Some(Box::new((
                            states_previous,
                            states_applied,
                            StatesGoal::from(states_target.into_inner()),
                        )))
                    },
                ))
                .with_execution_outcome_fetch(|resources| {
                    let states_previous = resources.try_remove::<StatesPrevious>();
                    let states_applied = resources.try_remove::<States<StatesTs>>();
                    let states_goal = resources.try_remove::<StatesGoal>();

                    if let Some(((states_previous, states_applied), states_goal)) = states_previous
                        .ok()
                        .zip(states_applied.ok())
                        .zip(states_goal.ok())
                    {
                        Some(EnsureExecChange::Some(Box::new((
                            states_previous,
                            states_applied,
                            states_goal,
                        ))))
                    } else {
                        Some(EnsureExecChange::None)
                    }
                })
                .build()
        };

        let ensure_exec_change = cmd_execution.exec(cmd_ctx).await?;

        // TODO: Should we run `StatesCurrentFn` again?
        //
        // i.e. is it part of `ApplyFns::exec`'s contract to return the state.
        //
        // * It may be duplication of code.
        // * `FileDownloadItem` needs to know the ETag from the last request, which:
        //     - in `StatesCurrentFn` comes from `StatesCurrent`
        //     - in `EnsureCmd` comes from `Ensured`
        // * `ShCmdItem` doesn't return the state in the apply script, so in the item we
        //   run the state current script after the apply exec script.

        Ok(ensure_exec_change)
    }

    // TODO: This duplicates a bit of code with `StatesDiscoverCmd`,
    async fn serialize_current(
        item_graph: &ItemGraph<<CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        resources: &Resources<SetUp>,
        states_applied: &StatesEnsured,
    ) -> Result<(), <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError> {
        use peace_state_rt::StatesSerializer;

        let flow_dir = resources.borrow::<FlowDir>();
        let storage = resources.borrow::<Storage>();
        let states_current_file = StatesCurrentFile::from(&*flow_dir);

        StatesSerializer::serialize(&storage, item_graph, states_applied, &states_current_file)
            .await?;

        drop(flow_dir);
        drop(storage);

        Ok(())
    }

    async fn serialize_goal(
        item_graph: &ItemGraph<<CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        resources: &Resources<SetUp>,
        states_goal: &StatesGoal,
    ) -> Result<(), <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError> {
        use peace_state_rt::StatesSerializer;

        let flow_dir = resources.borrow::<FlowDir>();
        let storage = resources.borrow::<Storage>();
        let states_goal_file = StatesGoalFile::from(&*flow_dir);

        StatesSerializer::serialize(&storage, item_graph, states_goal, &states_goal_file).await?;

        drop(flow_dir);
        drop(storage);

        Ok(())
    }
}

impl<CmdCtxTypesT> Default for EnsureCmd<CmdCtxTypesT> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[derive(Debug)]
enum EnsureExecChange<StatesTs> {
    /// Nothing changed, so nothing to serialize.
    None,
    /// Some state was changed, so serialization is required.
    ///
    /// This variant is used for both partial and complete execution, as long as
    /// some state was altered.
    Some(Box<(StatesPrevious, States<StatesTs>, StatesGoal)>),
}
