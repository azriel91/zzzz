use std::{
    any::TypeId,
    fmt::{Debug, Display},
    marker::PhantomData,
};

use peace_data::{
    fn_graph::{
        resman::{BorrowFail, Ref},
        DataAccess, DataAccessDyn, Resources, TypeIds,
    },
    Data,
};
use peace_item_model::ItemId;
use peace_resource_rt::{states::StatesCurrentStored, type_reg::untagged::DataType};
use serde::Serialize;

/// The previously stored `T` state, if any.
#[derive(Debug)]
pub struct Stored<'borrow, T> {
    /// ID of the item the state should be retrieved for.
    item_id: &'borrow ItemId,
    /// The borrowed `StatesCurrentStored`.
    states_current_stored: Option<Ref<'borrow, StatesCurrentStored>>,
    /// Marker.
    marker: PhantomData<T>,
}

impl<'borrow, T> Stored<'borrow, T>
where
    T: Clone + Debug + DataType + Display + Serialize + Send + Sync + 'static,
{
    pub fn get(&'borrow self) -> Option<&'borrow T> {
        self.states_current_stored
            .as_ref()
            .and_then(|states_current_stored| states_current_stored.get(self.item_id))
    }
}

impl<'borrow, T> Data<'borrow> for Stored<'borrow, T>
where
    T: Debug + Send + Sync + 'static,
{
    fn borrow(item_id: &'borrow ItemId, resources: &'borrow Resources) -> Self {
        let states_current_stored = resources
            .try_borrow::<StatesCurrentStored>()
            .map_err(|borrow_fail| match borrow_fail {
                e @ BorrowFail::ValueNotFound => e,
                BorrowFail::BorrowConflictImm | BorrowFail::BorrowConflictMut => {
                    panic!("Encountered {borrow_fail:?}")
                }
            })
            .ok();

        Self {
            item_id,
            states_current_stored,
            marker: PhantomData,
        }
    }
}

impl<T> DataAccess for Stored<'_, T> {
    fn borrows() -> TypeIds
    where
        Self: Sized,
    {
        let mut type_ids = TypeIds::new();
        type_ids.push(TypeId::of::<StatesCurrentStored>());
        type_ids
    }

    fn borrow_muts() -> TypeIds
    where
        Self: Sized,
    {
        TypeIds::new()
    }
}

impl<T> DataAccessDyn for Stored<'_, T> {
    fn borrows(&self) -> TypeIds
    where
        Self: Sized,
    {
        let mut type_ids = TypeIds::new();
        type_ids.push(TypeId::of::<StatesCurrentStored>());
        type_ids
    }

    fn borrow_muts(&self) -> TypeIds
    where
        Self: Sized,
    {
        TypeIds::new()
    }
}
