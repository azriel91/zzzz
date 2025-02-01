use std::{fmt::Debug, marker::PhantomData};

use futures::join;
use peace_cfg::FnCtx;
use peace_cmd::{ctx::CmdCtxTypesConstrained, scopes::SingleProfileSingleFlowView};
use peace_cmd_model::CmdBlockOutcome;
use peace_cmd_rt::{async_trait, CmdBlock};
use peace_item_model::ItemId;
use peace_resource_rt::{
    internal::StatesMut,
    resources::ts::SetUp,
    states::{
        ts::{Current, Goal},
        States, StatesCurrent, StatesGoal,
    },
    type_reg::untagged::BoxDtDisplay,
    ResourceFetchError, Resources,
};
use peace_rt_model::{fn_graph::StreamOpts, ItemBoxed};
use peace_rt_model_core::IndexMap;
use tokio::sync::mpsc::{self, Receiver};

use crate::BUFFERED_FUTURES_MAX;

cfg_if::cfg_if! {
    if #[cfg(feature = "output_progress")] {
        use peace_progress_model::{
            CmdBlockItemInteractionType,
            CmdProgressUpdate,
            ProgressComplete,
            ProgressDelta,
            ProgressMsgUpdate,
            ProgressSender,
            ProgressUpdate,
            ProgressUpdateAndId,
        };
        use tokio::sync::mpsc::Sender;
    }
}

/// Discovers current states.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiscoverForCurrent;

/// Discovers goal states.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiscoverForGoal;

/// Discovers current and goal states.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiscoverForCurrentAndGoal;

/// Discovers [`StatesCurrent`] and/or [`StatesGoal`].
pub struct StatesDiscoverCmdBlock<CmdCtxTypesT, DiscoverFor> {
    /// Whether or not to mark progress bars complete on success.
    #[cfg(feature = "output_progress")]
    progress_complete_on_success: bool,
    /// Marker.
    marker: PhantomData<(CmdCtxTypesT, DiscoverFor)>,
}

impl<CmdCtxTypesT, DiscoverFor> Debug for StatesDiscoverCmdBlock<CmdCtxTypesT, DiscoverFor> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("StatesDiscoverCmdBlock");
        #[cfg(feature = "output_progress")]
        debug_struct.field(
            "progress_complete_on_success",
            &self.progress_complete_on_success,
        );

        debug_struct.field("marker", &self.marker).finish()
    }
}

impl<CmdCtxTypesT> StatesDiscoverCmdBlock<CmdCtxTypesT, DiscoverForCurrent>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    /// Returns a block that discovers current states.
    pub fn current() -> Self {
        Self {
            #[cfg(feature = "output_progress")]
            progress_complete_on_success: false,
            marker: PhantomData,
        }
    }
}

impl<CmdCtxTypesT> StatesDiscoverCmdBlock<CmdCtxTypesT, DiscoverForGoal>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    /// Returns a block that discovers goal states.
    pub fn goal() -> Self {
        Self {
            #[cfg(feature = "output_progress")]
            progress_complete_on_success: false,
            marker: PhantomData,
        }
    }
}

impl<CmdCtxTypesT> StatesDiscoverCmdBlock<CmdCtxTypesT, DiscoverForCurrentAndGoal>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    /// Returns a block that discovers both current and goal states.
    pub fn current_and_goal() -> Self {
        Self {
            #[cfg(feature = "output_progress")]
            progress_complete_on_success: false,
            marker: PhantomData,
        }
    }
}

impl<CmdCtxTypesT, DiscoverFor> StatesDiscoverCmdBlock<CmdCtxTypesT, DiscoverFor>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
    DiscoverFor: Discover,
{
    /// Indicate that the progress tracker should be marked complete on success.
    ///
    /// This should be used only if this is the last `CmdBlock` in a
    /// `CmdExecution`.
    #[cfg(feature = "output_progress")]
    pub fn progress_complete_on_success(mut self) -> Self {
        self.progress_complete_on_success = true;
        self
    }

    async fn item_states_discover(
        #[cfg(feature = "output_progress")] progress_tx: &Sender<CmdProgressUpdate>,
        #[cfg(feature = "output_progress")] progress_complete_on_success: bool,
        params_specs: &peace_params::ParamsSpecs,
        resources: &Resources<SetUp>,
        outcomes_tx: &tokio::sync::mpsc::Sender<
            ItemDiscoverOutcome<<CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        >,
        item: &ItemBoxed<<CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
    ) {
        let item_id = item.id();
        let fn_ctx = FnCtx::new(
            item_id,
            #[cfg(feature = "output_progress")]
            ProgressSender::new(item_id, progress_tx),
        );

        let (states_current_result, states_goal_result) =
            DiscoverFor::discover(item, params_specs, resources, fn_ctx).await;

        // Send progress update.
        #[cfg(feature = "output_progress")]
        Self::discover_progress_update(
            progress_complete_on_success,
            states_current_result.as_ref(),
            states_goal_result.as_ref(),
            progress_tx,
            item_id,
        );

        let mut item_error = None;
        let state_current = if let Some(states_current_result) = states_current_result {
            match states_current_result {
                Ok(state_current_opt) => state_current_opt,
                Err(error) => {
                    item_error = Some(error);
                    None
                }
            }
        } else {
            None
        };

        let state_goal = if let Some(states_goal_result) = states_goal_result {
            match states_goal_result {
                Ok(state_goal_opt) => state_goal_opt,
                Err(error) => {
                    // It's probably more crucial to store the
                    // `states_current`
                    // error than the states goal error, if both err.
                    if item_error.is_none() {
                        item_error = Some(error);
                    }
                    None
                }
            }
        } else {
            None
        };

        if let Some(error) = item_error {
            outcomes_tx
                .send(ItemDiscoverOutcome::Fail {
                    item_id: item_id.clone(),
                    state_current,
                    state_goal,
                    error,
                })
                .await
                .expect("unreachable: `outcomes_rx` is in a sibling task.");
        } else {
            outcomes_tx
                .send(ItemDiscoverOutcome::Success {
                    item_id: item_id.clone(),
                    state_current,
                    state_goal,
                })
                .await
                .expect("unreachable: `outcomes_rx` is in a sibling task.");
        }
    }

    #[cfg(feature = "output_progress")]
    fn discover_progress_update(
        progress_complete_on_success: bool,
        states_current_result: Option<
            &Result<Option<BoxDtDisplay>, <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        >,
        states_goal_result: Option<
            &Result<Option<BoxDtDisplay>, <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        >,
        progress_tx: &Sender<CmdProgressUpdate>,
        item_id: &ItemId,
    ) {
        if let Some((progress_update, msg_update)) = DiscoverFor::progress_update(
            progress_complete_on_success,
            states_current_result,
            states_goal_result,
        ) {
            let _progress_send_unused = progress_tx.try_send(
                ProgressUpdateAndId {
                    item_id: item_id.clone(),
                    progress_update,
                    msg_update,
                }
                .into(),
            );
        }
    }
}

#[derive(Debug)]
pub enum ItemDiscoverOutcome<AppErrorT> {
    /// Discover succeeded.
    Success {
        item_id: ItemId,
        state_current: Option<BoxDtDisplay>,
        state_goal: Option<BoxDtDisplay>,
    },
    /// Discover failed.
    Fail {
        item_id: ItemId,
        state_current: Option<BoxDtDisplay>,
        state_goal: Option<BoxDtDisplay>,
        error: AppErrorT,
    },
}

#[async_trait(?Send)]
impl<CmdCtxTypesT> CmdBlock for StatesDiscoverCmdBlock<CmdCtxTypesT, DiscoverForCurrent>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    type CmdCtxTypes = CmdCtxTypesT;
    type InputT = ();
    type Outcome = States<Current>;

    #[cfg(feature = "output_progress")]
    fn cmd_block_item_interaction_type(&self) -> CmdBlockItemInteractionType {
        CmdBlockItemInteractionType::Read
    }

    fn input_fetch(&self, _resources: &mut Resources<SetUp>) -> Result<(), ResourceFetchError> {
        Ok(())
    }

    fn input_type_names(&self) -> Vec<String> {
        vec![]
    }

    async fn exec(
        &self,
        _input: Self::InputT,
        cmd_view: &mut SingleProfileSingleFlowView<'_, Self::CmdCtxTypes>,
        #[cfg(feature = "output_progress")] progress_tx: &Sender<CmdProgressUpdate>,
    ) -> Result<
        CmdBlockOutcome<Self::Outcome, <Self::CmdCtxTypes as CmdCtxTypesConstrained>::AppError>,
        <Self::CmdCtxTypes as CmdCtxTypesConstrained>::AppError,
    > {
        let SingleProfileSingleFlowView {
            interruptibility_state,
            flow,
            params_specs,
            resources,
            ..
        } = cmd_view;

        let (outcomes_tx, outcomes_rx) = mpsc::channel::<
            ItemDiscoverOutcome<<CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        >(flow.graph().node_count());

        let (stream_outcome, outcome_collate) = {
            let states_current_mut = StatesMut::<Current>::with_capacity(flow.graph().node_count());

            let item_states_discover_task = async move {
                let stream_outcome = flow
                    .graph()
                    .for_each_concurrent_with(
                        BUFFERED_FUTURES_MAX,
                        StreamOpts::new()
                            .interruptibility_state(interruptibility_state.reborrow())
                            .interrupted_next_item_include(false),
                        |item| {
                            Self::item_states_discover(
                                #[cfg(feature = "output_progress")]
                                progress_tx,
                                #[cfg(feature = "output_progress")]
                                self.progress_complete_on_success,
                                params_specs,
                                resources,
                                &outcomes_tx,
                                item,
                            )
                        },
                    )
                    .await;

                drop(outcomes_tx);

                stream_outcome
            };

            let outcome_collate_task = Self::outcome_collate_task(outcomes_rx, states_current_mut);

            join!(item_states_discover_task, outcome_collate_task)
        };

        outcome_collate.map(|(states_current, errors)| {
            let (stream_outcome, ()) = stream_outcome.replace(states_current);
            CmdBlockOutcome::ItemWise {
                stream_outcome,
                errors,
            }
        })
    }
}

impl<CmdCtxTypesT> StatesDiscoverCmdBlock<CmdCtxTypesT, DiscoverForCurrent>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    async fn outcome_collate_task(
        mut outcomes_rx: Receiver<
            ItemDiscoverOutcome<<CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        >,
        mut states_current_mut: StatesMut<Current>,
    ) -> Result<
        (
            States<Current>,
            IndexMap<ItemId, <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        ),
        <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError,
    > {
        let mut errors = IndexMap::new();
        while let Some(item_outcome) = outcomes_rx.recv().await {
            Self::outcome_collate(&mut states_current_mut, &mut errors, item_outcome)?;
        }

        let states_current = States::<Current>::from(states_current_mut);

        Ok((states_current, errors))
    }

    fn outcome_collate(
        states_current_mut: &mut StatesMut<Current>,
        errors: &mut IndexMap<ItemId, <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        outcome_partial: ItemDiscoverOutcome<<CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
    ) -> Result<(), <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError> {
        match outcome_partial {
            ItemDiscoverOutcome::Success {
                item_id,
                state_current,
                state_goal: _,
            } => {
                if let Some(state_current) = state_current {
                    states_current_mut.insert_raw(item_id.clone(), state_current);
                }
            }
            ItemDiscoverOutcome::Fail {
                item_id,
                state_current,
                state_goal: _,
                error,
            } => {
                errors.insert(item_id.clone(), error);

                if let Some(state_current) = state_current {
                    states_current_mut.insert_raw(item_id.clone(), state_current);
                }
            }
        }

        Ok(())
    }
}

#[async_trait(?Send)]
impl<CmdCtxTypesT> CmdBlock for StatesDiscoverCmdBlock<CmdCtxTypesT, DiscoverForGoal>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    type CmdCtxTypes = CmdCtxTypesT;
    type InputT = ();
    type Outcome = States<Goal>;

    #[cfg(feature = "output_progress")]
    fn cmd_block_item_interaction_type(&self) -> CmdBlockItemInteractionType {
        CmdBlockItemInteractionType::Read
    }

    fn input_fetch(&self, _resources: &mut Resources<SetUp>) -> Result<(), ResourceFetchError> {
        Ok(())
    }

    fn input_type_names(&self) -> Vec<String> {
        vec![]
    }

    async fn exec(
        &self,
        _input: Self::InputT,
        cmd_view: &mut SingleProfileSingleFlowView<'_, Self::CmdCtxTypes>,
        #[cfg(feature = "output_progress")] progress_tx: &Sender<CmdProgressUpdate>,
    ) -> Result<
        CmdBlockOutcome<Self::Outcome, <Self::CmdCtxTypes as CmdCtxTypesConstrained>::AppError>,
        <Self::CmdCtxTypes as CmdCtxTypesConstrained>::AppError,
    > {
        let SingleProfileSingleFlowView {
            interruptibility_state,
            flow,
            params_specs,
            resources,
            ..
        } = cmd_view;

        let (outcomes_tx, outcomes_rx) = mpsc::channel::<
            ItemDiscoverOutcome<<CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        >(flow.graph().node_count());

        let (stream_outcome, outcome_collate) = {
            let states_goal_mut = StatesMut::<Goal>::with_capacity(flow.graph().node_count());

            let item_states_discover_task = async move {
                let stream_outcome = flow
                    .graph()
                    .for_each_concurrent_with(
                        BUFFERED_FUTURES_MAX,
                        StreamOpts::new()
                            .interruptibility_state(interruptibility_state.reborrow())
                            .interrupted_next_item_include(false),
                        |item| {
                            Self::item_states_discover(
                                #[cfg(feature = "output_progress")]
                                progress_tx,
                                #[cfg(feature = "output_progress")]
                                self.progress_complete_on_success,
                                params_specs,
                                resources,
                                &outcomes_tx,
                                item,
                            )
                        },
                    )
                    .await;

                drop(outcomes_tx);

                stream_outcome
            };

            let outcome_collate_task = Self::outcome_collate_task(outcomes_rx, states_goal_mut);

            join!(item_states_discover_task, outcome_collate_task)
        };

        outcome_collate.map(|(states_goal, errors)| {
            let (stream_outcome, ()) = stream_outcome.replace(states_goal);
            CmdBlockOutcome::ItemWise {
                stream_outcome,
                errors,
            }
        })
    }
}

impl<CmdCtxTypesT> StatesDiscoverCmdBlock<CmdCtxTypesT, DiscoverForGoal>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    async fn outcome_collate_task(
        mut outcomes_rx: Receiver<
            ItemDiscoverOutcome<<CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        >,
        mut states_goal_mut: StatesMut<Goal>,
    ) -> Result<
        (
            States<Goal>,
            IndexMap<ItemId, <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        ),
        <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError,
    > {
        let mut errors = IndexMap::new();
        while let Some(item_outcome) = outcomes_rx.recv().await {
            Self::outcome_collate(&mut states_goal_mut, &mut errors, item_outcome)?;
        }

        let states_goal = States::<Goal>::from(states_goal_mut);

        Ok((states_goal, errors))
    }

    fn outcome_collate(
        states_goal_mut: &mut StatesMut<Goal>,
        errors: &mut IndexMap<ItemId, <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        outcome_partial: ItemDiscoverOutcome<<CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
    ) -> Result<(), <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError> {
        match outcome_partial {
            ItemDiscoverOutcome::Success {
                item_id,
                state_current: _,
                state_goal,
            } => {
                if let Some(state_goal) = state_goal {
                    states_goal_mut.insert_raw(item_id, state_goal);
                }
            }
            ItemDiscoverOutcome::Fail {
                item_id,
                state_current: _,
                state_goal,
                error,
            } => {
                errors.insert(item_id.clone(), error);

                if let Some(state_goal) = state_goal {
                    states_goal_mut.insert_raw(item_id, state_goal);
                }
            }
        }

        Ok(())
    }
}

#[async_trait(?Send)]
impl<CmdCtxTypesT> CmdBlock for StatesDiscoverCmdBlock<CmdCtxTypesT, DiscoverForCurrentAndGoal>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    type CmdCtxTypes = CmdCtxTypesT;
    type InputT = ();
    type Outcome = (States<Current>, States<Goal>);

    #[cfg(feature = "output_progress")]
    fn cmd_block_item_interaction_type(&self) -> CmdBlockItemInteractionType {
        CmdBlockItemInteractionType::Read
    }

    fn input_fetch(&self, _resources: &mut Resources<SetUp>) -> Result<(), ResourceFetchError> {
        Ok(())
    }

    fn input_type_names(&self) -> Vec<String> {
        vec![]
    }

    fn outcome_insert(&self, resources: &mut Resources<SetUp>, outcome: Self::Outcome) {
        let (states_current, states_goal) = outcome;
        resources.insert(states_current);
        resources.insert(states_goal);
    }

    fn outcome_type_names(&self) -> Vec<String> {
        vec![
            tynm::type_name::<StatesCurrent>(),
            tynm::type_name::<StatesGoal>(),
        ]
    }

    async fn exec(
        &self,
        _input: Self::InputT,
        cmd_view: &mut SingleProfileSingleFlowView<'_, Self::CmdCtxTypes>,
        #[cfg(feature = "output_progress")] progress_tx: &Sender<CmdProgressUpdate>,
    ) -> Result<
        CmdBlockOutcome<Self::Outcome, <Self::CmdCtxTypes as CmdCtxTypesConstrained>::AppError>,
        <Self::CmdCtxTypes as CmdCtxTypesConstrained>::AppError,
    > {
        let SingleProfileSingleFlowView {
            interruptibility_state,
            flow,
            params_specs,
            resources,
            ..
        } = cmd_view;

        let (outcomes_tx, outcomes_rx) = mpsc::channel::<
            ItemDiscoverOutcome<<CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        >(flow.graph().node_count());

        let (stream_outcome, outcome_collate) = {
            let states_current_mut = StatesMut::<Current>::with_capacity(flow.graph().node_count());
            let states_goal_mut = StatesMut::<Goal>::with_capacity(flow.graph().node_count());

            let item_states_discover_task = async move {
                let stream_outcome = flow
                    .graph()
                    .for_each_concurrent_with(
                        BUFFERED_FUTURES_MAX,
                        StreamOpts::new()
                            .interruptibility_state(interruptibility_state.reborrow())
                            .interrupted_next_item_include(false),
                        |item| {
                            Self::item_states_discover(
                                #[cfg(feature = "output_progress")]
                                progress_tx,
                                #[cfg(feature = "output_progress")]
                                self.progress_complete_on_success,
                                params_specs,
                                resources,
                                &outcomes_tx,
                                item,
                            )
                        },
                    )
                    .await;

                drop(outcomes_tx);

                stream_outcome
            };

            let outcome_collate_task =
                Self::outcome_collate_task(outcomes_rx, states_current_mut, states_goal_mut);

            join!(item_states_discover_task, outcome_collate_task)
        };

        outcome_collate.map(|(states_current, states_goal, errors)| {
            let (stream_outcome, ()) = stream_outcome.replace((states_current, states_goal));
            CmdBlockOutcome::ItemWise {
                stream_outcome,
                errors,
            }
        })
    }
}

impl<CmdCtxTypesT> StatesDiscoverCmdBlock<CmdCtxTypesT, DiscoverForCurrentAndGoal>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    async fn outcome_collate_task(
        mut outcomes_rx: Receiver<
            ItemDiscoverOutcome<<CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        >,
        mut states_current_mut: StatesMut<Current>,
        mut states_goal_mut: StatesMut<Goal>,
    ) -> Result<
        (
            States<Current>,
            States<Goal>,
            IndexMap<ItemId, <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        ),
        <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError,
    > {
        let mut errors = IndexMap::new();
        while let Some(item_outcome) = outcomes_rx.recv().await {
            Self::outcome_collate(
                &mut states_current_mut,
                &mut states_goal_mut,
                &mut errors,
                item_outcome,
            )?;
        }

        let states_current = States::<Current>::from(states_current_mut);
        let states_goal = States::<Goal>::from(states_goal_mut);

        Ok((states_current, states_goal, errors))
    }

    fn outcome_collate(
        states_current_mut: &mut StatesMut<Current>,
        states_goal_mut: &mut StatesMut<Goal>,
        errors: &mut IndexMap<ItemId, <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        outcome_partial: ItemDiscoverOutcome<<CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
    ) -> Result<(), <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError> {
        match outcome_partial {
            ItemDiscoverOutcome::Success {
                item_id,
                state_current,
                state_goal,
            } => {
                if let Some(state_current) = state_current {
                    states_current_mut.insert_raw(item_id.clone(), state_current);
                }
                if let Some(state_goal) = state_goal {
                    states_goal_mut.insert_raw(item_id, state_goal);
                }
            }
            ItemDiscoverOutcome::Fail {
                item_id,
                state_current,
                state_goal,
                error,
            } => {
                errors.insert(item_id.clone(), error);

                if let Some(state_current) = state_current {
                    states_current_mut.insert_raw(item_id.clone(), state_current);
                }
                if let Some(state_goal) = state_goal {
                    states_goal_mut.insert_raw(item_id, state_goal);
                }
            }
        }

        Ok(())
    }
}

/// Behaviour for each discover variant.
#[async_trait::async_trait(?Send)]
pub trait Discover {
    async fn discover<AppErrorT>(
        item: &ItemBoxed<AppErrorT>,
        params_specs: &peace_params::ParamsSpecs,
        resources: &Resources<SetUp>,
        fn_ctx: FnCtx<'_>,
    ) -> (
        Option<Result<Option<BoxDtDisplay>, AppErrorT>>,
        Option<Result<Option<BoxDtDisplay>, AppErrorT>>,
    )
    where
        AppErrorT: peace_value_traits::AppError + From<peace_rt_model::Error>;

    #[cfg(feature = "output_progress")]
    fn progress_update<AppErrorT>(
        progress_complete_on_success: bool,
        states_current_result: Option<&Result<Option<BoxDtDisplay>, AppErrorT>>,
        states_goal_result: Option<&Result<Option<BoxDtDisplay>, AppErrorT>>,
    ) -> Option<(ProgressUpdate, ProgressMsgUpdate)>
    where
        AppErrorT: peace_value_traits::AppError + From<peace_rt_model::Error>;
}

#[async_trait::async_trait(?Send)]
impl Discover for DiscoverForCurrent {
    async fn discover<AppErrorT>(
        item: &ItemBoxed<AppErrorT>,
        params_specs: &peace_params::ParamsSpecs,
        resources: &Resources<SetUp>,
        fn_ctx: FnCtx<'_>,
    ) -> (
        Option<Result<Option<BoxDtDisplay>, AppErrorT>>,
        Option<Result<Option<BoxDtDisplay>, AppErrorT>>,
    )
    where
        AppErrorT: peace_value_traits::AppError + From<peace_rt_model::Error>,
    {
        let states_current_result = item
            .state_current_try_exec(params_specs, resources, fn_ctx)
            .await;

        (Some(states_current_result), None)
    }

    #[cfg(feature = "output_progress")]
    fn progress_update<AppErrorT>(
        progress_complete_on_success: bool,
        states_current_result: Option<&Result<Option<BoxDtDisplay>, AppErrorT>>,
        _states_goal_result: Option<&Result<Option<BoxDtDisplay>, AppErrorT>>,
    ) -> Option<(ProgressUpdate, ProgressMsgUpdate)>
    where
        AppErrorT: peace_value_traits::AppError + From<peace_rt_model::Error>,
    {
        states_current_result.map(|states_current_result| match states_current_result {
            Ok(_) => {
                let progress_update = if progress_complete_on_success {
                    ProgressUpdate::Complete(ProgressComplete::Success)
                } else {
                    ProgressUpdate::Delta(ProgressDelta::Tick)
                };

                (progress_update, ProgressMsgUpdate::Clear)
            }
            Err(error) => (
                ProgressUpdate::Complete(ProgressComplete::Fail),
                ProgressMsgUpdate::Set(format!("{error}")),
            ),
        })
    }
}

#[async_trait::async_trait(?Send)]
impl Discover for DiscoverForGoal {
    async fn discover<AppErrorT>(
        item: &ItemBoxed<AppErrorT>,
        params_specs: &peace_params::ParamsSpecs,
        resources: &Resources<SetUp>,
        fn_ctx: FnCtx<'_>,
    ) -> (
        Option<Result<Option<BoxDtDisplay>, AppErrorT>>,
        Option<Result<Option<BoxDtDisplay>, AppErrorT>>,
    )
    where
        AppErrorT: peace_value_traits::AppError + From<peace_rt_model::Error>,
    {
        let states_goal_result = item
            .state_goal_try_exec(params_specs, resources, fn_ctx)
            .await;

        (None, Some(states_goal_result))
    }

    #[cfg(feature = "output_progress")]
    fn progress_update<AppErrorT>(
        progress_complete_on_success: bool,
        _states_current_result: Option<&Result<Option<BoxDtDisplay>, AppErrorT>>,
        states_goal_result: Option<&Result<Option<BoxDtDisplay>, AppErrorT>>,
    ) -> Option<(ProgressUpdate, ProgressMsgUpdate)>
    where
        AppErrorT: peace_value_traits::AppError + From<peace_rt_model::Error>,
    {
        states_goal_result.map(|states_goal_result| match states_goal_result {
            Ok(_) => {
                let progress_update = if progress_complete_on_success {
                    ProgressUpdate::Complete(ProgressComplete::Success)
                } else {
                    ProgressUpdate::Delta(ProgressDelta::Tick)
                };

                (progress_update, ProgressMsgUpdate::Clear)
            }
            Err(error) => (
                ProgressUpdate::Complete(ProgressComplete::Fail),
                ProgressMsgUpdate::Set(format!("{error}")),
            ),
        })
    }
}

#[async_trait::async_trait(?Send)]
impl Discover for DiscoverForCurrentAndGoal {
    async fn discover<AppErrorT>(
        item: &ItemBoxed<AppErrorT>,
        params_specs: &peace_params::ParamsSpecs,
        resources: &Resources<SetUp>,
        fn_ctx: FnCtx<'_>,
    ) -> (
        Option<Result<Option<BoxDtDisplay>, AppErrorT>>,
        Option<Result<Option<BoxDtDisplay>, AppErrorT>>,
    )
    where
        AppErrorT: peace_value_traits::AppError + From<peace_rt_model::Error>,
    {
        let states_current_result = item
            .state_current_try_exec(params_specs, resources, fn_ctx)
            .await;
        let states_goal_result = item
            .state_goal_try_exec(params_specs, resources, fn_ctx)
            .await;

        (Some(states_current_result), Some(states_goal_result))
    }

    #[cfg(feature = "output_progress")]
    fn progress_update<AppErrorT>(
        progress_complete_on_success: bool,
        states_current_result: Option<&Result<Option<BoxDtDisplay>, AppErrorT>>,
        states_goal_result: Option<&Result<Option<BoxDtDisplay>, AppErrorT>>,
    ) -> Option<(ProgressUpdate, ProgressMsgUpdate)>
    where
        AppErrorT: peace_value_traits::AppError + From<peace_rt_model::Error>,
    {
        states_current_result
            .zip(states_goal_result)
            .map(
                |states_current_and_states_goal_result| match states_current_and_states_goal_result
                {
                    (Ok(_), Ok(_)) => {
                        let progress_update = if progress_complete_on_success {
                            ProgressUpdate::Complete(ProgressComplete::Success)
                        } else {
                            ProgressUpdate::Delta(ProgressDelta::Tick)
                        };

                        (progress_update, ProgressMsgUpdate::Clear)
                    }
                    (Ok(_), Err(error)) | (Err(error), Ok(_)) => (
                        ProgressUpdate::Complete(ProgressComplete::Fail),
                        ProgressMsgUpdate::Set(format!("{error}")),
                    ),
                    (Err(error_current), Err(error_goal)) => (
                        ProgressUpdate::Complete(ProgressComplete::Fail),
                        ProgressMsgUpdate::Set(format!("{error_current}, {error_goal}")),
                    ),
                },
            )
    }
}
