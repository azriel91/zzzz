use std::ops::{Deref, DerefMut};

use peace_data::fn_graph::FnGraphBuilder;
use peace_rt_model::ItemBoxed;

use crate::ItemGraph;

/// Builder for an [`ItemGraph`], `FnGraphBuilder<ItemBoxed<E>>`
/// newtype.
#[derive(Debug)]
pub struct ItemGraphBuilder<E>(FnGraphBuilder<ItemBoxed<E>>);

impl<E> ItemGraphBuilder<E> {
    /// Returns a new `ItemGraphBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the inner [`FnGraphBuilder`].
    pub fn into_inner(self) -> FnGraphBuilder<ItemBoxed<E>> {
        self.0
    }

    /// Builds and returns the [`ItemGraph`].
    pub fn build(self) -> ItemGraph<E> {
        ItemGraph::from(self.0.build())
    }
}

impl<E> Default for ItemGraphBuilder<E> {
    fn default() -> Self {
        Self(FnGraphBuilder::default())
    }
}

impl<E> Deref for ItemGraphBuilder<E> {
    type Target = FnGraphBuilder<ItemBoxed<E>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E> DerefMut for ItemGraphBuilder<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<E> From<FnGraphBuilder<ItemBoxed<E>>> for ItemGraphBuilder<E> {
    fn from(graph: FnGraphBuilder<ItemBoxed<E>>) -> Self {
        Self(graph)
    }
}
