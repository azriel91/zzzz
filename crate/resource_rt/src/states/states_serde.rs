//! Resources that track current and goal states, and state diffs.

use std::{fmt::Debug, ops::Deref};

use peace_item_model::ItemId;
use serde::Serialize;
use type_reg::{
    common::UnknownEntriesSome,
    untagged::{BoxDtDisplay, TypeMapOpt},
};

/// Map of `State`s for all `Item`s. `TypeMapOpt<ItemId, Item::State>`
/// newtype.
///
/// Conceptually you can think of this as a `Map<ItemId, Option<Item::State>>`.
///
/// This map should:
///
/// * Always contain an entry for every item in the flow.
/// * Contain an unknown entry for deserialized unknown items.
///
/// This map can be initialized either through one of:
///
/// * Deserialization.
/// * `From<&ItemGraph<E>>`: All states are initialized to `None`.
/// * [`FromIterator::<(ItemId, Option<BoxDtDisplay>)>::from_iter`].
///
/// [`FromIterator::<(ItemId, Option<BoxDtDisplay>)>::from_iter`]: std::iter::FromIterator
#[derive(Debug, Serialize)]
#[serde(transparent)] // Needed to serialize as a map instead of a list.
pub struct StatesSerde<ValueT>(TypeMapOpt<ItemId, BoxDtDisplay, UnknownEntriesSome<ValueT>>)
where
    ValueT: Clone + Debug + PartialEq + Eq;

impl<ValueT> StatesSerde<ValueT>
where
    ValueT: Clone + Debug + PartialEq + Eq,
{
    /// Creates an empty `StatesSerde` map with the specified capacity.
    ///
    /// The `StatesSerde` will be able to hold at least capacity elements
    /// without reallocating. If capacity is 0, the map will not allocate.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(TypeMapOpt::with_capacity_typed(capacity))
    }

    /// Returns the inner map.
    pub fn into_inner(self) -> TypeMapOpt<ItemId, BoxDtDisplay, UnknownEntriesSome<ValueT>> {
        self.0
    }
}

impl<ValueT> Clone for StatesSerde<ValueT>
where
    ValueT: Clone + Debug + PartialEq + Eq,
{
    fn clone(&self) -> Self {
        let mut clone = Self(TypeMapOpt::with_capacity_typed(self.0.len()));
        clone.0.extend(
            self.0
                .iter()
                .map(|(item_id, state)| (item_id.clone(), state.clone())),
        );

        clone
    }
}

impl<ValueT> Deref for StatesSerde<ValueT>
where
    ValueT: Clone + Debug + PartialEq + Eq,
{
    type Target = TypeMapOpt<ItemId, BoxDtDisplay, UnknownEntriesSome<ValueT>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<ValueT> FromIterator<(ItemId, Option<BoxDtDisplay>)> for StatesSerde<ValueT>
where
    ValueT: Clone + Debug + PartialEq + Eq,
{
    fn from_iter<T: IntoIterator<Item = (ItemId, Option<BoxDtDisplay>)>>(iter: T) -> Self {
        iter.into_iter().fold(
            Self(TypeMapOpt::new_typed()),
            |mut states_serde, (item_id, state_boxed)| {
                states_serde.0.insert_raw(item_id, state_boxed);
                states_serde
            },
        )
    }
}

impl<ValueT> From<TypeMapOpt<ItemId, BoxDtDisplay, UnknownEntriesSome<ValueT>>>
    for StatesSerde<ValueT>
where
    ValueT: Clone + Debug + PartialEq + Eq,
{
    fn from(type_map_opt: TypeMapOpt<ItemId, BoxDtDisplay, UnknownEntriesSome<ValueT>>) -> Self {
        Self(type_map_opt)
    }
}
