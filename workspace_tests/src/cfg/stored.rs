use std::any::TypeId;

use peace::{
    cfg::accessors::Stored,
    data::{fn_graph::Resources, Data, DataAccess, DataAccessDyn, TypeIds},
    item_model::{item_id, ItemId},
    resource_rt::{internal::StatesMut, states::StatesCurrentStored},
};

const ITEM_SPEC_ID_TEST: &ItemId = &item_id!("item_id_test");
const ITEM_SPEC_ID_OTHER: &ItemId = &item_id!("item_id_other");

#[test]
fn retrieves_state_for_item() {
    let mut resources = Resources::new();
    let states_current_stored = {
        let mut states_mut = StatesMut::new();
        states_mut.insert(ITEM_SPEC_ID_TEST.clone(), 123u8);

        StatesCurrentStored::from(states_mut)
    };
    resources.insert(states_current_stored);

    let stored = Stored::<'_, u8>::borrow(ITEM_SPEC_ID_TEST, &resources);

    assert_eq!(Some(123), stored.get().copied());
}

#[test]
fn does_not_retrieve_state_for_item_other() {
    let mut resources = Resources::new();
    let states_current_stored = {
        let mut states_mut = StatesMut::new();
        states_mut.insert(ITEM_SPEC_ID_OTHER.clone(), 123u8);

        StatesCurrentStored::from(states_mut)
    };
    resources.insert(states_current_stored);

    let stored = Stored::<'_, u8>::borrow(ITEM_SPEC_ID_TEST, &resources);

    assert_eq!(None, stored.get().copied());
}

#[test]
fn data_access_borrows_returns_states_current_stored_type_id() {
    let mut type_ids = TypeIds::new();
    type_ids.push(TypeId::of::<StatesCurrentStored>());

    assert_eq!(type_ids, <Stored::<'_, u8> as DataAccess>::borrows());
}

#[test]
fn data_access_borrow_muts_is_empty() {
    let type_ids = TypeIds::new();

    assert_eq!(type_ids, <Stored::<'_, u8> as DataAccess>::borrow_muts());
}

#[test]
fn data_access_dyn_borrows_returns_states_current_stored_type_id() {
    let mut resources = Resources::new();
    let states_current_stored = StatesCurrentStored::from(StatesMut::new());
    resources.insert(states_current_stored);
    let stored = Stored::<'_, u8>::borrow(ITEM_SPEC_ID_TEST, &resources);

    let mut type_ids = TypeIds::new();
    type_ids.push(TypeId::of::<StatesCurrentStored>());

    assert_eq!(
        type_ids,
        <Stored::<'_, u8> as DataAccessDyn>::borrows(&stored)
    );
}

#[test]
fn data_access_dyn_borrow_muts_is_empty() {
    let mut resources = Resources::new();
    let states_current_stored = StatesCurrentStored::from(StatesMut::new());
    resources.insert(states_current_stored);
    let stored = Stored::<'_, u8>::borrow(ITEM_SPEC_ID_TEST, &resources);

    let type_ids = TypeIds::new();

    assert_eq!(
        type_ids,
        <Stored::<'_, u8> as DataAccessDyn>::borrow_muts(&stored)
    );
}

#[test]
fn debug() {
    let mut resources = Resources::new();
    let states_current_stored = {
        let mut states_mut = StatesMut::new();
        states_mut.insert(ITEM_SPEC_ID_TEST.clone(), 123u8);

        StatesCurrentStored::from(states_mut)
    };
    resources.insert(states_current_stored);

    let stored = Stored::<'_, u8>::borrow(ITEM_SPEC_ID_TEST, &resources);
    assert_eq!(
        r#"Stored { item_id: ItemId("item_id_test"), states_current_stored: Some(Ref { inner: States({ItemId("item_id_test"): TypedValue { type: "u8", value: 123 }}, PhantomData<peace_resource_rt::states::ts::CurrentStored>) }), marker: PhantomData<u8> }"#,
        format!("{stored:?}")
    );

    let stored = Stored::<'_, u8>::borrow(ITEM_SPEC_ID_OTHER, &resources);
    assert_eq!(
        r#"Stored { item_id: ItemId("item_id_other"), states_current_stored: Some(Ref { inner: States({ItemId("item_id_test"): TypedValue { type: "u8", value: 123 }}, PhantomData<peace_resource_rt::states::ts::CurrentStored>) }), marker: PhantomData<u8> }"#,
        format!("{stored:?}")
    );
}
