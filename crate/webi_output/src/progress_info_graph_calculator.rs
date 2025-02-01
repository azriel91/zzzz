use std::collections::HashMap;

use dot_ix_model::info_graph::InfoGraph;
use peace_flow_rt::Flow;
use peace_item_model::ItemId;
use peace_progress_model::ProgressStatus;

/// Calculates the actual `InfoGraph` for a flow's progress.
#[derive(Debug)]
pub struct ProgressInfoGraphCalculator;

impl ProgressInfoGraphCalculator {
    /// Returns the calculated `InfoGraph`.
    pub fn calculate<E>(
        flow: &Flow<E>,
        item_progress_statuses: &HashMap<ItemId, ProgressStatus>,
    ) -> InfoGraph
    where
        E: 'static,
    {
        let flow_spec_info = flow.flow_spec_info();
        flow_spec_info.to_progress_info_graph_with_statuses(item_progress_statuses)
    }
}
