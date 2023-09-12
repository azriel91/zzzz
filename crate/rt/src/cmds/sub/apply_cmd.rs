use std::{fmt::Debug, marker::PhantomData};

use peace_cmd::{
    ctx::CmdCtx,
    scopes::{SingleProfileSingleFlow, SingleProfileSingleFlowView},
};
use peace_cmd_rt::{CmdBlockWrapper, CmdExecution};

use peace_resources::{
    paths::{FlowDir, StatesCurrentFile, StatesGoalFile},
    resources::ts::SetUp,
    states::{States, StatesCurrent, StatesGoal, StatesPrevious},
    Resources,
};
use peace_rt_model::{
    outcomes::CmdOutcome, output::OutputWrite, params::ParamsKeys, Error, ItemGraph, Storage,
};

use crate::cmd_blocks::{
    apply_exec_cmd_block::{ApplyFor, StatesTsApplyExt},
    ApplyExecCmdBlock, ApplyStateSyncCheckCmdBlock, StatesCurrentReadCmdBlock,
    StatesDiscoverCmdBlock, StatesGoalReadCmdBlock,
};

pub use self::apply_stored_state_sync::ApplyStoredStateSync;

mod apply_stored_state_sync;

#[derive(Debug)]
pub struct ApplyCmd<E, O, PKeys, StatesTsApply, StatesTsApplyDry>(
    PhantomData<(E, O, PKeys, StatesTsApply, StatesTsApplyDry)>,
);

impl<E, O, PKeys, StatesTsApply, StatesTsApplyDry>
    ApplyCmd<E, O, PKeys, StatesTsApply, StatesTsApplyDry>
where
    E: std::error::Error + From<Error> + Send + Sync + Unpin + 'static,
    PKeys: ParamsKeys + 'static,
    O: OutputWrite<E>,
    StatesTsApply: Debug + StatesTsApplyExt + Send + Sync + Unpin + 'static,
    StatesTsApplyDry: Debug + StatesTsApplyExt + Send + Sync + Unpin + 'static,
    States<StatesTsApply>: From<StatesCurrent> + Send + Sync + Unpin + 'static,
    States<StatesTsApplyDry>: From<StatesCurrent> + Send + Sync + Unpin + 'static,
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
    /// The grouping of item functions run for a `Clean` execution to work
    /// is as follows:
    ///
    /// 1. Run [`StatesDiscoverCmd::current`] for all `Item`s in the
    ///   *forward* direction.
    ///
    ///     This populates `resources` with `Current<IS::State>`, needed for
    ///     `Item::try_state_current` during `ItemRt::clean_prepare`.
    ///
    /// 2. In the *reverse* direction, for each `Item` run
    ///    `ItemRt::clean_prepare`, which runs:
    ///
    ///     1. `Item::try_state_current`, which resolves parameters from the
    ///        *current* state.
    ///     2. `Item::state_goal`
    ///     3. `Item::apply_check`
    ///
    /// 3. For `Item`s that return `ApplyCheck::ExecRequired`, run
    ///    `Item::apply_exec_dry`.
    ///
    /// [`apply_exec_dry`]: peace_cfg::Item::apply_exec_dry
    /// [`Item::apply_check`]: peace_cfg::Item::apply_check
    /// [`Item::apply_exec_dry`]: peace_cfg::ItemRt::apply_exec_dry
    /// [`Item`]: peace_cfg::Item
    pub async fn exec_dry(
        cmd_ctx: &mut CmdCtx<SingleProfileSingleFlow<'_, E, O, PKeys, SetUp>>,
    ) -> Result<CmdOutcome<States<StatesTsApplyDry>, E>, E> {
        Self::exec_dry_with(cmd_ctx, ApplyStoredStateSync::Both).await
    }

    /// Conditionally runs [`Item::apply_exec_dry`] for each [`Item`].
    ///
    /// See [`Self::exec_dry`] for full documentation.
    ///
    /// This function exists so that this command can be executed as sub
    /// functionality of another command.
    pub async fn exec_dry_with(
        cmd_ctx: &mut CmdCtx<SingleProfileSingleFlow<'_, E, O, PKeys, SetUp>>,
        apply_stored_state_sync: ApplyStoredStateSync,
    ) -> Result<CmdOutcome<States<StatesTsApplyDry>, E>, E> {
        Self::exec_internal(cmd_ctx, apply_stored_state_sync)
            .await
            .map(|cmd_outcome| cmd_outcome.map(|(states_applied, _states_goal)| states_applied))
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
    /// The grouping of item functions run for a `Clean` execution to work
    /// is as follows:
    ///
    /// 1. Run [`StatesDiscoverCmd::current`] for all `Item`s in the
    ///   *forward* direction.
    ///
    ///     This populates `resources` with `Current<IS::State>`, needed for
    ///     `Item::try_state_current` during `ItemRt::clean_prepare`.
    ///
    /// 2. In the *reverse* direction, for each `Item` run
    ///    `ItemRt::clean_prepare`, which runs:
    ///
    ///     1. `Item::try_state_current`, which resolves parameters from the
    ///        *current* state.
    ///     2. `Item::state_goal`
    ///     3. `Item::apply_check`
    ///
    /// 3. For `Item`s that return `ApplyCheck::ExecRequired`, run
    ///    `Item::apply_exec`.
    ///
    /// [`apply_exec`]: peace_cfg::Item::apply_exec
    /// [`Item::apply_check`]: peace_cfg::Item::apply_check
    /// [`Item::apply_exec`]: peace_cfg::ItemRt::apply_exec
    /// [`Item`]: peace_cfg::Item
    pub async fn exec(
        cmd_ctx: &mut CmdCtx<SingleProfileSingleFlow<'_, E, O, PKeys, SetUp>>,
    ) -> Result<CmdOutcome<States<StatesTsApply>, E>, E> {
        Self::exec_with(cmd_ctx, ApplyStoredStateSync::Both).await
    }

    /// Conditionally runs [`Item::apply_exec`] for each [`Item`].
    ///
    /// See [`Self::exec`] for full documentation.
    ///
    /// This function exists so that this command can be executed as sub
    /// functionality of another command.
    pub async fn exec_with(
        cmd_ctx: &mut CmdCtx<SingleProfileSingleFlow<'_, E, O, PKeys, SetUp>>,
        apply_stored_state_sync: ApplyStoredStateSync,
    ) -> Result<CmdOutcome<States<StatesTsApply>, E>, E> {
        let CmdOutcome {
            value: (states_applied, states_goal),
            errors,
        } = Self::exec_internal(cmd_ctx, apply_stored_state_sync).await?;

        let SingleProfileSingleFlowView {
            flow, resources, ..
        } = cmd_ctx.view();
        let (item_graph, resources) = (flow.graph(), resources);

        Self::serialize_current(item_graph, resources, &states_applied).await?;

        let apply_for = StatesTsApply::apply_for();
        match apply_for {
            ApplyFor::Ensure => {
                Self::serialize_goal(item_graph, resources, &states_goal).await?;
            }
            ApplyFor::Clean => {}
        };

        let cmd_outcome = CmdOutcome {
            value: states_applied,
            errors,
        };
        Ok(cmd_outcome)
    }

    /// Conditionally runs [`ApplyFns`]`::`[`exec`] for each [`Item`].
    ///
    /// Same as [`Self::exec`], but does not change the type state, and returns
    /// [`States<StatesTsApply>`].
    ///
    /// [`exec`]: peace_cfg::ApplyFns::exec
    /// [`Item`]: peace_cfg::Item
    /// [`ApplyFns`]: peace_cfg::Item::ApplyFns
    async fn exec_internal<StatesTs>(
        cmd_ctx: &mut CmdCtx<SingleProfileSingleFlow<'_, E, O, PKeys, SetUp>>,
        apply_stored_state_sync: ApplyStoredStateSync,
    ) -> Result<CmdOutcome<(States<StatesTs>, StatesGoal), E>, E>
    where
        StatesTs: Debug + StatesTsApplyExt + Send + Sync + Unpin + 'static,
    {
        let mut cmd_execution = {
            let mut cmd_execution_builder =
                CmdExecution::<(StatesPrevious, States<StatesTs>, StatesGoal), _, _>::builder()
                    .with_cmd_block(CmdBlockWrapper::new(
                        StatesCurrentReadCmdBlock::new(),
                        |states_current_stored| {
                            (
                                StatesPrevious::from(states_current_stored.into_inner()),
                                States::<StatesTs>::new(),
                                StatesGoal::new(),
                            )
                        },
                    ))
                    .with_cmd_block(CmdBlockWrapper::new(
                        StatesGoalReadCmdBlock::new(),
                        |states_goal_stored| {
                            (
                                StatesPrevious::new(),
                                States::<StatesTs>::new(),
                                StatesGoal::from(states_goal_stored.into_inner()),
                            )
                        },
                    ));

            cmd_execution_builder = match apply_stored_state_sync {
                ApplyStoredStateSync::None => cmd_execution_builder.with_cmd_block(
                    CmdBlockWrapper::new(ApplyStateSyncCheckCmdBlock::goal(), |_| {
                        Default::default()
                    }),
                ),
                ApplyStoredStateSync::Current => cmd_execution_builder
                    .with_cmd_block(CmdBlockWrapper::new(
                        StatesDiscoverCmdBlock::current(),
                        |states_current_mut| {
                            (
                                StatesPrevious::from(StatesCurrent::from(states_current_mut)),
                                States::<StatesTs>::new(),
                                StatesGoal::new(),
                            )
                        },
                    ))
                    .with_cmd_block(CmdBlockWrapper::new(
                        ApplyStateSyncCheckCmdBlock::goal(),
                        |_| Default::default(),
                    )),
                ApplyStoredStateSync::Goal => cmd_execution_builder
                    .with_cmd_block(CmdBlockWrapper::new(
                        StatesDiscoverCmdBlock::current_and_goal(),
                        |states_current_and_goal_mut| {
                            let (states_current_mut, states_goal_mut) = states_current_and_goal_mut;

                            (
                                StatesPrevious::from(StatesCurrent::from(states_current_mut)),
                                States::<StatesTs>::new(),
                                StatesGoal::from(states_goal_mut),
                            )
                        },
                    ))
                    .with_cmd_block(CmdBlockWrapper::new(
                        ApplyStateSyncCheckCmdBlock::goal(),
                        |_| Default::default(),
                    )),
                ApplyStoredStateSync::Both => cmd_execution_builder
                    .with_cmd_block(CmdBlockWrapper::new(
                        StatesDiscoverCmdBlock::current_and_goal(),
                        |states_current_and_goal_mut| {
                            let (states_current_mut, states_goal_mut) = states_current_and_goal_mut;

                            (
                                StatesPrevious::from(StatesCurrent::from(states_current_mut)),
                                States::<StatesTs>::new(),
                                StatesGoal::from(states_goal_mut),
                            )
                        },
                    ))
                    .with_cmd_block(CmdBlockWrapper::new(
                        ApplyStateSyncCheckCmdBlock::current_and_goal(),
                        |_| Default::default(),
                    )),
            };

            cmd_execution_builder
                .with_cmd_block(CmdBlockWrapper::new(
                    ApplyExecCmdBlock::<E, PKeys, StatesTs>::new(),
                    |_| Default::default(),
                ))
                .with_execution_outcome_fetch(|resources| {
                    let states_previous =
                        resources.remove::<StatesPrevious>().unwrap_or_else(|| {
                            let states_previous = tynm::type_name::<States<StatesTs>>();
                            panic!("Expected `{states_previous}` to exist in `Resources`");
                        });
                    let states_applied =
                        resources.remove::<States<StatesTs>>().unwrap_or_else(|| {
                            let states_applied = tynm::type_name::<States<StatesTs>>();
                            panic!("Expected `{states_applied}` to exist in `Resources`");
                        });
                    let states_goal = resources.remove::<StatesGoal>().unwrap_or_else(|| {
                        let states_goal = tynm::type_name::<StatesGoal>();
                        panic!("Expected `{states_goal}` to exist in `Resources`");
                    });

                    (states_previous, states_applied, states_goal)
                })
                .build()
        };

        let cmd_outcome = cmd_execution.exec(cmd_ctx).await?;

        // TODO: Should we run `StatesCurrentFn` again?
        //
        // i.e. is it part of `ApplyFns::exec`'s contract to return the state.
        //
        // * It may be duplication of code.
        // * `FileDownloadItem` needs to know the ETag from the last request, which:
        //     - in `StatesCurrentFn` comes from `StatesCurrent`
        //     - in `ApplyCmd` comes from `StatesTsApply`
        // * `ShCmdItem` doesn't return the state in the apply script, so in the item we
        //   run the state current script after the apply exec script.

        let cmd_outcome = cmd_outcome.map(|(states_previous, states_applied, states_goal)| {
            cmd_ctx
                .view()
                .resources
                .insert::<StatesPrevious>(states_previous);

            (states_applied, states_goal)
        });

        Ok(cmd_outcome)
    }

    // TODO: This duplicates a bit of code with `StatesDiscoverCmd`,
    async fn serialize_current(
        item_graph: &ItemGraph<E>,
        resources: &Resources<SetUp>,
        states_applied: &States<StatesTsApply>,
    ) -> Result<(), E> {
        use peace_rt_model::StatesSerializer;

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
        item_graph: &ItemGraph<E>,
        resources: &Resources<SetUp>,
        states_goal: &StatesGoal,
    ) -> Result<(), E> {
        use peace_rt_model::StatesSerializer;

        let flow_dir = resources.borrow::<FlowDir>();
        let storage = resources.borrow::<Storage>();
        let states_goal_file = StatesGoalFile::from(&*flow_dir);

        StatesSerializer::serialize(&storage, item_graph, states_goal, &states_goal_file).await?;

        drop(flow_dir);
        drop(storage);

        Ok(())
    }
}

impl<E, O, PKeys, StatesTsApply, StatesTsApplyDry> Default
    for ApplyCmd<E, O, PKeys, StatesTsApply, StatesTsApplyDry>
{
    fn default() -> Self {
        Self(PhantomData)
    }
}
