use peace::{cfg::async_trait, fmt::Presentable, rt_model::output::OutputWrite};

cfg_if::cfg_if! {
    if #[cfg(feature = "output_progress")] {
        use peace::{
            item_interaction_model::ItemLocationState,
            item_model::ItemId,
            progress_model::{CmdBlockItemInteractionType, ProgressTracker, ProgressUpdateAndId},
            rt_model::CmdProgressTracker,
        };
    }
}

/// An `OutputWrite` implementation that does nothing.
#[derive(Debug)]
pub struct NoOpOutput;

#[async_trait(?Send)]
impl<E> OutputWrite<E> for NoOpOutput
where
    E: std::error::Error,
{
    #[cfg(feature = "output_progress")]
    async fn progress_begin(&mut self, _cmd_progress_tracker: &CmdProgressTracker) {}

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
    async fn progress_update(
        &mut self,
        _progress_tracker: &ProgressTracker,
        _progress_update_and_id: &ProgressUpdateAndId,
    ) {
    }

    #[cfg(feature = "output_progress")]
    async fn progress_end(&mut self, _cmd_progress_tracker: &CmdProgressTracker) {}

    async fn present<P>(&mut self, _presentable: P) -> Result<(), E>
    where
        P: Presentable,
    {
        Ok(())
    }

    async fn write_err(&mut self, _error: &E) -> Result<(), E> {
        Ok(())
    }
}
