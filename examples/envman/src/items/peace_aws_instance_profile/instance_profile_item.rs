use std::marker::PhantomData;

use aws_config::BehaviorVersion;
use peace::{
    cfg::{async_trait, ApplyCheck, FnCtx, Item},
    item_model::ItemId,
    params::Params,
    resource_rt::{resources::ts::Empty, Resources},
};

use crate::items::peace_aws_instance_profile::{
    InstanceProfileApplyFns, InstanceProfileData, InstanceProfileError, InstanceProfileParams,
    InstanceProfileState, InstanceProfileStateCurrentFn, InstanceProfileStateDiff,
    InstanceProfileStateDiffFn, InstanceProfileStateGoalFn,
};

/// Item to create an IAM instance profile and IAM role.
///
/// In sequence, this will:
///
/// * Create the IAM Role.
/// * Create the instance profile.
/// * Add the IAM role to the instance profile.
///
/// The `Id` type parameter is needed for each instance profile params to be a
/// distinct type.
///
/// # Type Parameters
///
/// * `Id`: A zero-sized type used to distinguish different instance profile
///   parameters from each other.
#[derive(Debug)]
pub struct InstanceProfileItem<Id> {
    /// ID of the instance profile item.
    item_id: ItemId,
    /// Marker for unique instance profile parameters type.
    marker: PhantomData<Id>,
}

impl<Id> InstanceProfileItem<Id> {
    /// Returns a new `InstanceProfileItem`.
    pub fn new(item_id: ItemId) -> Self {
        Self {
            item_id,
            marker: PhantomData,
        }
    }
}

impl<Id> Clone for InstanceProfileItem<Id> {
    fn clone(&self) -> Self {
        Self {
            item_id: self.item_id.clone(),
            marker: PhantomData,
        }
    }
}

#[async_trait(?Send)]
impl<Id> Item for InstanceProfileItem<Id>
where
    Id: Send + Sync + 'static,
{
    type Data<'exec> = InstanceProfileData<'exec, Id>;
    type Error = InstanceProfileError;
    type Params<'exec> = InstanceProfileParams<Id>;
    type State = InstanceProfileState;
    type StateDiff = InstanceProfileStateDiff;

    fn id(&self) -> &ItemId {
        &self.item_id
    }

    async fn setup(&self, resources: &mut Resources<Empty>) -> Result<(), InstanceProfileError> {
        if !resources.contains::<aws_sdk_iam::Client>() {
            let sdk_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
            let client = aws_sdk_iam::Client::new(&sdk_config);
            resources.insert(client);
        }
        Ok(())
    }

    #[cfg(feature = "item_state_example")]
    fn state_example(params: &Self::Params<'_>, _data: Self::Data<'_>) -> Self::State {
        use peace::cfg::state::Generated;

        use crate::items::peace_aws_instance_profile::model::InstanceProfileIdAndArn;

        let name = params.name().to_string();
        let path = params.path().to_string();
        let aws_account_id = "123456789012"; // Can this be looked up without calling AWS?
        let id = String::from("instance_profile_example_id");
        let arn = format!("arn:aws:iam::{aws_account_id}:instance-profile/{name}");

        InstanceProfileState::Some {
            name,
            path,
            instance_profile_id_and_arn: Generated::Value(InstanceProfileIdAndArn::new(id, arn)),
            role_associated: true,
        }
    }

    async fn try_state_current(
        fn_ctx: FnCtx<'_>,
        params_partial: &<Self::Params<'_> as Params>::Partial,
        data: InstanceProfileData<'_, Id>,
    ) -> Result<Option<Self::State>, InstanceProfileError> {
        InstanceProfileStateCurrentFn::try_state_current(fn_ctx, params_partial, data).await
    }

    async fn state_current(
        fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        data: InstanceProfileData<'_, Id>,
    ) -> Result<Self::State, InstanceProfileError> {
        InstanceProfileStateCurrentFn::state_current(fn_ctx, params, data).await
    }

    async fn try_state_goal(
        fn_ctx: FnCtx<'_>,
        params_partial: &<Self::Params<'_> as Params>::Partial,
        data: InstanceProfileData<'_, Id>,
    ) -> Result<Option<Self::State>, InstanceProfileError> {
        InstanceProfileStateGoalFn::try_state_goal(fn_ctx, params_partial, data).await
    }

    async fn state_goal(
        fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        data: InstanceProfileData<'_, Id>,
    ) -> Result<Self::State, InstanceProfileError> {
        InstanceProfileStateGoalFn::state_goal(fn_ctx, params, data).await
    }

    async fn state_diff(
        _params_partial: &<Self::Params<'_> as Params>::Partial,
        _data: Self::Data<'_>,
        state_current: &Self::State,
        state_goal: &Self::State,
    ) -> Result<Self::StateDiff, InstanceProfileError> {
        InstanceProfileStateDiffFn::state_diff(state_current, state_goal).await
    }

    async fn state_clean(
        _params_partial: &<Self::Params<'_> as Params>::Partial,
        _data: Self::Data<'_>,
    ) -> Result<Self::State, InstanceProfileError> {
        Ok(InstanceProfileState::None)
    }

    async fn apply_check(
        params: &Self::Params<'_>,
        data: Self::Data<'_>,
        state_current: &Self::State,
        state_target: &Self::State,
        diff: &Self::StateDiff,
    ) -> Result<ApplyCheck, Self::Error> {
        InstanceProfileApplyFns::<Id>::apply_check(params, data, state_current, state_target, diff)
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
        InstanceProfileApplyFns::<Id>::apply_dry(
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
        InstanceProfileApplyFns::<Id>::apply(
            fn_ctx,
            params,
            data,
            state_current,
            state_target,
            diff,
        )
        .await
    }

    #[cfg(feature = "item_interactions")]
    fn interactions(
        params: &Self::Params<'_>,
        _data: Self::Data<'_>,
    ) -> Vec<peace::item_interaction_model::ItemInteraction> {
        use peace::item_interaction_model::{
            ItemInteractionPush, ItemLocation, ItemLocationAncestors,
        };

        let instance_profile_name = format!("📝 {}", params.name());

        let item_interaction = ItemInteractionPush::new(
            ItemLocationAncestors::new(vec![ItemLocation::localhost()]),
            ItemLocationAncestors::new(vec![
                ItemLocation::group(String::from("IAM")),
                ItemLocation::group(String::from("Instance Profiles")),
                ItemLocation::path(instance_profile_name),
            ]),
        )
        .into();

        vec![item_interaction]
    }
}
