use std::marker::PhantomData;

use peace::{
    cfg::{async_trait, ApplyCheck, FnCtx, Item},
    item_model::ItemId,
    params::Params,
    resource_rt::{resources::ts::Empty, Resources},
};

use crate::{BlankApplyFns, BlankData, BlankError, BlankParams, BlankState, BlankStateDiff};

/// Item for copying a number.
///
/// The `Id` type parameter is needed for each blank params to be a
/// distinct type.
///
/// # Type Parameters
///
/// * `Id`: A zero-sized type used to distinguish different blank parameters
///   from each other.
#[derive(Debug)]
pub struct BlankItem<Id> {
    /// ID of the blank item.
    item_id: ItemId,
    /// Marker for unique blank parameters type.
    marker: PhantomData<Id>,
}

impl<Id> BlankItem<Id> {
    /// Returns a new `BlankItem`.
    pub fn new(item_id: ItemId) -> Self {
        Self {
            item_id,
            marker: PhantomData,
        }
    }
}

impl<Id> Clone for BlankItem<Id> {
    fn clone(&self) -> Self {
        Self {
            item_id: self.item_id.clone(),
            marker: PhantomData,
        }
    }
}

#[async_trait(?Send)]
impl<Id> Item for BlankItem<Id>
where
    Id: Send + Sync + 'static,
{
    type Data<'exec> = BlankData<'exec, Id>;
    type Error = BlankError;
    type Params<'exec> = BlankParams<Id>;
    type State = BlankState;
    type StateDiff = BlankStateDiff;

    fn id(&self) -> &ItemId {
        &self.item_id
    }

    async fn setup(&self, _resources: &mut Resources<Empty>) -> Result<(), BlankError> {
        Ok(())
    }

    #[cfg(feature = "item_state_example")]
    fn state_example(params: &Self::Params<'_>, _data: Self::Data<'_>) -> Self::State {
        BlankState(params.dest.0)
    }

    async fn try_state_current(
        _fn_ctx: FnCtx<'_>,
        params_partial: &<Self::Params<'_> as Params>::Partial,
        _data: BlankData<'_, Id>,
    ) -> Result<Option<Self::State>, BlankError> {
        Ok(params_partial.dest.clone().map(|dest| BlankState(dest.0)))
    }

    async fn state_current(
        _fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        _data: BlankData<'_, Id>,
    ) -> Result<Self::State, BlankError> {
        Ok(BlankState(params.dest.0))
    }

    async fn try_state_goal(
        _fn_ctx: FnCtx<'_>,
        params_partial: &<Self::Params<'_> as Params>::Partial,
        _data: BlankData<'_, Id>,
    ) -> Result<Option<Self::State>, BlankError> {
        Ok(params_partial
            .src
            .clone()
            .map(|src| BlankState(Some(src.0))))
    }

    async fn state_goal(
        _fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        _data: BlankData<'_, Id>,
    ) -> Result<Self::State, BlankError> {
        Ok(BlankState(Some(params.src.0)))
    }

    async fn state_diff(
        _params_partial: &<Self::Params<'_> as Params>::Partial,
        _data: Self::Data<'_>,
        state_current: &BlankState,
        state_goal: &BlankState,
    ) -> Result<Self::StateDiff, BlankError> {
        let diff = match (state_current, state_goal) {
            (BlankState(Some(current)), BlankState(Some(goal))) if current == goal => {
                BlankStateDiff::InSync { value: *current }
            }
            (BlankState(Some(current)), BlankState(Some(goal))) => BlankStateDiff::OutOfSync {
                diff: i64::from(goal - current),
            },
            (BlankState(None), BlankState(Some(goal))) => BlankStateDiff::Added { value: *goal },
            (BlankState(_), BlankState(None)) => unreachable!("goal state is always Some"),
        };

        Ok(diff)
    }

    async fn state_clean(
        _params_partial: &<Self::Params<'_> as Params>::Partial,
        _data: Self::Data<'_>,
    ) -> Result<BlankState, BlankError> {
        Ok(BlankState(None))
    }

    async fn apply_check(
        params: &Self::Params<'_>,
        data: Self::Data<'_>,
        state_current: &Self::State,
        state_target: &Self::State,
        diff: &Self::StateDiff,
    ) -> Result<ApplyCheck, Self::Error> {
        BlankApplyFns::<Id>::apply_check(params, data, state_current, state_target, diff).await
    }

    async fn apply_dry(
        fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        data: Self::Data<'_>,
        state_current: &Self::State,
        state_target: &Self::State,
        diff: &Self::StateDiff,
    ) -> Result<Self::State, Self::Error> {
        BlankApplyFns::<Id>::apply_dry(fn_ctx, params, data, state_current, state_target, diff)
            .await
    }

    async fn apply(
        fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        data: Self::Data<'_>,
        state_current: &Self::State,
        state_target: &Self::State,
        diff: &Self::StateDiff,
    ) -> Result<Self::State, Self::Error> {
        BlankApplyFns::<Id>::apply(fn_ctx, params, data, state_current, state_target, diff).await
    }

    #[cfg(feature = "item_interactions")]
    fn interactions(
        _params: &Self::Params<'_>,
        _data: Self::Data<'_>,
    ) -> Vec<peace::item_interaction_model::ItemInteraction> {
        use peace::item_interaction_model::{ItemInteractionWithin, ItemLocation};

        let item_interaction =
            ItemInteractionWithin::new(vec![ItemLocation::localhost()].into()).into();

        vec![item_interaction]
    }
}
