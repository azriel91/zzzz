use peace::{
    flow_rt::ItemGraphBuilder,
    rt_model::{fn_graph::FnGraphBuilder, Error, ItemBoxed},
};

#[test]
fn debug() {
    let builder = ItemGraphBuilder::<Error>::new();

    assert!(format!("{builder:?}").starts_with("ItemGraphBuilder"));
}

#[test]
fn into_inner() {
    let _fn_graph_builder = ItemGraphBuilder::<Error>::new().into_inner();
}

#[test]
fn deref() {
    let builder = ItemGraphBuilder::<Error>::new();

    let _fn_graph_builder: &FnGraphBuilder<ItemBoxed<Error>> = &builder;
}

#[test]
fn deref_mut() {
    let mut builder = ItemGraphBuilder::<Error>::new();

    let _fn_graph_builder: &mut FnGraphBuilder<ItemBoxed<Error>> = &mut builder;
}

#[test]
fn from() {
    let fn_graph_builder = FnGraphBuilder::<ItemBoxed<Error>>::new();
    let _builder = ItemGraphBuilder::<Error>::from(fn_graph_builder);
}
