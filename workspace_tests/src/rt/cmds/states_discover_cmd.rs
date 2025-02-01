use peace::{
    cfg::{app_name, profile},
    cmd::ctx::CmdCtx,
    cmd_model::CmdOutcome,
    flow_model::FlowId,
    flow_rt::{Flow, ItemGraphBuilder},
    item_model::ItemId,
    resource_rt::{
        paths::StatesGoalFile,
        states::{StatesCurrentStored, StatesGoal},
        type_reg::untagged::{BoxDtDisplay, TypeReg},
    },
    rt::cmds::{EnsureCmd, StatesCurrentReadCmd, StatesDiscoverCmd, StatesGoalReadCmd},
    rt_model::{Workspace, WorkspaceSpec},
};

use crate::{
    mock_item::{MockItem, MockItemError, MockSrc, MockState},
    peace_cmd_ctx_types::PeaceCmdCtxTypes,
    vec_copy_item::VecB,
    NoOpOutput, PeaceTestError, VecA, VecCopyItem, VecCopyState,
};

#[cfg(feature = "output_progress")]
use peace::progress_model::{ProgressComplete, ProgressStatus};

#[tokio::test]
async fn current_and_goal_discovers_both_states_current_and_goal(
) -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        graph_builder.add_fn(VecCopyItem::default().into());
        graph_builder.add_fn(MockItem::<()>::default().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let output = &mut NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile!("test_profile"))
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
    )
    .with_item_params::<MockItem<()>>(MockItem::<()>::ID_DEFAULT.clone(), MockSrc(1).into())
    .await?;

    let CmdOutcome::Complete {
        value: (states_current, states_goal),
        cmd_blocks_processed: _,
    } = StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesDiscoverCmd::current_and_goal` to complete successfully.");
    };
    let CmdOutcome::Complete {
        value: states_current_stored,
        cmd_blocks_processed: _,
    } = StatesCurrentReadCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesCurrentReadCmd::exec` to complete successfully.");
    };
    let CmdOutcome::Complete {
        value: states_goal_stored,
        cmd_blocks_processed: _,
    } = StatesGoalReadCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesGoalReadCmd::exec` to complete successfully.");
    };

    let vec_copy_current_state = states_current.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    let vec_copy_current_stored_state =
        states_current_stored.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    let vec_copy_goal_state = states_goal.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    let vec_copy_goal_stored_state =
        states_goal_stored.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);

    assert_eq!(Some(VecCopyState::new()).as_ref(), vec_copy_current_state);
    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        vec_copy_goal_state
    );
    assert_eq!(vec_copy_current_state, vec_copy_current_stored_state);
    assert_eq!(vec_copy_goal_state, vec_copy_goal_stored_state);

    let mock_current_state = states_current.get::<MockState, _>(MockItem::<()>::ID_DEFAULT);
    let mock_current_stored_state =
        states_current_stored.get::<MockState, _>(MockItem::<()>::ID_DEFAULT);
    let mock_goal_state = states_goal.get::<MockState, _>(MockItem::<()>::ID_DEFAULT);
    let mock_goal_stored_state = states_goal_stored.get::<MockState, _>(MockItem::<()>::ID_DEFAULT);

    assert_eq!(Some(MockState(0)).as_ref(), mock_current_state);
    assert_eq!(Some(MockState(1)).as_ref(), mock_goal_state);
    assert_eq!(mock_current_state, mock_current_stored_state);
    assert_eq!(mock_goal_state, mock_goal_stored_state);

    #[cfg(feature = "output_progress")]
    {
        let cmd_progress_tracker = cmd_ctx.cmd_progress_tracker();
        let progress_tracker = cmd_progress_tracker
            .progress_trackers()
            .get(VecCopyItem::ID_DEFAULT)
            .unwrap_or_else(
                #[cfg_attr(coverage_nightly, coverage(off))]
                || {
                    panic!(
                        "Expected `progress_tracker` to exist for {}",
                        VecCopyItem::ID_DEFAULT
                    )
                },
            );
        assert_eq!(
            &ProgressStatus::Complete(ProgressComplete::Success),
            progress_tracker.progress_status()
        );
    }

    Ok(())
}

#[tokio::test]
async fn current_runs_state_current_for_each_item() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        graph_builder.add_fn(VecCopyItem::default().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let output = &mut NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile!("test_profile"))
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
    )
    .await?;

    let CmdOutcome::Complete {
        value: states_current,
        cmd_blocks_processed: _,
    } = StatesDiscoverCmd::current(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesDiscoverCmd::current` to complete successfully.");
    };
    let CmdOutcome::Complete {
        value: states_current_stored,
        cmd_blocks_processed: _,
    } = StatesCurrentReadCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesCurrentReadCmd::exec` to complete successfully.");
    };

    let vec_copy_current_state = states_current.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    let vec_copy_current_stored_state =
        states_current_stored.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);

    assert_eq!(Some(VecCopyState::new()).as_ref(), vec_copy_current_state);
    assert_eq!(vec_copy_current_state, vec_copy_current_stored_state);

    Ok(())
}

#[tokio::test]
async fn current_inserts_states_current_stored_from_states_current_file(
) -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        graph_builder.add_fn(VecCopyItem::default().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let output = &mut NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile!("test_profile"))
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
    )
    .await?;

    // Writes to states_current_file.yaml
    StatesDiscoverCmd::current(&mut cmd_ctx).await?;

    // Execute again to ensure StatesCurrentStored is included
    //
    // Note: The actual logic is part of `CmdCtxBuilder::build`, implemented by
    // `impl_build.rs`.
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile!("test_profile"))
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
    )
    .await?;
    StatesDiscoverCmd::current(&mut cmd_ctx).await?;
    let resources = cmd_ctx.resources();
    let states_current_stored_from_cmd_ctx = resources.borrow::<StatesCurrentStored>();

    let output = &mut NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile!("test_profile"))
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
    )
    .await?;
    let CmdOutcome::Complete {
        value: states_current_stored,
        cmd_blocks_processed: _,
    } = StatesCurrentReadCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesCurrentReadCmd::exec` to complete successfully.");
    };

    let vec_copy_current_state =
        states_current_stored_from_cmd_ctx.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    assert_eq!(Some(VecCopyState::new()).as_ref(), vec_copy_current_state);
    assert_eq!(
        states_current_stored_from_cmd_ctx.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT),
        states_current_stored.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );

    Ok(())
}

#[tokio::test]
async fn current_returns_error_when_try_state_current_returns_error(
) -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        graph_builder.add_fn(VecCopyItem::default().into());
        graph_builder.add_fn(
            MockItem::<()>::default()
                .with_try_state_current(|_fn_ctx, _params_partial, _data| {
                    Err(MockItemError::Synthetic(String::from("synthetic")))
                })
                .into(),
        );
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let output = &mut NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile!("test_profile"))
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
    )
    .with_item_params::<MockItem<()>>(MockItem::<()>::ID_DEFAULT.clone(), MockSrc(1u8).into())
    .await?;

    let CmdOutcome::ItemError {
        item_stream_outcome,
        cmd_blocks_processed: _,
        cmd_blocks_not_processed: _,
        errors,
    } = StatesDiscoverCmd::current(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesDiscoverCmd::current` to complete with item error.");
    };
    let states_current = item_stream_outcome.value();

    let vec_copy_state = states_current.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    let CmdOutcome::Complete {
        value: states_on_disk,
        cmd_blocks_processed: _,
    } = StatesCurrentReadCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesCurrentReadCmd::exec` to complete successfully.");
    };
    assert_eq!(Some(VecCopyState::new()).as_ref(), vec_copy_state);
    assert_eq!(
        states_current.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT),
        states_on_disk.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );

    let error = errors.get(MockItem::<()>::ID_DEFAULT);
    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    &error,
                    Some(PeaceTestError::Mock(MockItemError::Synthetic(s)))
                    if s == "synthetic"
                ),
                "expected error to be `Some(PeaceTestError::Mock(MockItemError::Synthetic(\"synthetic\")))`,\n\
                but was {error:?}"
            )
        }
    })();

    Ok(())
}

#[tokio::test]
async fn goal_returns_error_when_try_state_goal_returns_error(
) -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        graph_builder.add_fn(VecCopyItem::default().into());
        graph_builder.add_fn(
            MockItem::<()>::default()
                .with_try_state_goal(|_fn_ctx, _params_partial, _data| {
                    Err(MockItemError::Synthetic(String::from("synthetic")))
                })
                .into(),
        );
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let output = &mut NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile!("test_profile"))
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
    )
    .with_item_params::<MockItem<()>>(MockItem::<()>::ID_DEFAULT.clone(), MockSrc(1u8).into())
    .await?;

    let CmdOutcome::ItemError {
        item_stream_outcome,
        cmd_blocks_processed: _,
        cmd_blocks_not_processed: _,
        errors,
    } = StatesDiscoverCmd::goal(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesDiscoverCmd::goal` to complete with item error.");
    };
    let states_goal = item_stream_outcome.value();

    let vec_copy_state = states_goal.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    let CmdOutcome::Complete {
        value: states_on_disk,
        cmd_blocks_processed: _,
    } = StatesGoalReadCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesGoalReadCmd::exec` to complete successfully.");
    };
    assert_eq!(
        Some(VecCopyState::from(vec![0, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        vec_copy_state
    );
    assert_eq!(
        states_goal.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT),
        states_on_disk.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );

    let error = errors.get(MockItem::<()>::ID_DEFAULT);
    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    &error,
                    Some(PeaceTestError::Mock(MockItemError::Synthetic(s)))
                    if s == "synthetic"
                ),
                "expected error to be `Some(PeaceTestError::Mock(MockItemError::Synthetic(\"synthetic\")))`,\n\
                but was {error:?}"
            )
        }
    })();

    Ok(())
}

#[tokio::test]
async fn current_and_goal_returns_error_when_try_state_current_returns_error(
) -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        graph_builder.add_fn(VecCopyItem::default().into());
        graph_builder.add_fn(
            MockItem::<()>::default()
                .with_try_state_current(|_fn_ctx, _params_partial, _data| {
                    Err(MockItemError::Synthetic(String::from("synthetic")))
                })
                .into(),
        );
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let output = &mut NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile!("test_profile"))
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
    )
    .with_item_params::<MockItem<()>>(MockItem::<()>::ID_DEFAULT.clone(), MockSrc(1u8).into())
    .await?;

    let CmdOutcome::ItemError {
        item_stream_outcome,
        cmd_blocks_processed: _,
        cmd_blocks_not_processed: _,
        errors,
    } = StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesDiscoverCmd::current_and_goal` to complete with item error.");
    };
    let (states_current, states_goal) = item_stream_outcome.value();

    // States current assertions
    let CmdOutcome::Complete {
        value: states_on_disk,
        cmd_blocks_processed: _,
    } = StatesCurrentReadCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesCurrentReadCmd::exec` to complete successfully.");
    };

    let vec_copy_state = states_current.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    assert_eq!(Some(VecCopyState::new()).as_ref(), vec_copy_state);
    assert_eq!(
        states_current.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT),
        states_on_disk.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );

    // States goal assertions
    let CmdOutcome::Complete {
        value: states_on_disk,
        cmd_blocks_processed: _,
    } = StatesGoalReadCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesGoalReadCmd::exec` to complete successfully.");
    };

    let vec_copy_state = states_goal.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    assert_eq!(
        Some(VecCopyState::from(vec![0, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        vec_copy_state
    );
    assert_eq!(
        states_goal.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT),
        states_on_disk.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );

    let mock_state = states_goal.get::<MockState, _>(MockItem::<()>::ID_DEFAULT);
    assert_eq!(Some(MockState(1u8)).as_ref(), mock_state);
    assert_eq!(
        states_goal.get::<MockState, _>(MockItem::<()>::ID_DEFAULT),
        states_on_disk.get::<MockState, _>(MockItem::<()>::ID_DEFAULT)
    );

    let error = errors.get(MockItem::<()>::ID_DEFAULT);
    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    &error,
                    Some(PeaceTestError::Mock(MockItemError::Synthetic(s)))
                    if s == "synthetic"
                ),
                "expected error to be `Some(PeaceTestError::Mock(MockItemError::Synthetic(\"synthetic\")))`,\n\
                but was {error:?}"
            )
        }
    })();

    Ok(())
}

#[tokio::test]
async fn current_and_goal_returns_error_when_try_state_goal_returns_error(
) -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        graph_builder.add_fn(VecCopyItem::default().into());
        graph_builder.add_fn(
            MockItem::<()>::default()
                .with_try_state_goal(|_fn_ctx, _params_partial, _data| {
                    Err(MockItemError::Synthetic(String::from("synthetic")))
                })
                .into(),
        );
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let output = &mut NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile!("test_profile"))
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
    )
    .with_item_params::<MockItem<()>>(MockItem::<()>::ID_DEFAULT.clone(), MockSrc(1u8).into())
    .await?;

    let CmdOutcome::ItemError {
        item_stream_outcome,
        cmd_blocks_processed: _,
        cmd_blocks_not_processed: _,
        errors,
    } = StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesDiscoverCmd::current_and_goal` to complete with item error.");
    };
    let (states_current, states_goal) = item_stream_outcome.value();

    // States current assertions
    let CmdOutcome::Complete {
        value: states_on_disk,
        cmd_blocks_processed: _,
    } = StatesCurrentReadCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesCurrentReadCmd::exec` to complete successfully.");
    };

    let vec_copy_state = states_current.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    assert_eq!(Some(VecCopyState::new()).as_ref(), vec_copy_state);
    assert_eq!(
        states_current.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT),
        states_on_disk.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );

    let mock_state = states_current.get::<MockState, _>(MockItem::<()>::ID_DEFAULT);
    assert_eq!(Some(MockState::new()).as_ref(), mock_state);
    assert_eq!(
        states_current.get::<MockState, _>(MockItem::<()>::ID_DEFAULT),
        states_on_disk.get::<MockState, _>(MockItem::<()>::ID_DEFAULT)
    );

    // States goal assertions
    let vec_copy_state = states_goal.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    let CmdOutcome::Complete {
        value: states_on_disk,
        cmd_blocks_processed: _,
    } = StatesGoalReadCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesGoalReadCmd::exec` to complete successfully.");
    };
    assert_eq!(
        Some(VecCopyState::from(vec![0, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        vec_copy_state
    );
    assert_eq!(
        states_goal.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT),
        states_on_disk.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );

    let error = errors.get(MockItem::<()>::ID_DEFAULT);
    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    &error,
                    Some(PeaceTestError::Mock(MockItemError::Synthetic(s)))
                    if s == "synthetic"
                ),
                "expected error to be `Some(PeaceTestError::Mock(MockItemError::Synthetic(\"synthetic\")))`,\n\
                but was {error:?}"
            )
        }
    })();

    Ok(())
}

#[tokio::test]
async fn current_and_goal_returns_current_error_when_both_try_state_current_and_try_state_goal_return_error(
) -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        graph_builder.add_fn(VecCopyItem::default().into());
        graph_builder.add_fn(
            MockItem::<()>::default()
                .with_try_state_current(|_fn_ctx, _params_partial, _data| {
                    Err(MockItemError::Synthetic(String::from("current_err")))
                })
                .with_try_state_goal(|_fn_ctx, _params_partial, _data| {
                    Err(MockItemError::Synthetic(String::from("goal_err")))
                })
                .into(),
        );
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let output = &mut NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile!("test_profile"))
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
    )
    .with_item_params::<MockItem<()>>(MockItem::<()>::ID_DEFAULT.clone(), MockSrc(1u8).into())
    .await?;

    let CmdOutcome::ItemError {
        item_stream_outcome,
        cmd_blocks_processed: _,
        cmd_blocks_not_processed: _,
        errors,
    } = StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesDiscoverCmd::current_and_goal` to complete with item error.");
    };
    let (states_current, states_goal) = item_stream_outcome.value();

    // States current assertions
    let CmdOutcome::Complete {
        value: states_on_disk,
        cmd_blocks_processed: _,
    } = StatesCurrentReadCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesCurrentReadCmd::exec` to complete successfully.");
    };

    let vec_copy_state = states_current.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    assert_eq!(Some(VecCopyState::new()).as_ref(), vec_copy_state);
    assert_eq!(
        states_current.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT),
        states_on_disk.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );

    // States goal assertions
    let vec_copy_state = states_goal.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    let CmdOutcome::Complete {
        value: states_on_disk,
        cmd_blocks_processed: _,
    } = StatesGoalReadCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesGoalReadCmd::exec` to complete successfully.");
    };
    assert_eq!(
        Some(VecCopyState::from(vec![0, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        vec_copy_state
    );
    assert_eq!(
        states_goal.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT),
        states_on_disk.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );

    let error = errors.get(MockItem::<()>::ID_DEFAULT);
    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    &error,
                    Some(PeaceTestError::Mock(MockItemError::Synthetic(s)))
                    if s == "current_err"
                ),
                "expected error to be `Some(PeaceTestError::Mock(MockItemError::Synthetic(\"current_err\")))`,\n\
                but was {error:?}"
            )
        }
    })();

    Ok(())
}

#[tokio::test]
async fn goal_runs_state_goal_for_each_item() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        graph_builder.add_fn(VecCopyItem::default().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let output = &mut NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile!("test_profile"))
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
    )
    .await?;

    let CmdOutcome::Complete {
        value: states_goal,
        cmd_blocks_processed: _,
    } = StatesDiscoverCmd::goal(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesDiscoverCmd::goal` to complete successfully.");
    };
    let resources = cmd_ctx.resources();

    let vec_copy_goal_state = states_goal.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    let states_goal_on_disk = {
        let states_goal_file = resources.borrow::<StatesGoalFile>();
        let states_slice = std::fs::read(&*states_goal_file)?;

        let mut type_reg = TypeReg::<ItemId, BoxDtDisplay>::new_typed();
        type_reg.register::<VecCopyState>(VecCopyItem::ID_DEFAULT.clone());

        let deserializer = serde_yaml::Deserializer::from_slice(&states_slice);
        StatesGoal::from(type_reg.deserialize_map(deserializer)?)
    };
    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        vec_copy_goal_state
    );
    assert_eq!(
        states_goal.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT),
        states_goal_on_disk.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );

    Ok(())
}

#[tokio::test]
async fn current_with_does_not_serialize_states_when_told_not_to(
) -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        graph_builder.add_fn(VecCopyItem::default().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let output = &mut NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile!("test_profile"))
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3]).into(),
    )
    .await?;

    // Write to disk first.
    assert!(matches!(
        StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?,
        CmdOutcome::Complete { .. }
    ));
    assert!(matches!(
        EnsureCmd::exec(&mut cmd_ctx).await?,
        CmdOutcome::Complete { .. }
    ));

    // Discover without serializing to storage.
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile!("test_profile"))
    .with_flow((&flow).into())
    .await?;
    // Overwrite states current.
    cmd_ctx
        .resources_mut()
        .insert(VecB(vec![0, 1, 2, 3, 4, 5, 6, 7]));

    let CmdOutcome::Complete {
        value: states_current,
        cmd_blocks_processed: _,
    } = StatesDiscoverCmd::current_with(&mut cmd_ctx, false).await?
    else {
        panic!("Expected `StatesDiscoverCmd::current_with` to complete successfully.");
    };

    let vec_copy_state = states_current.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    let CmdOutcome::Complete {
        value: states_current_stored,
        cmd_blocks_processed: _,
    } = StatesCurrentReadCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesCurrentReadCmd::exec` to complete successfully.");
    };
    assert_eq!(
        Some(VecCopyState::from(vec![0, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        vec_copy_state
    );
    assert_eq!(
        Some(&VecCopyState::from(vec![0, 1, 2, 3])),
        states_current_stored.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    assert_eq!(
        Some(&VecCopyState::from(vec![0, 1, 2, 3, 4, 5, 6, 7])),
        states_current.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT),
    );

    Ok(())
}

#[tokio::test]
async fn goal_with_does_not_serialize_states_when_told_not_to(
) -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        graph_builder.add_fn(VecCopyItem::default().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let output = &mut NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile!("test_profile"))
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3]).into(),
    )
    .await?;

    // Write to disk first.
    assert!(matches!(
        StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?,
        CmdOutcome::Complete { .. }
    ));

    // Discover without serializing to storage.
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile!("test_profile"))
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
    )
    .await?;

    let CmdOutcome::Complete {
        value: states_goal,
        cmd_blocks_processed: _,
    } = StatesDiscoverCmd::goal_with(&mut cmd_ctx, false).await?
    else {
        panic!("Expected `StatesDiscoverCmd::goal_with` to complete successfully.");
    };

    let vec_copy_state = states_goal.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    let CmdOutcome::Complete {
        value: states_goal_stored,
        cmd_blocks_processed: _,
    } = StatesGoalReadCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesGoalReadCmd::exec` to complete successfully.");
    };
    assert_eq!(
        Some(VecCopyState::from(vec![0, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        vec_copy_state
    );
    assert_eq!(
        Some(&VecCopyState::from(vec![0, 1, 2, 3])),
        states_goal_stored.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    assert_eq!(
        Some(&VecCopyState::from(vec![0, 1, 2, 3, 4, 5, 6, 7])),
        states_goal.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT),
    );

    Ok(())
}

#[cfg(feature = "output_progress")]
#[tokio::test]
async fn current_with_sets_progress_complete_for_successful_items(
) -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        graph_builder.add_fn(VecCopyItem::default().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let output = &mut NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile!("test_profile"))
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
    )
    .await?;

    let _cmd_outcome = StatesDiscoverCmd::<_>::current_with(&mut cmd_ctx, false).await;

    let cmd_progress_tracker = cmd_ctx.cmd_progress_tracker();
    let vec_copy_progress_tracker = cmd_progress_tracker
        .progress_trackers()
        .get(VecCopyItem::ID_DEFAULT)
        .unwrap_or_else(
            #[cfg_attr(coverage_nightly, coverage(off))]
            || {
                panic!(
                    "Expected `progress_tracker` to exist for {}",
                    VecCopyItem::ID_DEFAULT
                )
            },
        );
    let vec_copy_progress_status = vec_copy_progress_tracker.progress_status();
    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    vec_copy_progress_status,
                    ProgressStatus::Complete(ProgressComplete::Success),
                ),
                "expected `vec_copy_progress_status` to be `Initialized` or `Pending`, but was {vec_copy_progress_status:?}"
            );
        }
    })();

    Ok(())
}

#[cfg(feature = "output_progress")]
#[tokio::test]
async fn goal_with_sets_progress_complete_for_successful_items(
) -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        graph_builder.add_fn(VecCopyItem::default().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let output = &mut NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile!("test_profile"))
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
    )
    .await?;

    let _cmd_outcome = StatesDiscoverCmd::<_>::goal_with(&mut cmd_ctx, false).await?;

    let cmd_progress_tracker = cmd_ctx.cmd_progress_tracker();
    let vec_copy_progress_tracker = cmd_progress_tracker
        .progress_trackers()
        .get(VecCopyItem::ID_DEFAULT)
        .unwrap_or_else(
            #[cfg_attr(coverage_nightly, coverage(off))]
            || {
                panic!(
                    "Expected `progress_tracker` to exist for {}",
                    VecCopyItem::ID_DEFAULT
                )
            },
        );
    let vec_copy_progress_status = vec_copy_progress_tracker.progress_status();
    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    vec_copy_progress_status,
                    ProgressStatus::Complete(ProgressComplete::Success),
                ),
                "expected `vec_copy_progress_status` to be `Initialized` or `Pending`, but was {vec_copy_progress_status:?}"
            );
        }
    })();

    Ok(())
}

#[cfg(feature = "output_progress")]
#[tokio::test]
async fn current_and_goal_with_sets_progress_complete_for_successful_items(
) -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        graph_builder.add_fn(VecCopyItem::default().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let output = &mut NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile!("test_profile"))
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
    )
    .await?;

    let _cmd_outcome = StatesDiscoverCmd::<_>::current_and_goal_with(&mut cmd_ctx, false).await?;

    let cmd_progress_tracker = cmd_ctx.cmd_progress_tracker();
    let vec_copy_progress_tracker = cmd_progress_tracker
        .progress_trackers()
        .get(VecCopyItem::ID_DEFAULT)
        .unwrap_or_else(
            #[cfg_attr(coverage_nightly, coverage(off))]
            || {
                panic!(
                    "Expected `progress_tracker` to exist for {}",
                    VecCopyItem::ID_DEFAULT
                )
            },
        );
    let vec_copy_progress_status = vec_copy_progress_tracker.progress_status();
    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    vec_copy_progress_status,
                    ProgressStatus::Complete(ProgressComplete::Success),
                ),
                "expected `vec_copy_progress_status` to be `Initialized` or `Pending`, but was {vec_copy_progress_status:?}"
            );
        }
    })();

    Ok(())
}

#[test]
fn debug() {
    let debug_str = format!("{:?}", StatesDiscoverCmd::<PeaceCmdCtxTypes>::default());
    assert_eq!(
        r#"StatesDiscoverCmd(PhantomData<workspace_tests::peace_cmd_ctx_types::PeaceCmdCtxTypes>)"#,
        debug_str
    );
}
