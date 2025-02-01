use std::marker::PhantomData;

use aws_config::BehaviorVersion;
use peace::{
    cfg::{async_trait, ApplyCheck, FnCtx, Item},
    item_model::ItemId,
    params::Params,
    resource_rt::{resources::ts::Empty, Resources},
};

use crate::items::peace_aws_iam_policy::{
    IamPolicyApplyFns, IamPolicyData, IamPolicyError, IamPolicyParams, IamPolicyState,
    IamPolicyStateCurrentFn, IamPolicyStateDiff, IamPolicyStateDiffFn, IamPolicyStateGoalFn,
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
pub struct IamPolicyItem<Id> {
    /// ID of the instance profile item.
    item_id: ItemId,
    /// Marker for unique instance profile parameters type.
    marker: PhantomData<Id>,
}

impl<Id> IamPolicyItem<Id> {
    /// Returns a new `IamPolicyItem`.
    pub fn new(item_id: ItemId) -> Self {
        Self {
            item_id,
            marker: PhantomData,
        }
    }
}

impl<Id> Clone for IamPolicyItem<Id> {
    fn clone(&self) -> Self {
        Self {
            item_id: self.item_id.clone(),
            marker: PhantomData,
        }
    }
}

#[async_trait(?Send)]
impl<Id> Item for IamPolicyItem<Id>
where
    Id: Send + Sync + 'static,
{
    type Data<'exec> = IamPolicyData<'exec, Id>;
    type Error = IamPolicyError;
    type Params<'exec> = IamPolicyParams<Id>;
    type State = IamPolicyState;
    type StateDiff = IamPolicyStateDiff;

    fn id(&self) -> &ItemId {
        &self.item_id
    }

    async fn setup(&self, resources: &mut Resources<Empty>) -> Result<(), IamPolicyError> {
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

        use crate::items::peace_aws_iam_policy::model::PolicyIdArnVersion;

        let name = params.name().to_string();
        let path = params.path().to_string();
        let policy_document = params.policy_document().to_string();
        let policy_id_arn_version = {
            let aws_account_id = "123456789012"; // Can this be looked up without calling AWS?
            let id = String::from("iam_role_example_id");
            let arn = format!("arn:aws:iam::{aws_account_id}:policy/{name}");
            let version = String::from("v1");

            Generated::Value(PolicyIdArnVersion::new(id, arn, version))
        };

        IamPolicyState::Some {
            name,
            path,
            policy_document,
            policy_id_arn_version,
        }
    }

    async fn try_state_current(
        fn_ctx: FnCtx<'_>,
        params_partial: &<Self::Params<'_> as Params>::Partial,
        data: IamPolicyData<'_, Id>,
    ) -> Result<Option<Self::State>, IamPolicyError> {
        IamPolicyStateCurrentFn::try_state_current(fn_ctx, params_partial, data).await
    }

    async fn state_current(
        fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        data: IamPolicyData<'_, Id>,
    ) -> Result<Self::State, IamPolicyError> {
        IamPolicyStateCurrentFn::state_current(fn_ctx, params, data).await
    }

    async fn try_state_goal(
        fn_ctx: FnCtx<'_>,
        params_partial: &<Self::Params<'_> as Params>::Partial,
        data: IamPolicyData<'_, Id>,
    ) -> Result<Option<Self::State>, IamPolicyError> {
        IamPolicyStateGoalFn::try_state_goal(fn_ctx, params_partial, data).await
    }

    async fn state_goal(
        fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        data: IamPolicyData<'_, Id>,
    ) -> Result<Self::State, IamPolicyError> {
        IamPolicyStateGoalFn::state_goal(fn_ctx, params, data).await
    }

    async fn state_diff(
        _params_partial: &<Self::Params<'_> as Params>::Partial,
        _data: Self::Data<'_>,
        state_current: &Self::State,
        state_goal: &Self::State,
    ) -> Result<Self::StateDiff, IamPolicyError> {
        IamPolicyStateDiffFn::state_diff(state_current, state_goal).await
    }

    async fn state_clean(
        _params_partial: &<Self::Params<'_> as Params>::Partial,
        _data: Self::Data<'_>,
    ) -> Result<Self::State, IamPolicyError> {
        Ok(IamPolicyState::None)
    }

    async fn apply_check(
        params: &Self::Params<'_>,
        data: Self::Data<'_>,
        state_current: &Self::State,
        state_target: &Self::State,
        diff: &Self::StateDiff,
    ) -> Result<ApplyCheck, Self::Error> {
        IamPolicyApplyFns::<Id>::apply_check(params, data, state_current, state_target, diff).await
    }

    async fn apply_dry(
        fn_ctx: FnCtx<'_>,
        params: &Self::Params<'_>,
        data: Self::Data<'_>,
        state_current: &Self::State,
        state_target: &Self::State,
        diff: &Self::StateDiff,
    ) -> Result<Self::State, Self::Error> {
        IamPolicyApplyFns::<Id>::apply_dry(fn_ctx, params, data, state_current, state_target, diff)
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
        IamPolicyApplyFns::<Id>::apply(fn_ctx, params, data, state_current, state_target, diff)
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

        let iam_policy_name = format!("📝 {}", params.name());

        let item_interaction = ItemInteractionPush::new(
            ItemLocationAncestors::new(vec![ItemLocation::localhost()]),
            ItemLocationAncestors::new(vec![
                ItemLocation::group(String::from("IAM")),
                ItemLocation::group(String::from("Policies")),
                ItemLocation::path(iam_policy_name),
            ]),
        )
        .into();

        vec![item_interaction]
    }
}
