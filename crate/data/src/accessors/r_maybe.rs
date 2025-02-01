use std::{any::TypeId, fmt::Debug};

use fn_graph::{
    resman::{BorrowFail, Ref},
    DataAccess, DataAccessDyn, Resources, TypeIds,
};
use peace_item_model::ItemId;

use crate::Data;

/// A resource that may or may not exist.
///
/// For a mutable version of this, see [`WMaybe`].
///
/// [`WMaybe`]: crate::WMaybe
#[derive(Clone, Debug, PartialEq)]
pub struct RMaybe<'borrow, T>(Option<Ref<'borrow, T>>)
where
    T: Debug + Send + Sync + 'static;

impl<'borrow, T> From<Option<Ref<'borrow, T>>> for RMaybe<'borrow, T>
where
    T: Debug + Send + Sync + 'static,
{
    fn from(opt: Option<Ref<'borrow, T>>) -> Self {
        Self(opt)
    }
}

impl<'borrow, T> std::ops::Deref for RMaybe<'borrow, T>
where
    T: Debug + Send + Sync + 'static,
{
    type Target = Option<Ref<'borrow, T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'borrow, T> Data<'borrow> for RMaybe<'borrow, T>
where
    T: Debug + Send + Sync + 'static,
{
    fn borrow(_item_id: &'borrow ItemId, resources: &'borrow Resources) -> Self {
        resources
            .try_borrow::<T>()
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

impl<T> DataAccess for RMaybe<'_, T>
where
    T: Debug + Send + Sync + 'static,
{
    fn borrows() -> TypeIds
    where
        Self: Sized,
    {
        let mut type_ids = TypeIds::new();
        type_ids.push(TypeId::of::<T>());
        type_ids
    }

    fn borrow_muts() -> TypeIds
    where
        Self: Sized,
    {
        TypeIds::new()
    }
}

impl<T> DataAccessDyn for RMaybe<'_, T>
where
    T: Debug + Send + Sync + 'static,
{
    fn borrows(&self) -> TypeIds
    where
        Self: Sized,
    {
        let mut type_ids = TypeIds::new();
        type_ids.push(TypeId::of::<T>());
        type_ids
    }

    fn borrow_muts(&self) -> TypeIds
    where
        Self: Sized,
    {
        TypeIds::new()
    }
}
