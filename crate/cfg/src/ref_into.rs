use peace_item_interaction_model::ItemLocationState;

/// Returns `T` from a reference to `self`.
///
/// Allows setting a constraint on `Item::State`, such that `&State` can be
/// turned into an `peace_item_interaction_model::ItemLocationState`.
///
/// # Implementors
///
/// You should `impl<'state> From<&'state YourItemState> for ItemLocationState
/// {}`. There is a blanket implementation that implements
/// `RefInto<ItemLocationState> for S where ItemLocationState: From<&'state S>`
pub trait RefInto<T> {
    /// Returns `T` from a reference to `self`.
    fn into(&self) -> T;
}

impl<S> RefInto<ItemLocationState> for S
where
    for<'state> ItemLocationState: From<&'state S>,
    S: 'static,
{
    fn into(&self) -> ItemLocationState {
        ItemLocationState::from(self)
    }
}
