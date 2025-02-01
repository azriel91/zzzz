use peace::{
    data::fn_graph::{daggy2::Dag, Edge, WouldCycle},
    flow_model::{flow_id, FlowInfo, FlowSpecInfo, GraphInfo, ItemInfo, ItemSpecInfo},
    flow_rt::{Flow, ItemGraph, ItemGraphBuilder},
    item_model::item_id,
};
use peace_items::blank::BlankItem;

use crate::PeaceTestError;

#[test]
fn clone() -> Result<(), Box<dyn std::error::Error>> {
    let flow_info = flow_info()?;

    assert_eq!(flow_info, Clone::clone(&flow_info));
    Ok(())
}

#[test]
fn debug() -> Result<(), Box<dyn std::error::Error>> {
    let flow_info = flow_info()?;

    assert_eq!(
        "FlowInfo { \
            flow_id: FlowId(\"flow_id\"), \
            graph_info: GraphInfo { \
                graph: Dag { graph: Graph { Ty: \"Directed\", node_count: 6, edge_count: 9, edges: (0, 1), (0, 2), (1, 4), (2, 3), (3, 4), (5, 4), (1, 2), (5, 1), (0, 5), node weights: {0: ItemInfo { item_id: ItemId(\"a\") }, 1: ItemInfo { item_id: ItemId(\"b\") }, 2: ItemInfo { item_id: ItemId(\"c\") }, 3: ItemInfo { item_id: ItemId(\"d\") }, 4: ItemInfo { item_id: ItemId(\"e\") }, 5: ItemInfo { item_id: ItemId(\"f\") }}, edge weights: {0: Logic, 1: Logic, 2: Logic, 3: Logic, 4: Logic, 5: Logic, 6: Data, 7: Data, 8: Data} }, cycle_state: DfsSpace { dfs: Dfs { stack: [], discovered: FixedBitSet { data: 0x10, capacity: 0, length: 0 } } } } \
            } \
        }",
        format!("{flow_info:?}")
    );
    Ok(())
}

#[test]
fn serialize() -> Result<(), Box<dyn std::error::Error>> {
    let flow_info = flow_info()?;

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
      - Logic
    - - 0
      - 2
      - Logic
    - - 1
      - 4
      - Logic
    - - 2
      - 3
      - Logic
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
        serde_yaml::to_string(&flow_info)?
    );
    Ok(())
}

#[test]
fn deserialize() -> Result<(), Box<dyn std::error::Error>> {
    let flow_info = flow_info()?;

    assert_eq!(
        flow_info,
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
    - [0, 1, Logic]
    - [0, 2, Logic]
    - [1, 4, Logic]
    - [2, 3, Logic]
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

fn flow_info() -> Result<FlowInfo, WouldCycle<Edge>> {
    let flow = Flow::new(flow_id!("flow_id"), complex_graph()?);
    let FlowSpecInfo {
        flow_id,
        graph_info,
    } = flow.flow_spec_info();

    let mut graph = graph_info.iter_insertion_with_indices().fold(
        Dag::new(),
        |mut graph, (_, item_spec_info)| {
            let ItemSpecInfo { item_id } = item_spec_info;
            let item_info = ItemInfo::new(item_id.clone());
            graph.add_node(item_info);
            graph
        },
    );

    let edges = graph_info
        .raw_edges()
        .iter()
        .map(|e| (e.source(), e.target(), e.weight));

    graph.add_edges(edges).expect(
        "Edges are all directed from the original graph, \
        so this cannot cause a cycle.",
    );

    let graph_info = GraphInfo::new(graph);
    let flow_info = FlowInfo::new(flow_id, graph_info);
    Ok(flow_info)
}

fn complex_graph() -> Result<ItemGraph<PeaceTestError>, WouldCycle<Edge>> {
    // a - b --------- e
    //   \          / /
    //    '-- c - d  /
    //              /
    //   f --------'
    let mut item_graph_builder = ItemGraphBuilder::new();
    let [fn_id_a, fn_id_b, fn_id_c, fn_id_d, fn_id_e, fn_id_f] = item_graph_builder.add_fns([
        BlankItem::<()>::new(item_id!("a")).into(),
        BlankItem::<()>::new(item_id!("b")).into(),
        BlankItem::<()>::new(item_id!("c")).into(),
        BlankItem::<()>::new(item_id!("d")).into(),
        BlankItem::<()>::new(item_id!("e")).into(),
        BlankItem::<()>::new(item_id!("f")).into(),
    ]);
    item_graph_builder.add_logic_edges([
        (fn_id_a, fn_id_b),
        (fn_id_a, fn_id_c),
        (fn_id_b, fn_id_e),
        (fn_id_c, fn_id_d),
        (fn_id_d, fn_id_e),
        (fn_id_f, fn_id_e),
    ])?;
    let item_graph = item_graph_builder.build();
    Ok(item_graph)
}
