use peace::{
    item_model::item_id,
    resource_rt::{internal::StatesMut, states::ts::Current, type_reg::untagged::TypeMap},
};

#[test]
fn with_capacity_reserves_enough_capacity() {
    let states = StatesMut::<Current>::with_capacity(100);
    assert!(states.capacity() >= 100);
}

#[test]
fn into_inner() {
    let states = test_states();

    let type_map = states.into_inner();

    assert_eq!(1, type_map.len())
}

#[test]
fn deref_and_deref_mut() {
    let mut states = StatesMut::<Current>::new();

    // deref_mut
    states.insert(item_id!("key"), 123);

    // deref
    assert_eq!(1, states.len())
}

#[test]
fn from_type_map() {
    let _states = StatesMut::<Current>::from(TypeMap::new_typed());
}

#[test]
fn debug() {
    let states = test_states();

    let debug_str = format!("{states:?}");
    assert!(
        debug_str
            == r#"StatesMut({ItemId("key"): TypedValue { type: "i32", value: 123 }}, PhantomData<peace_resource_rt::states::ts::Current>)"#
            || debug_str
                == r#"StatesMut({ItemId("key"): TypedValue { type: "i32", value: 123 }}, PhantomData)"#
    );
}

fn test_states() -> StatesMut<Current> {
    let mut states = StatesMut::<Current>::new();
    states.insert(item_id!("key"), 123);

    states
}
