use std::{any::TypeId, fmt::Debug};

use fn_graph::{
    resman::{BorrowFail, RefMut},
    DataAccess, DataAccessDyn, Resources, TypeIds,
};
use peace_item_model::ItemId;

use crate::Data;

/// A mutable resource that may or may not exist.
///
/// For an immutable version of this, see [`RMaybe`].
///
/// [`RMaybe`]: crate::RMaybe
#[derive(Debug, PartialEq)]
pub struct WMaybe<'borrow, T>(Option<RefMut<'borrow, T>>)
where
    T: Debug + Send + Sync + 'static;

impl<'borrow, T> From<Option<RefMut<'borrow, T>>> for WMaybe<'borrow, T>
where
    T: Debug + Send + Sync + 'static,
{
    fn from(opt: Option<RefMut<'borrow, T>>) -> Self {
        Self(opt)
    }
}

impl<'borrow, T> std::ops::Deref for WMaybe<'borrow, T>
where
    T: Debug + Send + Sync + 'static,
{
    type Target = Option<RefMut<'borrow, T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for WMaybe<'_, T>
where
    T: Debug + Send + Sync + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'borrow, T> Data<'borrow> for WMaybe<'borrow, T>
where
    T: Debug + Send + Sync + 'static,
{
    fn borrow(_item_id: &'borrow ItemId, resources: &'borrow Resources) -> Self {
        resources
            .try_borrow_mut::<T>()
            .map_err(|borrow_fail| match borrow_fail {
                e @ BorrowFail::ValueNotFound => e,
                BorrowFail::BorrowConflictImm | BorrowFail::BorrowConflictMut => {
                    panic!("Encountered {borrow_fail:?}")
                }
            })
            .ok()
            .into()
    }
}

impl<T> DataAccess for WMaybe<'_, T>
where
    T: Debug + Send + Sync + 'static,
{
    fn borrows() -> TypeIds
    where
        Self: Sized,
    {
        TypeIds::new()
    }

    fn borrow_muts() -> TypeIds
    where
        Self: Sized,
    {
        let mut type_ids = TypeIds::new();
        type_ids.push(TypeId::of::<T>());
        type_ids
    }
}

impl<T> DataAccessDyn for WMaybe<'_, T>
where
    T: Debug + Send + Sync + 'static,
{
    fn borrows(&self) -> TypeIds
    where
        Self: Sized,
    {
        TypeIds::new()
    }

    fn borrow_muts(&self) -> TypeIds
    where
        Self: Sized,
    {
        let mut type_ids = TypeIds::new();
        type_ids.push(TypeId::of::<T>());
        type_ids
    }
}
