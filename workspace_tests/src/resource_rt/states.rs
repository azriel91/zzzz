use peace::{
    item_model::item_id,
    resource_rt::{internal::StatesMut, states::StatesCurrent, type_reg::untagged::TypeMap},
};

mod states_cleaned;
mod states_cleaned_dry;
mod states_current;
mod states_current_stored;
mod states_ensured;
mod states_ensured_dry;
mod states_goal;
mod states_goal_stored;
mod states_previous;
mod ts;

#[test]
fn with_capacity_reserves_enough_capacity() {
    let states = StatesCurrent::with_capacity(100);
    assert!(states.capacity() >= 100);
}

#[test]
fn into_inner() {
    let states = test_states();

    let type_map = states.into_inner();

    assert_eq!(1, type_map.len())
}

#[test]
fn deref() {
    let states = StatesCurrent::new();

    // deref
    assert_eq!(0, states.len())
}

#[test]
fn from_type_map() {
    let _states = StatesCurrent::from(TypeMap::new_typed());
}

#[test]
fn from_states_mut() {
    let _states = StatesCurrent::from(StatesMut::new());
}

#[test]
fn clone() {
    let states = Clone::clone(&test_states());

    assert_eq!(Some(123), states.get::<i32, _>(&item_id!("key")).copied());
}

#[test]
fn debug() {
    let states = test_states();

    assert_eq!(
        r#"States({ItemId("key"): TypedValue { type: "i32", value: 123 }}, PhantomData<peace_resource_rt::states::ts::Current>)"#,
        format!("{states:?}")
    );
}

fn test_states() -> StatesCurrent {
    let mut states = StatesMut::new();
    states.insert(item_id!("key"), 123i32);

    StatesCurrent::from(states)
}
