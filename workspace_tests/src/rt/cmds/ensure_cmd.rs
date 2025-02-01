use peace::{
    cfg::{app_name, profile},
    cmd::{
        ctx::CmdCtx,
        interruptible::{InterruptSignal, InterruptStrategy, Interruptibility},
    },
    cmd_model::{CmdBlockDesc, CmdOutcome},
    flow_model::FlowId,
    flow_rt::{Flow, ItemGraphBuilder},
    resource_rt::{
        paths::{StatesCurrentFile, StatesGoalFile},
        type_reg::untagged::BoxDataTypeDowncast,
    },
    rt::cmds::{ApplyStoredStateSync, EnsureCmd, StatesCurrentReadCmd, StatesDiscoverCmd},
    rt_model::{
        ApplyCmdError, Error as PeaceRtError, StateStoredAndDiscovered, Workspace, WorkspaceSpec,
    },
};
use tokio::sync::mpsc;

use crate::{
    mock_item::{MockItem, MockItemError, MockSrc, MockState},
    peace_cmd_ctx_types::PeaceCmdCtxTypes,
    vec_copy_item::VecB,
    NoOpOutput, PeaceTestError, VecA, VecCopyItem, VecCopyState,
};

#[tokio::test]
async fn resources_ensured_dry_does_not_alter_state() -> Result<(), Box<dyn std::error::Error>> {
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

    // Write current and goal states to disk.
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
    StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?;

    // Dry-ensured states.
    // The returned states are currently the same as `StatesCurrentStored`, but it
    // would be useful to return simulated ensured states.
    let CmdOutcome::Complete {
        value: states_ensured_dry,
        cmd_blocks_processed: _,
    } = EnsureCmd::exec_dry(&mut cmd_ctx).await?
    else {
        panic!("Expected `EnsureCmd::exec_dry` to complete successfully.");
    };

    // TODO: When EnsureCmd returns the execution report, assert on the state that
    // was discovered.
    //
    // ```rust,ignore
    // let states = resources.borrow::<StatesCurrent>();
    // let states_goal = resources.borrow::<StatesGoal>();
    // assert_eq!(
    //     Some(VecCopyState::new()).as_ref(),
    //     states.get::<VecCopyState, _>(&VecCopyItem::ID_DEFAULT)
    // );
    // assert_eq!(
    //     Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
    //     states_goal
    //         .get::<VecCopyState, _>(&VecCopyItem::ID_DEFAULT)
    //
    // );
    // ```

    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        states_ensured_dry.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    assert_eq!(
        Some(MockState(1)).as_ref(),
        states_ensured_dry.get::<MockState, _>(MockItem::<()>::ID_DEFAULT)
    );

    Ok(())
}

#[tokio::test]
async fn resources_ensured_contains_state_ensured_for_each_item_when_state_not_yet_ensured(
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

    // Write current and goal states to disk.
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
    StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?;

    // Alter states.
    let output = &mut NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile!("test_profile"))
    .with_flow((&flow).into())
    .await?;
    let CmdOutcome::Complete {
        value: states_ensured,
        cmd_blocks_processed: _,
    } = EnsureCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `EnsureCmd::exec` to complete successfully.");
    };

    // Re-read states from disk.
    let output = &mut NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile!("test_profile"))
    .with_flow((&flow).into())
    .await?;
    let CmdOutcome::Complete {
        value: states_current_stored,
        cmd_blocks_processed: _,
    } = StatesCurrentReadCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesCurrentReadCmd::exec` to complete successfully.");
    };

    // TODO: When EnsureCmd returns the execution report, assert on the state that
    // was discovered.
    //
    // ```rust,ignore
    // let ensured_states_before = resources_ensured.borrow::<StatesCurrent>();
    // let ensured_states_goal = resources_ensured.borrow::<StatesGoal>();
    // assert_eq!(
    //     Some(VecCopyState::new()).as_ref(),
    //     ensured_states_before.get::<VecCopyState, _>(&VecCopyItem::ID_DEFAULT)
    // );
    // assert_eq!(
    //     Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
    //     ensured_states_goal
    //         .get::<VecCopyState, _>(&VecCopyItem::ID_DEFAULT)
    //
    // );
    // ```

    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        states_ensured.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        states_current_stored.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    assert_eq!(
        Some(MockState(1)).as_ref(),
        states_ensured.get::<MockState, _>(MockItem::<()>::ID_DEFAULT)
    );
    assert_eq!(
        Some(MockState(1)).as_ref(),
        states_current_stored.get::<MockState, _>(MockItem::<()>::ID_DEFAULT)
    );

    Ok(())
}

#[tokio::test]
async fn resources_ensured_contains_state_ensured_for_each_item_when_state_already_ensured(
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

    // Write current and goal states to disk.
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
    StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?;

    // Alter states.
    let CmdOutcome::Complete {
        value: states_ensured,
        cmd_blocks_processed: _,
    } = EnsureCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `EnsureCmd::exec` to complete successfully.");
    };

    // Dry ensure states.
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
    .with_item_params::<MockItem<()>>(MockItem::<()>::ID_DEFAULT.clone(), MockSrc(0).into())
    .await?;
    // Changing params changes VecCopyItem state_goal
    StatesDiscoverCmd::goal(&mut cmd_ctx).await?;
    let CmdOutcome::Complete {
        value: states_ensured_dry,
        cmd_blocks_processed: _,
    } = EnsureCmd::exec_dry(&mut cmd_ctx).await?
    else {
        panic!("Expected `EnsureCmd::exec_dry` to complete successfully.");
    };

    // Re-read states from disk.
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
        value: states_current_stored,
        cmd_blocks_processed: _,
    } = StatesCurrentReadCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesCurrentReadCmd::exec` to complete successfully.");
    };

    // TODO: When EnsureCmd returns the execution report, assert on the state that
    // was discovered.
    //
    // ```rust,ignore
    // let ensured_states_before = // StatesCurrent passed in(?) to EnsureCmd
    // let ensured_states_goal = // StatesGoal passed in(?) to EnsureCmd
    // assert_eq!(
    //     Some(VecCopyState::new()).as_ref(),
    //     ensured_states_before.get::<VecCopyState,
    // _>(&VecCopyItem::ID_DEFAULT) );
    // assert_eq!(
    //     Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
    //     ensured_states_goal
    //         .get::<VecCopyState, _>(&VecCopyItem::ID_DEFAULT)
    //
    // );
    // ```
    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        states_ensured.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3])).as_ref(),
        states_ensured_dry.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        states_current_stored.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );

    assert_eq!(
        Some(MockState(1)).as_ref(),
        states_ensured.get::<MockState, _>(MockItem::<()>::ID_DEFAULT)
    );
    assert_eq!(
        Some(MockState(0)).as_ref(),
        states_ensured_dry.get::<MockState, _>(MockItem::<()>::ID_DEFAULT)
    );
    assert_eq!(
        Some(MockState(1)).as_ref(),
        states_current_stored.get::<MockState, _>(MockItem::<()>::ID_DEFAULT)
    );

    Ok(())
}

#[tokio::test]
async fn exec_dry_returns_sync_error_when_current_state_out_of_sync(
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

    // Write current and goal states to disk.
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
    .with_item_params::<MockItem<()>>(MockItem::<()>::ID_DEFAULT.clone(), MockSrc(0).into())
    .await?;
    StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?;

    // Alter states.
    let CmdOutcome::Complete {
        value: states_ensured,
        cmd_blocks_processed: _,
    } = EnsureCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `EnsureCmd::exec` to complete successfully.");
    };

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
    // Overwrite states current.
    cmd_ctx
        .resources_mut()
        .insert(VecB(vec![0, 1, 2, 3, 4, 5, 6, 7]));

    // Dry ensure states.
    let exec_dry_result =
        EnsureCmd::exec_dry_with(&mut cmd_ctx, ApplyStoredStateSync::Current).await;

    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3])).as_ref(),
        states_ensured.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    assert_eq!(
        Some(MockState(0)).as_ref(),
        states_ensured.get::<MockState, _>(MockItem::<()>::ID_DEFAULT)
    );
    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    &exec_dry_result,
                    Err(PeaceTestError::PeaceRt(PeaceRtError::ApplyCmdError(
                        ApplyCmdError::StatesCurrentOutOfSync { items_state_stored_stale }
                    )))
                    if items_state_stored_stale.len() == 1
                    && matches!(
                        items_state_stored_stale.iter().next(),
                        Some((item_id, state_stored_and_discovered))
                        if item_id == VecCopyItem::ID_DEFAULT
                        && matches!(
                            state_stored_and_discovered,
                            StateStoredAndDiscovered::ValuesDiffer { state_stored, state_discovered }
                            if matches!(
                                BoxDataTypeDowncast::<VecCopyState>::downcast_ref(state_stored),
                                Some(state_stored)
                                if **state_stored == [0, 1, 2, 3]
                            )
                            && matches!(
                                BoxDataTypeDowncast::<VecCopyState>::downcast_ref(state_discovered),
                                Some(state_discovered)
                                if **state_discovered == [0, 1, 2, 3, 4, 5, 6, 7]
                            )
                        ),
                    )
                ),
                "Expected `exec_dry_result` to be \
                `Err(.. {{ ApplyCmdError::StatesCurrentOutOfSync {{ .. }} }})`,\n\
                but was `{exec_dry_result:?}`",
            );
        }
    })();

    Ok(())
}

#[tokio::test]
async fn exec_dry_returns_sync_error_when_goal_state_out_of_sync(
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

    // Write current and goal states to disk.
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
    .with_item_params::<MockItem<()>>(MockItem::<()>::ID_DEFAULT.clone(), MockSrc(0).into())
    .await?;
    StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?;

    // Alter states.
    let CmdOutcome::Complete {
        value: states_ensured,
        cmd_blocks_processed: _,
    } = EnsureCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `EnsureCmd::exec` to complete successfully.");
    };

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

    // Dry ensure states.
    let exec_dry_result = EnsureCmd::exec_dry_with(&mut cmd_ctx, ApplyStoredStateSync::Goal).await;

    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3])).as_ref(),
        states_ensured.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    assert_eq!(
        Some(MockState(0)).as_ref(),
        states_ensured.get::<MockState, _>(MockItem::<()>::ID_DEFAULT)
    );
    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    &exec_dry_result,
                    Err(PeaceTestError::PeaceRt(PeaceRtError::ApplyCmdError(
                        ApplyCmdError::StatesGoalOutOfSync { items_state_stored_stale }
                    )))
                    if items_state_stored_stale.len() == 2
                    && matches!(
                        items_state_stored_stale.get(VecCopyItem::ID_DEFAULT),
                        Some(state_stored_and_discovered)
                        if matches!(
                            state_stored_and_discovered,
                            StateStoredAndDiscovered::ValuesDiffer { state_stored, state_discovered }
                            if matches!(
                                BoxDataTypeDowncast::<VecCopyState>::downcast_ref(state_stored),
                                Some(state_stored)
                                if **state_stored == [0, 1, 2, 3]
                            )
                            && matches!(
                                BoxDataTypeDowncast::<VecCopyState>::downcast_ref(state_discovered),
                                Some(state_discovered)
                                if **state_discovered == [0, 1, 2, 3, 4, 5, 6, 7]
                            )
                        ),
                    )
                    && matches!(
                        items_state_stored_stale.get(MockItem::<()>::ID_DEFAULT),
                        Some(state_stored_and_discovered)
                        if matches!(
                            state_stored_and_discovered,
                            StateStoredAndDiscovered::ValuesDiffer { state_stored, state_discovered }
                            if matches!(
                                BoxDataTypeDowncast::<MockState>::downcast_ref(state_stored),
                                Some(state_stored)
                                if **state_stored == 0
                            )
                            && matches!(
                                BoxDataTypeDowncast::<MockState>::downcast_ref(state_discovered),
                                Some(state_discovered)
                                if **state_discovered == 1
                            )
                        ),
                    )
                ),
                "Expected `exec_dry_result` to be \
                `Err(.. {{ ApplyCmdError::StatesGoalOutOfSync {{ .. }} }})`,\n\
                but was `{exec_dry_result:?}`",
            );
        }
    })();

    Ok(())
}

#[tokio::test]
async fn exec_returns_sync_error_when_current_state_out_of_sync(
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

    // Write current and goal states to disk.
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
    .with_item_params::<MockItem<()>>(MockItem::<()>::ID_DEFAULT.clone(), MockSrc(0).into())
    .await?;
    StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?;

    // Alter states.
    let CmdOutcome::Complete {
        value: states_ensured,
        cmd_blocks_processed: _,
    } = EnsureCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `EnsureCmd::exec` to complete successfully.");
    };

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
    // Overwrite states current.
    cmd_ctx
        .resources_mut()
        .insert(VecB(vec![0, 1, 2, 3, 4, 5, 6, 7]));

    // Ensure states.
    let exec_result = EnsureCmd::exec_with(&mut cmd_ctx, ApplyStoredStateSync::Current).await;

    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3])).as_ref(),
        states_ensured.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    assert_eq!(
        Some(MockState(0)).as_ref(),
        states_ensured.get::<MockState, _>(MockItem::<()>::ID_DEFAULT)
    );
    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    &exec_result,
                    Err(PeaceTestError::PeaceRt(PeaceRtError::ApplyCmdError(
                        ApplyCmdError::StatesCurrentOutOfSync { items_state_stored_stale }
                    )))
                    if items_state_stored_stale.len() == 1
                    && matches!(
                        items_state_stored_stale.iter().next(),
                        Some((item_id, state_stored_and_discovered))
                        if item_id == VecCopyItem::ID_DEFAULT
                        && matches!(
                            state_stored_and_discovered,
                            StateStoredAndDiscovered::ValuesDiffer { state_stored, state_discovered }
                            if matches!(
                                BoxDataTypeDowncast::<VecCopyState>::downcast_ref(state_stored),
                                Some(state_stored)
                                if **state_stored == [0, 1, 2, 3]
                            )
                            && matches!(
                                BoxDataTypeDowncast::<VecCopyState>::downcast_ref(state_discovered),
                                Some(state_discovered)
                                if **state_discovered == [0, 1, 2, 3, 4, 5, 6, 7]
                            )
                        ),
                    )
                ),
                "Expected `exec_result` to be \
                `Err(.. {{ ApplyCmdError::StatesCurrentOutOfSync {{ .. }} }})`,\n\
                but was {exec_result:?}",
            );
        }
    })();

    Ok(())
}

#[tokio::test]
async fn exec_returns_sync_error_when_goal_state_out_of_sync(
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

    // Write current and goal states to disk.
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
    .with_item_params::<MockItem<()>>(MockItem::<()>::ID_DEFAULT.clone(), MockSrc(0).into())
    .await?;
    StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?;

    // Alter states.
    let CmdOutcome::Complete {
        value: states_ensured,
        cmd_blocks_processed: _,
    } = EnsureCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `EnsureCmd::exec` to complete successfully.");
    };

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

    // Ensure states.
    let exec_result = EnsureCmd::exec_with(&mut cmd_ctx, ApplyStoredStateSync::Goal).await;

    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3])).as_ref(),
        states_ensured.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    assert_eq!(
        Some(MockState(0)).as_ref(),
        states_ensured.get::<MockState, _>(MockItem::<()>::ID_DEFAULT)
    );
    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    &exec_result,
                    Err(PeaceTestError::PeaceRt(PeaceRtError::ApplyCmdError(
                        ApplyCmdError::StatesGoalOutOfSync { items_state_stored_stale }
                    )))
                    if items_state_stored_stale.len() == 2
                    && matches!(
                        items_state_stored_stale.get(VecCopyItem::ID_DEFAULT),
                        Some(state_stored_and_discovered)
                        if matches!(
                            state_stored_and_discovered,
                            StateStoredAndDiscovered::ValuesDiffer { state_stored, state_discovered }
                            if matches!(
                                BoxDataTypeDowncast::<VecCopyState>::downcast_ref(state_stored),
                                Some(state_stored)
                                if **state_stored == [0, 1, 2, 3]
                            )
                            && matches!(
                                BoxDataTypeDowncast::<VecCopyState>::downcast_ref(state_discovered),
                                Some(state_discovered)
                                if **state_discovered == [0, 1, 2, 3, 4, 5, 6, 7]
                            )
                        ),
                    )
                    && matches!(
                        items_state_stored_stale.get(MockItem::<()>::ID_DEFAULT),
                        Some(state_stored_and_discovered)
                        if matches!(
                            state_stored_and_discovered,
                            StateStoredAndDiscovered::ValuesDiffer { state_stored, state_discovered }
                            if matches!(
                                BoxDataTypeDowncast::<MockState>::downcast_ref(state_stored),
                                Some(state_stored)
                                if **state_stored == 0
                            )
                            && matches!(
                                BoxDataTypeDowncast::<MockState>::downcast_ref(state_discovered),
                                Some(state_discovered)
                                if **state_discovered == 1
                            )
                        ),
                    )
                ),
                "Expected `exec_result` to be \
                `Err(.. {{ ApplyCmdError::StatesGoalOutOfSync {{ .. }} }})`,\n\
                but was {exec_result:?}",
            );
        }
    })();

    Ok(())
}

#[tokio::test]
async fn exec_dry_returns_item_error_when_item_discover_current_returns_error(
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

    // Write current and goal states to disk.
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
    .with_item_params::<MockItem<()>>(MockItem::<()>::ID_DEFAULT.clone(), MockSrc(0).into())
    .await?;
    StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?;

    // Alter states.
    let CmdOutcome::Complete {
        value: states_ensured,
        cmd_blocks_processed: _,
    } = EnsureCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `EnsureCmd::exec` to complete successfully.");
    };

    // Create new `cmd_ctx` with failing state current discovery.
    let output = &mut NoOpOutput;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        graph_builder.add_fn(VecCopyItem::default().into());
        graph_builder.add_fn(
            MockItem::<()>::default()
                .with_state_current(|_fn_ctx, _mock_src, _data| {
                    Err(MockItemError::Synthetic(String::from("state_current_err")))
                })
                .into(),
        );
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
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

    // Dry ensure states.
    let CmdOutcome::ItemError {
        item_stream_outcome,
        cmd_blocks_processed: _,
        cmd_blocks_not_processed: _,
        errors,
    } = EnsureCmd::exec_dry_with(&mut cmd_ctx, ApplyStoredStateSync::Current).await?
    else {
        panic!("Expected `EnsureCmd::exec_dry_with` to complete with item error.");
    };
    let states_ensured_dry = item_stream_outcome.value();

    assert_eq!(
        Some(VecCopyState::from(vec![0, 1, 2, 3])).as_ref(),
        states_ensured.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    assert_eq!(
        Some(MockState(0)).as_ref(),
        states_ensured.get::<MockState, _>(MockItem::<()>::ID_DEFAULT)
    );

    assert_eq!(
        Some(VecCopyState::from(vec![0, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        states_ensured_dry.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    let mock_error = errors.get(MockItem::<()>::ID_DEFAULT);
    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    mock_error,
                    Some(PeaceTestError::Mock(MockItemError::Synthetic(s)))
                    if s == "state_current_err"
                ),
                "Expected `mock_error` to be \
                `Err(.. {{ MockItemError::Synthetic {{ \"state_current_err\" }} }})`,\n\
                but was `{mock_error:?}`",
            );
        }
    })();

    Ok(())
}

#[tokio::test]
async fn exec_dry_returns_item_error_when_item_discover_goal_returns_error(
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

    // Write current and goal states to disk.
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
    .with_item_params::<MockItem<()>>(MockItem::<()>::ID_DEFAULT.clone(), MockSrc(0).into())
    .await?;
    StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?;

    // Alter states.
    let CmdOutcome::Complete {
        value: states_ensured,
        cmd_blocks_processed: _,
    } = EnsureCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `EnsureCmd::exec` to complete successfully.");
    };

    // Create new `cmd_ctx` with failing state current discovery.
    let output = &mut NoOpOutput;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        graph_builder.add_fn(VecCopyItem::default().into());
        graph_builder.add_fn(
            MockItem::<()>::default()
                .with_state_goal(|_fn_ctx, _mock_src, _data| {
                    Err(MockItemError::Synthetic(String::from("state_goal_err")))
                })
                .into(),
        );
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
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

    // Dry ensure states.
    let CmdOutcome::ItemError {
        item_stream_outcome,
        cmd_blocks_processed: _,
        cmd_blocks_not_processed: _,
        errors,
    } = EnsureCmd::exec_dry_with(&mut cmd_ctx, ApplyStoredStateSync::Current).await?
    else {
        panic!("Expected `EnsureCmd::exec_dry_with` to complete with item error.");
    };
    let states_ensured_dry = item_stream_outcome.value();

    assert_eq!(
        Some(VecCopyState::from(vec![0, 1, 2, 3])).as_ref(),
        states_ensured.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    assert_eq!(
        Some(MockState(0)).as_ref(),
        states_ensured.get::<MockState, _>(MockItem::<()>::ID_DEFAULT)
    );

    assert_eq!(
        Some(VecCopyState::from(vec![0, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        states_ensured_dry.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    let mock_error = errors.get(MockItem::<()>::ID_DEFAULT);
    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    mock_error,
                    Some(PeaceTestError::Mock(MockItemError::Synthetic(s)))
                    if s == "state_goal_err"
                ),
                "Expected `mock_error` to be \
                `Err(.. {{ MockItemError::Synthetic {{ \"state_goal_err\" }} }})`,\n\
                but was `{mock_error:?}`",
            );
        }
    })();

    Ok(())
}

#[tokio::test]
async fn exec_dry_returns_item_error_when_item_apply_check_returns_error(
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

    // Write current and goal states to disk.
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
    .with_item_params::<MockItem<()>>(MockItem::<()>::ID_DEFAULT.clone(), MockSrc(0).into())
    .await?;
    StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?;

    // Alter states.
    let CmdOutcome::Complete {
        value: states_ensured,
        cmd_blocks_processed: _,
    } = EnsureCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `EnsureCmd::exec` to complete successfully.");
    };

    // Create new `cmd_ctx` with failing state current discovery.
    let output = &mut NoOpOutput;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        graph_builder.add_fn(VecCopyItem::default().into());
        graph_builder.add_fn(
            MockItem::<()>::default()
                .with_apply_check(|_, _, _, _, _| {
                    Err(MockItemError::Synthetic(String::from("apply_check_err")))
                })
                .into(),
        );
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
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

    // Dry ensure states.
    let CmdOutcome::ItemError {
        item_stream_outcome,
        cmd_blocks_processed: _,
        cmd_blocks_not_processed: _,
        errors,
    } = EnsureCmd::exec_dry_with(&mut cmd_ctx, ApplyStoredStateSync::Current).await?
    else {
        panic!("Expected `EnsureCmd::exec_dry_with` to complete with item error.");
    };
    let states_ensured_dry = item_stream_outcome.value();

    assert_eq!(
        Some(VecCopyState::from(vec![0, 1, 2, 3])).as_ref(),
        states_ensured.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    assert_eq!(
        Some(MockState(0)).as_ref(),
        states_ensured.get::<MockState, _>(MockItem::<()>::ID_DEFAULT)
    );

    assert_eq!(
        Some(VecCopyState::from(vec![0, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        states_ensured_dry.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    let mock_error = errors.get(MockItem::<()>::ID_DEFAULT);
    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    mock_error,
                    Some(PeaceTestError::Mock(MockItemError::Synthetic(s)))
                    if s == "apply_check_err"
                ),
                "Expected `mock_error` to be \
                `Err(.. {{ MockItemError::Synthetic {{ \"apply_check_err\" }} }})`,\n\
                but was `{mock_error:?}`",
            );
        }
    })();

    Ok(())
}

#[tokio::test]
async fn exec_dry_returns_item_error_when_item_apply_dry_returns_error(
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

    // Write current and goal states to disk.
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
    .with_item_params::<MockItem<()>>(MockItem::<()>::ID_DEFAULT.clone(), MockSrc(0).into())
    .await?;
    StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?;

    // Alter states.
    let CmdOutcome::Complete {
        value: states_ensured,
        cmd_blocks_processed: _,
    } = EnsureCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `EnsureCmd::exec` to complete successfully.");
    };

    // Create new `cmd_ctx` with failing state current discovery.
    let output = &mut NoOpOutput;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        graph_builder.add_fn(VecCopyItem::default().into());
        graph_builder.add_fn(
            MockItem::<()>::default()
                .with_apply_dry(|_, _, _, _, _, _| {
                    Err(MockItemError::Synthetic(String::from("apply_dry_err")))
                })
                .into(),
        );
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
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

    // Dry ensure states.
    let CmdOutcome::ItemError {
        item_stream_outcome,
        cmd_blocks_processed: _,
        cmd_blocks_not_processed: _,
        errors,
    } = EnsureCmd::exec_dry_with(&mut cmd_ctx, ApplyStoredStateSync::Current).await?
    else {
        panic!("Expected `EnsureCmd::exec_dry_with` to complete with item error.");
    };
    let states_ensured_dry = item_stream_outcome.value();

    assert_eq!(
        Some(VecCopyState::from(vec![0, 1, 2, 3])).as_ref(),
        states_ensured.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    assert_eq!(
        Some(MockState(0)).as_ref(),
        states_ensured.get::<MockState, _>(MockItem::<()>::ID_DEFAULT)
    );

    assert_eq!(
        Some(VecCopyState::from(vec![0, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        states_ensured_dry.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    let mock_error = errors.get(MockItem::<()>::ID_DEFAULT);
    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    mock_error,
                    Some(PeaceTestError::Mock(MockItemError::Synthetic(s)))
                    if s == "apply_dry_err"
                ),
                "Expected `mock_error` to be \
                `Err(.. {{ MockItemError::Synthetic {{ \"apply_dry_err\" }} }})`,\n\
                but was `{mock_error:?}`",
            );
        }
    })();

    Ok(())
}

#[tokio::test]
async fn exec_returns_item_error_when_item_apply_returns_error(
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

    // Write current and goal states to disk.
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
    .with_item_params::<MockItem<()>>(MockItem::<()>::ID_DEFAULT.clone(), MockSrc(0).into())
    .await?;
    StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?;

    // Alter states.
    let CmdOutcome::Complete {
        value: states_ensured,
        cmd_blocks_processed: _,
    } = EnsureCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `EnsureCmd::exec` to complete successfully.");
    };

    // Create new `cmd_ctx` with failing state current discovery.
    let output = &mut NoOpOutput;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        graph_builder.add_fn(VecCopyItem::default().into());
        graph_builder.add_fn(
            MockItem::<()>::default()
                .with_apply(|_, _, _, _, _, _| {
                    Err(MockItemError::Synthetic(String::from("apply_err")))
                })
                .into(),
        );
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
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

    // Ensure states again.
    let CmdOutcome::ItemError {
        item_stream_outcome,
        cmd_blocks_processed: _,
        cmd_blocks_not_processed: _,
        errors,
    } = EnsureCmd::exec_with(&mut cmd_ctx, ApplyStoredStateSync::Current).await?
    else {
        panic!("Expected `EnsureCmd::exec_with` to complete with item error.");
    };
    let states_ensured_again = item_stream_outcome.value();

    assert_eq!(
        Some(VecCopyState::from(vec![0, 1, 2, 3])).as_ref(),
        states_ensured.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    assert_eq!(
        Some(MockState(0)).as_ref(),
        states_ensured.get::<MockState, _>(MockItem::<()>::ID_DEFAULT)
    );

    assert_eq!(
        Some(VecCopyState::from(vec![0, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        states_ensured_again.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    let mock_error = errors.get(MockItem::<()>::ID_DEFAULT);
    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    mock_error,
                    Some(PeaceTestError::Mock(MockItemError::Synthetic(s)))
                    if s == "apply_err"
                ),
                "Expected `mock_error` to be \
                `Err(.. {{ MockItemError::Synthetic {{ \"apply_err\" }} }})`,\n\
                but was `{mock_error:?}`",
            );
        }
    })();

    Ok(())
}

#[tokio::test]
async fn states_current_not_serialized_on_states_current_read_cmd_block_interrupt(
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

    let (interrupt_tx, interrupt_rx) = mpsc::channel::<InterruptSignal>(16);

    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_interruptibility(Interruptibility::new(
        interrupt_rx.into(),
        InterruptStrategy::FinishCurrent,
    ))
    .with_profile(profile!("test_profile"))
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
    )
    .with_item_params::<MockItem<()>>(MockItem::<()>::ID_DEFAULT.clone(), MockSrc(1).into())
    .await?;

    StatesDiscoverCmd::goal(&mut cmd_ctx).await?;
    // Note: Write custom states current file to disk.
    let flow_dir = cmd_ctx.flow_dir();
    let states_current_content = "\
        vec_copy: [0, 1, 2, 3]\n\
        mock: 123\n\
    ";
    let states_current_file = StatesCurrentFile::from(flow_dir);
    tokio::fs::write(&states_current_file, states_current_content.as_bytes()).await?;

    interrupt_tx.send(InterruptSignal).await?;
    let cmd_outcome = EnsureCmd::exec_with(&mut cmd_ctx, ApplyStoredStateSync::None).await?;
    let CmdOutcome::ExecutionInterrupted {
        value,
        cmd_blocks_processed,
        cmd_blocks_not_processed,
    } = cmd_outcome
    else {
        panic!(
            "Expected `EnsureCmd::exec_with` to complete with interruption,\n\
            but was:\n\
            \n\
            ```ron\n\
            {cmd_outcome:#?}\n\
            ```\n\
            "
        );
    };
    let states_ensured =
        value.expect("Expected `states_ensured` to be returned despite interruption.");

    // Early interruption returns empty `states_ensured`.
    assert!(states_ensured.is_empty());
    assert!(cmd_blocks_processed.is_empty());
    assert_eq!(
        &[
            "StatesCurrentReadCmdBlock",
            "StatesGoalReadCmdBlock",
            "StatesDiscoverCmdBlock",
            "ApplyExecCmdBlock",
        ],
        cmd_blocks_not_processed
            .iter()
            .map(CmdBlockDesc::cmd_block_name)
            .collect::<Vec<_>>()
            .as_slice()
    );
    let states_current_content_after_exec = tokio::fs::read_to_string(&states_current_file).await?;
    assert_eq!(states_current_content, states_current_content_after_exec);

    Ok(())
}

#[tokio::test]
async fn states_current_not_serialized_on_states_goal_read_cmd_block_interrupt(
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

    let (interrupt_tx, interrupt_rx) = mpsc::channel::<InterruptSignal>(16);

    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_interruptibility(Interruptibility::new(
        interrupt_rx.into(),
        InterruptStrategy::PollNextN(2),
    ))
    .with_profile(profile!("test_profile"))
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
    )
    .with_item_params::<MockItem<()>>(MockItem::<()>::ID_DEFAULT.clone(), MockSrc(1).into())
    .await?;

    StatesDiscoverCmd::goal(&mut cmd_ctx).await?;
    // Note: Write custom states current file to disk.
    let flow_dir = cmd_ctx.flow_dir();
    let states_current_content = "\
        vec_copy: [0, 1, 2, 3]\n\
        mock: 123\n\
    ";
    let states_current_file = StatesCurrentFile::from(flow_dir);
    tokio::fs::write(&states_current_file, states_current_content.as_bytes()).await?;

    interrupt_tx.send(InterruptSignal).await?;
    let cmd_outcome = EnsureCmd::exec_with(&mut cmd_ctx, ApplyStoredStateSync::None).await?;
    let CmdOutcome::ExecutionInterrupted {
        value,
        cmd_blocks_processed,
        cmd_blocks_not_processed,
    } = cmd_outcome
    else {
        panic!(
            "Expected `EnsureCmd::exec_with` to complete with interruption,\n\
            but was:\n\
            \n\
            ```ron\n\
            {cmd_outcome:#?}\n\
            ```\n\
            "
        );
    };
    let states_ensured =
        value.expect("Expected `states_ensured` to be returned despite interruption.");

    // Early interruption returns empty `states_ensured`.
    assert!(states_ensured.is_empty());
    assert_eq!(
        &["StatesCurrentReadCmdBlock", "StatesGoalReadCmdBlock",],
        cmd_blocks_processed
            .iter()
            .map(CmdBlockDesc::cmd_block_name)
            .collect::<Vec<_>>()
            .as_slice()
    );
    assert_eq!(
        &["StatesDiscoverCmdBlock", "ApplyExecCmdBlock",],
        cmd_blocks_not_processed
            .iter()
            .map(CmdBlockDesc::cmd_block_name)
            .collect::<Vec<_>>()
            .as_slice()
    );
    let states_current_content_after_exec = tokio::fs::read_to_string(&states_current_file).await?;
    assert_eq!(states_current_content, states_current_content_after_exec);

    Ok(())
}

#[tokio::test]
async fn states_current_not_serialized_on_states_discover_cmd_block_fail(
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

    // Note: Write goal states to disk.
    StatesDiscoverCmd::goal(&mut cmd_ctx).await?;
    // Note: Write custom states current file to disk.
    let flow_dir = cmd_ctx.flow_dir();
    let states_current_content = "\
        vec_copy: [0, 1, 2, 3]\n\
        mock: 123\n\
    ";
    let states_current_file = StatesCurrentFile::from(flow_dir);
    tokio::fs::write(&states_current_file, states_current_content.as_bytes()).await?;

    // Note: Change `MockItem` to fail on `try_state_current`.
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        let vec_copy_fn_id = graph_builder.add_fn(VecCopyItem::default().into());
        let mock_fn_id = graph_builder.add_fn(
            MockItem::<()>::default()
                .with_try_state_current(|_, _, _| {
                    Err(MockItemError::Synthetic(String::from(
                        "try_state_current_err",
                    )))
                })
                .into(),
        );
        graph_builder.add_logic_edge(vec_copy_fn_id, mock_fn_id)?;
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile!("test_profile"))
    .with_flow((&flow).into())
    .await?;

    let CmdOutcome::ItemError {
        item_stream_outcome,
        cmd_blocks_processed,
        cmd_blocks_not_processed,
        errors,
    } = EnsureCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `EnsureCmd::exec` to complete with item error.");
    };
    let states_ensured = item_stream_outcome.value();

    assert_eq!(
        None,
        states_ensured.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    assert_eq!(
        None,
        states_ensured.get::<MockState, _>(MockItem::<()>::ID_DEFAULT)
    );
    assert_eq!(
        &["StatesCurrentReadCmdBlock", "StatesGoalReadCmdBlock",],
        cmd_blocks_processed
            .iter()
            .map(CmdBlockDesc::cmd_block_name)
            .collect::<Vec<_>>()
            .as_slice()
    );
    assert_eq!(
        &[
            "StatesDiscoverCmdBlock",
            "ApplyStateSyncCheckCmdBlock",
            "ApplyExecCmdBlock",
        ],
        cmd_blocks_not_processed
            .iter()
            .map(CmdBlockDesc::cmd_block_name)
            .collect::<Vec<_>>()
            .as_slice()
    );

    let states_current_content_after_exec = tokio::fs::read_to_string(&states_current_file).await?;
    assert_eq!(states_current_content, states_current_content_after_exec);

    let mock_error = errors.get(MockItem::<()>::ID_DEFAULT);
    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    mock_error,
                    Some(PeaceTestError::Mock(MockItemError::Synthetic(s)))
                    if s == "try_state_current_err"
                ),
                "Expected `mock_error` to be \
                `Err(.. {{ MockItemError::Synthetic {{ \"try_state_current_err\" }} }})`,\n\
                but was `{mock_error:?}`",
            );
        }
    })();

    Ok(())
}

#[tokio::test]
async fn states_current_not_serialized_on_apply_state_sync_check_cmd_block_interrupt(
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

    let (interrupt_tx, interrupt_rx) = mpsc::channel::<InterruptSignal>(16);

    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_interruptibility(Interruptibility::new(
        interrupt_rx.into(),
        InterruptStrategy::PollNextN(7),
    ))
    .with_profile(profile!("test_profile"))
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
    )
    .with_item_params::<MockItem<()>>(MockItem::<()>::ID_DEFAULT.clone(), MockSrc(1).into())
    .await?;

    // Note: Write custom states current and states goal files to disk.
    let flow_dir = cmd_ctx.flow_dir();
    let states_current_content = "\
        vec_copy: []\n\
        mock: 0\n\
    ";
    let states_goal_content = "\
        vec_copy: [0, 1, 2, 3, 4, 5, 6, 7]\n\
        mock: 1\n\
    ";
    let states_current_file = StatesCurrentFile::from(flow_dir);
    tokio::fs::write(&states_current_file, states_current_content.as_bytes()).await?;
    let states_goal_file = StatesGoalFile::from(flow_dir);
    tokio::fs::write(&states_goal_file, states_goal_content.as_bytes()).await?;

    interrupt_tx.send(InterruptSignal).await?;
    let cmd_outcome = EnsureCmd::exec_with(&mut cmd_ctx, ApplyStoredStateSync::Both).await?;
    let CmdOutcome::ExecutionInterrupted {
        value,
        cmd_blocks_processed,
        cmd_blocks_not_processed,
    } = cmd_outcome
    else {
        panic!(
            "Expected `EnsureCmd::exec_with` to complete with interruption,\n\
            but was:\n\
            \n\
            ```ron\n\
            {cmd_outcome:#?}\n\
            ```\n\
            "
        );
    };
    let states_ensured =
        value.expect("Expected `states_ensured` to be returned despite interruption.");

    // Early interruption returns empty `states_ensured`.
    assert!(states_ensured.is_empty());
    assert_eq!(
        &[
            "StatesCurrentReadCmdBlock",
            "StatesGoalReadCmdBlock",
            "StatesDiscoverCmdBlock",
            "ApplyStateSyncCheckCmdBlock",
        ],
        cmd_blocks_processed
            .iter()
            .map(CmdBlockDesc::cmd_block_name)
            .collect::<Vec<_>>()
            .as_slice()
    );
    assert_eq!(
        &["ApplyExecCmdBlock",],
        cmd_blocks_not_processed
            .iter()
            .map(CmdBlockDesc::cmd_block_name)
            .collect::<Vec<_>>()
            .as_slice()
    );
    let states_current_content_after_exec = tokio::fs::read_to_string(&states_current_file).await?;
    assert_eq!(states_current_content, states_current_content_after_exec);
    let states_goal_content_after_exec = tokio::fs::read_to_string(&states_goal_file).await?;
    assert_eq!(states_goal_content, states_goal_content_after_exec);

    Ok(())
}

#[tokio::test]
async fn states_current_is_serialized_on_apply_exec_cmd_block_interrupt(
) -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        let vec_copy_id = graph_builder.add_fn(VecCopyItem::default().into());
        let mock_id = graph_builder.add_fn(MockItem::<()>::default().into());
        graph_builder.add_logic_edge(vec_copy_id, mock_id)?;
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let output = &mut NoOpOutput;

    let (interrupt_tx, interrupt_rx) = mpsc::channel::<InterruptSignal>(16);

    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_interruptibility(Interruptibility::new(
        interrupt_rx.into(),
        InterruptStrategy::PollNextN(9),
    ))
    .with_profile(profile!("test_profile"))
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
    )
    .with_item_params::<MockItem<()>>(MockItem::<()>::ID_DEFAULT.clone(), MockSrc(1).into())
    .await?;

    // Note: Write custom states current and states goal files to disk.
    let flow_dir = cmd_ctx.flow_dir();
    let states_current_content = "\
        vec_copy: []\n\
        mock: 0\n\
    ";
    let states_goal_content = "\
        vec_copy: [0, 1, 2, 3, 4, 5, 6, 7]\n\
        mock: 1\n\
    ";
    let states_current_file = StatesCurrentFile::from(flow_dir);
    tokio::fs::write(&states_current_file, states_current_content.as_bytes()).await?;
    let states_goal_file = StatesGoalFile::from(flow_dir);
    tokio::fs::write(&states_goal_file, states_goal_content.as_bytes()).await?;

    interrupt_tx.send(InterruptSignal).await?;
    let cmd_outcome = EnsureCmd::exec_with(&mut cmd_ctx, ApplyStoredStateSync::Both).await?;
    let CmdOutcome::BlockInterrupted {
        item_stream_outcome,
        cmd_blocks_processed,
        cmd_blocks_not_processed,
    } = cmd_outcome
    else {
        panic!(
            "Expected `EnsureCmd::exec_with` to complete with interruption,\n\
            but was:\n\
            \n\
            ```ron\n\
            {cmd_outcome:#?}\n\
            ```\n\
            "
        );
    };
    let states_ensured = item_stream_outcome.value();

    // Early interruption returns empty `states_ensured`.
    assert_eq!(2, states_ensured.len());
    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        states_ensured.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    assert_eq!(
        Some(MockState(0)).as_ref(),
        states_ensured.get::<MockState, _>(MockItem::<()>::ID_DEFAULT)
    );
    assert_eq!(
        &[
            "StatesCurrentReadCmdBlock",
            "StatesGoalReadCmdBlock",
            "StatesDiscoverCmdBlock",
            "ApplyStateSyncCheckCmdBlock",
        ],
        cmd_blocks_processed
            .iter()
            .map(CmdBlockDesc::cmd_block_name)
            .collect::<Vec<_>>()
            .as_slice()
    );
    assert_eq!(
        &["ApplyExecCmdBlock",],
        cmd_blocks_not_processed
            .iter()
            .map(CmdBlockDesc::cmd_block_name)
            .collect::<Vec<_>>()
            .as_slice()
    );

    // Note: Expect interruption to be between `vec_copy` and `mock` items.
    let states_current_content_expected = r#"vec_copy:
- 0
- 1
- 2
- 3
- 4
- 5
- 6
- 7
mock: 0
"#;
    // Note: states_goal_file is re-serialized, because we may have generated
    // information for earlier items that are used for later items' goal state.
    let states_goal_content_expected = r#"vec_copy:
- 0
- 1
- 2
- 3
- 4
- 5
- 6
- 7
mock: 1
"#;
    let states_current_content_after_exec = tokio::fs::read_to_string(&states_current_file).await?;
    assert_eq!(
        states_current_content_expected,
        states_current_content_after_exec
    );
    let states_goal_content_after_exec = tokio::fs::read_to_string(&states_goal_file).await?;
    assert_eq!(states_goal_content_expected, states_goal_content_after_exec);

    Ok(())
}

#[test]
fn debug() {
    let debug_str = format!("{:?}", EnsureCmd::<PeaceCmdCtxTypes>::default());
    assert_eq!(
        r#"EnsureCmd(PhantomData<workspace_tests::peace_cmd_ctx_types::PeaceCmdCtxTypes>)"#,
        debug_str,
    );
}
