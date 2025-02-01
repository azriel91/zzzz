use std::fmt::Debug;

use fn_graph::{DataAccess, DataAccessDyn, DataBorrow, Resources, R, W};
use peace_item_model::ItemId;

/// Defines the logic to instantiate and retrieve runtime data.
///
/// # Note for API Consumers
///
/// This trait is implemented by using the [`Data` derive].
///
/// [`Data` derive]: peace_data_derive::Data
pub trait Data<'borrow>: DataAccess + DataAccessDyn + Send {
    /// Borrows each of `Self`'s fields from the provided [`Resources`].
    ///
    /// This takes in the `item_id`, so that the type that implements this
    /// trait may further filter specific data within the borrowed data, scope
    /// to the item.
    ///
    /// # Parameters
    ///
    /// * `item_id`: ID of the item this borrow is used for.
    /// * `resources`: `Any` map to borrow the data from.
    fn borrow(item_id: &'borrow ItemId, resources: &'borrow Resources) -> Self;
}

impl<'borrow> Data<'borrow> for () {
    fn borrow(_item_id: &'borrow ItemId, _resources: &'borrow Resources) -> Self {}
}

impl<'borrow> Data<'borrow> for &'borrow () {
    fn borrow(_item_id: &'borrow ItemId, _resources: &'borrow Resources) -> Self {
        &()
    }
}

impl<'borrow, T> Data<'borrow> for R<'borrow, T>
where
    T: Debug + Send + Sync + 'static,
{
    fn borrow(_item_id: &'borrow ItemId, resources: &'borrow Resources) -> Self {
        <Self as DataBorrow>::borrow(resources)
    }
}

impl<'borrow, T> Data<'borrow> for W<'borrow, T>
where
    T: Debug + Send + Sync + 'static,
{
    fn borrow(_item_id: &'borrow ItemId, resources: &'borrow Resources) -> Self {
        <Self as DataBorrow>::borrow(resources)
    }
}
