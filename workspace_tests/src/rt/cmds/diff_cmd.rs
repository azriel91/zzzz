use diff::{VecDiff, VecDiffType};
use peace::{
    cfg::{app_name, profile},
    cli::output::CliOutput,
    cmd::ctx::CmdCtx,
    cmd_model::CmdOutcome,
    flow_model::FlowId,
    flow_rt::{Flow, ItemGraphBuilder},
    params::ParamsSpec,
    resource_rt::states::{
        ts::{Current, CurrentStored, Goal, GoalStored},
        StatesCurrent, StatesGoal,
    },
    rt::cmds::{DiffCmd, StatesDiscoverCmd},
    rt_model::{output::OutputWrite, Workspace, WorkspaceSpec},
};

use crate::{
    mock_item::{MockDest, MockDiff, MockItem, MockSrc, MockState},
    peace_cmd_ctx_types::PeaceCmdCtxTypes,
    NoOpOutput, PeaceTestError, VecA, VecB, VecCopyDiff, VecCopyItem, VecCopyState,
};

mod diff_info_spec;
mod diff_state_spec;

#[tokio::test]
async fn diff_stored_contains_state_diff_for_each_item() -> Result<(), Box<dyn std::error::Error>> {
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

    // Discover current and goal states.
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

    // Diff current and goal states.
    let CmdOutcome::Complete {
        value: state_diffs,
        cmd_blocks_processed: _,
    } = DiffCmd::diff_stored(&mut cmd_ctx).await?
    else {
        panic!("Expected `DiffCmd::diff_stored` to complete successfully.");
    };

    let vec_diff = state_diffs.get::<VecCopyDiff, _>(VecCopyItem::ID_DEFAULT);
    let vec_copy_current_state = states_current.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    let vec_copy_goal_state = states_goal.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    assert_eq!(Some(VecCopyState::new()).as_ref(), vec_copy_current_state);
    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        vec_copy_goal_state
    );
    assert_eq!(
        Some(VecCopyDiff::from(VecDiff(vec![VecDiffType::Inserted {
            index: 0,
            changes: vec![0u8, 1, 2, 3, 4, 5, 6, 7]
        }])))
        .as_ref(),
        vec_diff
    );

    let mock_diff = state_diffs.get::<MockDiff, _>(MockItem::<()>::ID_DEFAULT);
    let mock_current_state = states_current.get::<MockState, _>(MockItem::<()>::ID_DEFAULT);
    let mock_goal_state = states_goal.get::<MockState, _>(MockItem::<()>::ID_DEFAULT);

    assert_eq!(Some(MockState(0)).as_ref(), mock_current_state);
    assert_eq!(Some(MockState(1)).as_ref(), mock_goal_state);
    assert_eq!(Some(MockDiff(1)).as_ref(), mock_diff);

    Ok(())
}

#[tokio::test]
async fn diff_discover_current_on_demand() -> Result<(), Box<dyn std::error::Error>> {
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

    // Discover goal state.
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
        value: states_goal,
        cmd_blocks_processed: _,
    } = StatesDiscoverCmd::goal(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesDiscoverCmd::goal` to complete successfully.");
    };

    // Diff current and stored goal states.
    let CmdOutcome::Complete {
        value: state_diffs,
        cmd_blocks_processed: _,
    } = DiffCmd::diff::<Current, GoalStored>(&mut cmd_ctx).await?
    else {
        panic!("Expected `DiffCmd::diff` to complete successfully.");
    };

    // Note: discovered `StatesGoal` is not automatically serialized to storage.
    let resources = &cmd_ctx.view().resources;
    let states_current = resources.borrow::<StatesCurrent>();

    let vec_diff = state_diffs.get::<VecCopyDiff, _>(VecCopyItem::ID_DEFAULT);
    let vec_copy_current_state = states_current.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    let vec_copy_goal_state = states_goal.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    assert_eq!(Some(VecCopyState::new()).as_ref(), vec_copy_current_state);
    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        vec_copy_goal_state
    );
    assert_eq!(
        Some(VecCopyDiff::from(VecDiff(vec![VecDiffType::Inserted {
            index: 0,
            changes: vec![0u8, 1, 2, 3, 4, 5, 6, 7]
        }])))
        .as_ref(),
        vec_diff
    );

    let mock_diff = state_diffs.get::<MockDiff, _>(MockItem::<()>::ID_DEFAULT);
    let mock_current_state = states_current.get::<MockState, _>(MockItem::<()>::ID_DEFAULT);
    let mock_goal_state = states_goal.get::<MockState, _>(MockItem::<()>::ID_DEFAULT);

    assert_eq!(Some(MockState(0)).as_ref(), mock_current_state);
    assert_eq!(Some(MockState(1)).as_ref(), mock_goal_state);
    assert_eq!(Some(MockDiff(1)).as_ref(), mock_diff);

    Ok(())
}

#[tokio::test]
async fn diff_discover_goal_on_demand() -> Result<(), Box<dyn std::error::Error>> {
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

    // Discover current state.
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
        value: states_current,
        cmd_blocks_processed: _,
    } = StatesDiscoverCmd::current(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesDiscoverCmd::current` to complete successfully.");
    };

    // Diff current stored and goal states.
    let CmdOutcome::Complete {
        value: state_diffs,
        cmd_blocks_processed: _,
    } = DiffCmd::diff::<CurrentStored, Goal>(&mut cmd_ctx).await?
    else {
        panic!("Expected `DiffCmd::diff` to complete successfully.");
    };

    // Note: discovered `StatesGoal` is not automatically serialized to storage.
    let resources = &cmd_ctx.view().resources;
    let states_goal = resources.borrow::<StatesGoal>();

    let vec_diff = state_diffs.get::<VecCopyDiff, _>(VecCopyItem::ID_DEFAULT);
    let vec_copy_current_state = states_current.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    let vec_copy_goal_state = states_goal.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    assert_eq!(Some(VecCopyState::new()).as_ref(), vec_copy_current_state);
    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        vec_copy_goal_state
    );
    assert_eq!(
        Some(VecCopyDiff::from(VecDiff(vec![VecDiffType::Inserted {
            index: 0,
            changes: vec![0u8, 1, 2, 3, 4, 5, 6, 7]
        }])))
        .as_ref(),
        vec_diff
    );

    let mock_diff = state_diffs.get::<MockDiff, _>(MockItem::<()>::ID_DEFAULT);
    let mock_current_state = states_current.get::<MockState, _>(MockItem::<()>::ID_DEFAULT);
    let mock_goal_state = states_goal.get::<MockState, _>(MockItem::<()>::ID_DEFAULT);

    assert_eq!(Some(MockState(0)).as_ref(), mock_current_state);
    assert_eq!(Some(MockState(1)).as_ref(), mock_goal_state);
    assert_eq!(Some(MockDiff(1)).as_ref(), mock_diff);

    Ok(())
}

#[tokio::test]
async fn diff_discover_current_and_goal_on_demand() -> Result<(), Box<dyn std::error::Error>> {
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

    // No discovery of current or goal states before diffing.
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

    // Diff current and goal states.
    let CmdOutcome::Complete {
        value: state_diffs,
        cmd_blocks_processed: _,
    } = DiffCmd::diff::<Current, Goal>(&mut cmd_ctx).await?
    else {
        panic!("Expected `DiffCmd::diff` to complete successfully.");
    };

    // Note: discovered `StatesCurrent` and `StatesGoal` are not automatically
    // serialized to storage.
    let resources = &cmd_ctx.view().resources;
    let states_current = resources.borrow::<StatesCurrent>();
    let states_goal = resources.borrow::<StatesGoal>();

    let vec_diff = state_diffs.get::<VecCopyDiff, _>(VecCopyItem::ID_DEFAULT);
    let vec_copy_current_state = states_current.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    let vec_copy_goal_state = states_goal.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    assert_eq!(Some(VecCopyState::new()).as_ref(), vec_copy_current_state);
    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        vec_copy_goal_state
    );
    assert_eq!(
        Some(VecCopyDiff::from(VecDiff(vec![VecDiffType::Inserted {
            index: 0,
            changes: vec![0u8, 1, 2, 3, 4, 5, 6, 7]
        }])))
        .as_ref(),
        vec_diff
    );

    let mock_diff = state_diffs.get::<MockDiff, _>(MockItem::<()>::ID_DEFAULT);
    let mock_current_state = states_current.get::<MockState, _>(MockItem::<()>::ID_DEFAULT);
    let mock_goal_state = states_goal.get::<MockState, _>(MockItem::<()>::ID_DEFAULT);

    assert_eq!(Some(MockState(0)).as_ref(), mock_current_state);
    assert_eq!(Some(MockState(1)).as_ref(), mock_goal_state);
    assert_eq!(Some(MockDiff(1)).as_ref(), mock_diff);

    Ok(())
}

#[tokio::test]
async fn diff_stored_with_multiple_profiles() -> Result<(), Box<dyn std::error::Error>> {
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

    // profile_0
    let profile_0 = profile!("test_profile_0");
    let mut cmd_ctx_0 = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile_0.clone())
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
    )
    .with_item_params::<MockItem<()>>(MockItem::<()>::ID_DEFAULT.clone(), MockSrc(1).into())
    .await?;
    let resources = cmd_ctx_0.resources_mut();
    resources.insert(MockDest(1));
    let CmdOutcome::Complete {
        value: states_current_0,
        cmd_blocks_processed: _,
    } = StatesDiscoverCmd::current(&mut cmd_ctx_0).await?
    else {
        panic!("Expected `StatesDiscoverCmd::current` to complete successfully.");
    };

    // profile_1
    let profile_1 = profile!("test_profile_1");
    let mut cmd_ctx_1 = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile_1.clone())
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
    )
    .with_item_params::<MockItem<()>>(MockItem::<()>::ID_DEFAULT.clone(), MockSrc(3).into())
    .await?;
    let resources = cmd_ctx_1.resources_mut();
    resources.insert(VecB(vec![0, 1, 2, 3, 4, 5, 6, 7]));
    resources.insert(MockDest(3));
    let CmdOutcome::Complete {
        value: states_current_1,
        cmd_blocks_processed: _,
    } = StatesDiscoverCmd::current(&mut cmd_ctx_1).await?
    else {
        panic!("Expected `StatesDiscoverCmd::current` to complete successfully.");
    };

    let mut cmd_ctx_multi =
        CmdCtx::builder_multi_profile_single_flow::<PeaceTestError, NoOpOutput>(
            output.into(),
            (&workspace).into(),
        )
        .with_flow((&flow).into())
        .await?;

    // Diff current states for profile_0 and profile_1.
    let state_diffs =
        DiffCmd::diff_current_stored(&mut cmd_ctx_multi, &profile_0, &profile_1).await?;

    let vec_diff = state_diffs.get::<VecCopyDiff, _>(VecCopyItem::ID_DEFAULT);
    assert_eq!(
        Some(VecCopyState::new()).as_ref(),
        states_current_0.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        states_current_1.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    assert_eq!(
        Some(VecCopyDiff::from(VecDiff(vec![VecDiffType::Inserted {
            index: 0,
            changes: vec![0u8, 1, 2, 3, 4, 5, 6, 7]
        }])))
        .as_ref(),
        vec_diff
    );

    let vec_diff = state_diffs.get::<VecCopyDiff, _>(VecCopyItem::ID_DEFAULT);
    let vec_copy_0_current_state = states_current_0.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    let vec_copy_1_current_state = states_current_1.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    assert_eq!(Some(VecCopyState::new()).as_ref(), vec_copy_0_current_state);
    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        vec_copy_1_current_state
    );
    assert_eq!(
        Some(VecCopyDiff::from(VecDiff(vec![VecDiffType::Inserted {
            index: 0,
            changes: vec![0u8, 1, 2, 3, 4, 5, 6, 7]
        }])))
        .as_ref(),
        vec_diff
    );

    let mock_diff = state_diffs.get::<MockDiff, _>(MockItem::<()>::ID_DEFAULT);
    let mock_0_current_state = states_current_0.get::<MockState, _>(MockItem::<()>::ID_DEFAULT);
    let mock_1_current_state = states_current_1.get::<MockState, _>(MockItem::<()>::ID_DEFAULT);

    assert_eq!(Some(MockState(1)).as_ref(), mock_0_current_state);
    assert_eq!(Some(MockState(3)).as_ref(), mock_1_current_state);
    assert_eq!(Some(MockDiff(2)).as_ref(), mock_diff);

    Ok(())
}

#[tokio::test]
async fn diff_stored_with_missing_profile_0() -> Result<(), Box<dyn std::error::Error>> {
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

    // profile_0
    let profile_0 = profile!("test_profile_0");

    // profile_1
    let profile_1 = profile!("test_profile_1");
    let mut cmd_ctx_1 = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile_1.clone())
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
    )
    .await?;
    let resources = cmd_ctx_1.resources_mut();
    resources.insert(VecB(vec![0, 1, 2, 3, 4, 5, 6, 7]));
    StatesDiscoverCmd::current(&mut cmd_ctx_1).await?;

    let mut cmd_ctx_multi =
        CmdCtx::builder_multi_profile_single_flow::<PeaceTestError, NoOpOutput>(
            output.into(),
            (&workspace).into(),
        )
        .with_flow((&flow).into())
        .await?;

    let diff_result =
        DiffCmd::diff_current_stored(&mut cmd_ctx_multi, &profile_0, &profile_1).await;

    assert!(matches!(
            diff_result,
            Err(PeaceTestError::PeaceRt(
                peace::rt_model::Error::ProfileNotInScope { profile, profiles_in_scope }
            ))
            if profile == profile_0 && profiles_in_scope == vec![profile_1]));

    Ok(())
}

#[tokio::test]
async fn diff_stored_with_missing_profile_1() -> Result<(), Box<dyn std::error::Error>> {
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

    // profile_0
    let profile_0 = profile!("test_profile_0");
    let mut cmd_ctx_0 = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile_0.clone())
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
    )
    .await?;
    StatesDiscoverCmd::current(&mut cmd_ctx_0).await?;

    // profile_1
    let profile_1 = profile!("test_profile_1");

    let mut cmd_ctx_multi =
        CmdCtx::builder_multi_profile_single_flow::<PeaceTestError, NoOpOutput>(
            output.into(),
            (&workspace).into(),
        )
        .with_flow((&flow).into())
        .await?;

    let diff_result =
        DiffCmd::diff_current_stored(&mut cmd_ctx_multi, &profile_0, &profile_1).await;

    assert!(matches!(
            diff_result,
            Err(PeaceTestError::PeaceRt(
                peace::rt_model::Error::ProfileNotInScope { profile, profiles_in_scope }
            ))
            if profile == profile_1 && profiles_in_scope == vec![profile_0]));

    Ok(())
}

#[tokio::test]
async fn diff_stored_with_profile_0_missing_states_current(
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

    // profile_0
    let profile_0 = profile!("test_profile_0");
    let mut cmd_ctx_0 = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile_0.clone())
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
    )
    .await?;
    StatesDiscoverCmd::goal(&mut cmd_ctx_0).await?;

    // profile_1
    let profile_1 = profile!("test_profile_1");
    let mut cmd_ctx_1 = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile_1.clone())
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
    )
    .await?;
    let resources = cmd_ctx_1.resources_mut();
    resources.insert(VecB(vec![0, 1, 2, 3, 4, 5, 6, 7]));
    StatesDiscoverCmd::current(&mut cmd_ctx_1).await?;

    let mut cmd_ctx_multi =
        CmdCtx::builder_multi_profile_single_flow::<PeaceTestError, NoOpOutput>(
            output.into(),
            (&workspace).into(),
        )
        .with_flow((&flow).into())
        .await?;

    let diff_result =
        DiffCmd::diff_current_stored(&mut cmd_ctx_multi, &profile_0, &profile_1).await;

    assert!(matches!(
            diff_result,
            Err(PeaceTestError::PeaceRt(
                peace::rt_model::Error::ProfileStatesCurrentNotDiscovered { profile }
            ))
            if profile == profile_0));

    Ok(())
}

#[tokio::test]
async fn diff_stored_with_profile_1_missing_states_current(
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

    // profile_0
    let profile_0 = profile!("test_profile_0");
    let mut cmd_ctx_0 = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile_0.clone())
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
    )
    .await?;
    StatesDiscoverCmd::current(&mut cmd_ctx_0).await?;

    // profile_1
    let profile_1 = profile!("test_profile_1");
    let mut cmd_ctx_1 = CmdCtx::builder_single_profile_single_flow::<PeaceTestError, NoOpOutput>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile(profile_1.clone())
    .with_flow((&flow).into())
    .with_item_params::<VecCopyItem>(
        VecCopyItem::ID_DEFAULT.clone(),
        VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
    )
    .await?;
    StatesDiscoverCmd::goal(&mut cmd_ctx_1).await?;

    let mut cmd_ctx_multi =
        CmdCtx::builder_multi_profile_single_flow::<PeaceTestError, NoOpOutput>(
            output.into(),
            (&workspace).into(),
        )
        .with_flow((&flow).into())
        .await?;

    let diff_result =
        DiffCmd::diff_current_stored(&mut cmd_ctx_multi, &profile_0, &profile_1).await;

    assert!(matches!(
            diff_result,
            Err(PeaceTestError::PeaceRt(
                peace::rt_model::Error::ProfileStatesCurrentNotDiscovered { profile }
            ))
            if profile == profile_1));

    Ok(())
}

#[tokio::test]
async fn diff_with_multiple_changes() -> Result<(), Box<dyn std::error::Error>> {
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
    let mut buffer = Vec::with_capacity(256);
    let output = CliOutput::new_with_writer(&mut buffer);

    // Discover current and goal states.
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow::<
        PeaceTestError,
        CliOutput<&mut Vec<u8>>,
    >(output.into(), (&workspace).into())
    .with_profile(profile!("test_profile"))
    .with_flow((&flow).into())
    // overwrite initial state
    .with_resource(VecA(vec![0, 1, 2, 4, 5, 6, 8, 9]))
    .with_resource(VecB(vec![0, 1, 2, 3, 4, 5, 6, 7]))
    .with_item_params::<VecCopyItem>(VecCopyItem::ID_DEFAULT.clone(), ParamsSpec::InMemory)
    .await?;
    let CmdOutcome::Complete {
        value: (states_current, states_goal),
        cmd_blocks_processed: _,
    } = StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesDiscoverCmd::current_and_goal` to complete successfully.");
    };

    // Diff current and goal states.
    let CmdOutcome::Complete {
        value: state_diffs,
        cmd_blocks_processed: _,
    } = DiffCmd::diff_stored(&mut cmd_ctx).await?
    else {
        panic!("Expected `DiffCmd::diff_stored` to complete successfully.");
    };
    <_ as OutputWrite<PeaceTestError>>::present(cmd_ctx.output_mut(), &state_diffs).await?;

    let vec_diff = state_diffs.get::<VecCopyDiff, _>(VecCopyItem::ID_DEFAULT);
    assert_eq!(
        Some(VecCopyState::from(vec![0, 1, 2, 3, 4, 5, 6, 7]),).as_ref(),
        states_current.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 4, 5, 6, 8, 9])).as_ref(),
        states_goal.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    assert_eq!(
        Some(VecCopyDiff::from(VecDiff(vec![
            VecDiffType::Removed { index: 3, len: 1 },
            VecDiffType::Altered {
                index: 7,
                changes: vec![1] // 8 - 7 = 1
            },
            VecDiffType::Inserted {
                index: 8,
                changes: vec![9]
            },
        ])))
        .as_ref(),
        vec_diff
    );
    assert_eq!(
        "1. `vec_copy`: [(-)3..4, (~)7;1, (+)8;9, ]\n",
        String::from_utf8(buffer)?
    );

    Ok(())
}

#[test]
fn debug() {
    let debug_str = format!("{:?}", DiffCmd::<PeaceCmdCtxTypes, ()>::default());
    assert_eq!(
        r#"DiffCmd(PhantomData<(workspace_tests::peace_cmd_ctx_types::PeaceCmdCtxTypes, ())>)"#,
        debug_str,
    );
}
