use std::fmt::Debug;

use serde::{de::DeserializeOwned, Serialize};

use crate::FieldWiseSpecRt;

/// Input parameters to an item.
///
/// This trait is automatically implemented by
/// `#[derive(peace::params::Params)]`.
pub trait Params: Clone + Debug + Serialize + DeserializeOwned + Send + Sync + 'static {
    /// Convenience associated type for `ValueSpec<Self>`.
    type Spec: Clone + Debug + Serialize + DeserializeOwned + Send + Sync + 'static;
    /// The `Params` type, where each field is wrapped in [`Option`].
    type Partial: Clone + Debug + Default + Send + Sync + 'static;
    /// The `Params` type, where each field is wrapped with
    /// [`ParamsSpecFieldless<T>`].
    ///
    /// Specifies how to look up values for each field in the `Value`.
    ///
    /// [`ParamsSpecFieldless<T>`]: crate::ParamsSpecFieldless
    type FieldWiseSpec: FieldWiseSpecRt<ValueType = Self, Partial = Self::Partial>
        + Clone
        + Debug
        + Serialize
        + DeserializeOwned
        + Send
        + Sync
        + 'static;

    /// Builder to return the `FieldWiseSpec` type.
    type FieldWiseBuilder;

    /// Returns a builder to construct the `FieldWise` spec.
    fn field_wise_spec() -> Self::FieldWiseBuilder;
}
