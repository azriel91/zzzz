use dot_ix::model::{
    common::{EdgeId, Edges, NodeHierarchy, NodeId, NodeNames},
    info_graph::{GraphDir, GraphStyle, InfoGraph},
};
use fn_graph::{daggy2::Walker, Edge, GraphInfo};
use serde::{Deserialize, Serialize};

use crate::{FlowId, ItemSpecInfo};

cfg_if::cfg_if! {
    if #[cfg(feature = "output_progress")] {
        use std::collections::HashMap;

        use dot_ix::model::{
            common::AnyId,
            theme::{AnyIdOrDefaults, CssClassPartials, Theme, ThemeAttr},
        };
        use peace_item_model::ItemId;
        use peace_progress_model::{ProgressComplete, ProgressStatus};
    }
}

/// Serializable representation of how a [`Flow`] is configured.
///
/// [`Flow`]: https://docs.rs/peace_rt_model/latest/peace_rt_model/struct.Flow.html
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct FlowSpecInfo {
    /// ID of the flow.
    pub flow_id: FlowId,
    /// Serialized representation of the flow graph.
    pub graph_info: GraphInfo<ItemSpecInfo>,
}

impl FlowSpecInfo {
    /// Returns a new `FlowSpecInfo`.
    pub fn new(flow_id: FlowId, graph_info: GraphInfo<ItemSpecInfo>) -> Self {
        Self {
            flow_id,
            graph_info,
        }
    }

    /// Returns an [`InfoGraph`] that represents the progress of the flow's
    /// execution.
    pub fn to_progress_info_graph(&self) -> InfoGraph {
        self.to_progress_info_graph_internal(
            #[cfg(feature = "output_progress")]
            &HashMap::new(),
        )
    }

    /// Returns an [`InfoGraph`] that represents the progress of the flow's
    /// execution.
    #[cfg(feature = "output_progress")]
    pub fn to_progress_info_graph_with_statuses(
        &self,
        item_progress_statuses: &HashMap<ItemId, ProgressStatus>,
    ) -> InfoGraph {
        self.to_progress_info_graph_internal(item_progress_statuses)
    }

    fn to_progress_info_graph_internal(
        &self,
        #[cfg(feature = "output_progress")] item_progress_statuses: &HashMap<
            ItemId,
            ProgressStatus,
        >,
    ) -> InfoGraph {
        let graph_info = &self.graph_info;
        let item_count = graph_info.node_count();

        let hierarchy = graph_info.iter_insertion_with_indices().fold(
            NodeHierarchy::with_capacity(item_count),
            |mut hierarchy, (_node_index, item_spec_info)| {
                let node_id = item_spec_info_to_node_id(item_spec_info);
                // Progress nodes have no nested nodes.
                hierarchy.insert(node_id, NodeHierarchy::new());
                hierarchy
            },
        );

        let edges = progress_node_edges(graph_info);
        let node_names = node_names(graph_info);

        let info_graph = InfoGraph::default()
            .with_graph_style(GraphStyle::Circle)
            .with_direction(GraphDir::Vertical)
            .with_hierarchy(hierarchy)
            .with_edges(edges)
            .with_node_names(node_names);

        #[cfg(feature = "output_progress")]
        {
            if !item_progress_statuses.is_empty() {
                let theme = graph_info.iter_insertion_with_indices().fold(
                    Theme::new(),
                    |mut theme, (_node_index, item_spec_info)| {
                        let item_id = &item_spec_info.item_id;

                        if let Some(progress_status) = item_progress_statuses.get(item_id) {
                            let css_class_partials =
                                item_progress_css_class_partials(progress_status);

                            let any_id = AnyId::try_from(item_id.as_str().to_string()).expect(
                                "Expected `peace` `ItemId`s to be valid `dot_ix` `AnyId`s.`",
                            );

                            if !css_class_partials.is_empty() {
                                theme
                                    .styles
                                    .insert(AnyIdOrDefaults::AnyId(any_id), css_class_partials);
                            }
                        }
                        theme
                    },
                );

                return info_graph.with_theme(theme).with_css(String::from(
                    r#"
@keyframes ellipse-stroke-dashoffset-move {
  0%   { stroke-dashoffset: 30; }
  100% { stroke-dashoffset: 0; }
}
"#,
                ));
            }
        }

        info_graph
    }
}

#[cfg(feature = "output_progress")]
fn item_progress_css_class_partials(progress_status: &ProgressStatus) -> CssClassPartials {
    let mut css_class_partials = CssClassPartials::with_capacity(4);

    match progress_status {
        ProgressStatus::Initialized => {}
        ProgressStatus::Interrupted => {
            css_class_partials.insert(ThemeAttr::ShapeColor, "yellow".to_string());
        }
        ProgressStatus::ExecPending | ProgressStatus::Queued => {
            css_class_partials.insert(ThemeAttr::ShapeColor, "indigo".to_string());
        }
        ProgressStatus::Running => {
            css_class_partials.insert(ThemeAttr::StrokeStyle, "dashed".to_string());
            css_class_partials.insert(ThemeAttr::StrokeWidth, "[2px]".to_string());
            css_class_partials.insert(ThemeAttr::ShapeColor, "blue".to_string());
            css_class_partials.insert(
                ThemeAttr::Animate,
                "[ellipse-stroke-dashoffset-move_1s_linear_infinite]".to_string(),
            );
        }
        ProgressStatus::RunningStalled => {
            css_class_partials.insert(ThemeAttr::ShapeColor, "amber".to_string());
        }
        ProgressStatus::UserPending => {
            css_class_partials.insert(ThemeAttr::ShapeColor, "purple".to_string());
        }
        ProgressStatus::Complete(ProgressComplete::Success) => {
            css_class_partials.insert(ThemeAttr::ShapeColor, "green".to_string());
        }
        ProgressStatus::Complete(ProgressComplete::Fail) => {
            css_class_partials.insert(ThemeAttr::ShapeColor, "red".to_string());
        }
    }
    css_class_partials
}

/// Returns the list of edges between items in the graph for progress.
///
/// For progress graphs, an edge is rendered between pairs of predecessor and
/// successor items, regardless of whether their dependency is `Edge::Logic`
/// (adjacent) or `Edge::Contains` (nested).
fn progress_node_edges(graph_info: &GraphInfo<ItemSpecInfo>) -> Edges {
    graph_info.iter_insertion_with_indices().fold(
        Edges::with_capacity(graph_info.node_count()),
        |mut edges, (node_index, item_spec_info)| {
            //
            let children = graph_info.children(node_index);
            children
                .iter(graph_info)
                .filter_map(|(edge_index, child_node_index)| {
                    // For progress graphs, child nodes that:
                    //
                    // * are contained by parents nodes
                    // * reference data from parent nodes
                    //
                    // are both represented by forward edges, since this is their sequential
                    // ordering.
                    if matches!(
                        graph_info.edge_weight(edge_index).copied(),
                        Some(Edge::Logic | Edge::Contains)
                    ) {
                        Some(child_node_index)
                    } else {
                        None
                    }
                })
                .for_each(|child_node_index| {
                    let item_id = item_spec_info_to_node_id(item_spec_info);
                    let child_item_id = item_spec_info_to_node_id(&graph_info[child_node_index]);
                    let edge_id = EdgeId::try_from(format!("{item_id}__{child_item_id}")).expect(
                        "Expected concatenated `peace` `ItemId`s to be valid `dot_ix` `EdgeId`s.",
                    );
                    edges.insert(edge_id, [item_id, child_item_id]);
                });

            edges
        },
    )
}

/// Returns the list of edges between items in the graph.
fn node_names(graph_info: &GraphInfo<ItemSpecInfo>) -> NodeNames {
    graph_info.iter_insertion_with_indices().fold(
        NodeNames::with_capacity(graph_info.node_count()),
        |mut node_names, (_node_index, item_spec_info)| {
            let item_id = item_spec_info_to_node_id(item_spec_info);

            // Note: This does not have to be the ID, it can be a human readable name.
            let node_name = item_id.to_string();

            node_names.insert(item_id, node_name);

            node_names
        },
    )
}

fn item_spec_info_to_node_id(item_spec_info: &ItemSpecInfo) -> NodeId {
    NodeId::try_from(item_spec_info.item_id.to_string())
        .expect("Expected `peace` `ItemId`s to be valid `dot_ix` `NodeId`s.`")
}
