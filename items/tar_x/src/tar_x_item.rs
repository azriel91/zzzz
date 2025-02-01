use std::marker::PhantomData;

use peace::{
    cfg::{async_trait, ApplyCheck, FnCtx, Item},
    item_model::ItemId,
    params::Params,
    resource_rt::{resources::ts::Empty, Resources},
};

use crate::{
    FileMetadatas, TarXApplyFns, TarXData, TarXError, TarXParams, TarXStateCurrentFn,
    TarXStateDiff, TarXStateDiffFn, TarXStateGoalFn,
};

/// Item for extracting a tar file.
///
/// The `Id` type parameter is needed for each tar extraction params to be a
/// distinct type.
///
/// The following use cases are intended to be supported:
///
/// * A pristine directory with only the tar's contents and nothing else (in
///   progress).
/// * Extraction of a tar over an existing directory (not yet implemented).
///
/// # Type Parameters
///
/// * `Id`: A zero-sized type used to distinguish different tar extraction
///   parameters from each other.
#[derive(Debug)]
pub struct TarXItem<Id> {
    /// ID of the item to extract the tar.
    item_id: ItemId,
    /// Marker for unique tar extraction parameters type.
    marker: PhantomData<Id>,
}

impl<Id> Clone for TarXItem<Id> {
    fn clone(&self) -> Self {
        Self {
            item_id: self.item_id.clone(),
            marker: PhantomData,
        }
    }
}

impl<Id> TarXItem<Id> {
    /// Returns a new `TarXItem`.
    pub fn new(item_id: ItemId) -> Self {
        Self {
            item_id,
            marker: PhantomData,
        }
    }
}

#[async_trait(?Send)]
impl<Id> Item for TarXItem<Id>
where
    Id: Send + Sync + 'static,
{
    type Data<'exec> = TarXData<'exec, Id>;
    type Error = TarXError;
    type Params<'exec> = TarXParams<Id>;
    type State = FileMetadatas;
    type StateDiff = TarXStateDiff;

    fn id(&self) -> &ItemId {
        &self.item_id
    }

    async fn setup(&self, _resources: &mut Resources<Empty>) -> Result<(), TarXError> {
        Ok(())
    }

    #[cfg(feature = "item_state_example")]
    fn state_example(_params: &Self::Params<'_>, _data: Self::Data<'_>) -> Self::State {
        use std::{
            path::PathBuf,
            time::{Duration, SystemTime, UNIX_EPOCH},
        };

        use crate::FileMetadata;

        let mtime = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .as_ref()
            .map(Duration::as_secs)
            .unwrap_or(0u64);
        let files_extracted = vec![
            FileMetadata::new(PathBuf::from(String::from("tar_x_example_1.txt")), mtime),
            FileMetadata::new(PathBuf::from(String::from("tar_x_example_2.txt")), mtime),
        ];

        FileMetadatas::from(files_extracted)
    }

    async fn try_state_current(
        fn_ctx: FnCtx<'_>,
        params_partial: &<Self::Params<'_> as Params>::Partial,
        data: TarXData<'_, Id>,
    ) -> Result<Option<Self::State>, TarXError> {
        TarXStateCurrentFn::try_state_current(fn_ctx, params_partial, data).await
    }

    async fn state_current(
        fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        data: TarXData<'_, Id>,
    ) -> Result<Self::State, TarXError> {
        TarXStateCurrentFn::state_current(fn_ctx, params, data).await
    }

    async fn try_state_goal(
        fn_ctx: FnCtx<'_>,
        params_partial: &<Self::Params<'_> as Params>::Partial,
        data: TarXData<'_, Id>,
    ) -> Result<Option<Self::State>, TarXError> {
        TarXStateGoalFn::try_state_goal(fn_ctx, params_partial, data).await
    }

    async fn state_goal(
        fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        data: TarXData<'_, Id>,
    ) -> Result<Self::State, TarXError> {
        TarXStateGoalFn::state_goal(fn_ctx, params, data).await
    }

    async fn state_diff(
        _params_partial: &<Self::Params<'_> as Params>::Partial,
        _data: Self::Data<'_>,
        state_current: &Self::State,
        state_goal: &Self::State,
    ) -> Result<Self::StateDiff, TarXError> {
        TarXStateDiffFn::state_diff(state_current, state_goal).await
    }

    async fn state_clean(
        _params_partial: &<Self::Params<'_> as Params>::Partial,
        _data: Self::Data<'_>,
    ) -> Result<Self::State, TarXError> {
        Ok(FileMetadatas::default())
    }

    async fn apply_check(
        params: &Self::Params<'_>,
        data: Self::Data<'_>,
        state_current: &Self::State,
        state_target: &Self::State,
        diff: &Self::StateDiff,
    ) -> Result<ApplyCheck, Self::Error> {
        TarXApplyFns::<Id>::apply_check(params, data, state_current, state_target, diff).await
    }

    async fn apply_dry(
        fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        data: Self::Data<'_>,
        state_current: &Self::State,
        state_target: &Self::State,
        diff: &Self::StateDiff,
    ) -> Result<Self::State, Self::Error> {
        TarXApplyFns::<Id>::apply_dry(fn_ctx, params, data, state_current, state_target, diff).await
    }

    async fn apply(
        fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        data: Self::Data<'_>,
        state_current: &Self::State,
        state_target: &Self::State,
        diff: &Self::StateDiff,
    ) -> Result<Self::State, Self::Error> {
        TarXApplyFns::<Id>::apply(fn_ctx, params, data, state_current, state_target, diff).await
    }

    #[cfg(feature = "item_interactions")]
    fn interactions(
        params: &Self::Params<'_>,
        _data: Self::Data<'_>,
    ) -> Vec<peace::item_interaction_model::ItemInteraction> {
        use peace::item_interaction_model::{
            ItemInteractionWithin, ItemLocation, ItemLocationAncestors,
        };

        let location: ItemLocationAncestors = vec![
            ItemLocation::localhost(),
            ItemLocation::path(format!("📁 {}", params.dest().display())),
        ]
        .into();
        let item_interaction = ItemInteractionWithin::new(location).into();

        vec![item_interaction]
    }
}
