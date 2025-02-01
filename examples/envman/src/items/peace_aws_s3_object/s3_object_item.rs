use std::marker::PhantomData;

use aws_config::BehaviorVersion;
use peace::{
    cfg::{async_trait, ApplyCheck, FnCtx, Item},
    item_model::ItemId,
    params::Params,
    resource_rt::{resources::ts::Empty, Resources},
};

use crate::items::peace_aws_s3_object::{
    S3ObjectApplyFns, S3ObjectData, S3ObjectError, S3ObjectParams, S3ObjectState,
    S3ObjectStateCurrentFn, S3ObjectStateDiff, S3ObjectStateDiffFn, S3ObjectStateGoalFn,
};

/// Item to create an IAM S3 object and IAM role.
///
/// In sequence, this will:
///
/// * Create the IAM Role.
/// * Create the S3 object.
/// * Add the IAM role to the S3 object.
///
/// The `Id` type parameter is needed for each S3 object params to be a
/// distinct type.
///
/// # Type Parameters
///
/// * `Id`: A zero-sized type used to distinguish different S3 object parameters
///   from each other.
#[derive(Debug)]
pub struct S3ObjectItem<Id> {
    /// ID of the S3 object item.
    item_id: ItemId,
    /// Marker for unique S3 object parameters type.
    marker: PhantomData<Id>,
}

impl<Id> S3ObjectItem<Id> {
    /// Returns a new `S3ObjectItem`.
    pub fn new(item_id: ItemId) -> Self {
        Self {
            item_id,
            marker: PhantomData,
        }
    }
}

impl<Id> Clone for S3ObjectItem<Id> {
    fn clone(&self) -> Self {
        Self {
            item_id: self.item_id.clone(),
            marker: PhantomData,
        }
    }
}

#[async_trait(?Send)]
impl<Id> Item for S3ObjectItem<Id>
where
    Id: Send + Sync + 'static,
{
    type Data<'exec> = S3ObjectData<'exec, Id>;
    type Error = S3ObjectError;
    type Params<'exec> = S3ObjectParams<Id>;
    type State = S3ObjectState;
    type StateDiff = S3ObjectStateDiff;

    fn id(&self) -> &ItemId {
        &self.item_id
    }

    async fn setup(&self, resources: &mut Resources<Empty>) -> Result<(), S3ObjectError> {
        if !resources.contains::<aws_sdk_s3::Client>() {
            let sdk_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
            resources.insert(sdk_config.region().cloned());
            let client = aws_sdk_s3::Client::new(&sdk_config);
            resources.insert(client);
        }
        Ok(())
    }

    #[cfg(feature = "item_state_example")]
    fn state_example(params: &Self::Params<'_>, _data: Self::Data<'_>) -> Self::State {
        use std::fmt::Write;

        use peace::cfg::state::Generated;

        let example_content = b"s3_object_example";

        let content_md5_hexstr = {
            let content_md5_bytes = {
                let mut md5_ctx = md5_rs::Context::new();
                md5_ctx.read(example_content);
                md5_ctx.finish()
            };
            content_md5_bytes
                .iter()
                .try_fold(
                    String::with_capacity(content_md5_bytes.len() * 2),
                    |mut hexstr, x| {
                        write!(&mut hexstr, "{:02x}", x)?;
                        Result::<_, std::fmt::Error>::Ok(hexstr)
                    },
                )
                .expect("Failed to construct hexstring from S3 object MD5.")
        };

        S3ObjectState::Some {
            bucket_name: params.bucket_name().to_string(),
            object_key: params.object_key().to_string(),
            content_md5_hexstr: Some(content_md5_hexstr.clone()),
            e_tag: Generated::Value(content_md5_hexstr),
        }
    }

    async fn try_state_current(
        fn_ctx: FnCtx<'_>,
        params_partial: &<Self::Params<'_> as Params>::Partial,
        data: Self::Data<'_>,
    ) -> Result<Option<Self::State>, S3ObjectError> {
        S3ObjectStateCurrentFn::try_state_current(fn_ctx, params_partial, data).await
    }

    async fn state_current(
        fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        data: Self::Data<'_>,
    ) -> Result<Self::State, S3ObjectError> {
        S3ObjectStateCurrentFn::state_current(fn_ctx, params, data).await
    }

    async fn try_state_goal(
        fn_ctx: FnCtx<'_>,
        params_partial: &<Self::Params<'_> as Params>::Partial,
        data: Self::Data<'_>,
    ) -> Result<Option<Self::State>, S3ObjectError> {
        S3ObjectStateGoalFn::try_state_goal(fn_ctx, params_partial, data).await
    }

    async fn state_goal(
        fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        data: Self::Data<'_>,
    ) -> Result<Self::State, S3ObjectError> {
        S3ObjectStateGoalFn::state_goal(fn_ctx, params, data).await
    }

    async fn state_diff(
        _params_partial: &<Self::Params<'_> as Params>::Partial,
        _data: Self::Data<'_>,
        state_current: &Self::State,
        state_goal: &Self::State,
    ) -> Result<Self::StateDiff, S3ObjectError> {
        S3ObjectStateDiffFn::state_diff(state_current, state_goal).await
    }

    async fn state_clean(
        _params_partial: &<Self::Params<'_> as Params>::Partial,
        _data: Self::Data<'_>,
    ) -> Result<Self::State, S3ObjectError> {
        Ok(S3ObjectState::None)
    }

    async fn apply_check(
        params: &Self::Params<'_>,
        data: Self::Data<'_>,
        state_current: &Self::State,
        state_target: &Self::State,
        diff: &Self::StateDiff,
    ) -> Result<ApplyCheck, Self::Error> {
        S3ObjectApplyFns::<Id>::apply_check(params, data, state_current, state_target, diff).await
    }

    async fn apply_dry(
        fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        data: Self::Data<'_>,
        state_current: &Self::State,
        state_target: &Self::State,
        diff: &Self::StateDiff,
    ) -> Result<Self::State, Self::Error> {
        S3ObjectApplyFns::<Id>::apply_dry(fn_ctx, params, data, state_current, state_target, diff)
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
        S3ObjectApplyFns::<Id>::apply(fn_ctx, params, data, state_current, state_target, diff).await
    }

    #[cfg(feature = "item_interactions")]
    fn interactions(
        params: &Self::Params<'_>,
        _data: Self::Data<'_>,
    ) -> Vec<peace::item_interaction_model::ItemInteraction> {
        use peace::item_interaction_model::{
            ItemInteractionPush, ItemLocation, ItemLocationAncestors,
        };

        let file_path = format!("📄 {}", params.file_path().display());
        let bucket_name = format!("🪣 {}", params.bucket_name());
        let object_name = format!("📄 {}", params.object_key());

        let item_interaction = ItemInteractionPush::new(
            ItemLocationAncestors::new(vec![
                ItemLocation::localhost(),
                ItemLocation::path(file_path),
            ]),
            ItemLocationAncestors::new(vec![
                ItemLocation::path(bucket_name),
                ItemLocation::path(object_name),
            ]),
        )
        .into();

        vec![item_interaction]
    }
}
