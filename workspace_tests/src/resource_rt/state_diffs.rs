use peace::{
    item_model::item_id,
    resource_rt::{internal::StateDiffsMut, states::StateDiffs, type_reg::untagged::TypeMap},
};

#[test]
fn with_capacity_reserves_enough_capacity() {
    let state_diffs = StateDiffs::with_capacity(100);
    assert!(state_diffs.capacity() >= 100);
}

#[test]
fn into_inner() {
    let state_diffs = test_state_diffs();

    let type_map = state_diffs.into_inner();

    assert_eq!(1, type_map.len())
}

#[test]
fn deref() {
    let state_diffs = StateDiffs::new();

    // deref
    assert_eq!(0, state_diffs.len())
}

#[test]
fn from_type_map() {
    let _state_diffs = StateDiffs::from(TypeMap::new_typed());
}

#[test]
fn from_state_diffs_mut() {
    let _state_diffs = StateDiffs::from(StateDiffsMut::new());
}

#[test]
fn debug() {
    let state_diffs = test_state_diffs();

    assert_eq!(
        r#"StateDiffs({ItemId("key"): TypedValue { type: "i32", value: 123 }})"#,
        format!("{state_diffs:?}")
    );
}

fn test_state_diffs() -> StateDiffs {
    let mut state_diffs = StateDiffsMut::new();
    state_diffs.insert(item_id!("key"), 123);

    StateDiffs::from(state_diffs)
}
