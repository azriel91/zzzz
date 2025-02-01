use peace::{
    cfg::async_trait,
    fmt::Presentable,
    rt_model::{self, output::OutputWrite},
};

use crate::FnInvocation;

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

/// An `OutputWrite` implementation that tracks function invocations.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct FnTrackerOutput {
    /// List of function invocations.
    fn_invocations: Vec<FnInvocation>,
}

impl FnTrackerOutput {
    /// Returns a new `FnTrackerOutput`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the recorded function invocations.
    pub fn fn_invocations(&self) -> &[FnInvocation] {
        self.fn_invocations.as_ref()
    }
}

#[async_trait(?Send)]
impl<E> OutputWrite<E> for FnTrackerOutput
where
    E: std::error::Error + From<rt_model::Error>,
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

    async fn present<P>(&mut self, presentable: P) -> Result<(), E>
    where
        P: Presentable,
    {
        let presentable_serialized =
            serde_yaml::to_string(&presentable).map_err(rt_model::Error::PresentableSerialize)?;
        self.fn_invocations.push(FnInvocation::new(
            "present",
            vec![Some(presentable_serialized)],
        ));

        Ok(())
    }

    async fn write_err(&mut self, error: &E) -> Result<(), E> {
        self.fn_invocations.push(FnInvocation::new(
            "write_err",
            vec![Some(format!("{error:?}"))],
        ));
        Ok(())
    }
}
