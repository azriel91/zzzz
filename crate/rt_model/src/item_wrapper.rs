use std::{
    any::Any,
    fmt::{self, Debug},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use peace_cfg::{async_trait, ApplyCheck, FnCtx, Item};
use peace_data::{
    fn_graph::{DataAccess, DataAccessDyn, TypeIds},
    marker::{ApplyDry, Clean, Current, Goal},
    Data,
};
use peace_item_model::ItemId;
use peace_params::{
    Params, ParamsMergeExt, ParamsSpec, ParamsSpecs, ValueResolutionCtx, ValueResolutionMode,
};
use peace_resource_rt::{
    resources::ts::{Empty, SetUp},
    states::StatesCurrent,
    type_reg::untagged::{BoxDtDisplay, TypeMap},
    Resources,
};
use type_reg::untagged::BoxDataTypeDowncast;

use crate::{
    outcomes::{ItemApply, ItemApplyBoxed, ItemApplyPartial, ItemApplyPartialBoxed},
    ItemRt, ParamsSpecsTypeReg, StateDowncastError, StatesTypeReg,
};

#[cfg(feature = "output_progress")]
use peace_cfg::RefInto;
#[cfg(feature = "item_state_example")]
use peace_data::marker::Example;
#[cfg(feature = "output_progress")]
use peace_item_interaction_model::ItemLocationState;

/// Wraps a type implementing [`Item`].
///
/// # Type Parameters
///
/// * `I`: Item type to wrap.
/// * `E`: Application specific error type.
///
///     Notably, `E` here should be the application's error type, which is not
///     necessarily the item's error type (unless you have only one item
///     spec in the application).
#[allow(clippy::type_complexity)]
pub struct ItemWrapper<I, E>(I, PhantomData<E>);

impl<I, E> Clone for ItemWrapper<I, E>
where
    I: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<I, E> PartialEq for ItemWrapper<I, E> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<I, E> Eq for ItemWrapper<I, E> {}

impl<I, E> ItemWrapper<I, E>
where
    I: Debug + Item + Send + Sync,
    E: Debug
        + Send
        + Sync
        + std::error::Error
        + From<<I as Item>::Error>
        + From<crate::Error>
        + 'static,
    for<'params> <I as Item>::Params<'params>:
        TryFrom<<<I as Item>::Params<'params> as Params>::Partial>,
    for<'params> <I::Params<'params> as Params>::Partial: From<I::Params<'params>>,
{
    #[cfg(feature = "item_state_example")]
    fn state_example(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
    ) -> Result<I::State, E> {
        let state_example = {
            let params = self.params(params_specs, resources, ValueResolutionMode::Example)?;
            let data = <I::Data<'_> as Data>::borrow(self.id(), resources);
            I::state_example(&params, data)
        };
        resources.borrow_mut::<Example<I::State>>().0 = Some(state_example.clone());

        Ok(state_example)
    }

    async fn state_clean(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
    ) -> Result<I::State, E> {
        let state_clean = {
            let params_partial =
                self.params_partial(params_specs, resources, ValueResolutionMode::Clean)?;
            let data = <I::Data<'_> as Data>::borrow(self.id(), resources);
            I::state_clean(&params_partial, data).await?
        };
        resources.borrow_mut::<Clean<I::State>>().0 = Some(state_clean.clone());

        Ok(state_clean)
    }

    async fn state_current_try_exec(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        fn_ctx: FnCtx<'_>,
    ) -> Result<Option<I::State>, E> {
        let state_current = {
            let params_partial =
                self.params_partial(params_specs, resources, ValueResolutionMode::Current)?;
            let data = <I::Data<'_> as Data>::borrow(self.id(), resources);
            I::try_state_current(fn_ctx, &params_partial, data).await?
        };
        if let Some(state_current) = state_current.as_ref() {
            resources.borrow_mut::<Current<I::State>>().0 = Some(state_current.clone());

            #[cfg(feature = "output_progress")]
            fn_ctx
                .progress_sender()
                .item_location_state_send(RefInto::<ItemLocationState>::into(state_current));
        }

        Ok(state_current)
    }

    async fn state_current_exec(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        fn_ctx: FnCtx<'_>,
    ) -> Result<I::State, E> {
        let state_current = {
            let params = self.params(params_specs, resources, ValueResolutionMode::Current)?;
            let data = <I::Data<'_> as Data>::borrow(self.id(), resources);
            I::state_current(fn_ctx, &params, data).await?
        };
        resources.borrow_mut::<Current<I::State>>().0 = Some(state_current.clone());

        #[cfg(feature = "output_progress")]
        fn_ctx
            .progress_sender()
            .item_location_state_send(RefInto::<ItemLocationState>::into(&state_current));

        Ok(state_current)
    }

    async fn state_goal_try_exec(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        fn_ctx: FnCtx<'_>,
    ) -> Result<Option<I::State>, E> {
        let params_partial =
            self.params_partial(params_specs, resources, ValueResolutionMode::Goal)?;

        // If a predecessor's goal state is the same as current, then a successor's
        // `state_goal_try_exec` should kind of use `ValueResolutionMode::Current`.
        //
        // But really we should insert the predecessor's current state as the
        // `Goal<Predecessor::State>`.

        let data = <I::Data<'_> as Data>::borrow(self.id(), resources);
        let state_goal = I::try_state_goal(fn_ctx, &params_partial, data).await?;
        if let Some(state_goal) = state_goal.as_ref() {
            resources.borrow_mut::<Goal<I::State>>().0 = Some(state_goal.clone());
        }

        Ok(state_goal)
    }

    /// Returns the goal state for this item.
    ///
    /// `value_resolution_ctx` is passed in because:
    ///
    /// * When discovering the goal state for a flow, without altering any
    ///   items, the goal state of a successor is dependent on the goal state of
    ///   a predecessor.
    /// * When discovering the goal state of a successor, after a predecessor
    ///   has had state applied, the predecessor's goal state does not
    ///   necessarily contain the generated values, so we need to tell the
    ///   successor to look up the predecessor's newly current state.
    /// * When cleaning up, the predecessor's current state must be used to
    ///   resolve a successor's params. Since clean up is applied in reverse,
    ///   the `Goal` state of a predecessor may not exist, and cause the
    ///   successor to not be cleaned up.
    async fn state_goal_exec(
        &self,
        value_resolution_mode: ValueResolutionMode,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        fn_ctx: FnCtx<'_>,
    ) -> Result<I::State, E> {
        let params = self.params(params_specs, resources, value_resolution_mode)?;
        let data = <I::Data<'_> as Data>::borrow(self.id(), resources);
        let state_goal = I::state_goal(fn_ctx, &params, data).await?;
        resources.borrow_mut::<Goal<I::State>>().0 = Some(state_goal.clone());

        Ok(state_goal)
    }

    async fn state_diff_exec(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        states_a: &TypeMap<ItemId, BoxDtDisplay>,
        states_b: &TypeMap<ItemId, BoxDtDisplay>,
    ) -> Result<Option<I::StateDiff>, E> {
        let item_id = <I as Item>::id(self);
        let state_base = states_a.get::<I::State, _>(item_id);
        let state_goal = states_b.get::<I::State, _>(item_id);

        if let Some((state_base, state_goal)) = state_base.zip(state_goal) {
            let state_diff: I::StateDiff = self
                .state_diff_exec_with(params_specs, resources, state_base, state_goal)
                .await?;
            Ok(Some(state_diff))
        } else {
            // When we reach here, one of the following is true:
            //
            // * The current state cannot be retrieved, due to a predecessor's state not
            //   existing.
            // * The goal state cannot be retrieved, due to a predecessor's state not
            //   existing.
            // * A bug exists, e.g. the state is stored against the wrong type parameter.

            Ok(None)
        }
    }

    async fn state_diff_exec_with(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        state_a: &I::State,
        state_b: &I::State,
    ) -> Result<I::StateDiff, E> {
        let state_diff: I::StateDiff = {
            // Running `diff` for a single profile will be between the current and goal
            // states, and parameters are not really intended to be used for diffing.
            //
            // However for `ShCmdItem`, the shell script for diffing's path is in
            // params, which *likely* would be provided as direct `Value`s instead of
            // mapped from predecessors' state(s). Iff the values are mapped from a
            // predecessor's state, then we would want it to be the goal state, as that
            // is closest to the correct value -- `ValueResolutionMode::ApplyDry` is used in
            // `Item::apply_dry`, and `ValueResolutionMode::Apply` is used in
            // `Item::apply`.
            //
            // Running `diff` for multiple profiles will likely be between two profiles'
            // current states.
            let params_partial =
                self.params_partial(params_specs, resources, ValueResolutionMode::Goal)?;
            let data = <I::Data<'_> as Data>::borrow(self.id(), resources);
            I::state_diff(&params_partial, data, state_a, state_b)
                .await
                .map_err(Into::<E>::into)?
        };

        Ok(state_diff)
    }

    async fn apply_check(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        state_current: &I::State,
        state_target: &I::State,
        state_diff: &I::StateDiff,
        value_resolution_mode: ValueResolutionMode,
    ) -> Result<ApplyCheck, E> {
        // Normally an `apply_check` only compares the states / state diff.
        //
        // We use `ValueResolutionMode::Goal` because an apply is between the current
        // and goal states, and when resolving values, we want the target state's
        // parameters to be used. Note that during an apply, the goal state is
        // resolved as execution happens -- values that rely on predecessors' applied
        // state will be fed into successors' goal state.
        let params_partial = self.params_partial(params_specs, resources, value_resolution_mode)?;

        let data = <I::Data<'_> as Data>::borrow(self.id(), resources);
        if let Ok(params) = params_partial.try_into() {
            I::apply_check(&params, data, state_current, state_target, state_diff)
                .await
                .map_err(Into::<E>::into)
        } else {
            // > If we cannot resolve parameters, then this item, and its predecessor are
            // > cleaned up.
            //
            // The above is not necessarily true -- the user may have provided an incorrect
            // type to map from. However, it is more likely to be true than false.
            Ok(ApplyCheck::ExecNotRequired)
        }
    }

    async fn apply_exec_dry(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        fn_ctx: FnCtx<'_>,
        state_current: &I::State,
        state_goal: &I::State,
        state_diff: &I::StateDiff,
    ) -> Result<I::State, E> {
        let params = self.params(params_specs, resources, ValueResolutionMode::ApplyDry)?;
        let data = <I::Data<'_> as Data>::borrow(self.id(), resources);
        let state_ensured_dry =
            I::apply_dry(fn_ctx, &params, data, state_current, state_goal, state_diff)
                .await
                .map_err(Into::<E>::into)?;

        resources.borrow_mut::<ApplyDry<I::State>>().0 = Some(state_ensured_dry.clone());

        Ok(state_ensured_dry)
    }

    async fn apply_exec(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        fn_ctx: FnCtx<'_>,
        state_current: &I::State,
        state_goal: &I::State,
        state_diff: &I::StateDiff,
    ) -> Result<I::State, E> {
        let params = self.params(params_specs, resources, ValueResolutionMode::Current)?;
        let data = <I::Data<'_> as Data>::borrow(self.id(), resources);
        let state_ensured = I::apply(fn_ctx, &params, data, state_current, state_goal, state_diff)
            .await
            .map_err(Into::<E>::into)?;

        resources.borrow_mut::<Current<I::State>>().0 = Some(state_ensured.clone());

        #[cfg(feature = "output_progress")]
        fn_ctx
            .progress_sender()
            .item_location_state_send(RefInto::<ItemLocationState>::into(&state_ensured));

        Ok(state_ensured)
    }

    fn params_partial(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        value_resolution_mode: ValueResolutionMode,
    ) -> Result<<<I as Item>::Params<'_> as Params>::Partial, E> {
        let item_id = self.id();
        let params_spec = params_specs
            .get::<ParamsSpec<I::Params<'_>>, _>(item_id)
            .ok_or_else(|| crate::Error::ParamsSpecNotFound {
                item_id: item_id.clone(),
            })?;
        let mut value_resolution_ctx = ValueResolutionCtx::new(
            value_resolution_mode,
            item_id.clone(),
            tynm::type_name::<I::Params<'_>>(),
        );
        Ok(params_spec
            .resolve_partial(resources, &mut value_resolution_ctx)
            .map_err(crate::Error::ParamsResolveError)?)
    }

    fn params(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        value_resolution_mode: ValueResolutionMode,
    ) -> Result<<I as Item>::Params<'_>, E> {
        let item_id = self.id();
        let params_spec = params_specs
            .get::<ParamsSpec<I::Params<'_>>, _>(item_id)
            .ok_or_else(|| crate::Error::ParamsSpecNotFound {
                item_id: item_id.clone(),
            })?;
        let mut value_resolution_ctx = ValueResolutionCtx::new(
            value_resolution_mode,
            item_id.clone(),
            tynm::type_name::<I::Params<'_>>(),
        );
        Ok(params_spec
            .resolve(resources, &mut value_resolution_ctx)
            .map_err(crate::Error::ParamsResolveError)?)
    }
}

impl<I, E> Debug for ItemWrapper<I, E>
where
    I: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<I, E> Deref for ItemWrapper<I, E> {
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I, E> DerefMut for ItemWrapper<I, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<I, E> From<I> for ItemWrapper<I, E>
where
    I: Debug + Item + Send + Sync,
    E: Debug + Send + Sync + std::error::Error + From<<I as Item>::Error> + 'static,
{
    fn from(item: I) -> Self {
        Self(item, PhantomData)
    }
}

impl<I, E> DataAccess for ItemWrapper<I, E>
where
    I: Debug + Item + Send + Sync,
    E: Debug + Send + Sync + std::error::Error + From<<I as Item>::Error> + 'static,
{
    fn borrows() -> TypeIds {
        let mut type_ids = <I::Data<'_> as DataAccess>::borrows();
        type_ids.push(std::any::TypeId::of::<I::Params<'_>>());

        type_ids
    }

    fn borrow_muts() -> TypeIds {
        <I::Data<'_> as DataAccess>::borrow_muts()
    }
}

impl<I, E> DataAccessDyn for ItemWrapper<I, E>
where
    I: Debug + Item + Send + Sync,
    E: Debug + Send + Sync + std::error::Error + From<<I as Item>::Error> + 'static,
{
    fn borrows(&self) -> TypeIds {
        let mut type_ids = <I::Data<'_> as DataAccess>::borrows();
        type_ids.push(std::any::TypeId::of::<I::Params<'_>>());

        type_ids
    }

    fn borrow_muts(&self) -> TypeIds {
        <I::Data<'_> as DataAccess>::borrow_muts()
    }
}

#[async_trait(?Send)]
impl<I, E> ItemRt<E> for ItemWrapper<I, E>
where
    I: Clone + Debug + Item + Send + Sync + 'static,
    E: Debug
        + Send
        + Sync
        + std::error::Error
        + From<<I as Item>::Error>
        + From<crate::Error>
        + 'static,
    for<'params> I::Params<'params>:
        ParamsMergeExt + TryFrom<<I::Params<'params> as Params>::Partial>,
    for<'params> <I::Params<'params> as Params>::Partial: From<I::Params<'params>>
        + From<<I::Params<'params> as TryFrom<<I::Params<'params> as Params>::Partial>>::Error>,
{
    fn id(&self) -> &ItemId {
        <I as Item>::id(self)
    }

    fn eq(&self, other: &dyn ItemRt<E>) -> bool {
        if self.id() == other.id() {
            let other = other.as_any();
            if let Some(item_wrapper) = other.downcast_ref::<Self>() {
                self == item_wrapper
            } else {
                false
            }
        } else {
            false
        }
    }

    fn as_any(&self) -> &dyn Any
    where
        Self: 'static,
    {
        self
    }

    async fn setup(&self, resources: &mut Resources<Empty>) -> Result<(), E> {
        // Insert `XMarker<I::State>` to create entries in `Resources`.
        // This is used for referential param values (#94)
        #[cfg(feature = "item_state_example")]
        resources.insert(Example::<I::State>(None));
        resources.insert(Clean::<I::State>(None));
        resources.insert(Current::<I::State>(None));
        resources.insert(Goal::<I::State>(None));
        resources.insert(ApplyDry::<I::State>(None));

        // Run user defined setup.
        <I as Item>::setup(self, resources)
            .await
            .map_err(Into::<E>::into)
    }

    fn params_and_state_register(
        &self,
        params_specs_type_reg: &mut ParamsSpecsTypeReg,
        states_type_reg: &mut StatesTypeReg,
    ) {
        params_specs_type_reg.register::<ParamsSpec<I::Params<'_>>>(I::id(self).clone());
        states_type_reg.register::<I::State>(I::id(self).clone());
    }

    fn state_eq(&self, state_a: &BoxDtDisplay, state_b: &BoxDtDisplay) -> Result<bool, E> {
        let state_a_downcasted = BoxDataTypeDowncast::<I::State>::downcast_ref(state_a);
        let state_b_downcasted = BoxDataTypeDowncast::<I::State>::downcast_ref(state_b);

        match (state_a_downcasted, state_b_downcasted) {
            (None, None) => Err(crate::Error::StateDowncastError(StateDowncastError::Both {
                ty_name: tynm::type_name::<I::State>(),
                state_a: state_a.clone(),
                state_b: state_b.clone(),
            })
            .into()),
            (None, Some(_)) => Err(crate::Error::StateDowncastError(StateDowncastError::First {
                ty_name: tynm::type_name::<I::State>(),
                state_a: state_a.clone(),
            })
            .into()),
            (Some(_), None) => Err(
                crate::Error::StateDowncastError(StateDowncastError::Second {
                    ty_name: tynm::type_name::<I::State>(),
                    state_b: state_b.clone(),
                })
                .into(),
            ),
            (Some(state_a), Some(state_b)) => Ok(state_a == state_b),
        }
    }

    #[cfg(feature = "item_state_example")]
    fn state_example(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
    ) -> Result<BoxDtDisplay, E> {
        self.state_example(params_specs, resources)
            .map(BoxDtDisplay::new)
    }

    async fn state_clean(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
    ) -> Result<BoxDtDisplay, E> {
        self.state_clean(params_specs, resources)
            .await
            .map(BoxDtDisplay::new)
            .map_err(Into::<E>::into)
    }

    async fn state_current_try_exec(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        fn_ctx: FnCtx<'_>,
    ) -> Result<Option<BoxDtDisplay>, E> {
        self.state_current_try_exec(params_specs, resources, fn_ctx)
            .await
            .map(|state_current| state_current.map(BoxDtDisplay::new))
            .map_err(Into::<E>::into)
    }

    async fn state_current_exec(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        fn_ctx: FnCtx<'_>,
    ) -> Result<BoxDtDisplay, E> {
        self.state_current_exec(params_specs, resources, fn_ctx)
            .await
            .map(BoxDtDisplay::new)
            .map_err(Into::<E>::into)
    }

    async fn state_goal_try_exec(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        fn_ctx: FnCtx<'_>,
    ) -> Result<Option<BoxDtDisplay>, E> {
        self.state_goal_try_exec(params_specs, resources, fn_ctx)
            .await
            .map(|state_goal| state_goal.map(BoxDtDisplay::new))
            .map_err(Into::<E>::into)
    }

    async fn state_goal_exec(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        fn_ctx: FnCtx<'_>,
    ) -> Result<BoxDtDisplay, E> {
        self.state_goal_exec(
            // Use the would-be state of predecessor to discover goal state of this item.
            //
            // TODO: this means we may need to overlay the goal state with the predecessor's
            // current state.
            //
            // Or more feasibly, if predecessor's ApplyCheck is ExecNotRequired, then we can use
            // predecessor's current state.
            ValueResolutionMode::Goal,
            params_specs,
            resources,
            fn_ctx,
        )
        .await
        .map(BoxDtDisplay::new)
        .map_err(Into::<E>::into)
    }

    async fn state_diff_exec(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        states_a: &TypeMap<ItemId, BoxDtDisplay>,
        states_b: &TypeMap<ItemId, BoxDtDisplay>,
    ) -> Result<Option<BoxDtDisplay>, E> {
        self.state_diff_exec(params_specs, resources, states_a, states_b)
            .await
            .map(|state_diff_opt| state_diff_opt.map(BoxDtDisplay::new))
            .map_err(Into::<E>::into)
    }

    async fn ensure_prepare(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        fn_ctx: FnCtx<'_>,
    ) -> Result<ItemApplyBoxed, (E, ItemApplyPartialBoxed)> {
        let mut item_apply_partial = ItemApplyPartial::<I::State, I::StateDiff>::new();

        match self
            .state_current_exec(params_specs, resources, fn_ctx)
            .await
        {
            Ok(state_current) => item_apply_partial.state_current = Some(state_current),
            Err(error) => return Err((error, item_apply_partial.into())),
        }
        #[cfg(feature = "output_progress")]
        fn_ctx.progress_sender().reset_to_pending();
        match self
            .state_goal_exec(
                // Use current state of predecessor to discover goal state.
                ValueResolutionMode::Current,
                params_specs,
                resources,
                fn_ctx,
            )
            .await
        {
            Ok(state_goal) => item_apply_partial.state_target = Some(state_goal),
            Err(error) => return Err((error, item_apply_partial.into())),
        }
        #[cfg(feature = "output_progress")]
        fn_ctx.progress_sender().reset_to_pending();
        match self
            .state_diff_exec_with(
                params_specs,
                resources,
                item_apply_partial
                    .state_current
                    .as_ref()
                    .expect("unreachable: This is set just above."),
                item_apply_partial
                    .state_target
                    .as_ref()
                    .expect("unreachable: This is set just above."),
            )
            .await
        {
            Ok(state_diff) => item_apply_partial.state_diff = Some(state_diff),
            Err(error) => return Err((error, item_apply_partial.into())),
        }

        let (Some(state_current), Some(state_goal), Some(state_diff)) = (
            item_apply_partial.state_current.as_ref(),
            item_apply_partial.state_target.as_ref(),
            item_apply_partial.state_diff.as_ref(),
        ) else {
            unreachable!("These are set just above.");
        };

        let apply_check = self
            .apply_check(
                params_specs,
                resources,
                state_current,
                state_goal,
                state_diff,
                // Use current state of predecessor to discover goal state.
                ValueResolutionMode::Current,
            )
            .await;
        let state_applied = match apply_check {
            Ok(apply_check) => {
                item_apply_partial.apply_check = Some(apply_check);

                // TODO: write test for this case
                match apply_check {
                    #[cfg(not(feature = "output_progress"))]
                    ApplyCheck::ExecRequired => None,
                    #[cfg(feature = "output_progress")]
                    ApplyCheck::ExecRequired { .. } => None,
                    ApplyCheck::ExecNotRequired => item_apply_partial.state_current.clone(),
                }
            }
            Err(error) => return Err((error, item_apply_partial.into())),
        };

        Ok(ItemApply::try_from((item_apply_partial, state_applied))
            .expect("unreachable: All the fields are set above.")
            .into())
    }

    async fn apply_exec_dry(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        fn_ctx: FnCtx<'_>,
        item_apply_boxed: &mut ItemApplyBoxed,
    ) -> Result<(), E> {
        let Some(item_apply) = item_apply_boxed
            .as_data_type_mut()
            .downcast_mut::<ItemApply<I::State, I::StateDiff>>()
        else {
            panic!(
                "Failed to downcast `ItemApplyBoxed` to `{concrete_type}`.\n\
                    This is a bug in the Peace framework.",
                concrete_type = std::any::type_name::<ItemApply<I::State, I::StateDiff>>()
            )
        };

        let ItemApply {
            state_current_stored: _,
            state_current,
            state_target,
            state_diff,
            apply_check,
            state_applied,
        } = item_apply;

        match apply_check {
            #[cfg(not(feature = "output_progress"))]
            ApplyCheck::ExecRequired => {
                let state_applied_dry = self
                    .apply_exec_dry(
                        params_specs,
                        resources,
                        fn_ctx,
                        state_current,
                        state_target,
                        state_diff,
                    )
                    .await?;

                *state_applied = Some(state_applied_dry);
            }
            #[cfg(feature = "output_progress")]
            ApplyCheck::ExecRequired { progress_limit: _ } => {
                let state_applied_dry = self
                    .apply_exec_dry(
                        params_specs,
                        resources,
                        fn_ctx,
                        state_current,
                        state_target,
                        state_diff,
                    )
                    .await?;

                *state_applied = Some(state_applied_dry);
            }
            ApplyCheck::ExecNotRequired => {}
        }

        Ok(())
    }

    async fn clean_prepare(
        &self,
        states_current: &StatesCurrent,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
    ) -> Result<ItemApplyBoxed, (E, ItemApplyPartialBoxed)> {
        let mut item_apply_partial = ItemApplyPartial::<I::State, I::StateDiff>::new();

        if let Some(state_current) = states_current.get::<I::State, _>(self.id()) {
            item_apply_partial.state_current = Some(state_current.clone());
        } else {
            // Hack: Setting ItemApplyPartial state_current to state_clean is a hack,
            // which allows successor items to read the state of a predecessor, when
            // none can be discovered.
            //
            // This may not necessarily be a hack.
            match self.state_clean(params_specs, resources).await {
                Ok(state_clean) => item_apply_partial.state_current = Some(state_clean),
                Err(error) => return Err((error, item_apply_partial.into())),
            }
        }
        match self.state_clean(params_specs, resources).await {
            Ok(state_clean) => item_apply_partial.state_target = Some(state_clean),
            Err(error) => return Err((error, item_apply_partial.into())),
        }

        match self
            .state_diff_exec_with(
                params_specs,
                resources,
                item_apply_partial
                    .state_current
                    .as_ref()
                    .expect("unreachable: This is confirmed just above."),
                item_apply_partial
                    .state_target
                    .as_ref()
                    .expect("unreachable: This is set just above."),
            )
            .await
        {
            Ok(state_diff) => item_apply_partial.state_diff = Some(state_diff),
            Err(error) => return Err((error, item_apply_partial.into())),
        }

        let (Some(state_current), Some(state_clean), Some(state_diff)) = (
            item_apply_partial.state_current.as_ref(),
            item_apply_partial.state_target.as_ref(),
            item_apply_partial.state_diff.as_ref(),
        ) else {
            unreachable!("These are set just above.");
        };

        let apply_check = self
            .apply_check(
                params_specs,
                resources,
                state_current,
                state_clean,
                state_diff,
                // Use current state of predecessor to discover goal state.
                ValueResolutionMode::Current,
            )
            .await;

        let state_applied = match apply_check {
            Ok(apply_check) => {
                item_apply_partial.apply_check = Some(apply_check);

                // TODO: write test for this case
                match apply_check {
                    #[cfg(not(feature = "output_progress"))]
                    ApplyCheck::ExecRequired => None,
                    #[cfg(feature = "output_progress")]
                    ApplyCheck::ExecRequired { .. } => None,
                    ApplyCheck::ExecNotRequired => item_apply_partial.state_current.clone(),
                }
            }
            Err(error) => return Err((error, item_apply_partial.into())),
        };

        Ok(ItemApply::try_from((item_apply_partial, state_applied))
            .expect("unreachable: All the fields are set above.")
            .into())
    }

    async fn apply_exec(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        fn_ctx: FnCtx<'_>,
        item_apply_boxed: &mut ItemApplyBoxed,
    ) -> Result<(), E> {
        let Some(item_apply) = item_apply_boxed
            .as_data_type_mut()
            .downcast_mut::<ItemApply<I::State, I::StateDiff>>()
        else {
            panic!(
                "Failed to downcast `ItemApplyBoxed` to `{concrete_type}`.\n\
                    This is a bug in the Peace framework.",
                concrete_type = std::any::type_name::<ItemApply<I::State, I::StateDiff>>()
            )
        };

        let ItemApply {
            state_current_stored: _,
            state_current,
            state_target,
            state_diff,
            apply_check,
            state_applied,
        } = item_apply;

        match apply_check {
            #[cfg(not(feature = "output_progress"))]
            ApplyCheck::ExecRequired => {
                let state_applied_next = self
                    .apply_exec(
                        params_specs,
                        resources,
                        fn_ctx,
                        state_current,
                        state_target,
                        state_diff,
                    )
                    .await?;

                *state_applied = Some(state_applied_next);
            }
            #[cfg(feature = "output_progress")]
            ApplyCheck::ExecRequired { progress_limit: _ } => {
                let state_applied_next = self
                    .apply_exec(
                        params_specs,
                        resources,
                        fn_ctx,
                        state_current,
                        state_target,
                        state_diff,
                    )
                    .await?;

                *state_applied = Some(state_applied_next);
            }
            ApplyCheck::ExecNotRequired => {}
        }

        Ok(())
    }

    #[cfg(all(feature = "item_interactions", feature = "item_state_example"))]
    fn interactions_example(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
    ) -> Result<peace_item_interaction_model::ItemInteractionsExample, E> {
        let params = self.params(params_specs, resources, ValueResolutionMode::Example)?;

        let data = <I::Data<'_> as Data>::borrow(self.id(), resources);

        let item_interactions = I::interactions(&params, data);

        Ok(peace_item_interaction_model::ItemInteractionsExample::from(
            item_interactions,
        ))
    }

    #[cfg(all(feature = "item_interactions", feature = "item_state_example"))]
    fn interactions_try_current<'params>(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
    ) -> Result<peace_item_interaction_model::ItemInteractionsCurrentOrExample, E> {
        let params_partial_current =
            self.params_partial(params_specs, resources, ValueResolutionMode::Current)?;
        let mut params_example =
            self.params(params_specs, resources, ValueResolutionMode::Example)?;
        let params_current_result: Result<I::Params<'_>, _> =
            TryFrom::<_>::try_from(params_partial_current);

        let data = <I::Data<'_> as Data>::borrow(self.id(), resources);
        let item_interactions = match params_current_result {
            Ok(params_current) => {
                let item_interactions = I::interactions(&params_current, data);

                peace_item_interaction_model::ItemInteractionsCurrent::from(item_interactions)
                    .into()
            }
            Err(params_partial_current) => {
                // Rust cannot guarantee that `I::Params.try_from(params_partial)`'s
                // `TryFrom::Error` type is exactly the same as `Params::Partial`, so we have to
                // explicitly add the `ParamsPartial: From<TryFrom::Error>` bound, and call
                // `.into()` over here.
                ParamsMergeExt::merge(&mut params_example, params_partial_current.into());
                let params_merged = params_example;
                let item_interactions = I::interactions(&params_merged, data);

                peace_item_interaction_model::ItemInteractionsExample::from(item_interactions)
                    .into()
            }
        };

        Ok(item_interactions)
    }

    #[cfg(all(feature = "item_interactions", feature = "item_state_example"))]
    fn interactions_tag_name(&self) -> String {
        use std::borrow::Cow;

        let type_name = tynm::type_name::<I>();
        let (operation, prefix) = type_name
            .split_once("<")
            .map(|(operation, prefix_plus_extra)| {
                let prefix_end = prefix_plus_extra.find(['<', '>']);
                let prefix = prefix_end
                    .map(|prefix_end| &prefix_plus_extra[..prefix_end])
                    .unwrap_or(prefix_plus_extra);
                (Cow::Borrowed(operation), Some(Cow::Borrowed(prefix)))
            })
            .unwrap_or_else(|| (Cow::Borrowed(&type_name), None));

        // Subtract `Item` suffix
        let operation = match operation.rsplit_once("Item") {
            Some((operation_minus_item, _)) => Cow::Borrowed(operation_minus_item),
            None => operation,
        };

        match prefix {
            Some(prefix) => {
                let prefix = heck::AsTitleCase(prefix).to_string();
                let operation = heck::AsTitleCase(operation).to_string();
                format!("{prefix}: {operation}")
            }
            None => heck::AsTitleCase(operation).to_string(),
        }
    }
}
