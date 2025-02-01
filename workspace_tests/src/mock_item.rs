use std::{
    fmt,
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use peace::{
    cfg::{async_trait, ApplyCheck, FnCtx, Item},
    data::{
        accessors::{RMaybe, R, W},
        Data,
    },
    item_model::{item_id, ItemId},
    params::Params,
    resource_rt::{resources::ts::Empty, states::StatesCurrentStored, Resources},
};
#[cfg(feature = "output_progress")]
use peace::{
    item_interaction_model::ItemLocationState,
    progress_model::{ProgressLimit, ProgressMsgUpdate},
};
use serde::{Deserialize, Serialize};

/// Copies a number from `MockSrc` to `MockDest`.
///
/// This also allows each item function to be overridden.
#[derive(Clone, Debug)]
pub struct MockItem<Id>
where
    Id: Clone + Debug + Default + Send + Sync + 'static,
{
    /// ID of the item.
    id: ItemId,
    /// Marker.
    mock_fns: MockFns<Id>,
}

type FnTryState<Id> = fn(
    FnCtx<'_>,
    &<MockSrc as Params>::Partial,
    MockData<'_, Id>,
) -> Result<Option<MockState>, MockItemError>;

type FnStateClean<Id> =
    fn(&<MockSrc as Params>::Partial, MockData<'_, Id>) -> Result<MockState, MockItemError>;

type FnState<Id> = fn(FnCtx<'_>, &MockSrc, MockData<'_, Id>) -> Result<MockState, MockItemError>;

type FnApplyCheck<Id> = fn(
    &MockSrc,
    MockData<'_, Id>,
    &MockState,
    &MockState,
    &MockDiff,
) -> Result<ApplyCheck, MockItemError>;

type FnApply<Id> = fn(
    FnCtx<'_>,
    &MockSrc,
    MockData<'_, Id>,
    &MockState,
    &MockState,
    &MockDiff,
) -> Result<MockState, MockItemError>;

/// Copies bytes from `MockSrc` to `MockDest`.
#[derive(Clone, Debug, Default)]
pub struct MockFns<Id>
where
    Id: Clone + Debug + Default + Send + Sync + 'static,
{
    /// Override for `state_clean` function.
    state_clean: Option<FnStateClean<Id>>,
    /// Override for `try_state_current` function.
    try_state_current: Option<FnTryState<Id>>,
    /// Override for `state_current` function.
    state_current: Option<FnState<Id>>,
    /// Override for `try_state_goal` function.
    try_state_goal: Option<FnTryState<Id>>,
    /// Override for `state_goal` function.
    state_goal: Option<FnState<Id>>,
    /// Override for `apply_check` function.
    apply_check: Option<FnApplyCheck<Id>>,
    /// Override for `apply_dry` function.
    apply_dry: Option<FnApply<Id>>,
    /// Override for `apply` function.
    apply: Option<FnApply<Id>>,
    /// Marker.
    marker: PhantomData<Id>,
}

impl<Id> MockItem<Id>
where
    Id: Clone + Debug + Default + Send + Sync + 'static,
{
    pub const ID_DEFAULT: &'static ItemId = &item_id!("mock");

    pub fn new(id: ItemId) -> Self {
        Self {
            id,
            mock_fns: MockFns::<Id>::default(),
        }
    }

    pub fn with_state_clean(mut self, f: FnStateClean<Id>) -> Self {
        self.mock_fns.state_clean = Some(f);
        self
    }

    pub fn with_try_state_current(mut self, f: FnTryState<Id>) -> Self {
        self.mock_fns.try_state_current = Some(f);
        self
    }

    pub fn with_state_current(mut self, f: FnState<Id>) -> Self {
        self.mock_fns.state_current = Some(f);
        self
    }

    pub fn with_try_state_goal(mut self, f: FnTryState<Id>) -> Self {
        self.mock_fns.try_state_goal = Some(f);
        self
    }

    pub fn with_state_goal(mut self, f: FnState<Id>) -> Self {
        self.mock_fns.state_goal = Some(f);
        self
    }

    pub fn with_apply_check(mut self, f: FnApplyCheck<Id>) -> Self {
        self.mock_fns.apply_check = Some(f);
        self
    }

    pub fn with_apply_dry(mut self, f: FnApply<Id>) -> Self {
        self.mock_fns.apply_dry = Some(f);
        self
    }

    pub fn with_apply(mut self, f: FnApply<Id>) -> Self {
        self.mock_fns.apply = Some(f);
        self
    }

    async fn state_current_internal(
        fn_ctx: FnCtx<'_>,
        data: MockData<'_, Id>,
    ) -> Result<MockState, MockItemError> {
        #[cfg(not(feature = "output_progress"))]
        let _fn_ctx = fn_ctx;

        let mock_state = MockState(data.dest().0);

        #[cfg(feature = "output_progress")]
        {
            let n = u64::from(mock_state.0);
            fn_ctx.progress_sender.inc(n, ProgressMsgUpdate::NoChange);
        }

        Ok(mock_state)
    }

    async fn state_goal_internal(
        fn_ctx: FnCtx<'_>,
        mock_src: &u8,
    ) -> Result<MockState, MockItemError> {
        #[cfg(not(feature = "output_progress"))]
        let _fn_ctx = fn_ctx;
        let mock_state = MockState(*mock_src);

        #[cfg(feature = "output_progress")]
        {
            let n = u64::from(mock_state.0);
            fn_ctx.progress_sender.inc(n, ProgressMsgUpdate::NoChange);
        }

        Ok(mock_state)
    }
}

impl<Id> Default for MockItem<Id>
where
    Id: Clone + Debug + Default + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new(Self::ID_DEFAULT.clone())
    }
}

#[async_trait(?Send)]
impl<Id> Item for MockItem<Id>
where
    Id: Clone + Debug + Default + Send + Sync + 'static,
{
    type Data<'exec> = MockData<'exec, Id>;
    type Error = MockItemError;
    type Params<'exec> = MockSrc;
    type State = MockState;
    type StateDiff = MockDiff;

    fn id(&self) -> &ItemId {
        &self.id
    }

    #[cfg(feature = "item_state_example")]
    fn state_example(params: &Self::Params<'_>, _data: Self::Data<'_>) -> Self::State {
        MockState(params.0)
    }

    async fn state_clean(
        params_partial: &<Self::Params<'_> as Params>::Partial,
        data: Self::Data<'_>,
    ) -> Result<Self::State, MockItemError> {
        if let Some(state_clean) = data.mock_fns().state_clean.as_ref() {
            state_clean(params_partial, data)
        } else {
            Ok(MockState::new())
        }
    }

    async fn try_state_current(
        fn_ctx: FnCtx<'_>,
        params_partial: &<Self::Params<'_> as Params>::Partial,
        data: Self::Data<'_>,
    ) -> Result<Option<Self::State>, MockItemError> {
        if let Some(try_state_current) = data.mock_fns().try_state_current.as_ref() {
            try_state_current(fn_ctx, params_partial, data)
        } else {
            Self::state_current_internal(fn_ctx, data).await.map(Some)
        }
    }

    async fn state_current(
        fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        data: Self::Data<'_>,
    ) -> Result<Self::State, MockItemError> {
        if let Some(state_current) = data.mock_fns().state_current.as_ref() {
            state_current(fn_ctx, params, data)
        } else {
            Self::state_current_internal(fn_ctx, data).await
        }
    }

    async fn try_state_goal(
        fn_ctx: FnCtx<'_>,
        params_partial: &<Self::Params<'_> as Params>::Partial,
        data: Self::Data<'_>,
    ) -> Result<Option<Self::State>, MockItemError> {
        if let Some(try_state_goal) = data.mock_fns().try_state_goal.as_ref() {
            try_state_goal(fn_ctx, params_partial, data)
        } else if let Some(mock_src) = params_partial.0.as_ref() {
            Self::state_goal_internal(fn_ctx, mock_src).await.map(Some)
        } else {
            Ok(None)
        }
    }

    async fn state_goal(
        fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        data: Self::Data<'_>,
    ) -> Result<Self::State, MockItemError> {
        if let Some(state_goal) = data.mock_fns().state_goal.as_ref() {
            state_goal(fn_ctx, params, data)
        } else {
            Self::state_goal_internal(fn_ctx, &params.0).await
        }
    }

    async fn state_diff(
        _params_partial: &<Self::Params<'_> as Params>::Partial,
        _data: MockData<'_, Id>,
        state_current: &MockState,
        state_goal: &MockState,
    ) -> Result<Self::StateDiff, MockItemError> {
        Ok(MockDiff(
            i16::from(state_goal.0) - i16::from(state_current.0),
        ))
    }

    async fn apply_check(
        params: &Self::Params<'_>,
        data: Self::Data<'_>,
        state_current: &Self::State,
        state_target: &Self::State,
        diff: &Self::StateDiff,
    ) -> Result<ApplyCheck, Self::Error> {
        if let Some(apply_check) = data.mock_fns().apply_check.as_ref() {
            apply_check(params, data, state_current, state_target, diff)
        } else {
            let apply_check = if diff.0 == 0 {
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
                    let progress_limit = {
                        let byte_count = u64::from(state_current.0 + state_target.0);
                        ProgressLimit::Bytes(byte_count)
                    };

                    ApplyCheck::ExecRequired { progress_limit }
                }
            };
            Ok(apply_check)
        }
    }

    async fn apply_dry(
        fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        data: Self::Data<'_>,
        state_current: &Self::State,
        state_target: &Self::State,
        diff: &Self::StateDiff,
    ) -> Result<Self::State, Self::Error> {
        if let Some(apply_dry) = data.mock_fns().apply_dry.as_ref() {
            apply_dry(fn_ctx, params, data, state_current, state_target, diff)
        } else {
            // Would replace mock_dest's contents with mock_src's
            Ok(state_target.clone())
        }
    }

    async fn apply(
        fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        mut data: Self::Data<'_>,
        state_current: &Self::State,
        state_target: &Self::State,
        diff: &Self::StateDiff,
    ) -> Result<Self::State, Self::Error> {
        if let Some(apply) = data.mock_fns().apply.as_ref() {
            apply(fn_ctx, params, data, state_current, state_target, diff)
        } else {
            let dest = data.dest_mut();
            dest.0 = state_target.0;

            #[cfg(not(feature = "output_progress"))]
            let _fn_ctx = fn_ctx;
            #[cfg(feature = "output_progress")]
            {
                let n = state_target.0.into();
                fn_ctx.progress_sender().inc(n, ProgressMsgUpdate::NoChange);
            }

            Ok(state_target.clone())
        }
    }

    async fn setup(&self, resources: &mut Resources<Empty>) -> Result<(), MockItemError> {
        resources.insert(self.mock_fns.clone());

        let mock_dest = {
            let states_current_stored =
                <RMaybe<'_, StatesCurrentStored> as Data>::borrow(Self::ID_DEFAULT, resources);
            let mock_state_current_stored: Option<&'_ MockState> = states_current_stored
                .as_ref()
                .and_then(|states_current_stored| states_current_stored.get(self.id()));
            if let Some(mock_state) = mock_state_current_stored {
                MockDest(mock_state.0)
            } else {
                MockDest::default()
            }
        };
        resources.insert(mock_dest);
        Ok(())
    }

    #[cfg(feature = "item_interactions")]
    fn interactions(
        _params: &Self::Params<'_>,
        _data: Self::Data<'_>,
    ) -> Vec<peace::item_interaction_model::ItemInteraction> {
        use peace::item_interaction_model::{ItemInteractionWithin, ItemLocation};

        vec![ItemInteractionWithin::new(vec![ItemLocation::localhost()].into()).into()]
    }
}

#[cfg(feature = "error_reporting")]
use peace::miette;

/// Error while executing a Mock.
#[cfg_attr(feature = "error_reporting", derive(peace::miette::Diagnostic))]
#[derive(Debug, thiserror::Error)]
pub enum MockItemError {
    /// A synthetic error.
    #[error("Synthetic error: {}.", _0)]
    Synthetic(String),

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
pub struct MockData<'exec, Id>
where
    Id: Clone + Debug + Default + Send + Sync + 'static,
{
    /// Mock functions for this item.
    mock_fns: R<'exec, MockFns<Id>>,
    /// Destination `Vec` to write to.
    dest: W<'exec, MockDest>,
}

impl<Id> MockData<'_, Id>
where
    Id: Clone + Debug + Default + Send + Sync + 'static,
{
    pub fn mock_fns(&self) -> &MockFns<Id> {
        &self.mock_fns
    }

    pub fn dest(&self) -> &MockDest {
        &self.dest
    }

    pub fn dest_mut(&mut self) -> &mut MockDest {
        &mut self.dest
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Params)]
pub struct MockSrc(pub u8);

impl Deref for MockSrc {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MockSrc {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for MockSrc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MockDest(pub u8);

impl Deref for MockDest {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MockDest {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for MockDest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MockState(pub u8);

impl MockState {
    /// Returns an empty `MockState`.
    pub fn new() -> Self {
        Self(0)
    }
}

impl Deref for MockState {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MockState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for MockState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[cfg(feature = "output_progress")]
impl<'state> From<&'state MockState> for ItemLocationState {
    fn from(_mock_state: &'state MockState) -> ItemLocationState {
        ItemLocationState::Exists
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MockDiff(pub i16);

impl Deref for MockDiff {
    type Target = i16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MockDiff {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for MockDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use crate::mock_item::{MockDest, MockDiff, MockItem, MockSrc, MockState};

    #[test]
    fn clone() {
        let _mock_item = Clone::clone(&MockItem::<()>::default());
        let _mock_src = Clone::clone(&MockSrc::default());
        let _mock_dest = Clone::clone(&MockDest::default());
        let _mock_state = Clone::clone(&MockState::default());
        let _mock_diff = Clone::clone(&MockDiff::default());
    }

    #[test]
    fn deref() {
        let mock_src = MockSrc::default();
        let mock_dest = MockDest::default();
        let mock_state = MockState::default();
        let mock_diff = MockDiff::default();

        assert_eq!(0, *mock_src);
        assert_eq!(0, *mock_dest);
        assert_eq!(0, *mock_state);
        assert_eq!(0, *mock_diff);
    }

    #[test]
    fn deref_mut() {
        let mut mock_src = MockSrc::default();
        let mut mock_dest = MockDest::default();
        let mut mock_state = MockState::default();
        let mut mock_diff = MockDiff::default();

        *mock_src = 1;
        *mock_dest = 1;
        *mock_state = 1;
        *mock_diff = 1;

        assert_eq!(1, *mock_src);
        assert_eq!(1, *mock_dest);
        assert_eq!(1, *mock_state);
        assert_eq!(1, *mock_diff);
    }

    #[test]
    fn display() {
        let mock_src = MockSrc::default();
        let mock_dest = MockDest::default();
        let mock_state = MockState::default();
        let mock_diff = MockDiff::default();

        assert_eq!("0", format!("{mock_src}"));
        assert_eq!("0", format!("{mock_dest}"));
        assert_eq!("0", format!("{mock_state}"));
        assert_eq!("0", format!("{mock_diff}"));
    }

    #[test]
    fn debug() {
        let mock_item = MockItem::<()>::default();
        let mock_src = MockSrc::default();
        let mock_dest = MockDest::default();
        let mock_state = MockState::default();
        let mock_diff = MockDiff::default();

        assert_eq!(
            "MockItem { \
                id: ItemId(\"mock\"), \
                mock_fns: MockFns { \
                    state_clean: None, \
                    try_state_current: None, \
                    state_current: None, \
                    try_state_goal: None, \
                    state_goal: None, \
                    apply_check: None, \
                    apply_dry: None, \
                    apply: None, \
                    marker: PhantomData<()> \
                } \
             }",
            format!("{mock_item:?}")
        );
        assert_eq!("MockSrc(0)", format!("{mock_src:?}"));
        assert_eq!("MockDest(0)", format!("{mock_dest:?}"));
        assert_eq!("MockState(0)", format!("{mock_state:?}"));
        assert_eq!("MockDiff(0)", format!("{mock_diff:?}"));
    }
}
