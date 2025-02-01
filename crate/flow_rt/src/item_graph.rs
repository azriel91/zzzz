use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use peace_data::fn_graph::FnGraph;
use peace_resource_rt::states::{States, StatesSerde};
use peace_rt_model::ItemBoxed;

/// Graph of all [`Item`]s, `FnGraph<ItemBoxed<E>>` newtype.
///
/// [`Item`]: peace_cfg::Item
#[derive(Debug)]
pub struct ItemGraph<E>(FnGraph<ItemBoxed<E>>);

// Manual implementation because derive requires `E` to be `Clone`,
// which causes `graph.clone()` to call `FnGraph::clone`.
impl<E> Clone for ItemGraph<E> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<E> PartialEq for ItemGraph<E>
where
    E: 'static,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<E> Eq for ItemGraph<E> where E: 'static {}

impl<E> ItemGraph<E> {
    /// Returns the inner [`FnGraph`].
    pub fn into_inner(self) -> FnGraph<ItemBoxed<E>> {
        self.0
    }

    /// Returns a user-friendly serializable states map.
    ///
    /// This will contain an entry for all items, in order of flow item
    /// insertion, whether or not a state exists in the provided `states` map.
    pub fn states_serde<ValueT, TS>(&self, states: &States<TS>) -> StatesSerde<ValueT>
    where
        ValueT: Clone + Debug + PartialEq + Eq,
        E: 'static,
    {
        StatesSerde::from_iter(self.0.iter_insertion().map(|item| {
            let item_id = item.id();
            (item_id.clone(), states.get_raw(item_id).cloned())
        }))
    }
}

impl<E> Deref for ItemGraph<E> {
    type Target = FnGraph<ItemBoxed<E>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E> DerefMut for ItemGraph<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<E> From<FnGraph<ItemBoxed<E>>> for ItemGraph<E> {
    fn from(graph: FnGraph<ItemBoxed<E>>) -> Self {
        Self(graph)
    }
}

impl<'graph, ValueT, E> From<&'graph ItemGraph<E>> for StatesSerde<ValueT>
where
    ValueT: Clone + Debug + PartialEq + Eq,
    E: 'static,
{
    fn from(graph: &'graph ItemGraph<E>) -> Self {
        StatesSerde::from_iter(graph.iter_insertion().map(|item| (item.id().clone(), None)))
    }
}
