use fn_graph::StreamOutcome;
use peace_cmd_model::ItemStreamOutcome;
use peace_flow_rt::Flow;

/// Maps a `StreamOutcome<T>` to an `ItemStreamOutcome<T>`.
///
/// # Design Note
///
/// This resides in the `cmd_rt` package as the `Flow` type is needed for
/// mapping, and adding `rt_model` as a dependency of `cmd_model` creates a
/// dependency cycle with `rt_model_core`:
///
/// ```text
/// cmd_model -> rt_model
///    ^         /
///   /         v
/// rt_model_core
/// ```
pub struct ItemStreamOutcomeMapper;

impl ItemStreamOutcomeMapper {
    /// Maps `FnId`s into `ItemId`s for a better information abstraction level.
    pub fn map<T, E>(flow: &Flow<E>, stream_outcome: StreamOutcome<T>) -> ItemStreamOutcome<T>
    where
        E: 'static,
    {
        let StreamOutcome {
            value,
            state,
            fn_ids_processed,
            fn_ids_not_processed,
        } = stream_outcome;

        let item_ids_processed = fn_ids_processed
            .into_iter()
            .filter_map(|fn_id| {
                flow.graph()
                    .node_weight(fn_id)
                    .map(|item| item.id())
                    .cloned()
            })
            .collect::<Vec<_>>();
        let item_ids_not_processed = fn_ids_not_processed
            .into_iter()
            .filter_map(|fn_id| {
                flow.graph()
                    .node_weight(fn_id)
                    .map(|item| item.id())
                    .cloned()
            })
            .collect::<Vec<_>>();

        ItemStreamOutcome {
            value,
            state,
            item_ids_processed,
            item_ids_not_processed,
        }
    }
}
