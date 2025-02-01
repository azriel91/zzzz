use peace::{
    item_model::item_id,
    params::{ParamsSpec, ParamsSpecs},
};

use crate::mock_item::MockSrc;

#[test]
fn debug() {
    let mut params_specs = ParamsSpecs::new();
    params_specs.insert(item_id!("item_id"), ParamsSpec::<MockSrc>::InMemory);

    assert_eq!(
        "ParamsSpecs({\
        ItemId(\"item_id\"): \
        TypedValue { \
            type: \"peace_params::params_spec::ParamsSpec<workspace_tests::mock_item::MockSrc>\", \
            value: InMemory \
        }})",
        format!("{params_specs:?}")
    );
}

#[test]
fn into_inner() {
    let mut params_specs = ParamsSpecs::new();
    params_specs.insert(item_id!("item_id"), ParamsSpec::<MockSrc>::InMemory);

    let type_map = params_specs.into_inner();
    let params_spec = type_map.get::<ParamsSpec<MockSrc>, _>(&item_id!("item_id"));

    assert!(matches!(params_spec, Some(ParamsSpec::<MockSrc>::InMemory)));
}
