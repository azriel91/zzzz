use std::{fmt::Debug, marker::PhantomData};

use fn_graph::{StreamOpts, StreamOutcome};
use futures::FutureExt;
use peace_cmd::{
    ctx::CmdCtxTypesConstrained, interruptible::InterruptibilityState,
    scopes::SingleProfileSingleFlowView,
};
use peace_cmd_model::CmdBlockOutcome;
use peace_cmd_rt::{async_trait, CmdBlock};
use peace_flow_rt::Flow;
use peace_item_model::ItemId;
use peace_params::ParamsSpecs;
use peace_resource_rt::{
    internal::StateDiffsMut,
    resources::ts::SetUp,
    states::{
        ts::{Current, CurrentStored, Goal, GoalStored},
        StateDiffs, States,
    },
    type_reg::untagged::{BoxDtDisplay, TypeMap},
    ResourceFetchError, Resources,
};

use crate::cmds::DiffStateSpec;

cfg_if::cfg_if! {
    if #[cfg(feature = "output_progress")] {
        use peace_progress_model::{CmdBlockItemInteractionType, CmdProgressUpdate};
        use tokio::sync::mpsc::Sender;
    }
}

pub struct DiffCmdBlock<CmdCtxTypesT, StatesTs0, StatesTs1>(
    PhantomData<(CmdCtxTypesT, StatesTs0, StatesTs1)>,
);

impl<CmdCtxTypesT, StatesTs0, StatesTs1> Debug
    for DiffCmdBlock<CmdCtxTypesT, StatesTs0, StatesTs1>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("DiffCmdBlock").field(&self.0).finish()
    }
}

impl<CmdCtxTypesT, StatesTs0, StatesTs1> DiffCmdBlock<CmdCtxTypesT, StatesTs0, StatesTs1> {
    /// Returns a new `DiffCmdBlock`.
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<CmdCtxTypesT, StatesTs0, StatesTs1> DiffCmdBlock<CmdCtxTypesT, StatesTs0, StatesTs1>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
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
        interruptibility_state: InterruptibilityState<'_, '_>,
        flow: &Flow<<CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        states_a: &TypeMap<ItemId, BoxDtDisplay>,
        states_b: &TypeMap<ItemId, BoxDtDisplay>,
    ) -> Result<StreamOutcome<StateDiffs>, <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError> {
        let stream_outcome_result = flow
            .graph()
            .try_fold_async_with(
                StateDiffsMut::with_capacity(states_a.len()),
                StreamOpts::new()
                    .interruptibility_state(interruptibility_state)
                    .interrupted_next_item_include(false),
                |mut state_diffs_mut, item| {
                    async move {
                        let _params_specs = &params_specs;

                        let state_diff_opt = item
                            .state_diff_exec(params_specs, resources, states_a, states_b)
                            .await?;

                        if let Some(state_diff) = state_diff_opt {
                            state_diffs_mut.insert_raw(item.id().clone(), state_diff);
                        }

                        Result::<_, <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>::Ok(
                            state_diffs_mut,
                        )
                    }
                    .boxed_local()
                },
            )
            .await;

        stream_outcome_result.map(|stream_outcome| stream_outcome.map(StateDiffs::from))
    }
}

impl<CmdCtxTypesT, StatesTs0, StatesTs1> Default
    for DiffCmdBlock<CmdCtxTypesT, StatesTs0, StatesTs1>
{
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[async_trait(?Send)]
impl<CmdCtxTypesT, StatesTs0, StatesTs1> CmdBlock
    for DiffCmdBlock<CmdCtxTypesT, StatesTs0, StatesTs1>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
    StatesTs0: Debug + Send + Sync + 'static,
    StatesTs1: Debug + Send + Sync + 'static,
{
    type CmdCtxTypes = CmdCtxTypesT;
    type InputT = (States<StatesTs0>, States<StatesTs1>);
    type Outcome = (StateDiffs, Self::InputT);

    #[cfg(feature = "output_progress")]
    fn cmd_block_item_interaction_type(&self) -> CmdBlockItemInteractionType {
        CmdBlockItemInteractionType::Local
    }

    fn input_fetch(
        &self,
        resources: &mut Resources<SetUp>,
    ) -> Result<Self::InputT, ResourceFetchError> {
        let states_ts0 = resources.try_remove::<States<StatesTs0>>()?;
        let states_ts1 = resources.try_remove::<States<StatesTs1>>()?;

        Ok((states_ts0, states_ts1))
    }

    fn input_type_names(&self) -> Vec<String> {
        vec![
            tynm::type_name::<States<StatesTs0>>(),
            tynm::type_name::<States<StatesTs1>>(),
        ]
    }

    fn outcome_insert(&self, resources: &mut Resources<SetUp>, outcome: Self::Outcome) {
        let (state_diffs, (states_ts0, states_ts1)) = outcome;
        resources.insert(state_diffs);
        resources.insert(states_ts0);
        resources.insert(states_ts1);
    }

    fn outcome_type_names(&self) -> Vec<String> {
        vec![
            tynm::type_name::<StateDiffs>(),
            tynm::type_name::<States<StatesTs0>>(),
            tynm::type_name::<States<StatesTs1>>(),
        ]
    }

    async fn exec(
        &self,
        input: Self::InputT,
        cmd_view: &mut SingleProfileSingleFlowView<'_, Self::CmdCtxTypes>,
        #[cfg(feature = "output_progress")] _progress_tx: &Sender<CmdProgressUpdate>,
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

        let (states_ts0, states_ts1) = input;

        let stream_outcome = Self::diff_any(
            interruptibility_state.reborrow(),
            flow,
            params_specs,
            resources,
            &states_ts0,
            &states_ts1,
        )
        .await?
        .map(move |state_diffs| (state_diffs, (states_ts0, states_ts1)));

        Ok(CmdBlockOutcome::new_item_wise(stream_outcome))
    }
}

/// Infers the states to use in diffing, from a `StateTs`.
pub trait DiffCmdBlockStatesTsExt {
    /// Returns the `ApplyFor` this `StatesTs` is meant for.
    fn diff_state_spec() -> DiffStateSpec;
}

impl DiffCmdBlockStatesTsExt for Current {
    fn diff_state_spec() -> DiffStateSpec {
        DiffStateSpec::Current
    }
}

impl DiffCmdBlockStatesTsExt for CurrentStored {
    fn diff_state_spec() -> DiffStateSpec {
        DiffStateSpec::CurrentStored
    }
}
impl DiffCmdBlockStatesTsExt for Goal {
    fn diff_state_spec() -> DiffStateSpec {
        DiffStateSpec::Goal
    }
}
impl DiffCmdBlockStatesTsExt for GoalStored {
    fn diff_state_spec() -> DiffStateSpec {
        DiffStateSpec::GoalStored
    }
}
