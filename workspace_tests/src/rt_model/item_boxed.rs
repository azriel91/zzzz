use std::any::TypeId;

use peace::{
    data::{DataAccessDyn, TypeIds},
    item_model::item_id,
    rt_model::{ItemBoxed, ItemRt},
};

use crate::{
    vec_copy_item::{VecA, VecB},
    VecCopyError, VecCopyItem,
};

#[test]
fn deref_to_dyn_item_rt() {
    let vec_copy_item = VecCopyItem::default();
    let item_boxed: ItemBoxed<VecCopyError> = vec_copy_item.clone().into();
    let item_rt: &dyn ItemRt<_> = &*item_boxed;

    assert_eq!(format!("{vec_copy_item:?}"), format!("{item_rt:?}"));
}

#[test]
fn deref_mut_to_dyn_item_rt() {
    let vec_copy_item = VecCopyItem::default();
    let mut item_boxed: ItemBoxed<VecCopyError> = vec_copy_item.clone().into();
    let item_rt: &mut dyn ItemRt<_> = &mut *item_boxed;

    assert_eq!(format!("{vec_copy_item:?}"), format!("{item_rt:?}"));
}

#[test]
fn data_access_dyn_borrows() {
    let mut type_ids = TypeIds::new();
    type_ids.push(TypeId::of::<VecA>());

    let vec_copy_item = VecCopyItem::default();
    let item_boxed: ItemBoxed<VecCopyError> = vec_copy_item.into();

    assert_eq!(
        type_ids,
        <ItemBoxed<VecCopyError> as DataAccessDyn>::borrows(&item_boxed)
    );
}

#[test]
fn data_access_dyn_borrow_muts() {
    let mut type_ids = TypeIds::new();
    type_ids.push(TypeId::of::<VecB>());

    let vec_copy_item = VecCopyItem::default();
    let item_boxed: ItemBoxed<VecCopyError> = vec_copy_item.into();

    assert_eq!(
        type_ids,
        <ItemBoxed<VecCopyError> as DataAccessDyn>::borrow_muts(&item_boxed)
    );
}

#[test]
fn clone() {
    let vec_copy_item = VecCopyItem::default();
    let item_boxed: ItemBoxed<VecCopyError> = vec_copy_item.into();

    let _item_boxed = Clone::clone(&item_boxed);
}

#[test]
fn debug() {
    let vec_copy_item = VecCopyItem::default();
    let item_boxed: ItemBoxed<VecCopyError> = vec_copy_item.into();

    assert_eq!(
        "ItemBoxed(VecCopyItem { id: ItemId(\"vec_copy\") })",
        format!("{item_boxed:?}")
    );
}

#[test]
fn partial_eq() {
    let item_boxed_0: ItemBoxed<VecCopyError> = VecCopyItem::default().into();
    let item_boxed_1: ItemBoxed<VecCopyError> = VecCopyItem::default().into();
    let item_boxed_2: ItemBoxed<VecCopyError> = VecCopyItem::new(item_id!("rara")).into();

    assert_eq!(item_boxed_0, item_boxed_1);
    assert_ne!(item_boxed_0, item_boxed_2);
}
