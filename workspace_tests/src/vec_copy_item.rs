use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use diff::{Diff, VecDiff, VecDiffType};
use peace::{
    cfg::{async_trait, ApplyCheck, FnCtx, Item},
    data::{
        accessors::{RMaybe, W},
        Data,
    },
    item_model::{item_id, ItemId},
    params::Params,
    resource_rt::{resources::ts::Empty, states::StatesCurrentStored, Resources},
    rt_model::ItemWrapper,
};
#[cfg(feature = "output_progress")]
use peace::{
    item_interaction_model::ItemLocationState,
    progress_model::{ProgressLimit, ProgressMsgUpdate},
};
use serde::{Deserialize, Serialize};

/// Type alias for `ItemWrapper` with all of VecCopyItem's parameters.
pub type VecCopyItemWrapper = ItemWrapper<VecCopyItem, VecCopyError>;

/// Copies bytes from `VecA` to `VecB`.
#[derive(Clone, Debug)]
pub struct VecCopyItem {
    /// ID of the item.
    id: ItemId,
}

impl VecCopyItem {
    pub const ID_DEFAULT: &'static ItemId = &item_id!("vec_copy");

    pub fn new(id: ItemId) -> Self {
        Self { id }
    }

    async fn state_current_internal(
        fn_ctx: FnCtx<'_>,
        data: VecCopyData<'_>,
    ) -> Result<VecCopyState, VecCopyError> {
        #[cfg(not(feature = "output_progress"))]
        let _fn_ctx = fn_ctx;

        let vec_copy_state = VecCopyState::from(data.dest().0.clone());

        #[cfg(feature = "output_progress")]
        {
            if let Ok(len) = u64::try_from(vec_copy_state.len()) {
                fn_ctx.progress_sender.inc(len, ProgressMsgUpdate::NoChange);
            }
        }

        Ok(vec_copy_state)
    }

    async fn state_goal_internal(
        fn_ctx: FnCtx<'_>,
        vec_a: &[u8],
    ) -> Result<VecCopyState, VecCopyError> {
        #[cfg(not(feature = "output_progress"))]
        let _fn_ctx = fn_ctx;
        let vec_copy_state = VecCopyState::from(vec_a.to_vec());

        #[cfg(feature = "output_progress")]
        {
            if let Ok(len) = u64::try_from(vec_copy_state.len()) {
                fn_ctx.progress_sender.inc(len, ProgressMsgUpdate::NoChange);
            }
        }

        Ok(vec_copy_state)
    }
}

impl Default for VecCopyItem {
    fn default() -> Self {
        Self::new(Self::ID_DEFAULT.clone())
    }
}

#[async_trait(?Send)]
impl Item for VecCopyItem {
    type Data<'exec> = VecCopyData<'exec>;
    type Error = VecCopyError;
    type Params<'exec> = VecA;
    type State = VecCopyState;
    type StateDiff = VecCopyDiff;

    fn id(&self) -> &ItemId {
        &self.id
    }

    #[cfg(feature = "item_state_example")]
    fn state_example(params: &Self::Params<'_>, _data: Self::Data<'_>) -> Self::State {
        VecCopyState(params.0.clone())
    }

    async fn try_state_current(
        fn_ctx: FnCtx<'_>,
        _params_partial: &<Self::Params<'_> as Params>::Partial,
        data: Self::Data<'_>,
    ) -> Result<Option<Self::State>, VecCopyError> {
        Self::state_current_internal(fn_ctx, data).await.map(Some)
    }

    async fn state_current(
        fn_ctx: FnCtx<'_>,
        _params: &Self::Params<'_>,
        data: Self::Data<'_>,
    ) -> Result<Self::State, VecCopyError> {
        Self::state_current_internal(fn_ctx, data).await
    }

    async fn try_state_goal(
        fn_ctx: FnCtx<'_>,
        params_partial: &<Self::Params<'_> as Params>::Partial,
        _data: Self::Data<'_>,
    ) -> Result<Option<Self::State>, VecCopyError> {
        if let Some(vec_a) = params_partial.0.as_ref() {
            Self::state_goal_internal(fn_ctx, vec_a).await.map(Some)
        } else {
            Ok(None)
        }
    }

    async fn state_goal(
        fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        _data: Self::Data<'_>,
    ) -> Result<Self::State, VecCopyError> {
        Self::state_goal_internal(fn_ctx, params.0.as_ref()).await
    }

    async fn state_diff(
        _params_partial: &<Self::Params<'_> as Params>::Partial,
        _data: VecCopyData<'_>,
        state_current: &VecCopyState,
        state_goal: &VecCopyState,
    ) -> Result<Self::StateDiff, VecCopyError> {
        Ok(VecCopyDiff::from(state_current.diff(state_goal)))
    }

    async fn state_clean(
        _params_partial: &<Self::Params<'_> as Params>::Partial,
        _data: Self::Data<'_>,
    ) -> Result<Self::State, VecCopyError> {
        Ok(VecCopyState::new())
    }

    async fn apply_check(
        _params: &Self::Params<'_>,
        _data: Self::Data<'_>,
        state_current: &Self::State,
        state_target: &Self::State,
        diff: &Self::StateDiff,
    ) -> Result<ApplyCheck, Self::Error> {
        let apply_check = if diff.0 .0.is_empty() {
            ApplyCheck::ExecNotRequired
        } else {
            #[cfg(not(feature = "output_progress"))]
            {
                let _state_current = state_current;
                let _state_target = state_target;
                ApplyCheck::ExecRequired
            }
            #[cfg(feature = "output_progress")]
            {
                let progress_limit =
                    TryInto::<u64>::try_into(state_current.len() + state_target.len())
                        .map(ProgressLimit::Bytes)
                        .unwrap_or(ProgressLimit::Unknown);

                ApplyCheck::ExecRequired { progress_limit }
            }
        };
        Ok(apply_check)
    }

    async fn apply_dry(
        _fn_ctx: FnCtx<'_>,
        _params: &Self::Params<'_>,
        _data: Self::Data<'_>,
        _state_current: &Self::State,
        state_target: &Self::State,
        _diff: &Self::StateDiff,
    ) -> Result<Self::State, Self::Error> {
        // Would replace vec_b's contents with vec_a's
        Ok(state_target.clone())
    }

    async fn apply(
        fn_ctx: FnCtx<'_>,
        _params: &Self::Params<'_>,
        mut data: Self::Data<'_>,
        _state_current: &Self::State,
        state_target: &Self::State,
        _diff: &Self::StateDiff,
    ) -> Result<Self::State, Self::Error> {
        let dest = data.dest_mut();
        dest.0.clear();
        dest.0.extend_from_slice(state_target.as_slice());

        #[cfg(not(feature = "output_progress"))]
        let _fn_ctx = fn_ctx;
        #[cfg(feature = "output_progress")]
        if let Ok(n) = state_target.len().try_into() {
            fn_ctx.progress_sender().inc(n, ProgressMsgUpdate::NoChange);
        }

        Ok(state_target.clone())
    }

    async fn setup(&self, resources: &mut Resources<Empty>) -> Result<(), VecCopyError> {
        let vec_b = {
            let states_current_stored =
                <RMaybe<'_, StatesCurrentStored> as Data>::borrow(Self::ID_DEFAULT, resources);
            let vec_copy_state_current_stored: Option<&'_ VecCopyState> = states_current_stored
                .as_ref()
                .and_then(|states_current_stored| states_current_stored.get(self.id()));
            if let Some(vec_copy_state) = vec_copy_state_current_stored {
                VecB(vec_copy_state.to_vec())
            } else {
                VecB::default()
            }
        };
        resources.insert(vec_b);
        Ok(())
    }

    #[cfg(feature = "item_interactions")]
    fn interactions(
        _params: &Self::Params<'_>,
        _data: Self::Data<'_>,
    ) -> Vec<peace::item_interaction_model::ItemInteraction> {
        use peace::item_interaction_model::{ItemInteractionPush, ItemLocation};

        let item_interaction = ItemInteractionPush::new(
            vec![
                ItemLocation::localhost(),
                ItemLocation::path("Vec A".to_string()),
            ]
            .into(),
            vec![
                ItemLocation::localhost(),
                ItemLocation::path("Vec B".to_string()),
            ]
            .into(),
        )
        .into();

        vec![item_interaction]
    }
}

#[cfg(feature = "error_reporting")]
use peace::miette;

/// Error while executing a VecCopy.
#[cfg_attr(feature = "error_reporting", derive(peace::miette::Diagnostic))]
#[derive(Debug, thiserror::Error)]
pub enum VecCopyError {
    /// A `peace` runtime error occurred.
    #[error("A `peace` runtime error occurred.")]
    PeaceRtError(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        peace::rt_model::Error,
    ),
}

#[derive(Data, Debug)]
pub struct VecCopyData<'exec> {
    /// Destination `Vec` to write to.
    dest: W<'exec, VecB>,
}

impl VecCopyData<'_> {
    pub fn dest(&self) -> &VecB {
        &self.dest
    }

    pub fn dest_mut(&mut self) -> &mut VecB {
        &mut self.dest
    }
}

#[derive(Clone, Debug, Default, Params, PartialEq, Eq, Serialize, Deserialize)]
pub struct VecA(pub Vec<u8>);

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VecB(pub Vec<u8>);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecCopyState(Vec<u8>);

impl VecCopyState {
    /// Returns an empty `VecCopyState`.
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl From<Vec<u8>> for VecCopyState {
    fn from(v: Vec<u8>) -> Self {
        Self(v)
    }
}

impl Deref for VecCopyState {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VecCopyState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for VecCopyState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Default for VecCopyState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "output_progress")]
impl<'state> From<&'state VecCopyState> for ItemLocationState {
    fn from(_vec_copy_state: &'state VecCopyState) -> ItemLocationState {
        ItemLocationState::Exists
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecCopyDiff(VecDiff<u8>);

impl From<VecDiff<u8>> for VecCopyDiff {
    fn from(v: VecDiff<u8>) -> Self {
        Self(v)
    }
}

impl Deref for VecCopyDiff {
    type Target = VecDiff<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VecCopyDiff {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for VecCopyDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        self.0
             .0
            .iter()
            .try_for_each(|vec_diff_type| match vec_diff_type {
                VecDiffType::Removed { index, len } => {
                    let index_end = index + len;
                    write!(f, "(-){index}..{index_end}, ")
                }
                VecDiffType::Altered { index, changes } => {
                    write!(f, "(~){index};")?;
                    changes.iter().try_for_each(|value| write!(f, "{value}, "))
                }
                VecDiffType::Inserted { index, changes } => {
                    write!(f, "(+){index};")?;
                    changes.iter().try_for_each(|value| write!(f, "{value}, "))
                }
            })?;
        write!(f, "]")
    }
}
