use std::any::TypeId;

use peace::{
    data::{accessors::WMaybe, Data, DataAccess, DataAccessDyn, Resources, TypeIds},
    item_model::{item_id, ItemId},
};

const ITEM_SPEC_ID_UNUSED: &ItemId = &item_id!("test_item_id");

#[test]
fn data_borrow_returns_t_when_present() {
    let mut resources = Resources::new();
    resources.insert(1u8);

    let maybe_mut = <WMaybe<'_, u8> as Data>::borrow(ITEM_SPEC_ID_UNUSED, &resources);

    assert_eq!(Some(1u8), maybe_mut.as_deref().copied())
}

#[test]
fn data_borrow_returns_none_when_absent() {
    let resources = Resources::new();

    let maybe_mut = <WMaybe<'_, u8> as Data>::borrow(ITEM_SPEC_ID_UNUSED, &resources);

    assert_eq!(None, maybe_mut.as_deref().copied())
}

#[test]
fn data_access_borrows_returns_nothing() {
    let type_ids_actual = <WMaybe<'_, u8> as DataAccess>::borrows();
    let type_ids_expected = TypeIds::new();

    assert_eq!(type_ids_expected, type_ids_actual)
}

#[test]
fn data_access_borrow_muts_returns_t() {
    let type_ids_actual = <WMaybe<'_, u8> as DataAccess>::borrow_muts();
    let mut type_ids_expected = TypeIds::new();
    type_ids_expected.push(TypeId::of::<u8>());

    assert_eq!(type_ids_expected, type_ids_actual)
}

#[test]
fn data_access_dyn_borrows_returns_nothing() {
    let type_ids_actual = <WMaybe<'_, u8> as DataAccessDyn>::borrows(&WMaybe::from(None));
    let type_ids_expected = TypeIds::new();

    assert_eq!(type_ids_expected, type_ids_actual)
}

#[test]
fn data_access_dyn_borrow_muts_returns_t() {
    let type_ids_actual = <WMaybe<'_, u8> as DataAccessDyn>::borrow_muts(&WMaybe::from(None));
    let mut type_ids_expected = TypeIds::new();
    type_ids_expected.push(TypeId::of::<u8>());

    assert_eq!(type_ids_expected, type_ids_actual)
}

#[test]
fn debug() {
    let mut resources = Resources::new();
    resources.insert(1u8);
    let maybe_mut = <WMaybe<'_, u8> as Data>::borrow(ITEM_SPEC_ID_UNUSED, &resources);

    assert_eq!(
        r#"WMaybe(Some(RefMut { inner: 1 }))"#,
        format!("{maybe_mut:?}")
    )
}

#[test]
fn partial_eq() {
    let mut resources = Resources::new();
    resources.insert(1u8);
    let maybe_mut_0 = <WMaybe<'_, u8> as Data>::borrow(ITEM_SPEC_ID_UNUSED, &resources);

    let mut resources = Resources::new();
    resources.insert(1u8);
    let maybe_mut_1 = <WMaybe<'_, u8> as Data>::borrow(ITEM_SPEC_ID_UNUSED, &resources);

    assert_eq!(maybe_mut_0, maybe_mut_1)
}
