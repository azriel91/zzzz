use std::{fmt::Debug, marker::PhantomData};

use peace_cmd::{ctx::CmdCtxTypesConstrained, scopes::SingleProfileSingleFlowView};
use peace_cmd_model::CmdBlockOutcome;
use peace_cmd_rt::{async_trait, CmdBlock};
use peace_resource_rt::{
    resources::ts::SetUp,
    states::{States, StatesCurrent, StatesCurrentStored, StatesGoal, StatesGoalStored},
    ResourceFetchError, Resources,
};
use peace_rt_model::Error;
use peace_rt_model_core::{ApplyCmdError, ItemsStateStoredStale, StateStoredAndDiscovered};

cfg_if::cfg_if! {
    if #[cfg(feature = "output_progress")] {
        use peace_progress_model::{
            CmdBlockItemInteractionType,
            CmdProgressUpdate,
            ProgressComplete,
            ProgressDelta,
            ProgressMsgUpdate,
            ProgressUpdate,
            ProgressUpdateAndId,
        };
        use tokio::sync::mpsc::Sender;
    }
}

// Whether to block an apply operation if stored states are not in sync with
// discovered state.

/// Neither stored current states nor stored goal state need to be in sync
/// with the discovered current states and goal state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApplyStoreStateSyncNone;

/// The stored current states must be in sync with the discovered current
/// state for the apply to proceed.
///
/// The stored goal state does not need to be in sync with the discovered
/// goal state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApplyStoreStateSyncCurrent;

/// The stored goal state must be in sync with the discovered goal
/// state for the apply to proceed.
///
/// The stored current states does not need to be in sync with the
/// discovered current state.
///
/// For `CleanCmd`, this variant is equivalent to `None`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApplyStoreStateSyncGoal;

/// Both stored current states and stored goal state must be in sync with
/// the discovered current states and goal state for the apply to
/// proceed.
///
/// For `CleanCmd`, this variant is equivalent to `Current`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApplyStoreStateSyncCurrentAndGoal;

/// Stops a `CmdExecution` if stored states and discovered states are not in
/// sync.
pub struct ApplyStateSyncCheckCmdBlock<CmdCtxTypesT, ApplyStoreStateSync>(
    PhantomData<(CmdCtxTypesT, ApplyStoreStateSync)>,
);

impl<CmdCtxTypesT, ApplyStoreStateSync> Debug
    for ApplyStateSyncCheckCmdBlock<CmdCtxTypesT, ApplyStoreStateSync>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ApplyStateSyncCheckCmdBlock")
            .field(&self.0)
            .finish()
    }
}

impl<CmdCtxTypesT> ApplyStateSyncCheckCmdBlock<CmdCtxTypesT, ApplyStoreStateSyncNone>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    /// Returns a block that discovers current states.
    pub fn none() -> Self {
        Self(PhantomData)
    }
}

impl<CmdCtxTypesT> ApplyStateSyncCheckCmdBlock<CmdCtxTypesT, ApplyStoreStateSyncCurrent>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    /// Returns a block that discovers current states.
    pub fn current() -> Self {
        Self(PhantomData)
    }
}

impl<CmdCtxTypesT> ApplyStateSyncCheckCmdBlock<CmdCtxTypesT, ApplyStoreStateSyncGoal>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    /// Returns a block that discovers goal states.
    pub fn goal() -> Self {
        Self(PhantomData)
    }
}

impl<CmdCtxTypesT> ApplyStateSyncCheckCmdBlock<CmdCtxTypesT, ApplyStoreStateSyncCurrentAndGoal>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    /// Returns a block that discovers both current and goal states.
    pub fn current_and_goal() -> Self {
        Self(PhantomData)
    }
}

impl<CmdCtxTypesT, ApplyStoreStateSync>
    ApplyStateSyncCheckCmdBlock<CmdCtxTypesT, ApplyStoreStateSync>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    fn items_state_stored_stale<StatesTsStored, StatesTs>(
        cmd_view: &SingleProfileSingleFlowView<'_, CmdCtxTypesT>,
        states_stored: &States<StatesTsStored>,
        states_discovered: &States<StatesTs>,
        #[cfg(feature = "output_progress")] progress_tx: &Sender<CmdProgressUpdate>,
    ) -> Result<ItemsStateStoredStale, <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError> {
        let items_state_stored_stale = cmd_view.flow.graph().iter_insertion().try_fold(
            ItemsStateStoredStale::new(),
            |mut items_state_stored_stale, item_rt| {
                let item_id = item_rt.id();
                let state_stored = states_stored.get_raw(item_id);
                let state_discovered = states_discovered.get_raw(item_id);

                match (state_stored, state_discovered) {
                    (None, None) => {
                        // Item not discoverable, may be dependent on
                        // predecessor
                    }
                    (None, Some(state_discovered)) => {
                        let item_id = item_id.clone();
                        let state_discovered = state_discovered.clone();
                        items_state_stored_stale.insert(
                            item_id,
                            StateStoredAndDiscovered::OnlyDiscoveredExists { state_discovered },
                        );
                    }
                    (Some(state_stored), None) => {
                        let item_id = item_id.clone();
                        let state_stored = state_stored.clone();
                        items_state_stored_stale.insert(
                            item_id,
                            StateStoredAndDiscovered::OnlyStoredExists { state_stored },
                        );
                    }
                    (Some(state_stored), Some(state_discovered)) => {
                        let state_eq = item_rt.state_eq(state_stored, state_discovered);
                        match state_eq {
                            Ok(true) => {
                                #[cfg(feature = "output_progress")]
                                {
                                    let state_type = tynm::type_name::<StatesTs>();
                                    let _progress_send_unused = progress_tx.try_send(
                                        ProgressUpdateAndId {
                                            item_id: item_id.clone(),
                                            progress_update: ProgressUpdate::Delta(
                                                ProgressDelta::Tick,
                                            ),
                                            msg_update: ProgressMsgUpdate::Set(format!(
                                                "State {state_type} in sync"
                                            )),
                                        }
                                        .into(),
                                    );
                                }
                            }
                            Ok(false) => {
                                #[cfg(feature = "output_progress")]
                                {
                                    let state_type = tynm::type_name::<StatesTs>();
                                    let _progress_send_unused = progress_tx.try_send(
                                        ProgressUpdateAndId {
                                            item_id: item_id.clone(),
                                            progress_update: ProgressUpdate::Complete(
                                                ProgressComplete::Fail,
                                            ),
                                            msg_update: ProgressMsgUpdate::Set(format!(
                                                "State {state_type} out of sync"
                                            )),
                                        }
                                        .into(),
                                    );
                                }

                                let item_id = item_id.clone();
                                let state_stored = state_stored.clone();
                                let state_discovered = state_discovered.clone();
                                items_state_stored_stale.insert(
                                    item_id,
                                    StateStoredAndDiscovered::ValuesDiffer {
                                        state_stored,
                                        state_discovered,
                                    },
                                );
                            }
                            Err(error) => return Err(error),
                        }
                    }
                }

                Ok(items_state_stored_stale)
            },
        )?;

        Ok(items_state_stored_stale)
    }
}

#[async_trait(?Send)]
impl<CmdCtxTypesT> CmdBlock for ApplyStateSyncCheckCmdBlock<CmdCtxTypesT, ApplyStoreStateSyncNone>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    type CmdCtxTypes = CmdCtxTypesT;
    type InputT = ();
    type Outcome = Self::InputT;

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

    fn outcome_insert(&self, _resources: &mut Resources<SetUp>, _outcome: Self::Outcome) {}

    fn outcome_type_names(&self) -> Vec<String> {
        vec![]
    }

    async fn exec(
        &self,
        input: Self::InputT,
        _cmd_view: &mut SingleProfileSingleFlowView<'_, Self::CmdCtxTypes>,
        #[cfg(feature = "output_progress")] _progress_tx: &Sender<CmdProgressUpdate>,
    ) -> Result<
        CmdBlockOutcome<Self::Outcome, <Self::CmdCtxTypes as CmdCtxTypesConstrained>::AppError>,
        <Self::CmdCtxTypes as CmdCtxTypesConstrained>::AppError,
    > {
        outcome_collate(input, OutcomeResult::Ok)
    }
}

#[async_trait(?Send)]
impl<CmdCtxTypesT> CmdBlock
    for ApplyStateSyncCheckCmdBlock<CmdCtxTypesT, ApplyStoreStateSyncCurrent>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    type CmdCtxTypes = CmdCtxTypesT;
    type InputT = (StatesCurrentStored, StatesCurrent);
    type Outcome = Self::InputT;

    #[cfg(feature = "output_progress")]
    fn cmd_block_item_interaction_type(&self) -> CmdBlockItemInteractionType {
        CmdBlockItemInteractionType::Read
    }

    fn input_fetch(
        &self,
        resources: &mut Resources<SetUp>,
    ) -> Result<Self::InputT, ResourceFetchError> {
        input_fetch_current(resources)
    }

    fn input_type_names(&self) -> Vec<String> {
        vec![
            tynm::type_name::<StatesCurrentStored>(),
            tynm::type_name::<StatesCurrent>(),
        ]
    }

    fn outcome_insert(&self, resources: &mut Resources<SetUp>, outcome: Self::Outcome) {
        let (states_current_stored, states_current) = outcome;
        resources.insert(states_current_stored);
        resources.insert(states_current);
    }

    fn outcome_type_names(&self) -> Vec<String> {
        vec![
            tynm::type_name::<StatesCurrentStored>(),
            tynm::type_name::<StatesCurrent>(),
        ]
    }

    async fn exec(
        &self,
        mut input: Self::InputT,
        cmd_view: &mut SingleProfileSingleFlowView<'_, Self::CmdCtxTypes>,
        #[cfg(feature = "output_progress")] progress_tx: &Sender<CmdProgressUpdate>,
    ) -> Result<
        CmdBlockOutcome<Self::Outcome, <Self::CmdCtxTypes as CmdCtxTypesConstrained>::AppError>,
        <Self::CmdCtxTypes as CmdCtxTypesConstrained>::AppError,
    > {
        let (states_current_stored, states_current) = &mut input;

        let state_current_stale_result = Self::items_state_stored_stale(
            cmd_view,
            states_current_stored,
            states_current,
            #[cfg(feature = "output_progress")]
            progress_tx,
        );
        match state_current_stale_result {
            Ok(items_state_stored_stale) => {
                if items_state_stored_stale.stale() {
                    return outcome_collate(
                        input,
                        OutcomeResult::StatesCurrentOutOfSync {
                            items_state_stored_stale,
                        },
                    );
                }
            }
            Err(error) => {
                return outcome_collate(input, OutcomeResult::StatesDowncastError { error });
            }
        };

        outcome_collate(input, OutcomeResult::Ok)
    }
}

#[async_trait(?Send)]
impl<CmdCtxTypesT> CmdBlock for ApplyStateSyncCheckCmdBlock<CmdCtxTypesT, ApplyStoreStateSyncGoal>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    type CmdCtxTypes = CmdCtxTypesT;
    type InputT = (StatesGoalStored, StatesGoal);
    type Outcome = Self::InputT;

    #[cfg(feature = "output_progress")]
    fn cmd_block_item_interaction_type(&self) -> CmdBlockItemInteractionType {
        CmdBlockItemInteractionType::Read
    }

    fn input_fetch(
        &self,
        resources: &mut Resources<SetUp>,
    ) -> Result<Self::InputT, ResourceFetchError> {
        input_fetch_goal(resources)
    }

    fn input_type_names(&self) -> Vec<String> {
        vec![
            tynm::type_name::<StatesGoalStored>(),
            tynm::type_name::<StatesGoal>(),
        ]
    }

    fn outcome_insert(&self, resources: &mut Resources<SetUp>, outcome: Self::Outcome) {
        let (states_goal_stored, states_goal) = outcome;
        resources.insert(states_goal_stored);
        resources.insert(states_goal);
    }

    fn outcome_type_names(&self) -> Vec<String> {
        vec![
            tynm::type_name::<StatesGoalStored>(),
            tynm::type_name::<StatesGoal>(),
        ]
    }

    async fn exec(
        &self,
        mut input: Self::InputT,
        cmd_view: &mut SingleProfileSingleFlowView<'_, Self::CmdCtxTypes>,
        #[cfg(feature = "output_progress")] progress_tx: &Sender<CmdProgressUpdate>,
    ) -> Result<
        CmdBlockOutcome<Self::Outcome, <Self::CmdCtxTypes as CmdCtxTypesConstrained>::AppError>,
        <Self::CmdCtxTypes as CmdCtxTypesConstrained>::AppError,
    > {
        let (states_goal_stored, states_goal) = &mut input;

        let state_goal_stale_result = Self::items_state_stored_stale(
            cmd_view,
            states_goal_stored,
            states_goal,
            #[cfg(feature = "output_progress")]
            progress_tx,
        );
        match state_goal_stale_result {
            Ok(items_state_stored_stale) => {
                if items_state_stored_stale.stale() {
                    return outcome_collate(
                        input,
                        OutcomeResult::StatesGoalOutOfSync {
                            items_state_stored_stale,
                        },
                    );
                }
            }
            Err(error) => {
                return outcome_collate(input, OutcomeResult::StatesDowncastError { error });
            }
        };

        outcome_collate(input, OutcomeResult::Ok)
    }
}

#[async_trait(?Send)]
impl<CmdCtxTypesT> CmdBlock
    for ApplyStateSyncCheckCmdBlock<CmdCtxTypesT, ApplyStoreStateSyncCurrentAndGoal>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    type CmdCtxTypes = CmdCtxTypesT;
    type InputT = (
        StatesCurrentStored,
        StatesCurrent,
        StatesGoalStored,
        StatesGoal,
    );
    type Outcome = Self::InputT;

    #[cfg(feature = "output_progress")]
    fn cmd_block_item_interaction_type(&self) -> CmdBlockItemInteractionType {
        CmdBlockItemInteractionType::Read
    }

    fn input_fetch(
        &self,
        resources: &mut Resources<SetUp>,
    ) -> Result<Self::InputT, ResourceFetchError> {
        let (states_current_stored, states_current) = input_fetch_current(resources)?;
        let (states_goal_stored, states_goal) = input_fetch_goal(resources)?;

        Ok((
            states_current_stored,
            states_current,
            states_goal_stored,
            states_goal,
        ))
    }

    fn input_type_names(&self) -> Vec<String> {
        vec![
            tynm::type_name::<StatesCurrentStored>(),
            tynm::type_name::<StatesCurrent>(),
            tynm::type_name::<StatesGoalStored>(),
            tynm::type_name::<StatesGoal>(),
        ]
    }

    fn outcome_insert(&self, resources: &mut Resources<SetUp>, outcome: Self::Outcome) {
        let (states_current_stored, states_current, states_goal_stored, states_goal) = outcome;
        resources.insert(states_current_stored);
        resources.insert(states_current);
        resources.insert(states_goal_stored);
        resources.insert(states_goal);
    }

    fn outcome_type_names(&self) -> Vec<String> {
        vec![
            tynm::type_name::<StatesCurrentStored>(),
            tynm::type_name::<StatesCurrent>(),
            tynm::type_name::<StatesGoalStored>(),
            tynm::type_name::<StatesGoal>(),
        ]
    }

    async fn exec(
        &self,
        mut input: Self::InputT,
        cmd_view: &mut SingleProfileSingleFlowView<'_, Self::CmdCtxTypes>,
        #[cfg(feature = "output_progress")] progress_tx: &Sender<CmdProgressUpdate>,
    ) -> Result<
        CmdBlockOutcome<Self::Outcome, <Self::CmdCtxTypes as CmdCtxTypesConstrained>::AppError>,
        <Self::CmdCtxTypes as CmdCtxTypesConstrained>::AppError,
    > {
        let (states_current_stored, states_current, states_goal_stored, states_goal) = &mut input;

        let state_current_stale_result = Self::items_state_stored_stale(
            cmd_view,
            states_current_stored,
            states_current,
            #[cfg(feature = "output_progress")]
            progress_tx,
        );
        match state_current_stale_result {
            Ok(items_state_stored_stale) => {
                if items_state_stored_stale.stale() {
                    return outcome_collate(
                        input,
                        OutcomeResult::StatesCurrentOutOfSync {
                            items_state_stored_stale,
                        },
                    );
                }
            }
            Err(error) => {
                return outcome_collate(input, OutcomeResult::StatesDowncastError { error });
            }
        };

        let state_goal_stale_result = Self::items_state_stored_stale(
            cmd_view,
            states_goal_stored,
            states_goal,
            #[cfg(feature = "output_progress")]
            progress_tx,
        );
        match state_goal_stale_result {
            Ok(items_state_stored_stale) => {
                if items_state_stored_stale.stale() {
                    return outcome_collate(
                        input,
                        OutcomeResult::StatesGoalOutOfSync {
                            items_state_stored_stale,
                        },
                    );
                }
            }
            Err(error) => {
                return outcome_collate(input, OutcomeResult::StatesDowncastError { error });
            }
        }

        outcome_collate(input, OutcomeResult::Ok)
    }
}

#[derive(Debug)]
enum OutcomeResult<E> {
    /// States that are desired to be in sync are in sync.
    Ok,
    /// Stored current states are not in sync with the actual current state.
    StatesCurrentOutOfSync {
        /// Items whose stored current state is out of sync with the discovered
        /// state.
        items_state_stored_stale: ItemsStateStoredStale,
    },
    /// Stored goal states are not in sync with the actual goal state.
    StatesGoalOutOfSync {
        /// Items whose stored goal state is out of sync with the discovered
        /// state.
        items_state_stored_stale: ItemsStateStoredStale,
    },
    /// Error downcasting a boxed item state to its concrete stype.
    StatesDowncastError {
        /// The error from state downcast.
        error: E,
    },
}

// Use trampolining to decrease compiled code size..

fn input_fetch_current(
    resources: &mut Resources<SetUp>,
) -> Result<(StatesCurrentStored, StatesCurrent), ResourceFetchError> {
    let states_current_stored = resources.try_remove::<StatesCurrentStored>()?;
    let states_current = resources.try_remove::<StatesCurrent>()?;

    Ok((states_current_stored, states_current))
}

fn input_fetch_goal(
    resources: &mut Resources<SetUp>,
) -> Result<(StatesGoalStored, StatesGoal), ResourceFetchError> {
    let states_goal_stored = resources.try_remove::<StatesGoalStored>()?;
    let states_goal = resources.try_remove::<StatesGoal>()?;

    Ok((states_goal_stored, states_goal))
}

fn outcome_collate<AppErrorT, InputT>(
    states_stored_and_discovered: InputT,
    outcome_result: OutcomeResult<AppErrorT>,
) -> Result<CmdBlockOutcome<InputT, AppErrorT>, AppErrorT>
where
    AppErrorT: peace_value_traits::AppError + From<peace_rt_model::Error>,
{
    match outcome_result {
        OutcomeResult::Ok => Ok(CmdBlockOutcome::Single(states_stored_and_discovered)),
        OutcomeResult::StatesCurrentOutOfSync {
            items_state_stored_stale,
        } => Err(AppErrorT::from(Error::ApplyCmdError(
            ApplyCmdError::StatesCurrentOutOfSync {
                items_state_stored_stale,
            },
        ))),
        OutcomeResult::StatesGoalOutOfSync {
            items_state_stored_stale,
        } => Err(AppErrorT::from(Error::ApplyCmdError(
            ApplyCmdError::StatesGoalOutOfSync {
                items_state_stored_stale,
            },
        ))),
        OutcomeResult::StatesDowncastError { error } => Err(error),
    }
}
