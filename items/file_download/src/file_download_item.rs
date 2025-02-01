use std::{marker::PhantomData, path::Path};

use peace::{
    cfg::{async_trait, state::FetchedOpt, ApplyCheck, FnCtx, Item},
    item_model::ItemId,
    params::Params,
    resource_rt::{resources::ts::Empty, Resources},
};

use crate::{
    FileDownloadApplyFns, FileDownloadData, FileDownloadError, FileDownloadParams,
    FileDownloadState, FileDownloadStateCurrentFn, FileDownloadStateDiff, FileDownloadStateDiffFn,
    FileDownloadStateGoalFn, FileDownloadStateLogical,
};

/// Item for downloading a file.
///
/// The `Id` type parameter is needed for each file download params to be a
/// distinct type.
///
/// # Type Parameters
///
/// * `Id`: A zero-sized type used to distinguish different file download
///   parameters from each other.
#[derive(Debug)]
pub struct FileDownloadItem<Id> {
    /// ID of the item to download the file.
    item_id: ItemId,
    /// Marker for unique download parameters type.
    marker: PhantomData<Id>,
}

impl<Id> Clone for FileDownloadItem<Id> {
    fn clone(&self) -> Self {
        Self {
            item_id: self.item_id.clone(),
            marker: PhantomData,
        }
    }
}

impl<Id> FileDownloadItem<Id> {
    /// Returns a new `FileDownloadItem`.
    pub fn new(item_id: ItemId) -> Self {
        Self {
            item_id,
            marker: PhantomData,
        }
    }
}

#[async_trait(?Send)]
impl<Id> Item for FileDownloadItem<Id>
where
    Id: Send + Sync + 'static,
{
    type Data<'exec> = FileDownloadData<'exec, Id>;
    type Error = FileDownloadError;
    type Params<'exec> = FileDownloadParams<Id>;
    type State = FileDownloadState;
    type StateDiff = FileDownloadStateDiff;

    fn id(&self) -> &ItemId {
        &self.item_id
    }

    async fn setup(&self, resources: &mut Resources<Empty>) -> Result<(), FileDownloadError> {
        resources.insert::<reqwest::Client>(reqwest::Client::new());

        Ok(())
    }

    #[cfg(feature = "item_state_example")]
    fn state_example(params: &Self::Params<'_>, _data: Self::Data<'_>) -> Self::State {
        let dest = params.dest();

        FileDownloadState::new(
            FileDownloadStateLogical::StringContents {
                path: dest.to_path_buf(),
                contents: "example contents".to_string(),
            },
            FetchedOpt::None,
        )
    }

    async fn try_state_current(
        fn_ctx: FnCtx<'_>,
        params_partial: &<Self::Params<'_> as Params>::Partial,
        data: FileDownloadData<'_, Id>,
    ) -> Result<Option<Self::State>, FileDownloadError> {
        FileDownloadStateCurrentFn::try_state_current(fn_ctx, params_partial, data).await
    }

    async fn state_current(
        fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        data: FileDownloadData<'_, Id>,
    ) -> Result<Self::State, FileDownloadError> {
        FileDownloadStateCurrentFn::state_current(fn_ctx, params, data).await
    }

    async fn try_state_goal(
        fn_ctx: FnCtx<'_>,
        params_partial: &<Self::Params<'_> as Params>::Partial,
        data: FileDownloadData<'_, Id>,
    ) -> Result<Option<Self::State>, FileDownloadError> {
        FileDownloadStateGoalFn::try_state_goal(fn_ctx, params_partial, data).await
    }

    async fn state_goal(
        fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        data: FileDownloadData<'_, Id>,
    ) -> Result<Self::State, FileDownloadError> {
        FileDownloadStateGoalFn::state_goal(fn_ctx, params, data).await
    }

    async fn state_diff(
        _params_partial: &<Self::Params<'_> as Params>::Partial,
        _data: Self::Data<'_>,
        state_a: &Self::State,
        state_b: &Self::State,
    ) -> Result<Self::StateDiff, FileDownloadError> {
        FileDownloadStateDiffFn::state_diff(state_a, state_b).await
    }

    async fn state_clean(
        params_partial: &<Self::Params<'_> as Params>::Partial,
        _data: Self::Data<'_>,
    ) -> Result<Self::State, FileDownloadError> {
        let path = params_partial.dest().map(Path::to_path_buf);
        let state =
            FileDownloadState::new(FileDownloadStateLogical::None { path }, FetchedOpt::Tbd);
        Ok(state)
    }

    async fn apply_check(
        params: &Self::Params<'_>,
        data: Self::Data<'_>,
        state_current: &Self::State,
        state_target: &Self::State,
        diff: &Self::StateDiff,
    ) -> Result<ApplyCheck, Self::Error> {
        FileDownloadApplyFns::<Id>::apply_check(params, data, state_current, state_target, diff)
            .await
    }

    async fn apply_dry(
        fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        data: Self::Data<'_>,
        state_current: &Self::State,
        state_target: &Self::State,
        diff: &Self::StateDiff,
    ) -> Result<Self::State, Self::Error> {
        FileDownloadApplyFns::<Id>::apply_dry(
            fn_ctx,
            params,
            data,
            state_current,
            state_target,
            diff,
        )
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
        FileDownloadApplyFns::<Id>::apply(fn_ctx, params, data, state_current, state_target, diff)
            .await
    }

    #[cfg(feature = "item_interactions")]
    fn interactions(
        params: &Self::Params<'_>,
        _data: Self::Data<'_>,
    ) -> Vec<peace::item_interaction_model::ItemInteraction> {
        use peace::item_interaction_model::{
            ItemInteractionPull, ItemLocation, ItemLocationAncestors,
        };

        let location_server: ItemLocationAncestors = vec![
            ItemLocation::host_from_url(params.src()),
            ItemLocation::path(params.src().to_string()),
        ]
        .into();

        let location_client: ItemLocationAncestors = vec![
            ItemLocation::localhost(),
            ItemLocation::path(format!("📄 {}", params.dest().display())),
        ]
        .into();

        let item_interaction = ItemInteractionPull {
            location_client,
            location_server,
        }
        .into();

        vec![item_interaction]
    }
}
