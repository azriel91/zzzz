use peace_fmt::Presentable;
use peace_rt_model_core::{async_trait, output::OutputWrite};

use crate::Error;

cfg_if::cfg_if! {
    if #[cfg(feature = "output_progress")] {
        use peace_item_interaction_model::ItemLocationState;
        use peace_item_model::ItemId;
        use peace_progress_model::{
            CmdBlockItemInteractionType,
            ProgressTracker,
            ProgressUpdateAndId,
        };

        use crate::CmdProgressTracker;
    }
}

/// An `OutputWrite` implementation that writes to the command line.
///
/// Currently this only outputs return values or errors, not progress.
#[derive(Debug, Default)]
pub struct InMemoryTextOutput {
    /// Buffer to write to.
    buffer: String,
}

impl InMemoryTextOutput {
    /// Returns a new `InMemoryTextOutput`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the inner buffer.
    pub fn into_inner(self) -> String {
        self.buffer
    }
}

/// Simple serialization implementations for now.
///
/// See <https://github.com/azriel91/peace/issues/28> for further improvements.
#[async_trait(?Send)]
impl<E> OutputWrite<E> for InMemoryTextOutput
where
    E: std::error::Error + From<Error>,
{
    #[cfg(feature = "output_progress")]
    async fn progress_begin(&mut self, _cmd_progress_tracker: &CmdProgressTracker) {}

    #[cfg(feature = "output_progress")]
    async fn progress_update(
        &mut self,
        _progress_tracker: &ProgressTracker,
        _progress_update_and_id: &ProgressUpdateAndId,
    ) {
    }

    #[cfg(feature = "output_progress")]
    async fn cmd_block_start(
        &mut self,
        _cmd_block_item_interaction_type: CmdBlockItemInteractionType,
    ) {
    }

    #[cfg(feature = "output_progress")]
    async fn item_location_state(
        &mut self,
        _item_id: ItemId,
        _item_location_state: ItemLocationState,
    ) {
    }

    #[cfg(feature = "output_progress")]
    async fn progress_end(&mut self, _cmd_progress_tracker: &CmdProgressTracker) {}

    async fn present<P>(&mut self, presentable: P) -> Result<(), E>
    where
        P: Presentable,
    {
        self.buffer = serde_yaml::to_string(&presentable).map_err(Error::StatesSerialize)?;

        Ok(())
    }

    async fn write_err(&mut self, error: &E) -> Result<(), E> {
        self.buffer = format!("{error}\n");

        Ok(())
    }
}
