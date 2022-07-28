use std::fmt::Debug;

use fn_graph::{DataAccess, DataAccessDyn};
use peace_cfg::{async_trait, FullSpecId};
use peace_resources::{
    resources_type_state::{Empty, SetUp, WithStates},
    type_reg::untagged::DataType,
    Resources, StatesDesiredMut, StatesMut,
};

/// Internal trait that erases the types from [`FullSpec`]
///
/// This exists so that different implementations of [`FullSpec`] can be held
/// under the same boxed trait.
///
/// [`FullSpec`]: peace_cfg::FullSpec
#[async_trait]
pub trait FullSpecRt<E>: Debug + DataAccess + DataAccessDyn
where
    E: Debug + std::error::Error,
{
    /// Returns the ID of this full spec.
    ///
    /// See [`FullSpec::id`];
    ///
    /// [`FullSpec::id`]: peace_cfg::FullSpec::id
    fn id(&self) -> FullSpecId;

    /// Initializes data for the operation's check and `exec` functions.
    async fn setup(&self, resources: &mut Resources<Empty>) -> Result<(), E>;

    /// Runs [`FullSpec::StateNowFnSpec`]`::`[`exec`].
    ///
    /// [`FullSpec::StateNowFnSpec`]: peace_cfg::FullSpec::StateNowFnSpec
    /// [`exec`]: peace_cfg::FnSpec::exec
    async fn state_now_fn_exec(&self, resources: &Resources<SetUp>) -> Result<(), E>;

    /// Runs [`FullSpec::StateDesiredFnSpec`]`::`[`desired`].
    ///
    /// [`FullSpec::StateDesiredFnSpec`]: peace_cfg::FullSpec::StateDesiredFnSpec
    /// [`desired`]: peace_cfg::FnSpec::desired
    async fn state_desired_fn_exec(&self, resources: &Resources<SetUp>) -> Result<(), E>;

    /// Returns the diff between the current and desired [`State`]s.
    ///
    /// [`State`]: peace_cfg::State
    fn diff(&self, states: &StatesMut, states_desired: &StatesDesiredMut) -> Box<dyn DataType>;

    /// Runs [`FullSpec::EnsureOpSpec`]`::`[`check`].
    ///
    /// [`FullSpec::EnsureOpSpec`]: peace_cfg::FullSpec::EnsureOpSpec
    /// [`check`]: peace_cfg::OpSpec::check
    async fn ensure_op_check(&self, resources: &Resources<WithStates>) -> Result<(), E>;

    /// Runs [`FullSpec::EnsureOpSpec`]`::`[`exec_dry`].
    ///
    /// [`FullSpec::EnsureOpSpec`]: peace_cfg::FullSpec::EnsureOpSpec
    /// [`exec_dry`]: peace_cfg::OpSpec::exec_dry
    async fn ensure_op_exec_dry(&self, resources: &Resources<WithStates>) -> Result<(), E>;

    /// Runs [`FullSpec::EnsureOpSpec`]`::`[`exec`].
    ///
    /// [`FullSpec::EnsureOpSpec`]: peace_cfg::FullSpec::EnsureOpSpec
    /// [`exec`]: peace_cfg::OpSpec::exec
    async fn ensure_op_exec(&self, resources: &Resources<WithStates>) -> Result<(), E>;
}
