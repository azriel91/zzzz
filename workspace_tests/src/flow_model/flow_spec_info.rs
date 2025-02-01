use peace::{
    data::fn_graph::{Edge, WouldCycle},
    flow_model::{
        dot_ix::{
            self,
            model::{
                common::{Edges, NodeHierarchy, NodeNames},
                edge_id,
                info_graph::{GraphDir, GraphStyle, InfoGraph},
                node_id,
            },
        },
        flow_id, FlowSpecInfo,
    },
    flow_rt::{Flow, ItemGraph, ItemGraphBuilder},
    item_model::item_id,
};
use peace_items::blank::BlankItem;

use crate::PeaceTestError;

#[test]
fn to_progress_info_graph() -> Result<(), Box<dyn std::error::Error>> {
    let flow_spec_info = flow_spec_info()?;

    let info_graph = flow_spec_info.to_progress_info_graph();

    let info_graph_expected = {
        let mut node_hierarchy = NodeHierarchy::new();
        node_hierarchy.insert(node_id!("a"), NodeHierarchy::new());
        node_hierarchy.insert(node_id!("b"), NodeHierarchy::new());
        node_hierarchy.insert(node_id!("c"), NodeHierarchy::new());
        node_hierarchy.insert(node_id!("d"), NodeHierarchy::new());
        node_hierarchy.insert(node_id!("e"), NodeHierarchy::new());
        node_hierarchy.insert(node_id!("f"), NodeHierarchy::new());

        let mut edges = Edges::new();
        edges.insert(edge_id!("a__b"), [node_id!("a"), node_id!("b")]);
        edges.insert(edge_id!("a__c"), [node_id!("a"), node_id!("c")]);
        edges.insert(edge_id!("b__e"), [node_id!("b"), node_id!("e")]);
        edges.insert(edge_id!("c__d"), [node_id!("c"), node_id!("d")]);
        edges.insert(edge_id!("d__e"), [node_id!("d"), node_id!("e")]);
        edges.insert(edge_id!("f__e"), [node_id!("f"), node_id!("e")]);

        let mut node_names = NodeNames::new();
        node_names.insert(node_id!("a"), String::from("a"));
        node_names.insert(node_id!("b"), String::from("b"));
        node_names.insert(node_id!("c"), String::from("c"));
        node_names.insert(node_id!("d"), String::from("d"));
        node_names.insert(node_id!("e"), String::from("e"));
        node_names.insert(node_id!("f"), String::from("f"));

        InfoGraph::default()
            .with_graph_style(GraphStyle::Circle)
            .with_direction(GraphDir::Vertical)
            .with_hierarchy(node_hierarchy)
            .with_node_names(node_names)
            .with_edges(edges)
    };

    assert_eq!(info_graph_expected, info_graph);
    Ok(())
}

#[test]
fn clone() -> Result<(), Box<dyn std::error::Error>> {
    let flow_spec_info = flow_spec_info()?;

    assert_eq!(flow_spec_info, Clone::clone(&flow_spec_info));
    Ok(())
}

#[test]
fn debug() -> Result<(), Box<dyn std::error::Error>> {
    let flow_spec_info = flow_spec_info()?;

    assert_eq!(
        "FlowSpecInfo { \
            flow_id: FlowId(\"flow_id\"), \
            graph_info: GraphInfo { \
                graph: Dag { graph: Graph { Ty: \"Directed\", node_count: 6, edge_count: 9, edges: (0, 1), (0, 2), (1, 4), (2, 3), (3, 4), (5, 4), (1, 2), (5, 1), (0, 5), node weights: {0: ItemSpecInfo { item_id: ItemId(\"a\") }, 1: ItemSpecInfo { item_id: ItemId(\"b\") }, 2: ItemSpecInfo { item_id: ItemId(\"c\") }, 3: ItemSpecInfo { item_id: ItemId(\"d\") }, 4: ItemSpecInfo { item_id: ItemId(\"e\") }, 5: ItemSpecInfo { item_id: ItemId(\"f\") }}, edge weights: {0: Contains, 1: Logic, 2: Logic, 3: Contains, 4: Logic, 5: Logic, 6: Data, 7: Data, 8: Data} }, cycle_state: DfsSpace { dfs: Dfs { stack: [], discovered: FixedBitSet { data: 0x10, capacity: 0, length: 0 } } } } \
            } \
        }",
        format!("{flow_spec_info:?}")
    );
    Ok(())
}

#[test]
fn serialize() -> Result<(), Box<dyn std::error::Error>> {
    let flow_spec_info = flow_spec_info()?;

    assert_eq!(
        r#"flow_id: flow_id
graph_info:
  graph:
    nodes:
    - item_id: a
    - item_id: b
    - item_id: c
    - item_id: d
    - item_id: e
    - item_id: f
    node_holes: []
    edge_property: directed
    edges:
    - - 0
      - 1
      - Contains
    - - 0
      - 2
      - Logic
    - - 1
      - 4
      - Logic
    - - 2
      - 3
      - Contains
    - - 3
      - 4
      - Logic
    - - 5
      - 4
      - Logic
    - - 1
      - 2
      - Data
    - - 5
      - 1
      - Data
    - - 0
      - 5
      - Data
"#,
        serde_yaml::to_string(&flow_spec_info)?
    );
    Ok(())
}

#[test]
fn deserialize() -> Result<(), Box<dyn std::error::Error>> {
    let flow_spec_info = flow_spec_info()?;

    assert_eq!(
        flow_spec_info,
        serde_yaml::from_str(
            r#"flow_id: flow_id
graph_info:
  graph:
    nodes:
    - item_id: a
    - item_id: b
    - item_id: c
    - item_id: d
    - item_id: e
    - item_id: f
    node_holes: []
    edge_property: directed
    edges:
    - [0, 1, Contains]
    - [0, 2, Logic]
    - [1, 4, Logic]
    - [2, 3, Contains]
    - [3, 4, Logic]
    - [5, 4, Logic]
    - [1, 2, Data]
    - [5, 1, Data]
    - [0, 5, Data]
"#
        )?
    );
    Ok(())
}

fn flow_spec_info() -> Result<FlowSpecInfo, WouldCycle<Edge>> {
    let flow_spec_info: FlowSpecInfo = {
        let flow = Flow::new(flow_id!("flow_id"), complex_graph()?);
        flow.flow_spec_info()
    };
    Ok(flow_spec_info)
}

fn complex_graph() -> Result<ItemGraph<PeaceTestError>, WouldCycle<Edge>> {
    // Progress:
    //
    // ```text
    // a - b --------- e
    //   \          / /
    //    '-- c - d  /
    //              /
    //   f --------'
    // ```
    //
    // Outcome:
    //
    // ```text
    // .-a---.     .-e-.
    // |.-b-.|     '---'
    // |'---'| .-c---.
    // '-----' |.-d-.|
    //         |'---'|
    // .-f-.   '-----'
    // '---'
    // ```
    let mut item_graph_builder = ItemGraphBuilder::new();
    let [fn_id_a, fn_id_b, fn_id_c, fn_id_d, fn_id_e, fn_id_f] = item_graph_builder.add_fns([
        BlankItem::<()>::new(item_id!("a")).into(),
        BlankItem::<()>::new(item_id!("b")).into(),
        BlankItem::<()>::new(item_id!("c")).into(),
        BlankItem::<()>::new(item_id!("d")).into(),
        BlankItem::<()>::new(item_id!("e")).into(),
        BlankItem::<()>::new(item_id!("f")).into(),
    ]);
    item_graph_builder.add_contains_edge(fn_id_a, fn_id_b)?;
    item_graph_builder.add_logic_edge(fn_id_a, fn_id_c)?;
    item_graph_builder.add_logic_edge(fn_id_b, fn_id_e)?;
    item_graph_builder.add_contains_edge(fn_id_c, fn_id_d)?;
    item_graph_builder.add_logic_edge(fn_id_d, fn_id_e)?;
    item_graph_builder.add_logic_edge(fn_id_f, fn_id_e)?;

    let item_graph = item_graph_builder.build();
    Ok(item_graph)
}
