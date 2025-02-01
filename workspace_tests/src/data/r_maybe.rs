use std::any::TypeId;

use peace::{
    data::{accessors::RMaybe, Data, DataAccess, DataAccessDyn, Resources, TypeIds},
    item_model::{item_id, ItemId},
};

const ITEM_SPEC_ID_UNUSED: &ItemId = &item_id!("test_item_id");

#[test]
fn data_borrow_returns_t_when_present() {
    let mut resources = Resources::new();
    resources.insert(1u8);

    let maybe = <RMaybe<'_, u8> as Data>::borrow(ITEM_SPEC_ID_UNUSED, &resources);

    assert_eq!(Some(1u8), maybe.as_deref().copied())
}

#[test]
fn data_borrow_returns_none_when_absent() {
    let resources = Resources::new();

    let maybe = <RMaybe<'_, u8> as Data>::borrow(ITEM_SPEC_ID_UNUSED, &resources);

    assert_eq!(None, maybe.as_deref().copied())
}

#[test]
fn data_access_borrows_returns_t() {
    let type_ids_actual = <RMaybe<'_, u8> as DataAccess>::borrows();
    let mut type_ids_expected = TypeIds::new();
    type_ids_expected.push(TypeId::of::<u8>());

    assert_eq!(type_ids_expected, type_ids_actual)
}

#[test]
fn data_access_borrow_muts_returns_nothing() {
    let type_ids_actual = <RMaybe<'_, u8> as DataAccess>::borrow_muts();
    let type_ids_expected = TypeIds::new();

    assert_eq!(type_ids_expected, type_ids_actual)
}

#[test]
fn data_access_dyn_borrows_returns_t() {
    let type_ids_actual = <RMaybe<'_, u8> as DataAccessDyn>::borrows(&RMaybe::from(None));
    let mut type_ids_expected = TypeIds::new();
    type_ids_expected.push(TypeId::of::<u8>());

    assert_eq!(type_ids_expected, type_ids_actual)
}

#[test]
fn data_access_dyn_borrow_muts_returns_t() {
    let type_ids_actual = <RMaybe<'_, u8> as DataAccessDyn>::borrow_muts(&RMaybe::from(None));
    let type_ids_expected = TypeIds::new();

    assert_eq!(type_ids_expected, type_ids_actual)
}

#[test]
fn clone() {
    let mut resources = Resources::new();
    resources.insert(1u8);
    let maybe = <RMaybe<'_, u8> as Data>::borrow(ITEM_SPEC_ID_UNUSED, &resources);

    let maybe_clone = maybe.clone();

    assert_eq!(maybe, maybe_clone)
}

#[test]
fn debug() {
    let mut resources = Resources::new();
    resources.insert(1u8);
    let maybe = <RMaybe<'_, u8> as Data>::borrow(ITEM_SPEC_ID_UNUSED, &resources);

    assert_eq!(r#"RMaybe(Some(Ref { inner: 1 }))"#, format!("{maybe:?}"))
}
