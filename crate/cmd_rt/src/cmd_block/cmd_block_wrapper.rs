use std::{fmt::Debug, marker::PhantomData, pin::Pin};

use async_trait::async_trait;
use peace_cfg::ItemId;
use peace_cmd::scopes::SingleProfileSingleFlowView;
use peace_resources::{resources::ts::SetUp, Resource};
use peace_rt_model::{outcomes::CmdOutcome, params::ParamsKeys, IndexMap};
use tokio::sync::mpsc;

use crate::{CmdBlock, CmdBlockRt};

cfg_if::cfg_if! {
    if #[cfg(feature = "output_progress")] {
        use peace_cfg::progress::ProgressUpdateAndId;
        use tokio::sync::mpsc::Sender;
    }
}

/// Wraps a [`CmdBlock`] so that the type erased [`CmdBlockRt`] trait can be
/// implemented on the type within this crate.
///
/// [`CmdBlockRt`]: crate::CmdBlockRt
#[derive(Debug)]
pub struct CmdBlockWrapper<CB, E, PKeys, OutcomeT, ItemOutcomeT, WorkingT, InputT> {
    /// Underlying `CmdBlock` implementation.
    ///
    /// The trait constraints are applied on impl blocks.
    cmd_block: CB,
    /// Initialization function for `WorkingT`, based on the block's input type.
    fn_working_t_init: fn(Box<InputT>) -> WorkingT,
    /// Transforms `WorkingT` into `OutcomeT`.
    fn_cmd_outcome_from_working: fn(WorkingT) -> OutcomeT,
    /// Marker.
    marker: PhantomData<(E, PKeys, OutcomeT, ItemOutcomeT)>,
}

impl<CB, E, PKeys, OutcomeT, ItemOutcomeT, WorkingT, InputT>
    CmdBlockWrapper<CB, E, PKeys, OutcomeT, ItemOutcomeT, WorkingT, InputT>
where
    CB: CmdBlock<
            Error = E,
            PKeys = PKeys,
            OutcomeT = OutcomeT,
            ItemOutcomeT = ItemOutcomeT,
            WorkingT = WorkingT,
        >,
{
    pub fn new(
        cmd_block: CB,
        fn_working_t_init: fn(Box<InputT>) -> WorkingT,
        fn_cmd_outcome_from_working: fn(WorkingT) -> OutcomeT,
    ) -> Self {
        Self {
            cmd_block,
            fn_working_t_init,
            fn_cmd_outcome_from_working,
            marker: PhantomData,
        }
    }
}

impl<CB, E, PKeys, OutcomeT, ItemOutcomeT, WorkingT, InputT>
    From<(CB, fn(Box<InputT>) -> WorkingT, fn(WorkingT) -> OutcomeT)>
    for CmdBlockWrapper<CB, E, PKeys, OutcomeT, ItemOutcomeT, WorkingT, InputT>
where
    CB: CmdBlock<
            Error = E,
            PKeys = PKeys,
            OutcomeT = OutcomeT,
            ItemOutcomeT = ItemOutcomeT,
            WorkingT = WorkingT,
        >,
{
    fn from(
        (cmd_block, fn_working_t_init, fn_cmd_outcome_from_working): (
            CB,
            fn(Box<InputT>) -> WorkingT,
            fn(WorkingT) -> OutcomeT,
        ),
    ) -> Self {
        Self::new(cmd_block, fn_working_t_init, fn_cmd_outcome_from_working)
    }
}

#[async_trait(?Send)]
impl<CB, E, PKeys, OutcomeT, ItemOutcomeT, WorkingT, InputT> CmdBlockRt
    for CmdBlockWrapper<CB, E, PKeys, OutcomeT, ItemOutcomeT, WorkingT, InputT>
where
    CB: CmdBlock<
            Error = E,
            PKeys = PKeys,
            ItemOutcomeT = ItemOutcomeT,
            WorkingT = WorkingT,
            InputT = InputT,
        > + Unpin,
    E: Debug + std::error::Error + From<peace_rt_model::Error> + Send + Unpin + 'static,
    PKeys: Debug + ParamsKeys + Unpin + 'static,
    OutcomeT: Debug + Unpin + 'static,
    ItemOutcomeT: Debug + Unpin + 'static,
    WorkingT: Debug + Resource + 'static,
    InputT: Debug + Resource + 'static,
{
    type Error = E;
    type OutcomeT = OutcomeT;
    type PKeys = PKeys;

    async fn exec(
        self: Pin<Box<Self>>,
        cmd_view: &mut SingleProfileSingleFlowView<'_, Self::Error, Self::PKeys, SetUp>,
        #[cfg(feature = "output_progress")] progress_tx: Sender<ProgressUpdateAndId>,
        input: Box<dyn Resource>,
    ) -> Result<CmdOutcome<Self::OutcomeT, Self::Error>, Self::Error> {
        let input = input.downcast().unwrap_or_else(|input| {
            let input_type_name = tynm::type_name::<InputT>();
            let actual_type_name = Resource::type_name(&*input);
            panic!(
                "Expected to downcast input to `{input_type_name}`.\n\
                The actual type name is `{actual_type_name:?}`\n\
                This is a bug in the Peace framework."
            );
        });
        let cmd_block = &self.cmd_block;

        let (outcomes_tx, mut outcomes_rx) = mpsc::unbounded_channel::<ItemOutcomeT>();
        let mut cmd_outcome = {
            let outcome = (self.fn_working_t_init)(input);
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
                    cmd_view,
                    outcomes_tx,
                    #[cfg(feature = "output_progress")]
                    progress_tx,
                )
                .await;

            // `progress_tx` is dropped here, so `progress_rx` will safely end.
        };

        let ((), outcome_result) = futures::join!(execution_task, outcomes_rx_task);

        outcome_result
            .map(|()| cmd_outcome.map(|working_t| (self.fn_cmd_outcome_from_working)(working_t)))
    }
}
