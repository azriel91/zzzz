use fn_graph::GraphInfo;

use serde::{Deserialize, Serialize};

use crate::{FlowId, ItemInfo};

/// Serializable representation of values in a [`Flow`].
///
/// This includes values passed into, or produced by `Item`s in the `Flow`.
///
/// [`Flow`]: https://docs.rs/peace_rt_model/latest/peace_rt_model/struct.Flow.html
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct FlowInfo {
    /// ID of the flow.
    pub flow_id: FlowId,
    /// Serialized representation of the flow graph.
    pub graph_info: GraphInfo<ItemInfo>,
}

impl FlowInfo {
    /// Returns a new `FlowInfo`.
    pub fn new(flow_id: FlowId, graph_info: GraphInfo<ItemInfo>) -> Self {
        Self {
            flow_id,
            graph_info,
        }
    }
}
