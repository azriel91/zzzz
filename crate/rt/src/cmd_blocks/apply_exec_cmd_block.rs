use std::{fmt::Debug, marker::PhantomData};

use fn_graph::{StreamOpts, StreamOutcome};
use futures::join;
use peace_cfg::{ApplyCheck, FnCtx};
use peace_cmd::{ctx::CmdCtxTypesConstrained, scopes::SingleProfileSingleFlowView};
use peace_cmd_model::CmdBlockOutcome;
use peace_cmd_rt::{async_trait, CmdBlock};
use peace_item_model::ItemId;
use peace_params::ParamsSpecs;
use peace_resource_rt::{
    internal::StatesMut,
    resources::ts::SetUp,
    states::{
        ts::{Clean, Cleaned, CleanedDry, Ensured, EnsuredDry, Goal},
        States, StatesCurrent, StatesPrevious,
    },
    ResourceFetchError, Resources,
};
use peace_rt_model::{
    outcomes::{ItemApplyBoxed, ItemApplyPartialBoxed},
    ItemBoxed, ItemRt,
};
use tokio::sync::mpsc::{self, Receiver};

use peace_rt_model_core::IndexMap;
use tokio::sync::mpsc::Sender;

use crate::BUFFERED_FUTURES_MAX;

cfg_if::cfg_if! {
    if #[cfg(feature = "output_progress")] {
        use std::error::Error;

        use peace_progress_model::{
            CmdBlockItemInteractionType,
            CmdProgressUpdate,
            ProgressComplete,
            ProgressMsgUpdate,
            ProgressUpdate,
            ProgressUpdateAndId,
            ProgressSender,
        };
    }
}

/// Stops a `CmdExecution` if stored states and discovered states are not in
/// sync.
pub struct ApplyExecCmdBlock<CmdCtxTypesT, StatesTs>(PhantomData<(CmdCtxTypesT, StatesTs)>);

impl<CmdCtxTypesT, StatesTs> Debug for ApplyExecCmdBlock<CmdCtxTypesT, StatesTs> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ApplyExecCmdBlock").field(&self.0).finish()
    }
}

impl<CmdCtxTypesT, StatesTs> ApplyExecCmdBlock<CmdCtxTypesT, StatesTs> {
    /// Returns an `ApplyExecCmdBlock`.
    ///
    /// This is a generic constructor where `StatesTs` determines whether the
    /// goal state or clean state is the target state.
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<CmdCtxTypesT, StatesTs> Default for ApplyExecCmdBlock<CmdCtxTypesT, StatesTs> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<CmdCtxTypesT> ApplyExecCmdBlock<CmdCtxTypesT, Ensured>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    /// Returns an `ApplyExecCmdBlock` with the goal state as the target state.
    pub fn ensure() -> Self {
        Self(PhantomData)
    }
}

impl<CmdCtxTypesT> ApplyExecCmdBlock<CmdCtxTypesT, EnsuredDry>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    /// Returns an `ApplyExecCmdBlock` with the goal state as the target state.
    pub fn ensure_dry() -> Self {
        Self(PhantomData)
    }
}

impl<CmdCtxTypesT> ApplyExecCmdBlock<CmdCtxTypesT, Cleaned>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    /// Returns an `ApplyExecCmdBlock` with the clean state as the target state.
    pub fn clean() -> Self {
        Self(PhantomData)
    }
}

impl<CmdCtxTypesT> ApplyExecCmdBlock<CmdCtxTypesT, CleanedDry>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    /// Returns an `ApplyExecCmdBlock` with the clean state as the target state.
    pub fn clean_dry() -> Self {
        Self(PhantomData)
    }
}

impl<CmdCtxTypesT, StatesTs> ApplyExecCmdBlock<CmdCtxTypesT, StatesTs>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
    StatesTs: StatesTsApplyExt + Debug + Send,
{
    ///
    /// # Implementation Note
    ///
    /// Tried passing through the function to execute instead of a `dry_run`
    /// parameter, but couldn't convince the compiler that the lifetimes match
    /// up:
    ///
    /// ```rust,ignore
    /// async fn item_apply_exec<F, Fut>(
    ///     resources: &Resources<SetUp>,
    ///     outcomes_tx: &Sender<ItemApplyOutcome<<CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>>,
    ///     item: FnRef<'_, ItemBoxed<<CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>>,
    ///     f: F,
    /// ) -> bool
    /// where
    ///     F: (Fn(&dyn ItemRt<<CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>, fn_ctx: OpCtx<'_>, &Resources<SetUp>, &mut ItemApplyBoxed) -> Fut) + Copy,
    ///     Fut: Future<Output = Result<(), <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>>,
    /// ```
    async fn item_apply_exec(
        item_apply_exec_ctx: ItemApplyExecCtx<
            '_,
            <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError,
        >,
        item: &ItemBoxed<<CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
    ) -> Result<(), ()> {
        let ItemApplyExecCtx {
            params_specs,
            resources,
            apply_for_internal,
            #[cfg(feature = "output_progress")]
            progress_tx,
            outcomes_tx,
        } = item_apply_exec_ctx;

        let item_id = item.id();

        // Indicate this item is running, so that an `Interrupt` message from
        // `CmdExecution` does not cause it to be rendered as `Interrupted`.
        #[cfg(feature = "output_progress")]
        let _progress_send_unused = progress_tx.try_send(
            ProgressUpdateAndId {
                item_id: item_id.clone(),
                progress_update: ProgressUpdate::Queued,
                msg_update: ProgressMsgUpdate::NoChange,
            }
            .into(),
        );

        let apply_fn = if StatesTs::dry_run() {
            ItemRt::apply_exec_dry
        } else {
            ItemRt::apply_exec
        };

        let fn_ctx = FnCtx::new(
            item_id,
            #[cfg(feature = "output_progress")]
            ProgressSender::new(item_id, progress_tx),
        );
        let item_apply = match apply_for_internal {
            ApplyForInternal::Ensure => {
                ItemRt::ensure_prepare(&**item, params_specs, resources, fn_ctx).await
            }
            ApplyForInternal::Clean { states_current } => {
                ItemRt::clean_prepare(&**item, states_current, params_specs, resources).await
            }
        };

        match item_apply {
            Ok(mut item_apply) => {
                match item_apply.apply_check() {
                    #[cfg(not(feature = "output_progress"))]
                    ApplyCheck::ExecRequired => {}
                    #[cfg(feature = "output_progress")]
                    ApplyCheck::ExecRequired { progress_limit } => {
                        // Update `OutputWrite`s with progress limit.
                        let _progress_send_unused = progress_tx.try_send(
                            ProgressUpdateAndId {
                                item_id: item_id.clone(),
                                progress_update: ProgressUpdate::Limit(progress_limit),
                                msg_update: ProgressMsgUpdate::Set(String::from("in progress")),
                            }
                            .into(),
                        );
                    }
                    ApplyCheck::ExecNotRequired => {
                        #[cfg(feature = "output_progress")]
                        let _progress_send_unused = progress_tx.try_send(
                            ProgressUpdateAndId {
                                item_id: item_id.clone(),
                                progress_update: ProgressUpdate::Complete(
                                    ProgressComplete::Success,
                                ),
                                msg_update: ProgressMsgUpdate::Set(String::from("nothing to do!")),
                            }
                            .into(),
                        );

                        // TODO: write test for this case
                        // In case of an interrupt or power failure, we may not have written states
                        // to disk.
                        outcomes_tx
                            .send(ItemApplyOutcome::Success {
                                item_id: item.id().clone(),
                                item_apply,
                            })
                            .await
                            .expect("unreachable: `outcomes_rx` is in a sibling task.");

                        // short-circuit
                        return Ok(());
                    }
                }
                match apply_fn(&**item, params_specs, resources, fn_ctx, &mut item_apply).await {
                    Ok(()) => {
                        // apply succeeded

                        #[cfg(feature = "output_progress")]
                        let _progress_send_unused = progress_tx.try_send(
                            ProgressUpdateAndId {
                                item_id: item_id.clone(),
                                progress_update: ProgressUpdate::Complete(
                                    ProgressComplete::Success,
                                ),
                                msg_update: ProgressMsgUpdate::Set(String::from("done!")),
                            }
                            .into(),
                        );

                        outcomes_tx
                            .send(ItemApplyOutcome::Success {
                                item_id: item.id().clone(),
                                item_apply,
                            })
                            .await
                            .expect("unreachable: `outcomes_rx` is in a sibling task.");

                        Ok(())
                    }
                    Err(error) => {
                        // apply failed

                        #[cfg(feature = "output_progress")]
                        let _progress_send_unused = progress_tx.try_send(
                            ProgressUpdateAndId {
                                item_id: item_id.clone(),
                                progress_update: ProgressUpdate::Complete(ProgressComplete::Fail),
                                msg_update: ProgressMsgUpdate::Set(
                                    error
                                        .source()
                                        .map(|source| format!("{source}"))
                                        .unwrap_or_else(|| format!("{error}")),
                                ),
                            }
                            .into(),
                        );

                        outcomes_tx
                            .send(ItemApplyOutcome::Fail {
                                item_id: item.id().clone(),
                                item_apply,
                                error,
                            })
                            .await
                            .expect("unreachable: `outcomes_rx` is in a sibling task.");

                        // we should stop processing.
                        Err(())
                    }
                }
            }
            Err((error, item_apply_partial)) => {
                #[cfg(feature = "output_progress")]
                let _progress_send_unused = progress_tx.try_send(
                    ProgressUpdateAndId {
                        item_id: item.id().clone(),
                        progress_update: ProgressUpdate::Complete(ProgressComplete::Fail),
                        msg_update: ProgressMsgUpdate::Set(
                            error
                                .source()
                                .map(|source| format!("{source}"))
                                .unwrap_or_else(|| format!("{error}")),
                        ),
                    }
                    .into(),
                );

                outcomes_tx
                    .send(ItemApplyOutcome::PrepareFail {
                        item_id: item.id().clone(),
                        item_apply_partial,
                        error,
                    })
                    .await
                    .expect("unreachable: `outcomes_rx` is in a sibling task.");

                Err(())
            }
        }
    }

    async fn outcome_collate_task(
        mut outcomes_rx: Receiver<
            ItemApplyOutcome<<CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        >,
        mut states_applied_mut: StatesMut<StatesTs>,
        mut states_target_mut: StatesMut<StatesTs::TsTarget>,
    ) -> Result<
        (
            States<StatesTs>,
            States<StatesTs::TsTarget>,
            IndexMap<ItemId, <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        ),
        <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError,
    > {
        let mut errors = IndexMap::new();
        while let Some(item_outcome) = outcomes_rx.recv().await {
            Self::outcome_collate(
                &mut states_applied_mut,
                &mut states_target_mut,
                &mut errors,
                item_outcome,
            )?;
        }

        let states_applied = States::<StatesTs>::from(states_applied_mut);
        let states_target = States::<StatesTs::TsTarget>::from(states_target_mut);

        Ok((states_applied, states_target, errors))
    }

    fn outcome_collate(
        states_applied_mut: &mut StatesMut<StatesTs>,
        states_target_mut: &mut StatesMut<StatesTs::TsTarget>,
        errors: &mut IndexMap<ItemId, <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        outcome_partial: ItemApplyOutcome<<CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
    ) -> Result<(), <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError> {
        let apply_for = StatesTs::apply_for();

        match outcome_partial {
            ItemApplyOutcome::PrepareFail {
                item_id,
                item_apply_partial,
                error,
            } => {
                errors.insert(item_id.clone(), error);

                // Save `state_target` (which is `state_goal`) if we are not cleaning
                // up.
                match apply_for {
                    ApplyFor::Ensure => {
                        if let Some(state_target) = item_apply_partial.state_target() {
                            states_target_mut.insert_raw(item_id, state_target);
                        }
                    }
                    ApplyFor::Clean => {}
                }
            }
            ItemApplyOutcome::Success {
                item_id,
                item_apply,
            } => {
                if let Some(state_applied) = item_apply.state_applied() {
                    states_applied_mut.insert_raw(item_id.clone(), state_applied);
                } else {
                    // Item was already in the goal state.
                    // No change to current state.
                }

                // Save `state_target` (which is state_target) if we are not cleaning
                // up.
                match apply_for {
                    ApplyFor::Ensure => {
                        let state_target = item_apply.state_target();
                        states_target_mut.insert_raw(item_id, state_target);
                    }
                    ApplyFor::Clean => {}
                }
            }
            ItemApplyOutcome::Fail {
                item_id,
                item_apply,
                error,
            } => {
                errors.insert(item_id.clone(), error);
                if let Some(state_applied) = item_apply.state_applied() {
                    states_applied_mut.insert_raw(item_id.clone(), state_applied);
                }

                // Save `state_target` (which is `state_goal`) if we are not cleaning
                // up.
                match apply_for {
                    ApplyFor::Ensure => {
                        let state_target = item_apply.state_target();
                        states_target_mut.insert_raw(item_id, state_target);
                    }
                    ApplyFor::Clean => {}
                }
            }
        }

        Ok(())
    }
}

#[async_trait(?Send)]
impl<CmdCtxTypesT, StatesTs> CmdBlock for ApplyExecCmdBlock<CmdCtxTypesT, StatesTs>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
    StatesTs: StatesTsApplyExt + Debug + Send + Sync + 'static,
{
    type CmdCtxTypes = CmdCtxTypesT;
    type InputT = (StatesCurrent, States<StatesTs::TsTarget>);
    type Outcome = (StatesPrevious, States<StatesTs>, States<StatesTs::TsTarget>);

    #[cfg(feature = "output_progress")]
    fn cmd_block_item_interaction_type(&self) -> CmdBlockItemInteractionType {
        CmdBlockItemInteractionType::Write
    }

    fn input_fetch(
        &self,
        resources: &mut Resources<SetUp>,
    ) -> Result<Self::InputT, ResourceFetchError> {
        let states_current = resources.try_remove::<StatesCurrent>()?;

        let states_target = resources.try_remove::<States<StatesTs::TsTarget>>()?;

        Ok((states_current, states_target))
    }

    fn input_type_names(&self) -> Vec<String> {
        vec![
            tynm::type_name::<StatesCurrent>(),
            tynm::type_name::<States<StatesTs::TsTarget>>(),
        ]
    }

    fn outcome_insert(&self, resources: &mut Resources<SetUp>, outcome: Self::Outcome) {
        let (states_previous, states_applied, states_target) = outcome;
        resources.insert(states_previous);
        resources.insert(states_applied);
        resources.insert(states_target);
    }

    fn outcome_type_names(&self) -> Vec<String> {
        vec![
            tynm::type_name::<StatesPrevious>(),
            tynm::type_name::<States<StatesTs>>(),
            tynm::type_name::<States<StatesTs::TsTarget>>(),
        ]
    }

    async fn exec(
        &self,
        input: Self::InputT,
        cmd_view: &mut SingleProfileSingleFlowView<'_, Self::CmdCtxTypes>,
        #[cfg(feature = "output_progress")] progress_tx: &Sender<CmdProgressUpdate>,
    ) -> Result<
        CmdBlockOutcome<Self::Outcome, <Self::CmdCtxTypes as CmdCtxTypesConstrained>::AppError>,
        <Self::CmdCtxTypes as CmdCtxTypesConstrained>::AppError,
    > {
        let (states_current, states_target) = input;
        let (states_previous, states_applied_mut, states_target_mut) = {
            let states_previous = StatesPrevious::from(states_current.clone());
            // `Ensured`, `EnsuredDry`, `Cleaned`, `CleanedDry` states start as the current
            // state, and are altered.
            let states_applied_mut =
                StatesMut::<StatesTs>::from(states_current.clone().into_inner());
            let states_target_mut =
                StatesMut::<StatesTs::TsTarget>::from(states_target.clone().into_inner());

            (states_previous, states_applied_mut, states_target_mut)
        };

        let SingleProfileSingleFlowView {
            interruptibility_state,
            flow,
            params_specs,
            resources,
            ..
        } = cmd_view;

        let item_graph = flow.graph();
        let resources_ref = &*resources;
        let apply_for = StatesTs::apply_for();
        let apply_for_internal = match apply_for {
            ApplyFor::Ensure => ApplyForInternal::Ensure,
            ApplyFor::Clean => ApplyForInternal::Clean { states_current },
        };

        let (outcomes_tx, outcomes_rx) = mpsc::channel::<
            ItemApplyOutcome<<CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        >(item_graph.node_count());

        let stream_opts = {
            let stream_opts = StreamOpts::new()
                .interruptibility_state(interruptibility_state.reborrow())
                .interrupted_next_item_include(false);
            match apply_for {
                ApplyFor::Ensure => stream_opts,
                ApplyFor::Clean => stream_opts.rev(),
            }
        };

        let (stream_outcome_result, outcome_collate) = {
            let item_apply_exec_task = async move {
                let stream_outcome = item_graph
                    .try_for_each_concurrent_with(BUFFERED_FUTURES_MAX, stream_opts, |item| {
                        let item_apply_exec_ctx = ItemApplyExecCtx {
                            params_specs,
                            resources: resources_ref,
                            apply_for_internal: &apply_for_internal,
                            #[cfg(feature = "output_progress")]
                            progress_tx,
                            outcomes_tx: &outcomes_tx,
                        };
                        Self::item_apply_exec(item_apply_exec_ctx, item)
                    })
                    .await;

                drop(outcomes_tx);

                stream_outcome
            };
            let outcome_collate_task =
                Self::outcome_collate_task(outcomes_rx, states_applied_mut, states_target_mut);

            join!(item_apply_exec_task, outcome_collate_task)
        };
        let (states_applied, states_target, errors) = outcome_collate?;

        let stream_outcome = {
            let (Ok(stream_outcome) | Err((stream_outcome, ()))) = stream_outcome_result.map_err(
                |(stream_outcome, _vec_unit): (StreamOutcome<()>, Vec<()>)| (stream_outcome, ()),
            );

            stream_outcome.map(|()| (states_previous, states_applied, states_target))
        };

        Ok(CmdBlockOutcome::ItemWise {
            stream_outcome,
            errors,
        })
    }
}

/// Whether the `ApplyCmd` is for `Ensure` or `Clean`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplyFor {
    /// The apply target state is `state_goal`.
    Ensure,
    /// The apply target state is `state_clean`.
    Clean,
}

/// Whether the `ApplyCmd` is for `Ensure` or `Clean`.
#[derive(Debug)]
enum ApplyForInternal {
    Ensure,
    Clean { states_current: StatesCurrent },
}

struct ItemApplyExecCtx<'f, E> {
    /// Map of item ID to its params' specs.
    params_specs: &'f ParamsSpecs,
    /// Map of all types at runtime.
    resources: &'f Resources<SetUp>,
    /// Whether the `ApplyCmd` is for `Ensure` or `Clean`.
    apply_for_internal: &'f ApplyForInternal,
    /// Channel sender for `CmdBlock` item outcomes.
    #[cfg(feature = "output_progress")]
    progress_tx: &'f Sender<CmdProgressUpdate>,
    outcomes_tx: &'f Sender<ItemApplyOutcome<E>>,
}

#[derive(Debug)]
pub enum ItemApplyOutcome<E> {
    /// Error occurred when discovering current state, goal states, state
    /// diff, or `ApplyCheck`.
    PrepareFail {
        item_id: ItemId,
        item_apply_partial: ItemApplyPartialBoxed,
        error: E,
    },
    /// Ensure execution succeeded.
    Success {
        item_id: ItemId,
        item_apply: ItemApplyBoxed,
    },
    /// Ensure execution failed.
    Fail {
        item_id: ItemId,
        item_apply: ItemApplyBoxed,
        error: E,
    },
}

/// Infers the target state, ensure or clean, and dry run, from a `StateTs`.
pub trait StatesTsApplyExt {
    type TsTarget: Debug + Send + Sync + Unpin + 'static;

    /// Returns the `ApplyFor` this `StatesTs` is meant for.
    fn apply_for() -> ApplyFor;
    /// Returns whether this `StatesTs` is for a dry run.
    fn dry_run() -> bool;
}

impl StatesTsApplyExt for Ensured {
    type TsTarget = Goal;

    fn apply_for() -> ApplyFor {
        ApplyFor::Ensure
    }

    fn dry_run() -> bool {
        false
    }
}

impl StatesTsApplyExt for EnsuredDry {
    type TsTarget = Goal;

    fn apply_for() -> ApplyFor {
        ApplyFor::Ensure
    }

    fn dry_run() -> bool {
        true
    }
}

impl StatesTsApplyExt for Cleaned {
    type TsTarget = Clean;

    fn apply_for() -> ApplyFor {
        ApplyFor::Clean
    }

    fn dry_run() -> bool {
        false
    }
}

impl StatesTsApplyExt for CleanedDry {
    type TsTarget = Clean;

    fn apply_for() -> ApplyFor {
        ApplyFor::Clean
    }

    fn dry_run() -> bool {
        true
    }
}
