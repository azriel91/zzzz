use std::{collections::VecDeque, fmt::Debug};

use futures::{future, stream, Future, StreamExt, TryStreamExt};
use interruptible::InterruptSignal;
use peace_cmd::{
    ctx::{CmdCtx, CmdCtxTypes, CmdCtxTypesConstrained},
    scopes::{
        SingleProfileSingleFlow, SingleProfileSingleFlowView, SingleProfileSingleFlowViewAndOutput,
    },
};
use peace_cmd_model::{CmdBlockDesc, CmdOutcome};
use peace_resource_rt::{resources::ts::SetUp, Resources};

use crate::{CmdBlockError, CmdBlockRtBox, ItemStreamOutcomeMapper};

cfg_if::cfg_if! {
    if #[cfg(feature = "output_progress")] {
        use peace_progress_model::CmdProgressUpdate;
        use peace_rt_model::{output::OutputWrite, CmdProgressTracker};
        use tokio::sync::mpsc::{self, Sender};

        use crate::Progress;
    }
}

pub use self::{
    cmd_execution_builder::CmdExecutionBuilder,
    cmd_execution_error_builder::CmdExecutionErrorBuilder,
};

mod cmd_execution_builder;
mod cmd_execution_error_builder;

/// List of [`CmdBlock`]s to run for a `*Cmd`.
///
/// A `CmdExecution` is interruptible if [`CmdExecutionBuilder::interruptible`]
/// is called during construction.
///
/// # Design
///
/// Interruptibility is implemented as type state. It could be implemented as a
/// feature flag as well; I don't know if developers want certain command
/// executions to be interruptible, and others not.
///
/// [`CmdBlock`]: crate::CmdBlock
#[derive(Debug)]
pub struct CmdExecution<'types, ExecutionOutcome, CmdCtxTypesT>
where
    CmdCtxTypesT: CmdCtxTypes,
{
    /// Blocks of commands to run.
    cmd_blocks: VecDeque<CmdBlockRtBox<'types, CmdCtxTypesT, ExecutionOutcome>>,
    /// Logic to extract the `ExecutionOutcome` from `Resources`.
    execution_outcome_fetch: fn(&mut Resources<SetUp>) -> Option<ExecutionOutcome>,
    /// Whether or not to render progress.
    #[cfg(feature = "output_progress")]
    progress_render_enabled: bool,
}

impl<'types, ExecutionOutcome, CmdCtxTypesT> CmdExecution<'types, ExecutionOutcome, CmdCtxTypesT>
where
    ExecutionOutcome: Debug + Send + Sync + Unpin + 'static,
    CmdCtxTypesT: CmdCtxTypesConstrained + 'types,
{
    pub fn builder() -> CmdExecutionBuilder<'types, ExecutionOutcome, CmdCtxTypesT> {
        CmdExecutionBuilder::new()
    }

    /// Returns the result of executing the command.
    pub async fn exec(
        &mut self,
        cmd_ctx: &mut CmdCtx<SingleProfileSingleFlow<'_, CmdCtxTypesT>>,
    ) -> Result<
        CmdOutcome<ExecutionOutcome, <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
        <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError,
    > {
        let Self {
            cmd_blocks,
            execution_outcome_fetch,
            #[cfg(feature = "output_progress")]
            progress_render_enabled,
        } = self;
        #[cfg(feature = "output_progress")]
        let progress_render_enabled = *progress_render_enabled;

        cfg_if::cfg_if! {
            if #[cfg(feature = "output_progress")] {
                let SingleProfileSingleFlowViewAndOutput {
                    output,
                    cmd_progress_tracker,
                    mut cmd_view,
                    ..
                } = cmd_ctx.view_and_output();

                let (cmd_progress_tx, cmd_progress_rx) =
                    mpsc::channel::<CmdProgressUpdate>(crate::CMD_PROGRESS_COUNT_MAX);

                let cmd_progress_tx_for_interruptibility_state = cmd_progress_tx.clone().downgrade();

                cmd_view.interruptibility_state
                    .set_fn_interrupt_activate(Some(move || {
                        if let Some(cmd_progress_tx) = cmd_progress_tx_for_interruptibility_state.upgrade() {
                            let _cmd_progress_send_result =
                            cmd_progress_tx.try_send(CmdProgressUpdate::Interrupt);
                            drop(cmd_progress_tx);
                        }
                    }));
            } else {
                let SingleProfileSingleFlowViewAndOutput {
                    mut cmd_view,
                    ..
                } = cmd_ctx.view_and_output();
            }
        }

        let cmd_outcome_task = cmd_outcome_task(
            cmd_blocks,
            execution_outcome_fetch,
            &mut cmd_view,
            #[cfg(feature = "output_progress")]
            cmd_progress_tx,
        );

        #[cfg(not(feature = "output_progress"))]
        {
            exec_internal(cmd_outcome_task).await
        }

        #[cfg(feature = "output_progress")]
        {
            exec_internal(
                cmd_outcome_task,
                progress_render_enabled,
                output,
                cmd_progress_tracker,
                cmd_progress_rx,
            )
            .await
        }
    }

    // pub fn exec_bg -> CmdExecId
}

/// Executes and returns the `CmdOutcome`.
///
/// This also runs the progress task if the `"output_progress"` feature is
/// enabled.
async fn exec_internal<ExecutionOutcome, E, #[cfg(feature = "output_progress")] O: OutputWrite<E>>(
    cmd_outcome_task: impl Future<Output = Result<CmdOutcome<ExecutionOutcome, E>, E>>,
    #[cfg(feature = "output_progress")] progress_render_enabled: bool,
    #[cfg(feature = "output_progress")] output: &mut O,
    #[cfg(feature = "output_progress")] cmd_progress_tracker: &mut CmdProgressTracker,
    #[cfg(feature = "output_progress")] mut cmd_progress_rx: mpsc::Receiver<CmdProgressUpdate>,
) -> Result<CmdOutcome<ExecutionOutcome, E>, E>
where
    ExecutionOutcome: Debug + Send + Sync + Unpin + 'static,
    E: std::error::Error + From<peace_rt_model::Error> + Send + Sync + Unpin + 'static,
{
    #[cfg(not(feature = "output_progress"))]
    {
        cmd_outcome_task.await
    }

    #[cfg(feature = "output_progress")]
    if progress_render_enabled {
        output.progress_begin(cmd_progress_tracker).await;
        let progress_trackers = &mut cmd_progress_tracker.progress_trackers;
        let progress_render_task =
            Progress::progress_render(output, progress_trackers, cmd_progress_rx);

        let (cmd_outcome, ()) = futures::join!(cmd_outcome_task, progress_render_task);

        output.progress_end(cmd_progress_tracker).await;

        cmd_outcome
    } else {
        // When `progress_render_enabled` is false, still consumes progress updates
        // and drop them.
        let progress_render_task = async move { while cmd_progress_rx.recv().await.is_some() {} };

        let (cmd_outcome, ()) = futures::join!(cmd_outcome_task, progress_render_task);

        cmd_outcome
    }
}

async fn cmd_outcome_task<'types: 'view, 'view, 'view_ref, ExecutionOutcome, CmdCtxTypesT>(
    cmd_blocks: &VecDeque<CmdBlockRtBox<'types, CmdCtxTypesT, ExecutionOutcome>>,
    execution_outcome_fetch: &mut fn(&mut Resources<SetUp>) -> Option<ExecutionOutcome>,
    cmd_view: &mut SingleProfileSingleFlowView<'view, CmdCtxTypesT>,
    #[cfg(feature = "output_progress")] cmd_progress_tx: Sender<CmdProgressUpdate>,
) -> Result<
    CmdOutcome<ExecutionOutcome, <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
    <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError,
>
where
    ExecutionOutcome: Debug + Send + Sync + Unpin + 'static,
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    let cmd_view_and_progress_result: Result<
        CmdViewAndProgress<'_, '_, _>,
        CmdBlockStreamBreak<'_, '_, _, _>,
    > = stream::unfold(cmd_blocks.iter(), |mut cmd_blocks| {
        let cmd_block_next = cmd_blocks.next();
        future::ready(cmd_block_next.map(|cmd_block_next| (cmd_block_next, cmd_blocks)))
    })
    .enumerate()
    .map(Result::<_, CmdBlockStreamBreak<'_, '_, ExecutionOutcome, CmdCtxTypesT>>::Ok)
    .try_fold(
        CmdViewAndProgress {
            cmd_view,
            #[cfg(feature = "output_progress")]
            cmd_progress_tx,
        },
        // `progress_tx` is moved into this closure, and dropped at the very end, so
        // that `progress_render_task` will actually end.
        |cmd_view_and_progress, (cmd_block_index, cmd_block_rt)| async move {
            let CmdViewAndProgress {
                cmd_view,
                #[cfg(feature = "output_progress")]
                cmd_progress_tx,
            } = cmd_view_and_progress;

            #[cfg(feature = "output_progress")]
            if cmd_block_index != 0 {
                cmd_progress_tx
                    .send(CmdProgressUpdate::ResetToPending)
                    .await
                    .expect(
                        "Expected `CmdProgressUpdate` channel to remain open \
                        while iterating over `CmdBlock`s.",
                    );
            }

            // Check if we are interrupted before we execute this `CmdBlock`.
            if let Some(interrupt_signal) =
                cmd_view.interruptibility_state.item_interrupt_poll(true)
            {
                let cmd_view_and_progress = CmdViewAndProgress {
                    cmd_view,
                    #[cfg(feature = "output_progress")]
                    cmd_progress_tx,
                };
                return Err(CmdBlockStreamBreak::Interrupt {
                    cmd_view_and_progress,
                    cmd_block_index_next: cmd_block_index,
                    interrupt_signal,
                });
            }

            #[cfg(feature = "output_progress")]
            {
                let cmd_block_item_interaction_type =
                    cmd_block_rt.cmd_block_item_interaction_type();
                cmd_progress_tx
                    .send(CmdProgressUpdate::CmdBlockStart {
                        cmd_block_item_interaction_type,
                    })
                    .await
                    .expect(
                        "Expected `CmdProgressUpdate` channel to remain open \
                            while iterating over `CmdBlock`s.",
                    );
            }

            let block_cmd_outcome_result = cmd_block_rt
                .exec(
                    cmd_view,
                    #[cfg(feature = "output_progress")]
                    cmd_progress_tx.clone(),
                )
                .await;

            // `CmdBlock` block logic errors are propagated.
            let cmd_view_and_progress = CmdViewAndProgress {
                cmd_view,
                #[cfg(feature = "output_progress")]
                cmd_progress_tx,
            };

            match block_cmd_outcome_result {
                Ok(()) => Ok(cmd_view_and_progress),
                Err(cmd_block_error) => Err(CmdBlockStreamBreak::BlockErr(CmdViewAndErr {
                    cmd_view_and_progress,
                    cmd_block_index,
                    cmd_block_error,
                })),
            }
        },
    )
    .await;

    outcome_extract::<ExecutionOutcome, CmdCtxTypesT>(
        cmd_view_and_progress_result,
        cmd_blocks,
        execution_outcome_fetch,
    )
}

/// Extracts the `ExecutionOutcome` from the intermediate outcome collating
/// types.
///
/// # Parameters
///
/// * `cmd_view_and_progress_result`: The command context, progress, and maybe
///   error.
/// * `cmd_blocks`: `CmdBlock`s in this execution, used to build a useful error
///   message if needed.
/// * `execution_outcome_fetch`: Logic to extract the `ExecutionOutcome` type.
fn outcome_extract<'types: 'view, 'view, 'view_ref, ExecutionOutcome, CmdCtxTypesT>(
    cmd_view_and_progress_result: Result<
        CmdViewAndProgress<'view, 'view_ref, CmdCtxTypesT>,
        CmdBlockStreamBreak<'view, 'view_ref, ExecutionOutcome, CmdCtxTypesT>,
    >,
    cmd_blocks: &'view_ref VecDeque<CmdBlockRtBox<'types, CmdCtxTypesT, ExecutionOutcome>>,
    execution_outcome_fetch: &mut fn(&mut Resources<SetUp>) -> Option<ExecutionOutcome>,
) -> Result<
    CmdOutcome<ExecutionOutcome, <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
    <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError,
>
where
    ExecutionOutcome: Debug + Send + Sync + Unpin + 'static,
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    let (cmd_view_and_progress, cmd_block_index_and_error, cmd_block_index_next) =
        match cmd_view_and_progress_result {
            Ok(cmd_view_and_progress) => (cmd_view_and_progress, None, None),
            Err(cmd_block_stream_break) => match cmd_block_stream_break {
                CmdBlockStreamBreak::Interrupt {
                    cmd_view_and_progress,
                    cmd_block_index_next,
                    interrupt_signal: InterruptSignal,
                } => (cmd_view_and_progress, None, Some(cmd_block_index_next)),
                CmdBlockStreamBreak::BlockErr(CmdViewAndErr {
                    cmd_view_and_progress,
                    cmd_block_index,
                    cmd_block_error,
                }) => (
                    cmd_view_and_progress,
                    Some((cmd_block_index, cmd_block_error)),
                    Some(cmd_block_index),
                ),
            },
        };

    let CmdViewAndProgress {
        cmd_view: SingleProfileSingleFlowView {
            flow, resources, ..
        },
        #[cfg(feature = "output_progress")]
        cmd_progress_tx,
    } = cmd_view_and_progress;

    #[cfg(feature = "output_progress")]
    drop(cmd_progress_tx);

    if let Some((cmd_block_index, cmd_block_error)) = cmd_block_index_and_error {
        match cmd_block_error {
            CmdBlockError::InputFetch(resource_fetch_error) => {
                Err(<CmdCtxTypesT as CmdCtxTypesConstrained>::AppError::from(
                    peace_rt_model::Error::from(CmdExecutionErrorBuilder::build::<_, _, _>(
                        cmd_blocks.iter(),
                        cmd_block_index,
                        resource_fetch_error,
                    )),
                ))
            }
            CmdBlockError::Exec(error) => Err(error),
            CmdBlockError::Interrupt { stream_outcome } => {
                let item_stream_outcome = ItemStreamOutcomeMapper::map(flow, stream_outcome);
                let cmd_blocks_processed = cmd_blocks
                    .range(0..cmd_block_index)
                    .map(|cmd_block_rt| cmd_block_rt.cmd_block_desc())
                    .collect::<Vec<CmdBlockDesc>>();

                let cmd_blocks_not_processed = cmd_blocks
                    .range(cmd_block_index..)
                    .map(|cmd_block_rt| cmd_block_rt.cmd_block_desc())
                    .collect::<Vec<CmdBlockDesc>>();
                let cmd_outcome = CmdOutcome::BlockInterrupted {
                    item_stream_outcome,
                    cmd_blocks_processed,
                    cmd_blocks_not_processed,
                };

                Ok(cmd_outcome)
            }
            CmdBlockError::ItemError {
                stream_outcome,
                errors,
            } => {
                let item_stream_outcome = ItemStreamOutcomeMapper::map(flow, stream_outcome);
                let cmd_blocks_processed = cmd_blocks
                    .range(0..cmd_block_index)
                    .map(|cmd_block_rt| cmd_block_rt.cmd_block_desc())
                    .collect::<Vec<CmdBlockDesc>>();

                let cmd_blocks_not_processed = cmd_blocks
                    .range(cmd_block_index..)
                    .map(|cmd_block_rt| cmd_block_rt.cmd_block_desc())
                    .collect::<Vec<CmdBlockDesc>>();

                let cmd_outcome = CmdOutcome::ItemError {
                    item_stream_outcome,
                    cmd_blocks_processed,
                    cmd_blocks_not_processed,
                    errors,
                };

                Ok(cmd_outcome)
            }
        }
    } else {
        let execution_outcome = execution_outcome_fetch(resources);
        let cmd_outcome = if let Some(cmd_block_index_next) = cmd_block_index_next {
            let cmd_blocks_processed = cmd_blocks
                .range(0..cmd_block_index_next)
                .map(|cmd_block_rt| cmd_block_rt.cmd_block_desc())
                .collect::<Vec<CmdBlockDesc>>();

            let cmd_blocks_not_processed = cmd_blocks
                .range(cmd_block_index_next..)
                .map(|cmd_block_rt| cmd_block_rt.cmd_block_desc())
                .collect::<Vec<CmdBlockDesc>>();

            CmdOutcome::ExecutionInterrupted {
                value: execution_outcome,
                cmd_blocks_processed,
                cmd_blocks_not_processed,
            }
        } else {
            let cmd_blocks_processed = cmd_blocks
                .iter()
                .map(|cmd_block_rt| cmd_block_rt.cmd_block_desc())
                .collect::<Vec<CmdBlockDesc>>();

            CmdOutcome::Complete {
                value: execution_outcome.unwrap_or_else(|| {
                    let execution_outcome_type_name = tynm::type_name::<ExecutionOutcome>();
                    panic!(
                        "Expected `{execution_outcome_type_name}` to exist in `Resources`.\n\
                        Make sure the final `CmdBlock` has that type as its `Outcome`.\n\
                        \n\
                        You may wish to call `CmdExecutionBuilder::with_execution_outcome_fetch`\n\
                        to specify how to fetch the `ExecutionOutcome`."
                    );
                }),
                cmd_blocks_processed,
            }
        };
        Ok(cmd_outcome)
    }
}

struct CmdViewAndProgress<'view, 'view_ref, CmdCtxTypesT>
where
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    cmd_view: &'view_ref mut SingleProfileSingleFlowView<'view, CmdCtxTypesT>,
    #[cfg(feature = "output_progress")]
    cmd_progress_tx: Sender<CmdProgressUpdate>,
}

/// Reasons to stop processing `CmdBlock`s.
enum CmdBlockStreamBreak<'view, 'view_ref, ExecutionOutcome, CmdCtxTypesT>
where
    ExecutionOutcome: Debug,
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    /// An interruption happened between `CmdBlock` executions.
    Interrupt {
        cmd_view_and_progress: CmdViewAndProgress<'view, 'view_ref, CmdCtxTypesT>,
        /// Index of the next `CmdBlock` that hasn't been processed.
        cmd_block_index_next: usize,
        interrupt_signal: InterruptSignal,
    },
    /// A `CmdBlockError` was returned from `CmdBlockRt::exec`.
    BlockErr(CmdViewAndErr<'view, 'view_ref, ExecutionOutcome, CmdCtxTypesT>),
}

struct CmdViewAndErr<'view, 'view_ref, ExecutionOutcome, CmdCtxTypesT>
where
    ExecutionOutcome: Debug,
    CmdCtxTypesT: CmdCtxTypesConstrained,
{
    cmd_view_and_progress: CmdViewAndProgress<'view, 'view_ref, CmdCtxTypesT>,
    /// Index of the `CmdBlock` that erred.
    cmd_block_index: usize,
    cmd_block_error:
        CmdBlockError<ExecutionOutcome, <CmdCtxTypesT as CmdCtxTypesConstrained>::AppError>,
}
