use peace::{
    item_model::item_id,
    resource_rt::{internal::StateDiffsMut, type_reg::untagged::TypeMap},
};

#[test]
fn with_capacity_reserves_enough_capacity() {
    let state_diffs_mut = StateDiffsMut::with_capacity(100);
    assert!(state_diffs_mut.capacity() >= 100);
}

#[test]
fn into_inner() {
    let state_diffs_mut = test_state_diffs_mut();

    let type_map = state_diffs_mut.into_inner();

    assert_eq!(1, type_map.len())
}

#[test]
fn deref_and_deref_mut() {
    let mut state_diffs_mut = StateDiffsMut::new();

    // deref_mut
    state_diffs_mut.insert(item_id!("key"), 123);

    // deref
    assert_eq!(1, state_diffs_mut.len())
}

#[test]
fn from_type_map() {
    let _state_diffs_mut = StateDiffsMut::from(TypeMap::new_typed());
}

#[test]
fn debug() {
    let state_diffs_mut = test_state_diffs_mut();

    assert_eq!(
        r#"StateDiffsMut({ItemId("key"): TypedValue { type: "i32", value: 123 }})"#,
        format!("{state_diffs_mut:?}")
    );
}

fn test_state_diffs_mut() -> StateDiffsMut {
    let mut state_diffs_mut = StateDiffsMut::new();
    state_diffs_mut.insert(item_id!("key"), 123);

    state_diffs_mut
}
