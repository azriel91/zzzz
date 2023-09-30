use std::{fmt::Debug, marker::PhantomData};

use async_trait::async_trait;
use peace_cfg::ItemId;
use peace_cmd::scopes::SingleProfileSingleFlowView;
use peace_cmd_model::CmdBlockDesc;
use peace_resources::{resources::ts::SetUp, Resource};
use peace_rt_model::{outcomes::CmdOutcome, params::ParamsKeys, IndexMap};
use tokio::sync::mpsc;
use tynm::TypeParamsFmtOpts;

use crate::{CmdBlock, CmdBlockError, CmdBlockRt};

cfg_if::cfg_if! {
    if #[cfg(feature = "output_progress")] {
        use peace_cfg::progress::ProgressUpdateAndId;
        use tokio::sync::mpsc::Sender;
    }
}

/// Wraps a [`CmdBlock`] and holds a partial execution handler.
///
/// The following are the technical reasons for this type's existence:
///
/// * Being in the `peace_cmd` crate, the type erased [`CmdBlockRt`] trait can
///   be implemented on this type within this crate.
/// * The partial execution handler specifies how a command execution should
///   finish, if execution is interrupted or there is an error with one item
///   within the flow.
///
/// [`CmdBlockRt`]: crate::CmdBlockRt
#[derive(Debug)]
pub struct CmdBlockWrapper<
    CB,
    E,
    PKeys,
    ExecutionOutcome,
    BlockOutcome,
    BlockOutcomeAcc,
    BlockOutcomePartial,
    InputT,
> {
    /// Underlying `CmdBlock` implementation.
    ///
    /// The trait constraints are applied on impl blocks.
    cmd_block: CB,
    /// Function to run if interruption or an item failure happens while
    /// executing this `CmdBlock`.
    fn_partial_exec_handler: fn(BlockOutcomeAcc) -> ExecutionOutcome,
    /// Marker.
    marker: PhantomData<(E, PKeys, BlockOutcome, BlockOutcomePartial, InputT)>,
}

impl<CB, E, PKeys, ExecutionOutcome, BlockOutcome, BlockOutcomeAcc, BlockOutcomePartial, InputT>
    CmdBlockWrapper<
        CB,
        E,
        PKeys,
        ExecutionOutcome,
        BlockOutcome,
        BlockOutcomeAcc,
        BlockOutcomePartial,
        InputT,
    >
where
    CB: CmdBlock<
            Error = E,
            PKeys = PKeys,
            OutcomeAcc = BlockOutcomeAcc,
            OutcomePartial = BlockOutcomePartial,
            InputT = InputT,
        >,
{
    /// Returns a new `CmdBlockWrapper`.
    ///
    /// # Parameters
    ///
    /// * `cmd_block`: The `CmdBlock` implementation.
    /// * `fn_partial_exec_handler`: How the `CmdExecution` should end, if
    ///   execution ends with this `CmdBlock`.
    ///
    ///     This could be due to interruption, or a `CmdOutcome` with an item
    ///     failure.
    pub fn new(
        cmd_block: CB,
        fn_partial_exec_handler: fn(BlockOutcomeAcc) -> ExecutionOutcome,
    ) -> Self {
        Self {
            cmd_block,
            fn_partial_exec_handler,
            marker: PhantomData,
        }
    }
}

#[async_trait(?Send)]
impl<CB, E, PKeys, ExecutionOutcome, BlockOutcome, BlockOutcomeAcc, BlockOutcomePartial, InputT>
    CmdBlockRt
    for CmdBlockWrapper<
        CB,
        E,
        PKeys,
        ExecutionOutcome,
        BlockOutcome,
        BlockOutcomeAcc,
        BlockOutcomePartial,
        InputT,
    >
where
    CB: CmdBlock<
            Error = E,
            PKeys = PKeys,
            Outcome = BlockOutcome,
            OutcomeAcc = BlockOutcomeAcc,
            OutcomePartial = BlockOutcomePartial,
            InputT = InputT,
        > + Unpin,
    E: Debug + std::error::Error + From<peace_rt_model::Error> + Send + Unpin + 'static,
    PKeys: ParamsKeys + 'static,
    ExecutionOutcome: Debug + Unpin + Send + Sync + 'static,
    BlockOutcome: Debug + Unpin + Send + Sync + 'static,
    BlockOutcomeAcc: Debug + Resource + Unpin + 'static,
    BlockOutcomePartial: Debug + Unpin + 'static,
    InputT: Debug + Resource + Unpin + 'static,
{
    type Error = E;
    type ExecutionOutcome = ExecutionOutcome;
    type PKeys = PKeys;

    async fn exec(
        &self,
        cmd_view: &mut SingleProfileSingleFlowView<'_, Self::Error, Self::PKeys, SetUp>,
        #[cfg(feature = "output_progress")] progress_tx: Sender<ProgressUpdateAndId>,
    ) -> Result<(), CmdBlockError<ExecutionOutcome, Self::Error>> {
        let cmd_block = &self.cmd_block;
        let input = cmd_block.input_fetch(cmd_view.resources)?;

        let (outcomes_tx, mut outcomes_rx) = mpsc::unbounded_channel::<BlockOutcomePartial>();
        let mut cmd_outcome = {
            let outcome = cmd_block.outcome_acc_init(&input);
            let errors = IndexMap::<ItemId, E>::new();
            CmdOutcome {
                value: outcome,
                errors,
            }
        };
        let outcomes_rx_task = async {
            while let Some(item_outcome) = outcomes_rx.recv().await {
                cmd_block.outcome_collate(&mut cmd_outcome, item_outcome)?;
            }

            Result::<(), E>::Ok(())
        };

        let execution_task = async move {
            let outcomes_tx = &outcomes_tx;
            #[cfg(feature = "output_progress")]
            let progress_tx = &progress_tx;

            cmd_block
                .exec(
                    input,
                    cmd_view,
                    outcomes_tx,
                    #[cfg(feature = "output_progress")]
                    progress_tx,
                )
                .await;

            cmd_view
            // `progress_tx` is dropped here, so `progress_rx` will safely end.
        };

        let (cmd_view, outcome_result) = futures::join!(execution_task, outcomes_rx_task);
        let () = outcome_result.map_err(CmdBlockError::Block)?;

        if cmd_outcome.is_ok() {
            let CmdOutcome {
                value: outcome_acc,
                errors: _,
            } = cmd_outcome;

            let outcome = cmd_block.outcome_from_acc(outcome_acc);
            cmd_block.outcome_insert(cmd_view.resources, outcome);

            Ok(())
        } else {
            // If possible, `CmdBlock` outcomes with item errors need to be mapped to
            // the `CmdExecution` outcome type, so we still return the item errors.
            //
            // e.g. `StatesCurrentMut` should be mapped into `StatesEnsured` when some
            // items fail to be ensured.
            //
            // Note, when discovering current and goal states for diffing, and an item
            // error occurs, mapping the partially accumulated `(StatesCurrentMut,
            // StatesGoalMut)` into `StateDiffs` may or may not be semantically
            // meaningful.

            let cmd_outcome = cmd_outcome.map(self.fn_partial_exec_handler);
            Err(CmdBlockError::Outcome(cmd_outcome))
        }
    }

    fn cmd_block_desc(&self) -> CmdBlockDesc {
        let cmd_block_name = tynm::type_name_opts::<CB>(TypeParamsFmtOpts::Std);
        let cmd_block_input_names = self.cmd_block.input_type_names();
        let cmd_block_outcome_names = self.cmd_block.outcome_type_names();

        CmdBlockDesc::new(
            cmd_block_name,
            cmd_block_input_names,
            cmd_block_outcome_names,
        )
    }
}
