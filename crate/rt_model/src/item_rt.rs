use std::{any::Any, fmt::Debug};

use dyn_clone::DynClone;
use peace_cfg::{async_trait, FnCtx};
use peace_data::fn_graph::{DataAccess, DataAccessDyn};
use peace_item_model::ItemId;
use peace_params::ParamsSpecs;
use peace_resource_rt::{
    resources::ts::{Empty, SetUp},
    states::StatesCurrent,
    type_reg::untagged::{BoxDtDisplay, TypeMap},
    Resources,
};

use crate::{
    outcomes::{ItemApplyBoxed, ItemApplyPartialBoxed},
    ParamsSpecsTypeReg, StatesTypeReg,
};

/// Internal trait that erases the types from [`Item`]
///
/// This exists so that different implementations of [`Item`] can be held
/// under the same boxed trait.
///
/// [`Item`]: peace_cfg::Item
#[async_trait(?Send)]
pub trait ItemRt<E>:
    Any + Debug + DataAccess + DataAccessDyn + DynClone + Send + Sync + 'static
{
    /// Returns the ID of this item.
    ///
    /// See [`Item::id`];
    ///
    /// [`Item::id`]: peace_cfg::Item::id
    fn id(&self) -> &ItemId;

    /// Returns whether this item is equal to the other.
    fn eq(&self, other: &dyn ItemRt<E>) -> bool;

    /// Returns `&self` as `&dyn Any`.
    ///
    /// This is needed to upcast to `&dyn Any` and satisfy the upcast lifetime
    /// requirement.
    fn as_any(&self) -> &dyn Any;

    /// Initializes data for the item's functions.
    async fn setup(&self, resources: &mut Resources<Empty>) -> Result<(), E>
    where
        E: Debug + std::error::Error;

    /// Registers params and state types with the type registries for
    /// deserializing from disk.
    ///
    /// This is necessary to deserialize `ItemParamsFile`,
    /// `ParamsSpecsFile`, `StatesCurrentFile`, and `StatesGoalFile`.
    fn params_and_state_register(
        &self,
        params_specs_type_reg: &mut ParamsSpecsTypeReg,
        states_type_reg: &mut StatesTypeReg,
    );

    /// Returns if the given two states equal.
    ///
    /// This returns an error if the boxed states could not be downcasted to
    /// this item's state, which indicates one of the following:
    ///
    /// * Peace contains a bug, and passed an incorrect box to this item.
    /// * Item IDs were swapped, such that `ItemA`'s state is passed to `ItemB`.
    ///
    ///     This needs some rework on how item IDs are implemented -- as in,
    ///     whether we should use a string newtype for `ItemId`s, or redesign
    ///     how `Item`s or related types are keyed.
    ///
    /// Note: it is impossible to call this method if an `Item`'s state type has
    /// changed -- it would have failed on deserialization.
    fn state_eq(&self, state_a: &BoxDtDisplay, state_b: &BoxDtDisplay) -> Result<bool, E>
    where
        E: Debug + std::error::Error;

    /// Returns an example fully deployed state of the managed item.
    ///
    /// # Design
    ///
    /// This is *expected* to always return a value, as it is used to:
    ///
    /// * Display a diagram that shows the user what the item looks like when it
    ///   is fully deployed, without actually interacting with any external
    ///   state.
    ///
    /// As much as possible, use the values in the provided params and data.
    ///
    /// This function should **NOT** interact with any external services, or
    /// read from files that are part of the automation process, e.g.
    /// querying data from a web endpoint, or reading files that may be
    /// downloaded by a predecessor.
    ///
    /// ## Fallibility
    ///
    /// [`Item::state_example`] is deliberately infallible to signal to
    /// implementors that calling an external service / read from a file is
    /// incorrect implementation for this method -- values in params / data
    /// may be example values from other items that may not resolve.
    ///
    /// [`ItemRt::state_example`] *is* fallible as value resolution for
    /// parameters may fail, e.g. if there is a bug in Peace, or an item's
    /// parameters requests a type that doesn't exist in [`Resources`].
    ///
    /// ## Non-async
    ///
    /// This signals to implementors that this function should be a cheap
    /// example state computation that is relatively realistic rather than
    /// determining an accurate value.
    ///
    /// [`Item::state_example`]: peace_cfg::Item::Item::state_example
    #[cfg(feature = "item_state_example")]
    fn state_example(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
    ) -> Result<BoxDtDisplay, E>
    where
        E: Debug + std::error::Error;

    /// Runs [`Item::state_clean`].
    ///
    /// [`Item::state_clean`]: peace_cfg::Item::state_clean
    async fn state_clean(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
    ) -> Result<BoxDtDisplay, E>
    where
        E: Debug + std::error::Error;

    /// Runs [`Item::state_current`]`::`[`try_exec`].
    ///
    /// [`Item::state_current`]: peace_cfg::Item::state_current
    /// [`try_exec`]: peace_cfg::TryFnSpec::try_exec
    async fn state_current_try_exec(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        fn_ctx: FnCtx<'_>,
    ) -> Result<Option<BoxDtDisplay>, E>
    where
        E: Debug + std::error::Error;

    /// Runs [`Item::state_current`]`::`[`exec`].
    ///
    /// [`Item::state_current`]: peace_cfg::Item::state_current
    /// [`exec`]: peace_cfg::TryFnSpec::exec
    async fn state_current_exec(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        fn_ctx: FnCtx<'_>,
    ) -> Result<BoxDtDisplay, E>
    where
        E: Debug + std::error::Error;

    /// Runs [`Item::state_goal`]`::`[`try_exec`].
    ///
    /// [`Item::state_goal`]: peace_cfg::Item::state_goal
    /// [`try_exec`]: peace_cfg::TryFnSpec::try_exec
    async fn state_goal_try_exec(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        fn_ctx: FnCtx<'_>,
    ) -> Result<Option<BoxDtDisplay>, E>
    where
        E: Debug + std::error::Error;

    /// Runs [`Item::state_goal`]`::`[`exec`].
    ///
    /// [`Item::state_goal`]: peace_cfg::Item::state_goal
    /// [`exec`]: peace_cfg::TryFnSpec::exec
    async fn state_goal_exec(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        fn_ctx: FnCtx<'_>,
    ) -> Result<BoxDtDisplay, E>
    where
        E: Debug + std::error::Error;

    /// Returns the diff between the given [`State`]s.
    ///
    /// Given `states_a` and `states_b` represent current states and goal
    /// states, then this method returns `None` in the following cases:
    ///
    /// * The current state cannot be retrieved, due to a predecessor's state
    ///   not existing.
    /// * The goal state cannot be retrieved, due to a predecessor's state not
    ///   existing.
    /// * A bug exists, e.g. the state is stored against the wrong type
    ///   parameter.
    ///
    /// [`State`]: peace_cfg::State
    async fn state_diff_exec(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        states_a: &TypeMap<ItemId, BoxDtDisplay>,
        states_b: &TypeMap<ItemId, BoxDtDisplay>,
    ) -> Result<Option<BoxDtDisplay>, E>
    where
        E: Debug + std::error::Error;

    /// Discovers the information needed for an ensure execution.
    ///
    /// This runs the following functions in order:
    ///
    /// * [`Item::state_current`]
    /// * [`Item::state_goal`]
    /// * [`Item::state_diff`]
    /// * [`ApplyFns::check`]
    ///
    /// [`Item::state_current`]: peace_cfg::Item::state_current
    /// [`Item::state_goal`]: peace_cfg::Item::state_goal
    /// [`Item::state_diff`]: peace_cfg::Item::state_diff
    /// [`ApplyFns::check`]: peace_cfg::Item::ApplyFns
    async fn ensure_prepare(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        fn_ctx: FnCtx<'_>,
    ) -> Result<ItemApplyBoxed, (E, ItemApplyPartialBoxed)>
    where
        E: Debug + std::error::Error;

    /// Discovers the information needed for a clean execution.
    ///
    /// This runs the following functions in order:
    ///
    /// * [`Item::state_current`]
    /// * [`Item::state_clean`]
    /// * [`Item::state_diff`]
    /// * [`ApplyFns::check`]
    ///
    /// [`Item::state_current`]: peace_cfg::Item::state_current
    /// [`Item::state_clean`]: peace_cfg::Item::state_clean
    /// [`Item::state_diff`]: peace_cfg::Item::state_diff
    /// [`ApplyFns::check`]: peace_cfg::Item::ApplyFns
    async fn clean_prepare(
        &self,
        states_current: &StatesCurrent,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
    ) -> Result<ItemApplyBoxed, (E, ItemApplyPartialBoxed)>
    where
        E: Debug + std::error::Error;

    /// Dry applies the item from its current state to its goal state.
    ///
    /// This runs the following function in order, passing in the information
    /// collected from [`ensure_prepare`] or [`clean_prepare`]:
    ///
    /// * [`ApplyFns::exec_dry`]
    ///
    /// # Parameters
    ///
    /// * `resources`: The resources in the current execution.
    /// * `item_apply`: The information collected in `self.ensure_prepare`.
    ///
    /// [`ApplyFns::exec_dry`]: peace_cfg::Item::ApplyFns
    async fn apply_exec_dry(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        fn_ctx: FnCtx<'_>,
        item_apply: &mut ItemApplyBoxed,
    ) -> Result<(), E>
    where
        E: Debug + std::error::Error;

    /// Applies the item from its current state to its goal state.
    ///
    /// This runs the following function in order, passing in the information
    /// collected from [`ensure_prepare`] or [`clean_prepare`]:
    ///
    /// * [`ApplyFns::exec`]
    ///
    /// # Parameters
    ///
    /// * `resources`: The resources in the current execution.
    /// * `item_apply`: The information collected in `self.ensure_prepare`.
    ///
    /// [`ApplyFns::exec`]: peace_cfg::Item::ApplyFns
    async fn apply_exec(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        fn_ctx: FnCtx<'_>,
        item_apply: &mut ItemApplyBoxed,
    ) -> Result<(), E>
    where
        E: Debug + std::error::Error;

    /// Returns the physical resources that this item interacts with, purely
    /// using example state.
    ///
    /// # Design
    ///
    /// This method returns interactions from [`Item::interactions`], passing in
    /// parameters computed from example state.
    ///
    /// ## Fallibility
    ///
    /// [`Item::interactions`] is infallible as computing `ItemInteractions`
    /// should purely be instantiating objects.
    ///
    /// [`ItemRt::interactions_example`] *is* fallible as value resolution for
    /// parameters may fail, e.g. if there is a bug in Peace, or an item's
    /// parameters requests a type that doesn't exist in [`Resources`].
    #[cfg(all(feature = "item_interactions", feature = "item_state_example"))]
    fn interactions_example(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
    ) -> Result<peace_item_interaction_model::ItemInteractionsExample, E>;

    /// Returns the physical resources that this item interacts with, merging
    /// any available current state over example state.
    ///
    /// # Design
    ///
    /// This method returns interactions from [`Item::interactions`], passing in
    /// parameters computed from current state, or if not available, example
    /// state.
    ///
    /// For tracking which item interactions are known, for the purpose of
    /// styling unknown state differently, we could return the
    /// `ItemInteractions` alongside with how they were constructed:
    ///
    /// 1. One for `ItemInteraction`s where params are fully computed using
    ///    fully known state.
    /// 2. One for `ItemInteraction`s where params are computed using some or
    ///    all example state.
    ///
    /// ## Fallibility
    ///
    /// [`Item::interactions`] is infallible as computing `ItemInteractions`
    /// should purely be instantiating objects.
    ///
    /// [`ItemRt::interactions_current`] *is* fallible as value resolution
    /// for parameters may fail, e.g. if there is a bug in Peace, or an
    /// item's parameters requests a type that doesn't exist in
    /// [`Resources`].
    #[cfg(all(feature = "item_interactions", feature = "item_state_example"))]
    fn interactions_try_current(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
    ) -> Result<peace_item_interaction_model::ItemInteractionsCurrentOrExample, E>;

    /// Returns a human readable tag name that represents this item.
    ///
    /// For example, a `FileDownloadItem<WebApp>` should return a string similar
    /// to: `"Web App: File Download"`. This allows tags to be grouped by the
    /// concept / information they are associated with, rather than grouping
    /// tags by the type of operation.
    #[cfg(all(feature = "item_interactions", feature = "item_state_example"))]
    fn interactions_tag_name(&self) -> String;
}
