use std::fmt::Debug;

use async_trait::async_trait;

use peace_cmd::{ctx::CmdCtxTypesConstrained, scopes::SingleProfileSingleFlowView};
use peace_cmd_model::CmdBlockDesc;

use crate::CmdBlockError;

cfg_if::cfg_if! {
    if #[cfg(feature = "output_progress")] {
        use peace_progress_model::{CmdBlockItemInteractionType, CmdProgressUpdate};
        use tokio::sync::mpsc::Sender;
    }
}

/// Type erased [`CmdBlock`]
///
/// [`CmdBlock`]: crate::CmdBlock
#[async_trait(?Send)]
pub trait CmdBlockRt: Debug + Unpin {
    /// Type parameters passed to the `CmdCtx`.
    type CmdCtxTypes: CmdCtxTypesConstrained;
    /// Outcome type of the command execution.
    type ExecutionOutcome: Debug + 'static;

    /// Executes this command block.
    async fn exec(
        &self,
        view: &mut SingleProfileSingleFlowView<'_, Self::CmdCtxTypes>,
        #[cfg(feature = "output_progress")] progress_tx: Sender<CmdProgressUpdate>,
    ) -> Result<
        (),
        CmdBlockError<
            Self::ExecutionOutcome,
            <Self::CmdCtxTypes as CmdCtxTypesConstrained>::AppError,
        >,
    >;

    /// Returns the type of interactions the `CmdBlock` has with
    /// `ItemLocation`s.
    #[cfg(feature = "output_progress")]
    fn cmd_block_item_interaction_type(&self) -> CmdBlockItemInteractionType;

    /// Returns the `String` representation of the `CmdBlock` in a
    /// `CmdExecution`.
    ///
    /// This is used to provide a well-formatted error message so that
    /// developers can identify where a bug lies more easily.
    fn cmd_block_desc(&self) -> CmdBlockDesc;
}
