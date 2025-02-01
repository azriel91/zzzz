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
    states::{StatesCurrent, StatesGoal},
    Resources,
};
use peace_rt_model::Storage;

use crate::cmd_blocks::StatesDiscoverCmdBlock;

pub struct StatesDiscoverCmd<CmdCtxTypesT>(PhantomData<CmdCtxTypesT>);

impl<CmdCtxTypesT> Debug for StatesDiscoverCmd<CmdCtxTypesT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("StatesDiscoverCmd").field(&self.0).finish()
    }
}

impl<CmdCtxTypesT> StatesDiscoverCmd<CmdCtxTypesT>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    /// Runs [`try_state_current`] for each [`Item`].
    ///
    /// At the end of this function, [`Resources`] will be populated with
    /// [`StatesCurrent`], and will be serialized to
    /// `$flow_dir/states_current.yaml`.
    ///
    /// If any `state_current` function needs to read the `State` from a
    /// previous `Item`, it may automatically be referenced using [`Current<T>`]
    /// where `T` us the predecessor's state. Peace will have automatically
    /// inserted it into `Resources`, and the successor should references it
    /// in their [`Data`].
    ///
    /// This function will always serialize states to storage.
    ///
    /// [`Current<T>`]: https://docs.rs/peace_data/latest/peace_data/marker/struct.Current.html
    /// [`Data`]: peace_cfg::TryFnSpec::Data
    /// [`Item`]: peace_cfg::Item
    /// [`try_state_current`]: peace_cfg::Item::try_state_current
    pub async fn current<'ctx>(
        cmd_ctx: &mut CmdCtx<SingleProfileSingleFlow<'ctx, CmdCtxTypesT>>,
    ) -> Result<
        CmdOutcome<StatesCurrent, <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError,
    >
    where
        CmdCtxTypesT: 'ctx,
    {
        Self::current_with(cmd_ctx, true).await
    }

    /// Runs [`try_state_current`] for each [`Item`].
    ///
    /// See [`Self::current`] for full documentation.
    ///
    /// This function exists so that this command can be executed as sub
    /// functionality of another command.
    ///
    /// # Parameters
    ///
    /// * `cmd_ctx`: Information needed to execute a command.
    /// * `serialize_to_storage`: Whether to write states to storage after
    ///   discovery.
    ///
    /// [`try_state_current`]: peace_cfg::Item::try_state_current
    pub async fn current_with<'ctx>(
        cmd_ctx: &mut CmdCtx<SingleProfileSingleFlow<'ctx, CmdCtxTypesT>>,
        serialize_to_storage: bool,
    ) -> Result<
        CmdOutcome<StatesCurrent, <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError,
    >
    where
        CmdCtxTypesT: 'ctx,
    {
        let mut cmd_execution = CmdExecution::<StatesCurrent, _>::builder()
            .with_cmd_block(CmdBlockWrapper::new(
                #[cfg(not(feature = "output_progress"))]
                StatesDiscoverCmdBlock::current(),
                #[cfg(feature = "output_progress")]
                StatesDiscoverCmdBlock::current().progress_complete_on_success(),
                StatesCurrent::from,
            ))
            .build();

        let cmd_outcome = cmd_execution.exec(cmd_ctx).await?;

        if let Some(states_current) = cmd_outcome.value() {
            let SingleProfileSingleFlowView {
                flow, resources, ..
            } = cmd_ctx.view();
            let (item_graph, resources) = (flow.graph(), resources);

            if serialize_to_storage {
                Self::serialize_current(item_graph, resources, states_current).await?;
            }
        }

        Ok(cmd_outcome)
    }

    /// Runs [`try_state_goal`] for each [`Item`].
    ///
    /// At the end of this function, [`Resources`] will be populated with
    /// [`StatesGoal`], and will be serialized to
    /// `$flow_dir/states_goal.yaml`.
    ///
    /// If any `state_goal` function needs to read the `State` from a
    /// previous `Item`, it may automatically be referenced using [`Goal<T>`]
    /// where `T` us the predecessor's state. Peace will have automatically
    /// inserted it into `Resources`, and the successor should references it
    /// in their [`Data`].
    ///
    /// This function will always serialize states to storage.
    ///
    /// [`Data`]: peace_cfg::TryFnSpec::Data
    /// [`Goal<T>`]: https://docs.rs/peace_data/latest/peace_data/marker/struct.Goal.html
    /// [`Item`]: peace_cfg::Item
    /// [`try_state_goal`]: peace_cfg::Item::try_state_goal
    pub async fn goal<'ctx>(
        cmd_ctx: &mut CmdCtx<SingleProfileSingleFlow<'ctx, CmdCtxTypesT>>,
    ) -> Result<
        CmdOutcome<StatesGoal, <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError,
    >
    where
        CmdCtxTypesT: 'ctx,
    {
        Self::goal_with(cmd_ctx, true).await
    }

    /// Runs [`try_state_goal`] for each [`Item`].
    ///
    /// See [`Self::goal`] for full documentation.
    ///
    /// This function exists so that this command can be executed as sub
    /// functionality of another command.
    ///
    /// # Parameters
    ///
    /// * `cmd_ctx`: Information needed to execute a command.
    /// * `serialize_to_storage`: Whether to write states to storage after
    ///   discovery.
    ///
    /// [`try_state_goal`]: peace_cfg::Item::try_state_goal
    pub async fn goal_with<'ctx>(
        cmd_ctx: &mut CmdCtx<SingleProfileSingleFlow<'ctx, CmdCtxTypesT>>,
        serialize_to_storage: bool,
    ) -> Result<
        CmdOutcome<StatesGoal, <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError,
    >
    where
        CmdCtxTypesT: 'ctx,
    {
        let mut cmd_execution = CmdExecution::<StatesGoal, _>::builder()
            .with_cmd_block(CmdBlockWrapper::new(
                #[cfg(not(feature = "output_progress"))]
                StatesDiscoverCmdBlock::goal(),
                #[cfg(feature = "output_progress")]
                StatesDiscoverCmdBlock::goal().progress_complete_on_success(),
                StatesGoal::from,
            ))
            .build();

        let cmd_outcome = cmd_execution.exec(cmd_ctx).await?;

        if let Some(states_goal) = cmd_outcome.value() {
            let SingleProfileSingleFlowView {
                flow, resources, ..
            } = cmd_ctx.view();
            let (item_graph, resources) = (flow.graph(), resources);

            if serialize_to_storage {
                Self::serialize_goal(item_graph, resources, states_goal).await?;
            }
        }

        Ok(cmd_outcome)
    }

    /// Runs [`try_state_current`] and [`try_state_goal`]` for each
    /// [`Item`].
    ///
    /// At the end of this function, [`Resources`] will be populated with
    /// [`StatesCurrent`] and [`StatesGoal`], and states will be serialized
    /// to `$flow_dir/states_current.yaml` and
    /// `$flow_dir/states_goal.yaml`.
    ///
    /// If any `state_current` function needs to read the `State` from a
    /// previous `Item`, the predecessor should insert a copy / clone of
    /// their state into `Resources`, and the successor should references it
    /// in their [`Data`].
    ///
    /// If any `state_goal` function needs to read the `State` from a
    /// previous `Item`, it may automatically be referenced using
    /// [`Goal<T>`] where `T` us the predecessor's state. Peace will have
    /// automatically inserted it into `Resources`, and the successor should
    /// references it in their [`Data`].
    ///
    /// This function will always serialize states to storage.
    ///
    /// [`Current<T>`]: https://docs.rs/peace_data/latest/peace_data/marker/struct.Current.html
    /// [`Data`]: peace_cfg::TryFnSpec::Data
    /// [`Goal<T>`]: https://docs.rs/peace_data/latest/peace_data/marker/struct.Goal.html
    /// [`Item`]: peace_cfg::Item
    /// [`try_state_current`]: peace_cfg::Item::try_state_current
    /// [`try_state_goal`]: peace_cfg::Item::try_state_goal
    pub async fn current_and_goal<'ctx>(
        cmd_ctx: &mut CmdCtx<SingleProfileSingleFlow<'ctx, CmdCtxTypesT>>,
    ) -> Result<
        CmdOutcome<(StatesCurrent, StatesGoal), <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError,
    >
    where
        CmdCtxTypesT: 'ctx,
    {
        Self::current_and_goal_with(cmd_ctx, true).await
    }

    /// Runs [`try_state_current`] and [`try_state_goal`]` for each
    /// [`Item`].
    ///
    /// See [`Self::goal`] for full documentation.
    ///
    /// This function exists so that this command can be executed as sub
    /// functionality of another command.
    ///
    /// # Parameters
    ///
    /// * `cmd_ctx`: Information needed to execute a command.
    /// * `serialize_to_storage`: Whether to write states to storage after
    ///   discovery.
    ///
    /// [`try_state_current`]: peace_cfg::Item::try_state_current
    /// [`try_state_goal`]: peace_cfg::Item::try_state_goal
    pub async fn current_and_goal_with<'ctx>(
        cmd_ctx: &mut CmdCtx<SingleProfileSingleFlow<'ctx, CmdCtxTypesT>>,
        serialize_to_storage: bool,
    ) -> Result<
        CmdOutcome<(StatesCurrent, StatesGoal), <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError,
    >
    where
        CmdCtxTypesT: 'ctx,
    {
        let mut cmd_execution = CmdExecution::<(StatesCurrent, StatesGoal), _>::builder()
            .with_cmd_block(CmdBlockWrapper::new(
                #[cfg(not(feature = "output_progress"))]
                StatesDiscoverCmdBlock::current_and_goal(),
                #[cfg(feature = "output_progress")]
                StatesDiscoverCmdBlock::current_and_goal().progress_complete_on_success(),
                |states_current_and_goal_mut| {
                    let (states_current_mut, states_goal_mut) = states_current_and_goal_mut;

                    (
                        StatesCurrent::from(states_current_mut),
                        StatesGoal::from(states_goal_mut),
                    )
                },
            ))
            .with_execution_outcome_fetch(|resources| {
                let states_current = resources.try_remove::<StatesCurrent>();
                let states_goal = resources.try_remove::<StatesGoal>();

                states_current.ok().zip(states_goal.ok())
            })
            .build();

        let cmd_outcome = cmd_execution.exec(cmd_ctx).await?;

        if let Some((states_current, states_goal)) = cmd_outcome.value() {
            let SingleProfileSingleFlowView {
                flow, resources, ..
            } = cmd_ctx.view();
            let (item_graph, resources) = (flow.graph(), resources);

            if serialize_to_storage {
                Self::serialize_current(item_graph, resources, states_current).await?;
                Self::serialize_goal(item_graph, resources, states_goal).await?;
            }
        }

        Ok(cmd_outcome)
    }

    // TODO: This duplicates a bit of code with `EnsureCmd` and `CleanCmd`.
    async fn serialize_current(
        item_graph: &ItemGraph<<CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        resources: &mut Resources<SetUp>,
        states_current: &StatesCurrent,
    ) -> Result<(), <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError> {
        use peace_state_rt::StatesSerializer;

        let flow_dir = resources.borrow::<FlowDir>();
        let storage = resources.borrow::<Storage>();
        let states_current_file = StatesCurrentFile::from(&*flow_dir);

        StatesSerializer::serialize(&storage, item_graph, states_current, &states_current_file)
            .await?;

        drop(flow_dir);
        drop(storage);

        resources.insert(states_current_file);

        Ok(())
    }

    async fn serialize_goal(
        item_graph: &ItemGraph<<CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        resources: &mut Resources<SetUp>,
        states_goal: &StatesGoal,
    ) -> Result<(), <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError> {
        use peace_state_rt::StatesSerializer;

        let flow_dir = resources.borrow::<FlowDir>();
        let storage = resources.borrow::<Storage>();
        let states_goal_file = StatesGoalFile::from(&*flow_dir);

        StatesSerializer::serialize(&storage, item_graph, states_goal, &states_goal_file).await?;

        drop(flow_dir);
        drop(storage);

        resources.insert(states_goal_file);

        Ok(())
    }
}

impl<CmdCtxTypesT> Default for StatesDiscoverCmd<CmdCtxTypesT> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
