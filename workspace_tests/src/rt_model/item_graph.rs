use peace::{
    flow_rt::{ItemGraph, ItemGraphBuilder},
    item_model::item_id,
    resource_rt::{
        internal::StatesMut,
        states::{StatesCurrent, StatesSerde},
    },
    rt_model::{fn_graph::FnGraph, ItemBoxed},
};

use crate::{
    mock_item::{MockItem, MockState},
    vec_copy_item::VecCopyState,
    PeaceTestError, VecCopyItem,
};

#[test]
fn clone() {
    let item_graph = ItemGraphBuilder::<PeaceTestError>::new().build();

    let _item_graph = Clone::clone(&item_graph);
}

#[test]
fn debug() {
    let item_graph = ItemGraphBuilder::<PeaceTestError>::new().build();

    assert!(format!("{item_graph:?}").starts_with("ItemGraph"));
}

#[test]
fn partial_eq() {
    let item_graph_0 = {
        let mut item_graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        item_graph_builder.add_fn(VecCopyItem::default().into());
        item_graph_builder.build()
    };
    let item_graph_1 = {
        let mut item_graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        item_graph_builder.add_fn(VecCopyItem::default().into());
        item_graph_builder.build()
    };
    let item_graph_2 = {
        let mut item_graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        item_graph_builder.add_fn(VecCopyItem::new(item_id!("rara")).into());
        item_graph_builder.build()
    };

    assert_eq!(item_graph_0, item_graph_1);
    assert_ne!(item_graph_0, item_graph_2);
}

#[test]
fn into_inner() {
    let _fn_graph = ItemGraphBuilder::<PeaceTestError>::new()
        .build()
        .into_inner();
}

#[test]
fn deref() {
    let item_graph = ItemGraphBuilder::<PeaceTestError>::new().build();

    let _fn_graph: &FnGraph<ItemBoxed<PeaceTestError>> = &item_graph;
}

#[test]
fn deref_mut() {
    let mut item_graph = ItemGraphBuilder::<PeaceTestError>::new().build();

    let _fn_graph: &mut FnGraph<ItemBoxed<PeaceTestError>> = &mut item_graph;
}

#[test]
fn from() {
    let fn_graph = FnGraph::<ItemBoxed<PeaceTestError>>::new();
    let _item_graph = ItemGraph::<PeaceTestError>::from(fn_graph);
}

#[test]
fn states_serde_from_item_graph() {
    let one = item_id!("one");
    let two = item_id!("two");
    let three = item_id!("three");

    let item_graph = {
        let mut item_graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        item_graph_builder.add_fns([
            VecCopyItem::new(one.clone()).into(),
            MockItem::<()>::new(two.clone()).into(),
            MockItem::<()>::new(three.clone()).into(),
        ]);
        item_graph_builder.build()
    };

    let states_serde = StatesSerde::<serde_yaml::Value>::from(&item_graph);

    assert_eq!(Some(None), states_serde.get::<VecCopyState, _>(&one));
    assert_eq!(Some(None), states_serde.get::<MockState, _>(&two));
    assert_eq!(Some(None), states_serde.get::<MockState, _>(&three));

    let mut states_serde_keys = states_serde.keys();
    assert_eq!(Some(&one), states_serde_keys.next());
    assert_eq!(Some(&two), states_serde_keys.next());
    assert_eq!(Some(&three), states_serde_keys.next());
    assert_eq!(None, states_serde_keys.next());
}

#[test]
fn states_serde_from_item_graph_and_states() {
    let one = item_id!("one");
    let two = item_id!("two");
    let three = item_id!("three");

    let item_graph = {
        let mut item_graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        item_graph_builder.add_fns([
            VecCopyItem::new(one.clone()).into(),
            MockItem::<()>::new(two.clone()).into(),
            MockItem::<()>::new(three.clone()).into(),
        ]);
        item_graph_builder.build()
    };
    let states = {
        let mut states_mut = StatesMut::new();
        states_mut.insert(one.clone(), VecCopyState::from(vec![1u8]));
        states_mut.insert(two.clone(), MockState(2u8));

        StatesCurrent::from(states_mut)
    };

    let states_serde = item_graph.states_serde::<serde_yaml::Value, _>(&states);

    assert_eq!(
        Some(Some(&VecCopyState::from(vec![1u8]))),
        states_serde.get::<VecCopyState, _>(&one)
    );
    assert_eq!(
        Some(Some(&MockState(2u8))),
        states_serde.get::<MockState, _>(&two)
    );
    assert_eq!(Some(None), states_serde.get::<MockState, _>(&three));

    let mut states_serde_keys = states_serde.keys();
    assert_eq!(Some(&one), states_serde_keys.next());
    assert_eq!(Some(&two), states_serde_keys.next());
    assert_eq!(Some(&three), states_serde_keys.next());
    assert_eq!(None, states_serde_keys.next());
}
