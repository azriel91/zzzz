use diff::{VecDiff, VecDiffType};
use peace::{
    cfg::{ApplyCheck, FnCtx},
    data::marker::{ApplyDry, Clean, Current, Goal},
    item_model::item_id,
    params::{ParamsSpec, ParamsSpecs},
    resource_rt::{
        internal::StatesMut,
        resources::ts::SetUp,
        states::{self, StatesCurrent, StatesCurrentStored, StatesGoal},
        type_reg::untagged::{BoxDataTypeDowncast, BoxDtDisplay},
        Resources,
    },
    rt_model::{Error as PeaceRtError, ItemRt, ItemWrapper, StateDowncastError},
};
use peace_items::blank::BlankItem;
cfg_if::cfg_if! {
    if #[cfg(feature = "output_progress")] {
        use peace::progress_model::{ProgressLimit, ProgressSender};
        use tokio::sync::mpsc;
    }
}

use crate::{
    PeaceTestError, VecA, VecB, VecCopyDiff, VecCopyError, VecCopyItem, VecCopyItemWrapper,
    VecCopyState,
};

#[test]
fn eq_returns_true_for_same_item_type() {
    let item_wrapper = ItemWrapper::<_, PeaceTestError>::from(VecCopyItem::default());
    let item_rt_0: &dyn ItemRt<_> = &item_wrapper;

    let item_wrapper = ItemWrapper::<_, PeaceTestError>::from(VecCopyItem::default());
    let item_rt_1: &dyn ItemRt<_> = &item_wrapper;

    assert!(item_rt_0.eq(item_rt_1));

    let item_wrapper =
        ItemWrapper::<_, PeaceTestError>::from(BlankItem::<()>::new(item_id!("blank")));
    let item_rt_0: &dyn ItemRt<_> = &item_wrapper;

    let item_wrapper =
        ItemWrapper::<_, PeaceTestError>::from(BlankItem::<()>::new(item_id!("blank")));
    let item_rt_1: &dyn ItemRt<_> = &item_wrapper;

    assert!(item_rt_0.eq(item_rt_1));
}

#[test]
fn eq_returns_false_for_different_item_type() {
    let item_wrapper = ItemWrapper::<_, PeaceTestError>::from(VecCopyItem::new(item_id!("blank")));
    let item_rt_0: &dyn ItemRt<_> = &item_wrapper;

    let item_wrapper =
        ItemWrapper::<_, PeaceTestError>::from(BlankItem::<()>::new(item_id!("blank")));
    let item_rt_1: &dyn ItemRt<_> = &item_wrapper;

    assert!(!item_rt_0.eq(item_rt_1));
    assert!(!item_rt_1.eq(item_rt_0));
}

#[test]
fn eq_returns_false_for_different_item_id() {
    let item_wrapper =
        ItemWrapper::<_, PeaceTestError>::from(BlankItem::<()>::new(item_id!("blank_0")));
    let item_rt_0: &dyn ItemRt<_> = &item_wrapper;

    let item_wrapper =
        ItemWrapper::<_, PeaceTestError>::from(BlankItem::<()>::new(item_id!("blank_1")));
    let item_rt_1: &dyn ItemRt<_> = &item_wrapper;

    assert!(!item_rt_0.eq(item_rt_1));
}

#[test]
fn state_eq_returns_true_for_same_value() {
    let item_wrapper = ItemWrapper::<_, PeaceTestError>::from(VecCopyItem::default());
    let state_eq = item_wrapper.state_eq(
        &BoxDtDisplay::new(VecCopyState::from(vec![0u8])),
        &BoxDtDisplay::new(VecCopyState::from(vec![0u8])),
    );

    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(&state_eq, Ok(true)),
                "expected `state_eq` to be `Ok(true)`, but was `{state_eq:?}`."
            );
        }
    })();
}

#[test]
fn state_eq_returns_false_for_different_value() {
    let item_wrapper = ItemWrapper::<_, PeaceTestError>::from(VecCopyItem::default());
    let state_eq = item_wrapper.state_eq(
        &BoxDtDisplay::new(VecCopyState::from(vec![0u8])),
        &BoxDtDisplay::new(VecCopyState::from(vec![1u8])),
    );

    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(&state_eq, Ok(false)),
                "expected `state_eq` to be `Ok(false)`, but was `{state_eq:?}`."
            );
        }
    })();
}

#[test]
fn state_eq_returns_err_first_when_first_fails_downcast() {
    let item_wrapper = ItemWrapper::<_, PeaceTestError>::from(VecCopyItem::default());
    let state_eq = item_wrapper.state_eq(
        &BoxDtDisplay::new(String::from("string_a")),
        &BoxDtDisplay::new(VecCopyState::from(vec![1u8])),
    );

    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    &state_eq,
                    Err(PeaceTestError::PeaceRt(PeaceRtError::StateDowncastError(
                        StateDowncastError::First { ty_name, state_a }
                    ))) if
                    ty_name == "VecCopyState" &&
                    format!("{state_a}") == "string_a"
                ),
                "expected `state_eq` to be `Err( .. {{ StateDowncastError::First {{ .. }} }})`,\n\
        but was `{state_eq:?}`."
            );
        }
    })();
}

#[test]
fn state_eq_returns_err_second_when_second_fails_downcast() {
    let item_wrapper = ItemWrapper::<_, PeaceTestError>::from(VecCopyItem::default());
    let state_eq = item_wrapper.state_eq(
        &BoxDtDisplay::new(VecCopyState::from(vec![0u8])),
        &BoxDtDisplay::new(String::from("string_b")),
    );

    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    &state_eq,
                    Err(PeaceTestError::PeaceRt(PeaceRtError::StateDowncastError(
                        StateDowncastError::Second { ty_name, state_b }
                    ))) if
                    ty_name == "VecCopyState" &&
                    format!("{state_b}") == "string_b"
                ),
                "expected `state_eq` to be `Err( .. {{ StateDowncastError::Second {{ .. }} }})`,\n\
                but was `{state_eq:?}`."
            );
        }
    })();
}

#[test]
fn state_eq_returns_err_both_when_both_fail_downcast() {
    let item_wrapper = ItemWrapper::<_, PeaceTestError>::from(VecCopyItem::default());
    let state_eq = item_wrapper.state_eq(
        &BoxDtDisplay::new(String::from("string_a")),
        &BoxDtDisplay::new(String::from("string_b")),
    );

    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    &state_eq,
                    Err(PeaceTestError::PeaceRt(PeaceRtError::StateDowncastError(
                        StateDowncastError::Both { ty_name, state_a, state_b }
                    ))) if
                    ty_name == "VecCopyState" &&
                    format!("{state_a}") == "string_a" &&
                    format!("{state_b}") == "string_b"
                ),
                "expected `state_eq` to be `Err( .. {{ StateDowncastError::Both {{ .. }} }})`,\n\
                but was `{state_eq:?}`."
            );
        }
    })();
}

#[tokio::test]
async fn deref_to_dyn_item_rt() {
    let vec_copy_item = VecCopyItem::default();
    let item_wrapper = ItemWrapper::<_, VecCopyError>::from(vec_copy_item.clone());
    let item_rt: &dyn ItemRt<_> = &item_wrapper;

    assert_eq!(format!("{vec_copy_item:?}"), format!("{item_rt:?}"));
}

#[tokio::test]
async fn deref_mut_to_dyn_item_rt() {
    let vec_copy_item = VecCopyItem::default();
    let item_wrapper = ItemWrapper::<_, VecCopyError>::from(vec_copy_item.clone());
    let item_rt: &dyn ItemRt<_> = &item_wrapper;

    assert_eq!(format!("{vec_copy_item:?}"), format!("{item_rt:?}"));
}

#[tokio::test]
async fn setup() -> Result<(), Box<dyn std::error::Error>> {
    let vec_copy_item = VecCopyItem::default();
    let item_wrapper = ItemWrapper::<_, VecCopyError>::from(vec_copy_item);
    let mut resources = Resources::new();
    <dyn ItemRt<_>>::setup(&item_wrapper, &mut resources).await?;
    resources.insert(VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]));

    assert!(resources.try_borrow::<VecA>().is_ok());
    // Automatic `Current<State>` and `Goal<State>` insertion.
    assert!(resources.try_borrow::<Current<VecCopyState>>().is_ok());
    assert!(resources.borrow::<Current<VecCopyState>>().is_none());
    assert!(resources.try_borrow::<Goal<VecCopyState>>().is_ok());
    assert!(resources.borrow::<Goal<VecCopyState>>().is_none());

    Ok(())
}

#[tokio::test]
async fn state_current_try_exec() -> Result<(), Box<dyn std::error::Error>> {
    let vec_copy_item = VecCopyItem::default();
    let item_wrapper = ItemWrapper::<_, VecCopyError>::from(vec_copy_item);
    let (params_specs, resources) = resources_set_up(&item_wrapper).await?;
    cfg_if::cfg_if! {
        if #[cfg(feature = "output_progress")] {
            let (progress_tx, _progress_rx) = mpsc::channel(10);
            let progress_sender = ProgressSender::new(
                VecCopyItem::ID_DEFAULT,
                &progress_tx,
            );
        }
    }
    let fn_ctx = FnCtx::new(
        VecCopyItem::ID_DEFAULT,
        #[cfg(feature = "output_progress")]
        progress_sender,
    );

    let state = item_wrapper
        .state_current_try_exec(&params_specs, &resources, fn_ctx)
        .await?
        .unwrap();

    assert_eq!(
        Some(VecCopyState::new()).as_ref(),
        BoxDataTypeDowncast::<VecCopyState>::downcast_ref(&state)
    );
    // Automatic `Current<State>` insertion.
    assert_eq!(
        Some(VecCopyState::new()).as_ref(),
        resources.borrow::<Current<VecCopyState>>().as_ref()
    );

    Ok(())
}

#[tokio::test]
async fn state_goal_try_exec() -> Result<(), VecCopyError> {
    let vec_copy_item = VecCopyItem::default();
    let item_wrapper = ItemWrapper::<_, VecCopyError>::from(vec_copy_item);
    let (params_specs, resources) = resources_set_up(&item_wrapper).await?;
    cfg_if::cfg_if! {
        if #[cfg(feature = "output_progress")] {
            let (progress_tx, _progress_rx) = mpsc::channel(10);
            let progress_sender = ProgressSender::new(
                VecCopyItem::ID_DEFAULT,
                &progress_tx,
            );
        }
    }
    let fn_ctx = FnCtx::new(
        VecCopyItem::ID_DEFAULT,
        #[cfg(feature = "output_progress")]
        progress_sender,
    );

    let state_goal = item_wrapper
        .state_goal_try_exec(&params_specs, &resources, fn_ctx)
        .await?
        .unwrap();

    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        BoxDataTypeDowncast::<VecCopyState>::downcast_ref(&state_goal)
    );
    // Automatic `Goal<State>` insertion.
    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        resources.borrow::<Goal<VecCopyState>>().as_ref()
    );

    Ok(())
}

#[tokio::test]
async fn state_diff_exec() -> Result<(), VecCopyError> {
    let vec_copy_item = VecCopyItem::default();
    let item_wrapper = ItemWrapper::<_, VecCopyError>::from(vec_copy_item);

    let (params_specs, resources, states_current_stored, states_goal) =
        resources_and_states_current_stored_and_goal(&item_wrapper).await?;

    let state_diff = item_wrapper
        .state_diff_exec(
            &params_specs,
            &resources,
            &states_current_stored,
            &states_goal,
        )
        .await?
        .expect(
            "Expected state_diff to be Some when state_current_stored and state_goal both exist.",
        );

    assert_eq!(
        Some(VecCopyDiff::from(VecDiff(vec![VecDiffType::Inserted {
            index: 0,
            changes: vec![0u8, 1, 2, 3, 4, 5, 6, 7]
        }])))
        .as_ref(),
        BoxDataTypeDowncast::<VecCopyDiff>::downcast_ref(&state_diff)
    );

    Ok(())
}

#[tokio::test]
async fn ensure_prepare() -> Result<(), VecCopyError> {
    let vec_copy_item = VecCopyItem::default();
    let item_wrapper = ItemWrapper::<_, VecCopyError>::from(vec_copy_item);
    let (params_specs, resources) = resources_set_up(&item_wrapper).await?;
    cfg_if::cfg_if! {
        if #[cfg(feature = "output_progress")] {
            let (progress_tx, _progress_rx) = mpsc::channel(10);
            let progress_sender = ProgressSender::new(
                VecCopyItem::ID_DEFAULT,
                &progress_tx,
            );
        }
    }
    let fn_ctx = FnCtx::new(
        VecCopyItem::ID_DEFAULT,
        #[cfg(feature = "output_progress")]
        progress_sender,
    );

    match <dyn ItemRt<_>>::ensure_prepare(&item_wrapper, &params_specs, &resources, fn_ctx).await {
        Ok(item_apply) => {
            #[cfg(not(feature = "output_progress"))]
            assert_eq!(ApplyCheck::ExecRequired, item_apply.apply_check());
            #[cfg(feature = "output_progress")]
            assert_eq!(
                ApplyCheck::ExecRequired {
                    progress_limit: ProgressLimit::Bytes(8)
                },
                item_apply.apply_check()
            );

            Ok(())
        }
        Err((error, _item_apply_partial)) => Err(error),
    }
}

#[tokio::test]
async fn apply_exec_dry_for_ensure() -> Result<(), VecCopyError> {
    let vec_copy_item = VecCopyItem::default();
    let item_wrapper = ItemWrapper::<_, VecCopyError>::from(vec_copy_item);
    let (params_specs, resources) = resources_set_up(&item_wrapper).await?;
    cfg_if::cfg_if! {
        if #[cfg(feature = "output_progress")] {
            let (progress_tx, _progress_rx) = mpsc::channel(10);
            let progress_sender = ProgressSender::new(
                VecCopyItem::ID_DEFAULT,
                &progress_tx,
            );
        }
    }
    let fn_ctx = FnCtx::new(
        VecCopyItem::ID_DEFAULT,
        #[cfg(feature = "output_progress")]
        progress_sender,
    );

    let mut item_apply_boxed =
        <dyn ItemRt<_>>::ensure_prepare(&item_wrapper, &params_specs, &resources, fn_ctx)
            .await
            .map_err(|(error, _)| error)?;
    cfg_if::cfg_if! {
        if #[cfg(feature = "output_progress")] {
            let (progress_tx, _progress_rx) = mpsc::channel(10);
            let progress_sender = ProgressSender::new(
                VecCopyItem::ID_DEFAULT,
                &progress_tx,
            );
        }
    }
    let fn_ctx = FnCtx::new(
        VecCopyItem::ID_DEFAULT,
        #[cfg(feature = "output_progress")]
        progress_sender,
    );

    <dyn ItemRt<_>>::apply_exec_dry(
        &item_wrapper,
        &params_specs,
        &resources,
        fn_ctx,
        &mut item_apply_boxed,
    )
    .await?;

    let vec_b = resources.borrow::<VecB>();
    assert_eq!(&[0u8; 0], &*vec_b.0);

    // Automatic `Current<State>` insertion.
    assert_eq!(
        Some(VecCopyState::new()).as_ref(),
        resources.borrow::<Current<VecCopyState>>().as_ref()
    );
    // Automatic `ApplyDry<State>` insertion.
    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        resources.borrow::<ApplyDry<VecCopyState>>().as_ref()
    );
    // Goal should also exist.
    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        resources.borrow::<Goal<VecCopyState>>().as_ref()
    );

    Ok(())
}

#[tokio::test]
async fn apply_exec_for_ensure() -> Result<(), VecCopyError> {
    let vec_copy_item = VecCopyItem::default();
    let item_wrapper = ItemWrapper::<_, VecCopyError>::from(vec_copy_item);
    let (params_specs, resources) = resources_set_up(&item_wrapper).await?;
    cfg_if::cfg_if! {
        if #[cfg(feature = "output_progress")] {
            let (progress_tx, _progress_rx) = mpsc::channel(10);
            let progress_sender = ProgressSender::new(
                VecCopyItem::ID_DEFAULT,
                &progress_tx,
            );
        }
    }
    let fn_ctx = FnCtx::new(
        VecCopyItem::ID_DEFAULT,
        #[cfg(feature = "output_progress")]
        progress_sender,
    );

    let mut item_apply_boxed =
        <dyn ItemRt<_>>::ensure_prepare(&item_wrapper, &params_specs, &resources, fn_ctx)
            .await
            .map_err(|(error, _)| error)?;
    cfg_if::cfg_if! {
        if #[cfg(feature = "output_progress")] {
            let (progress_tx, _progress_rx) = mpsc::channel(10);
            let progress_sender = ProgressSender::new(
                VecCopyItem::ID_DEFAULT,
                &progress_tx,
            );
        }
    }
    let fn_ctx = FnCtx::new(
        VecCopyItem::ID_DEFAULT,
        #[cfg(feature = "output_progress")]
        progress_sender,
    );

    <dyn ItemRt<_>>::apply_exec(
        &item_wrapper,
        &params_specs,
        &resources,
        fn_ctx,
        &mut item_apply_boxed,
    )
    .await?;

    let vec_b = resources.borrow::<VecB>();
    assert_eq!(&[0u8, 1, 2, 3, 4, 5, 6, 7], &*vec_b.0);

    // Automatic `Current<State>` insertion.
    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        resources.borrow::<Current<VecCopyState>>().as_ref()
    );
    // Goal should also exist.
    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        resources.borrow::<Goal<VecCopyState>>().as_ref()
    );

    Ok(())
}

#[tokio::test]
async fn clean_prepare() -> Result<(), VecCopyError> {
    let vec_copy_item = VecCopyItem::default();
    let item_wrapper = ItemWrapper::<_, VecCopyError>::from(vec_copy_item);
    let (params_specs, resources, states_current) =
        resources_set_up_with_pre_stored_state(&item_wrapper).await?;

    match <dyn ItemRt<_>>::clean_prepare(&item_wrapper, &states_current, &params_specs, &resources)
        .await
    {
        Ok(item_apply) => {
            #[cfg(not(feature = "output_progress"))]
            assert_eq!(ApplyCheck::ExecRequired, item_apply.apply_check());
            #[cfg(feature = "output_progress")]
            assert_eq!(
                ApplyCheck::ExecRequired {
                    progress_limit: ProgressLimit::Bytes(8)
                },
                item_apply.apply_check()
            );

            Ok(())
        }
        Err((error, _item_apply_partial)) => Err(error),
    }
}

#[tokio::test]
async fn apply_exec_dry_for_clean() -> Result<(), VecCopyError> {
    let vec_copy_item = VecCopyItem::default();
    let item_wrapper = ItemWrapper::<_, VecCopyError>::from(vec_copy_item);
    let (params_specs, resources, states_current) =
        resources_set_up_with_pre_stored_state(&item_wrapper).await?;

    let mut item_apply_boxed =
        <dyn ItemRt<_>>::clean_prepare(&item_wrapper, &states_current, &params_specs, &resources)
            .await
            .map_err(|(error, _)| error)?;
    cfg_if::cfg_if! {
        if #[cfg(feature = "output_progress")] {
            let (progress_tx, _progress_rx) = mpsc::channel(10);
            let progress_sender = ProgressSender::new(
                VecCopyItem::ID_DEFAULT,
                &progress_tx,
            );
        }
    }
    let fn_ctx = FnCtx::new(
        VecCopyItem::ID_DEFAULT,
        #[cfg(feature = "output_progress")]
        progress_sender,
    );

    <dyn ItemRt<_>>::apply_exec_dry(
        &item_wrapper,
        &params_specs,
        &resources,
        fn_ctx,
        &mut item_apply_boxed,
    )
    .await?;

    let vec_b = resources.borrow::<VecB>();
    assert_eq!(&[0u8, 1, 2, 3, 4, 5, 6, 7], &*vec_b.0);

    // Automatic `ApplyDry<State>` insertion.
    assert_eq!(
        Some(VecCopyState::new()).as_ref(),
        resources.borrow::<ApplyDry<VecCopyState>>().as_ref()
    );
    // Clean should also exist.
    assert_eq!(
        Some(VecCopyState::new()).as_ref(),
        resources.borrow::<Clean<VecCopyState>>().as_ref()
    );

    Ok(())
}

#[tokio::test]
async fn apply_exec_for_clean() -> Result<(), VecCopyError> {
    let vec_copy_item = VecCopyItem::default();
    let item_wrapper = ItemWrapper::<_, VecCopyError>::from(vec_copy_item);
    let (params_specs, resources, states_current) =
        resources_set_up_with_pre_stored_state(&item_wrapper).await?;

    let mut item_apply_boxed =
        <dyn ItemRt<_>>::clean_prepare(&item_wrapper, &states_current, &params_specs, &resources)
            .await
            .map_err(|(error, _)| error)?;
    cfg_if::cfg_if! {
        if #[cfg(feature = "output_progress")] {
            let (progress_tx, _progress_rx) = mpsc::channel(10);
            let progress_sender = ProgressSender::new(
                VecCopyItem::ID_DEFAULT,
                &progress_tx,
            );
        }
    }
    let fn_ctx = FnCtx::new(
        VecCopyItem::ID_DEFAULT,
        #[cfg(feature = "output_progress")]
        progress_sender,
    );

    <dyn ItemRt<_>>::apply_exec(
        &item_wrapper,
        &params_specs,
        &resources,
        fn_ctx,
        &mut item_apply_boxed,
    )
    .await?;

    let vec_b = resources.borrow::<VecB>();
    assert_eq!(&[0u8; 0], &*vec_b.0);

    // Automatic `Current<State>` insertion.
    assert_eq!(
        Some(VecCopyState::new()).as_ref(),
        resources.borrow::<Current<VecCopyState>>().as_ref()
    );
    // Clean should also exist.
    assert_eq!(
        Some(VecCopyState::new()).as_ref(),
        resources.borrow::<Clean<VecCopyState>>().as_ref()
    );

    Ok(())
}

async fn resources_set_up(
    item_wrapper: &VecCopyItemWrapper,
) -> Result<(ParamsSpecs, Resources<SetUp>), VecCopyError> {
    let mut params_specs = ParamsSpecs::new();
    params_specs.insert(
        VecCopyItem::ID_DEFAULT.clone(),
        ParamsSpec::Value {
            value: VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]),
        },
    );

    let mut resources = Resources::new();
    <dyn ItemRt<_>>::setup(item_wrapper, &mut resources).await?;
    let resources = Resources::<SetUp>::from(resources);

    Ok((params_specs, resources))
}

async fn resources_set_up_with_pre_stored_state(
    item_wrapper: &VecCopyItemWrapper,
) -> Result<(ParamsSpecs, Resources<SetUp>, StatesCurrent), VecCopyError> {
    let (params_specs, mut resources) = resources_set_up(item_wrapper).await?;
    let stored_state = vec![0, 1, 2, 3, 4, 5, 6, 7];
    resources.insert(VecB(stored_state.clone()));
    resources.insert(Current(Some(VecCopyState::from(stored_state.clone()))));
    let states_current = {
        let mut states_mut = StatesMut::new();
        states_mut.insert(
            VecCopyItem::ID_DEFAULT.clone(),
            VecCopyState::from(stored_state),
        );
        StatesCurrent::from(states_mut)
    };

    Ok((params_specs, resources, states_current))
}

async fn resources_and_states_current_stored_and_goal(
    item_wrapper: &VecCopyItemWrapper,
) -> Result<
    (
        ParamsSpecs,
        Resources<SetUp>,
        StatesCurrentStored,
        StatesGoal,
    ),
    VecCopyError,
> {
    let (params_specs, resources) = resources_set_up(item_wrapper).await?;
    cfg_if::cfg_if! {
        if #[cfg(feature = "output_progress")] {
            let (progress_tx, _progress_rx) = mpsc::channel(10);
            let progress_sender = ProgressSender::new(
                VecCopyItem::ID_DEFAULT,
                &progress_tx,
            );
        }
    }
    let fn_ctx = FnCtx::new(
        VecCopyItem::ID_DEFAULT,
        #[cfg(feature = "output_progress")]
        progress_sender,
    );

    let states_current_stored = {
        let mut states_mut = StatesMut::new();
        let state = <dyn ItemRt<_>>::state_current_try_exec(
            item_wrapper,
            &params_specs,
            &resources,
            fn_ctx,
        )
        .await?;
        if let Some(state) = state {
            states_mut.insert_raw(<dyn ItemRt<_>>::id(item_wrapper).clone(), state);
        }

        Into::<StatesCurrentStored>::into(StatesCurrent::from(states_mut))
    };
    let states_goal = {
        let mut states_goal_mut = StatesMut::<states::ts::Goal>::new();
        let state_goal = item_wrapper
            .state_goal_try_exec(&params_specs, &resources, fn_ctx)
            .await?
            .unwrap();
        states_goal_mut.insert_raw(<dyn ItemRt<_>>::id(item_wrapper).clone(), state_goal);

        StatesGoal::from(states_goal_mut)
    };
    Ok((params_specs, resources, states_current_stored, states_goal))
}
