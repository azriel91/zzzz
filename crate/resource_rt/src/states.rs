//! Resources that track current and goal states, and state diffs.

pub use self::{
    state_diffs::StateDiffs, states_clean::StatesClean, states_cleaned::StatesCleaned,
    states_cleaned_dry::StatesCleanedDry, states_current::StatesCurrent,
    states_current_stored::StatesCurrentStored, states_ensured::StatesEnsured,
    states_ensured_dry::StatesEnsuredDry, states_goal::StatesGoal,
    states_goal_stored::StatesGoalStored, states_previous::StatesPrevious,
    states_serde::StatesSerde,
};

pub mod ts;

use std::{marker::PhantomData, ops::Deref};

use peace_fmt::{Presentable, Presenter};
use peace_item_model::ItemId;
use serde::Serialize;
use type_reg::untagged::{BoxDtDisplay, TypeMap};

use crate::internal::StatesMut;

mod state_diffs;
mod states_clean;
mod states_cleaned;
mod states_cleaned_dry;
mod states_current;
mod states_current_stored;
mod states_ensured;
mod states_ensured_dry;
mod states_goal;
mod states_goal_stored;
mod states_previous;
mod states_serde;

/// Map of `State`s for all `Item`s. `TypeMap<ItemId, Item::State>` newtype.
///
/// # Type Parameters
///
/// * `TS`: Type state to distinguish the purpose of the `States` map.
///
/// # Serialization
///
/// [`StatesSerde`] is used for serialization and deserialization.
///
/// # Design
///
/// When states are serialized, we want there to be an entry for each item.
///
/// 1. This means the `States` map should contain an entry for each item,
///    regardless of whether a `State` is recorded for that item.
///
/// 2. Inserting an `Option<_>` layer around the `Item::State` turns the map
///    into a `Map<ItemId, Option<Item::State>>`.
///
/// 3. Calling `states.get(item_id)` returns `Option<Option<Item::State>>`, the
///    outer layer for whether the item had an entry, and the inner layer for
///    whether there was any `State` recorded.
///
/// 4. If we can guarantee the item ID is valid -- an ID of an item in the flow
///    -- we could remove that outer `Option` layer. Currently we cannot make
///    this guarantee, as:
///
///     - item IDs are constructed by developer code, without any constraints
///       for which items are inserted into the Flow, and which are inserted
///       into `States` -- although insertion into `States` is largely managed
///       by `peace`.
///
///     - `States` may contain different items across different versions of an
///       automation tool, so it is possible (and valid) to:
///
///         + Deserialize `States` that contain states for `Item`s that are no
///           longer in the flow.
///         + Deserialize `States` that do not contain states for `Item`s that
///           are newly added to the flow.
///         + Have a combination of the above for renamed items.
///
/// 5. For clarity of each of these `Option` layers, we can wrap them in a
///    newtype.
///
/// 6. For code cleanliness, this additional layer requires calling
///    [`flatten()`] on `states.get(item_id)`.
///
/// 7. We *could* introduce a different type during serialization that handles
///    this additional layer, to remove the additional `flatten()`. How do we
///    handle flow upgrades smoothly?
///
///     - **Development:** Compile time API support with runtime errors may be
///       sufficient.
///     - **User:** Developers *may* require users to decide how to migrate
///       data. This use case hopefully is less common.
///
/// ## `StatesSerde` Separate Type
///
/// Newtype for `Map<ItemId, Option<Item::State>>`.
///
/// ### Item Additions
///
/// * Flow contains the `Item`.
/// * Stored state doesn't contain an entry for the item.
/// * Deserialized `StatesSerde` should contain `(item_id!("new"), None)` -- may
///   need custom deserialization code.
///
/// ### Item Removals
///
/// * Flow does not contain the `Item`.
/// * Stored state contains an entry for the item, but cannot be deserialized.
/// * Deserialized `StatesSerde` would not contain any entry.
/// * Deserialization will return the unable to be deserialized item state in
///   the return value. Meaning, `StatesSerde` will contain it in a separate
///   "removed" field.
///
/// After deserialization, `StatesSerde` is explicitly mapped into `States`, and
/// we can inform the developer and/or user of the removed items if it is
/// useful.
///
/// ## `States` With Optional Item State
///
/// Developers will frequently use `states.get(item_id).flatten()` to access
/// state.
///
/// Deserialization has all the same properties as the `StatesSerde` separate
/// type. However, the entries that fail to be deserialized are retained in the
/// `States` type (or are lost, if we deliberately ignore entries that fail to
/// be deserialized).
///
/// Should `Flow`s be versionable, and we migrate them to the latest version as
/// encountered? If so, then:
///
/// * `peace` should store the version of the flow in the stored states files
/// * items that have ever been used in flows must be shipped in the automation
///   software, in order to support safe upgrades.
///
/// How would this work?
///
/// * Newly added items just work.
/// * Removed items need to be removed:
///     - Successors may need their parameters specified from new predecessors.
///     - If removing multiple items, we need to clean them in reverse.
/// * Renamed items may need to be re-applied, or potentially cleaned and
///   re-ensured. This doesn't support data retention if a predecessor needs to
///   be cleaned, forcing successors to be cleaned, and reensured after. Unless,
///   `peace` supports backup and restore.
///
/// [`flatten()`]: std::option::Option::flatten
#[derive(Debug, Serialize)]
#[serde(transparent)] // Needed to serialize as a map instead of a list.
pub struct States<TS>(
    pub(crate) TypeMap<ItemId, BoxDtDisplay>,
    pub(crate) PhantomData<TS>,
);

impl<TS> States<TS> {
    /// Returns a new `States` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an empty `States` map with the specified capacity.
    ///
    /// The `States` will be able to hold at least capacity elements
    /// without reallocating. If capacity is 0, the map will not allocate.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(TypeMap::with_capacity_typed(capacity), PhantomData)
    }

    /// Returns the inner map.
    pub fn into_inner(self) -> TypeMap<ItemId, BoxDtDisplay> {
        self.0
    }
}

impl<TS> Clone for States<TS> {
    fn clone(&self) -> Self {
        let mut clone = Self(TypeMap::with_capacity_typed(self.0.len()), PhantomData);
        clone.0.extend(
            self.0
                .iter()
                .map(|(item_id, state)| (item_id.clone(), state.clone())),
        );

        clone
    }
}

impl<TS> Default for States<TS> {
    fn default() -> Self {
        Self(TypeMap::default(), PhantomData)
    }
}

impl<TS> Deref for States<TS> {
    type Target = TypeMap<ItemId, BoxDtDisplay>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<TS> From<TypeMap<ItemId, BoxDtDisplay>> for States<TS> {
    fn from(type_map: TypeMap<ItemId, BoxDtDisplay>) -> Self {
        Self(type_map, PhantomData)
    }
}

impl<TS> From<StatesMut<TS>> for States<TS> {
    fn from(states_mut: StatesMut<TS>) -> Self {
        Self(states_mut.into_inner(), PhantomData)
    }
}

#[peace_fmt::async_trait(?Send)]
impl<TS> Presentable for States<TS> {
    async fn present<'output, PR>(&self, presenter: &mut PR) -> Result<(), PR::Error>
    where
        PR: Presenter<'output>,
    {
        presenter
            .list_numbered_with(self.iter(), |(item_id, state)| {
                (item_id, format!(": {state}"))
            })
            .await
    }
}
