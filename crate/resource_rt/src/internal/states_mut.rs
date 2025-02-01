use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use peace_item_model::ItemId;
use serde::Serialize;
use type_reg::untagged::{BoxDtDisplay, TypeMap};

/// `State`s for all `Item`s. `TypeMap<ItemId, BoxDtDisplay>` newtype.
///
/// # Implementors
///
/// To reference State from another `Item`, in `Item::Data`, you should
/// reference [`Current<T>`] or [`Goal<T>`], where `T` is the predecessor
/// item's state.
///
/// # Type Parameters
///
/// * `TS`: Type state to distinguish the purpose of the `States` map.
///
/// [`Current<T>`]: peace_data::marker::Current
/// [`Data`]: peace_data::Data
/// [`Goal<T>`]: peace_data::marker::Goal
/// [`Resources`]: crate::Resources
/// [`StatesCurrent`]: crate::StatesCurrent
/// [`StatesRw`]: crate::StatesRw
#[derive(Debug, Serialize)]
pub struct StatesMut<TS>(TypeMap<ItemId, BoxDtDisplay>, PhantomData<TS>);

impl<TS> StatesMut<TS> {
    /// Returns a new `StatesMut` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an empty `StatesMut` map with the specified capacity.
    ///
    /// The `StatesMut` will be able to hold at least capacity elements
    /// without reallocating. If capacity is 0, the map will not allocate.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(TypeMap::with_capacity_typed(capacity), PhantomData)
    }

    /// Returns the inner map.
    pub fn into_inner(self) -> TypeMap<ItemId, BoxDtDisplay> {
        self.0
    }
}

impl<TS> Default for StatesMut<TS> {
    fn default() -> Self {
        Self(TypeMap::default(), PhantomData)
    }
}

impl<TS> Deref for StatesMut<TS> {
    type Target = TypeMap<ItemId, BoxDtDisplay>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<TS> DerefMut for StatesMut<TS> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<TS> From<TypeMap<ItemId, BoxDtDisplay>> for StatesMut<TS> {
    fn from(type_map: TypeMap<ItemId, BoxDtDisplay>) -> Self {
        Self(type_map, PhantomData)
    }
}

impl<TS> Extend<(ItemId, BoxDtDisplay)> for StatesMut<TS> {
    fn extend<T: IntoIterator<Item = (ItemId, BoxDtDisplay)>>(&mut self, iter: T) {
        iter.into_iter().for_each(|(item_id, state)| {
            self.insert_raw(item_id, state);
        });
    }
}
