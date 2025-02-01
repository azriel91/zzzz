use std::fmt::Debug;

use async_trait::async_trait;
use peace_cmd::{ctx::CmdCtxTypesConstrained, scopes::SingleProfileSingleFlowView};
use peace_cmd_model::CmdBlockOutcome;
use peace_resource_rt::{resources::ts::SetUp, Resource, ResourceFetchError, Resources};

cfg_if::cfg_if! {
    if #[cfg(feature = "output_progress")] {
        use peace_progress_model::{CmdBlockItemInteractionType, CmdProgressUpdate};
        use tokio::sync::mpsc::Sender;
    }
}

pub use self::{
    cmd_block_error::CmdBlockError, cmd_block_rt::CmdBlockRt, cmd_block_rt_box::CmdBlockRtBox,
    cmd_block_wrapper::CmdBlockWrapper,
};

mod cmd_block_error;
mod cmd_block_rt;
mod cmd_block_rt_box;
mod cmd_block_wrapper;

/// Runs one [`Item::*`] function for one iteration of items.
///
/// A command may consist of:
///
/// 1. Discovering the current state of an environment.
/// 2. Ensuring new items that are not blocked, e.g. launch new servers before
///    taking old servers away.
/// 3. Cleaning unused items that block new items from being ensured, e.g.
///    terminating servers before resizing a subnet's CIDR block.
/// 4. Ensuring new / modified items that are newly unblocked, e.g. launching
///    new servers in the resized subnet.
/// 5. Cleaning unused items that are no longer needed, e.g. removing an old
///    service.
///
/// Each of these is an iteration through items, running one of the [`Item::*`]
/// functions.
///
/// A `CmdBlock` is the unit of one iteration logic.
///
/// [`Item::*`]: peace_cfg::Item
#[async_trait(?Send)]
pub trait CmdBlock: Debug {
    /// Type parameters passed to the `CmdCtx`.
    ///
    /// `CmdBlock` uses the `AppError` and `ParamsKeys` associated type.
    type CmdCtxTypes: CmdCtxTypesConstrained;
    /// Outcome type of the command block, e.g. `(StatesCurrent, StatesGoal)`.
    type Outcome: Debug + Send + Sync + 'static;
    /// Input type of the command block, e.g. `StatesCurrent`.
    type InputT: Resource + 'static;

    /// Returns the type of interactions the `CmdBlock` has with
    /// `ItemLocation`s.
    #[cfg(feature = "output_progress")]
    fn cmd_block_item_interaction_type(&self) -> CmdBlockItemInteractionType;

    /// Fetch function for `InputT`.
    ///
    /// This is overridable so that `CmdBlock`s can change how their `InputT` is
    /// looked up.
    ///
    /// The most common use case for overriding this is for unit `()` inputs,
    /// which should provide an empty implementation.
    ///
    /// # Maintainers / Developers
    ///
    /// Whenever this method is overridden, `input_type_names` should be
    /// overridden as well.
    fn input_fetch(
        &self,
        resources: &mut Resources<SetUp>,
    ) -> Result<Self::InputT, ResourceFetchError> {
        resources.try_remove::<Self::InputT>()
    }

    /// Returns the short type name(s) of `CmdBlock::InputT`.
    ///
    /// If this `CmdBlock::InputT` is a tuple, and each member type is inserted
    /// separately into `resources`, then this method must return the short type
    /// name per member type.
    ///
    /// # Design
    ///
    /// This is a separate method to `input_fetch` as it is invoked separately.
    /// Though of course we *could* also change `input_fetch` to return a
    /// closure that returns the input type names.
    ///
    /// # Maintainers / Developers
    ///
    /// Example implementations are as follows.
    ///
    /// Within the `peace` framework:
    ///
    /// ```rust,ignore
    /// // type InputT = (StatesCurrent, StatesGoal);
    ///
    /// vec![tynm::type_name::<StatesCurrent>(),
    /// tynm::type_name::<StatesGoal>()]
    /// ```
    ///
    /// Outside the `peace` framework:
    ///
    /// ```rust,ignore
    /// // type InputT = (StatesCurrent, StatesGoal);
    ///
    /// vec![
    ///     peace::cmd_rt::tynm::type_name::<StatesCurrent>(),
    ///     peace::cmd_rt::tynm::type_name::<StatesGoal>(),
    /// ]
    /// ```
    fn input_type_names(&self) -> Vec<String> {
        vec![tynm::type_name::<Self::InputT>()]
    }

    /// Inserts the `CmdBlock::Outcome` into `Resources`.
    ///
    /// This is overridable so that `CmdBlock`s can change how their `Outcome`
    /// is inserted.
    ///
    /// The most common use case for overriding this is for unit `()` inputs,
    /// which should provide an empty implementation.
    ///
    /// # Maintainers / Developers
    ///
    /// Whenever this method is overridden, `outcome_type_names` should be
    /// overridden as well.
    fn outcome_insert(&self, resources: &mut Resources<SetUp>, outcome: Self::Outcome) {
        resources.insert(outcome);
    }

    /// Returns the short type name(s) of `CmdBlock::Outcome`.
    ///
    /// If this `CmdBlock::Outcome` is a tuple, and each member type is inserted
    /// separately into `resources`, then this method must return the short type
    /// name per member type.
    ///
    /// # Maintainers / Developers
    ///
    /// Example implementations are as follows.
    ///
    /// Within the `peace` framework:
    ///
    /// ```rust,ignore
    /// // type Outcome = (StatesCurrent, StatesGoal);
    ///
    /// vec![tynm::type_name::<StatesCurrent>(),
    /// tynm::type_name::<StatesGoal>()]
    /// ```
    ///
    /// Outside the `peace` framework:
    ///
    /// ```rust,ignore
    /// // type Outcome = (StatesCurrent, StatesGoal);
    ///
    /// vec![
    ///     peace::cmd_rt::tynm::type_name::<StatesCurrent>(),
    ///     peace::cmd_rt::tynm::type_name::<StatesGoal>(),
    /// ]
    /// ```
    fn outcome_type_names(&self) -> Vec<String> {
        vec![tynm::type_name::<Self::Outcome>()]
    }

    /// Producer function to process all items.
    ///
    /// This is infallible because errors are expected to be returned associated
    /// with an item. This may change if there are errors that are related to
    /// the block that are not associated with a specific item.
    ///
    /// # Implementors
    ///
    /// `StreamOutcome<()>` should be returned if the `CmdBlock` streams the
    /// items, as this captures whether or not the block execution was
    /// interrupted.
    ///
    /// If the block does not stream items, `None` should be returned.
    async fn exec(
        &self,
        input: Self::InputT,
        cmd_view: &mut SingleProfileSingleFlowView<'_, Self::CmdCtxTypes>,
        #[cfg(feature = "output_progress")] progress_tx: &Sender<CmdProgressUpdate>,
    ) -> Result<
        CmdBlockOutcome<Self::Outcome, <Self::CmdCtxTypes as CmdCtxTypesConstrained>::AppError>,
        <Self::CmdCtxTypes as CmdCtxTypesConstrained>::AppError,
    >;
}
