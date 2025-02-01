use peace_fmt::Presentable;
use peace_rt_model_core::{async_trait, output::OutputWrite};
use peace_value_traits::AppError;
use peace_webi_model::WebUiUpdate;
use tokio::sync::mpsc;

cfg_if::cfg_if! {
    if #[cfg(feature = "output_progress")] {
        use peace_item_model::ItemId;
        use peace_item_interaction_model::ItemLocationState;
        use peace_progress_model::{
            CmdBlockItemInteractionType,
            // ProgressComplete,
            // ProgressLimit,
            // ProgressStatus,
            ProgressTracker,
            // ProgressUpdate,
            ProgressUpdateAndId,
        };
        use peace_rt_model_core::CmdProgressTracker;
    }
}

/// An `OutputWrite` implementation that writes to web elements.
#[derive(Clone, Debug)]
pub struct WebiOutput {
    /// Channel to notify the `CmdExecution` task / `leptos` to update the UI.
    ///
    /// This can be:
    ///
    /// * Progress `InfoGraph` diagram needs to be restyled.
    /// * Outcome `InfoGraph` diagram needs to be restyled.
    /// * Execution result to show to the user.
    web_ui_update_tx: Option<mpsc::Sender<WebUiUpdate>>,
}

impl WebiOutput {
    /// Returns a new `WebiOutput`.
    pub fn new(web_ui_update_tx: mpsc::Sender<WebUiUpdate>) -> Self {
        Self {
            web_ui_update_tx: Some(web_ui_update_tx),
        }
    }

    pub fn clone_without_tx(&self) -> Self {
        Self {
            web_ui_update_tx: None,
        }
    }
}

#[async_trait(?Send)]
impl<AppErrorT> OutputWrite<AppErrorT> for WebiOutput
where
    AppErrorT: AppError,
{
    #[cfg(feature = "output_progress")]
    async fn progress_begin(&mut self, _cmd_progress_tracker: &CmdProgressTracker) {}

    #[cfg(feature = "output_progress")]
    async fn cmd_block_start(
        &mut self,
        cmd_block_item_interaction_type: CmdBlockItemInteractionType,
    ) {
        if let Some(web_ui_update_tx) = self.web_ui_update_tx.as_ref() {
            let _result = web_ui_update_tx
                .send(WebUiUpdate::CmdBlockStart {
                    cmd_block_item_interaction_type,
                })
                .await;
        }
    }

    #[cfg(feature = "output_progress")]
    async fn item_location_state(
        &mut self,
        item_id: ItemId,
        item_location_state: ItemLocationState,
    ) {
        if let Some(web_ui_update_tx) = self.web_ui_update_tx.as_ref() {
            let _result = web_ui_update_tx
                .send(WebUiUpdate::ItemLocationState {
                    item_id,
                    item_location_state,
                })
                .await;
        }
    }

    #[cfg(feature = "output_progress")]
    async fn progress_update(
        &mut self,
        progress_tracker: &ProgressTracker,
        progress_update_and_id: &ProgressUpdateAndId,
    ) {
        let item_id = progress_update_and_id.item_id.clone();
        let progress_status = progress_tracker.progress_status().clone();
        let progress_limit = progress_tracker.progress_limit();
        let message = progress_tracker.message().cloned();

        if let Some(web_ui_update_tx) = self.web_ui_update_tx.as_ref() {
            let _result = web_ui_update_tx
                .send(WebUiUpdate::ItemProgressStatus {
                    item_id,
                    progress_status,
                    progress_limit,
                    message,
                })
                .await;
        }
    }

    #[cfg(feature = "output_progress")]
    async fn progress_end(&mut self, _cmd_progress_tracker: &CmdProgressTracker) {}

    async fn present<P>(&mut self, _presentable: P) -> Result<(), AppErrorT>
    where
        AppErrorT: std::error::Error,
        P: Presentable,
    {
        // TODO: send rendered / renderable markdown to the channel.
        let markdown_src = String::from("TODO: presentable.present(md_presenter).");
        if let Some(web_ui_update_tx) = self.web_ui_update_tx.as_ref() {
            let _result = web_ui_update_tx
                .send(WebUiUpdate::Markdown { markdown_src })
                .await;
        }
        Ok(())
    }

    async fn write_err(&mut self, _error: &AppErrorT) -> Result<(), AppErrorT>
    where
        AppErrorT: std::error::Error,
    {
        todo!()
    }
}
