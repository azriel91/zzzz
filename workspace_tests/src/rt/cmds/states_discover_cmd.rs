use peace::{
    cfg::{app_name, profile, AppName, FlowId, ItemId, Profile},
    cmd::ctx::CmdCtx,
    resources::{
        paths::{StatesCurrentFile, StatesGoalFile},
        states::{StatesCurrent, StatesCurrentStored, StatesGoal},
        type_reg::untagged::{BoxDtDisplay, TypeReg},
    },
    rt::cmds::{
        cmd_ctx_internal::CmdIndependence, EnsureCmd, StatesCurrentReadCmd, StatesDiscoverCmd,
        StatesGoalReadCmd,
    },
    rt_model::{outcomes::CmdOutcome, Flow, ItemGraphBuilder, Workspace, WorkspaceSpec},
};

use crate::{
    vec_copy_item::VecB, NoOpOutput, PeaceTestError, VecA, VecCopyError, VecCopyItem, VecCopyState,
};

#[cfg(feature = "output_progress")]
use peace::cfg::progress::{ProgressComplete, ProgressStatus};

#[tokio::test]
async fn current_and_goal_discovers_both_states_current_and_goal()
-> Result<(), Box<dyn std::error::Error>> {
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
    let mut output = NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .with_item_params::<VecCopyItem>(
            VecCopyItem::ID_DEFAULT.clone(),
            VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
        )
        .await?;

    let CmdOutcome {
        value: (states_current, states_goal),
        errors: _,
    } = StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?;
    let resources = cmd_ctx.resources();

    let vec_copy_state = states_current.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    let states_on_disk = {
        let states_current_file = resources.borrow::<StatesCurrentFile>();
        let states_slice = std::fs::read(&*states_current_file)?;

        let mut type_reg = TypeReg::<ItemId, BoxDtDisplay>::new_typed();
        type_reg.register::<VecCopyState>(VecCopyItem::ID_DEFAULT.clone());

        let deserializer = serde_yaml::Deserializer::from_slice(&states_slice);
        StatesCurrent::from(type_reg.deserialize_map(deserializer)?)
    };
    let vec_copy_goal_state = states_goal.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    let states_goal_on_disk = {
        let states_goal_file = resources.borrow::<StatesGoalFile>();
        let states_slice = std::fs::read(&*states_goal_file)?;

        let mut type_reg = TypeReg::<ItemId, BoxDtDisplay>::new_typed();
        type_reg.register::<VecCopyState>(VecCopyItem::ID_DEFAULT.clone());

        let deserializer = serde_yaml::Deserializer::from_slice(&states_slice);
        StatesGoal::from(type_reg.deserialize_map(deserializer)?)
    };
    assert_eq!(Some(VecCopyState::new()).as_ref(), vec_copy_state);
    assert_eq!(
        states_current.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT),
        states_on_disk.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );
    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        vec_copy_goal_state
    );
    assert_eq!(
        states_goal.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT),
        states_goal_on_disk.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );

    #[cfg(feature = "output_progress")]
    {
        let cmd_progress_tracker = cmd_ctx.cmd_progress_tracker();
        let progress_tracker = cmd_progress_tracker
            .progress_trackers()
            .get(VecCopyItem::ID_DEFAULT)
            .unwrap_or_else(|| {
                panic!(
                    "Expected `progress_tracker` to exist for {}",
                    VecCopyItem::ID_DEFAULT
                )
            });
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
    let mut output = NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .with_item_params::<VecCopyItem>(
            VecCopyItem::ID_DEFAULT.clone(),
            VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
        )
        .await?;

    let CmdOutcome {
        value: states_current,
        errors: _,
    } = StatesDiscoverCmd::current(&mut cmd_ctx).await?;
    let resources = cmd_ctx.resources();

    let vec_copy_state = states_current.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    let states_on_disk = {
        let states_current_file = resources.borrow::<StatesCurrentFile>();
        let states_slice = std::fs::read(&*states_current_file)?;

        let mut type_reg = TypeReg::<ItemId, BoxDtDisplay>::new_typed();
        type_reg.register::<VecCopyState>(VecCopyItem::ID_DEFAULT.clone());

        let deserializer = serde_yaml::Deserializer::from_slice(&states_slice);
        StatesCurrent::from(type_reg.deserialize_map(deserializer)?)
    };
    assert_eq!(Some(VecCopyState::new()).as_ref(), vec_copy_state);
    assert_eq!(
        states_current.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT),
        states_on_disk.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );

    Ok(())
}

#[tokio::test]
async fn current_inserts_states_current_stored_from_states_current_file()
-> Result<(), Box<dyn std::error::Error>> {
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
    let mut output = NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .with_item_params::<VecCopyItem>(
            VecCopyItem::ID_DEFAULT.clone(),
            VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
        )
        .await?;

    // Writes to states_current_file.yaml
    StatesDiscoverCmd::current(&mut cmd_ctx).await?;

    // Execute again to ensure StatesCurrentStored is included
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .with_item_params::<VecCopyItem>(
            VecCopyItem::ID_DEFAULT.clone(),
            VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
        )
        .await?;
    StatesDiscoverCmd::current(&mut cmd_ctx).await?;
    let resources = cmd_ctx.resources();

    let states_current_stored = resources.borrow::<StatesCurrentStored>();
    let vec_copy_state = states_current_stored.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    let states_on_disk = {
        let states_current_file = resources.borrow::<StatesCurrentFile>();
        let states_slice = std::fs::read(&*states_current_file)?;

        let mut type_reg = TypeReg::<ItemId, BoxDtDisplay>::new_typed();
        type_reg.register::<VecCopyState>(VecCopyItem::ID_DEFAULT.clone());

        let deserializer = serde_yaml::Deserializer::from_slice(&states_slice);
        StatesCurrent::from(type_reg.deserialize_map(deserializer)?)
    };
    assert_eq!(Some(VecCopyState::new()).as_ref(), vec_copy_state);
    assert_eq!(
        states_current_stored.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT),
        states_on_disk.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT)
    );

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
    let mut output = NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .with_item_params::<VecCopyItem>(
            VecCopyItem::ID_DEFAULT.clone(),
            VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
        )
        .await?;

    let CmdOutcome {
        value: states_goal,
        errors: _,
    } = StatesDiscoverCmd::goal(&mut cmd_ctx).await?;
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
async fn current_with_does_not_serialize_states_when_told_not_to()
-> Result<(), Box<dyn std::error::Error>> {
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
    let mut output = NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .with_item_params::<VecCopyItem>(
            VecCopyItem::ID_DEFAULT.clone(),
            VecA(vec![0, 1, 2, 3]).into(),
        )
        .await?;

    // Write to disk first.
    assert!(
        StatesDiscoverCmd::current_and_goal(&mut cmd_ctx)
            .await?
            .is_ok()
    );
    assert!(EnsureCmd::exec(&mut cmd_ctx).await?.is_ok());

    // Discover without serializing to storage.
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .await?;
    // Overwrite states current.
    cmd_ctx
        .resources_mut()
        .insert(VecB(vec![0, 1, 2, 3, 4, 5, 6, 7]));

    let CmdOutcome {
        value: states_current,
        errors: _,
    } = StatesDiscoverCmd::<_, NoOpOutput, _>::current_with(
        &mut CmdIndependence::SubCmd {
            cmd_view: &mut cmd_ctx.view(),
        },
        false,
    )
    .await?;

    let vec_copy_state = states_current.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    let states_current_stored = StatesCurrentReadCmd::exec(&mut cmd_ctx).await?;
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
async fn goal_with_does_not_serialize_states_when_told_not_to()
-> Result<(), Box<dyn std::error::Error>> {
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
    let mut output = NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .with_item_params::<VecCopyItem>(
            VecCopyItem::ID_DEFAULT.clone(),
            VecA(vec![0, 1, 2, 3]).into(),
        )
        .await?;

    // Write to disk first.
    assert!(
        StatesDiscoverCmd::current_and_goal(&mut cmd_ctx)
            .await?
            .is_ok()
    );

    // Discover without serializing to storage.
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .with_item_params::<VecCopyItem>(
            VecCopyItem::ID_DEFAULT.clone(),
            VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
        )
        .await?;

    let CmdOutcome {
        value: states_goal,
        errors: _,
    } = StatesDiscoverCmd::<_, NoOpOutput, _>::goal_with(
        &mut CmdIndependence::SubCmd {
            cmd_view: &mut cmd_ctx.view(),
        },
        false,
    )
    .await?;

    let vec_copy_state = states_goal.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    let states_goal_stored = StatesGoalReadCmd::exec(&mut cmd_ctx).await?;
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
async fn sub_cmd_current_with_send_progress_tick_instead_of_complete()
-> Result<(), Box<dyn std::error::Error>> {
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
    let mut output = NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .with_item_params::<VecCopyItem>(
            VecCopyItem::ID_DEFAULT.clone(),
            VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
        )
        .await?;

    let _cmd_outcome = StatesDiscoverCmd::<_, NoOpOutput, _>::current_with(
        &mut CmdIndependence::SubCmd {
            cmd_view: &mut cmd_ctx.view(),
        },
        false,
    )
    .await?;

    let cmd_progress_tracker = cmd_ctx.cmd_progress_tracker();
    let progress_tracker = cmd_progress_tracker
        .progress_trackers()
        .get(VecCopyItem::ID_DEFAULT)
        .unwrap_or_else(|| {
            panic!(
                "Expected `progress_tracker` to exist for {}",
                VecCopyItem::ID_DEFAULT
            )
        });
    let progress_status = progress_tracker.progress_status();
    assert!(
        matches!(
            progress_status,
            ProgressStatus::Initialized | ProgressStatus::Running,
        ),
        "expected `progress_status` to be `Initialized` or `Pending`, but was {progress_status:?}"
    );

    Ok(())
}

#[cfg(feature = "output_progress")]
#[tokio::test]
async fn sub_cmd_goal_with_send_progress_tick_instead_of_complete()
-> Result<(), Box<dyn std::error::Error>> {
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
    let mut output = NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .with_item_params::<VecCopyItem>(
            VecCopyItem::ID_DEFAULT.clone(),
            VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
        )
        .await?;

    let _cmd_outcome = StatesDiscoverCmd::<_, NoOpOutput, _>::goal_with(
        &mut CmdIndependence::SubCmd {
            cmd_view: &mut cmd_ctx.view(),
        },
        false,
    )
    .await?;

    let cmd_progress_tracker = cmd_ctx.cmd_progress_tracker();
    let progress_tracker = cmd_progress_tracker
        .progress_trackers()
        .get(VecCopyItem::ID_DEFAULT)
        .unwrap_or_else(|| {
            panic!(
                "Expected `progress_tracker` to exist for {}",
                VecCopyItem::ID_DEFAULT
            )
        });
    let progress_status = progress_tracker.progress_status();
    assert!(
        matches!(
            progress_status,
            ProgressStatus::Initialized | ProgressStatus::Running,
        ),
        "expected `progress_status` to be `Initialized` or `Pending`, but was {progress_status:?}"
    );

    Ok(())
}

#[cfg(feature = "output_progress")]
#[tokio::test]
async fn sub_cmd_current_and_goal_with_send_progress_tick_instead_of_complete()
-> Result<(), Box<dyn std::error::Error>> {
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
    let mut output = NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .with_item_params::<VecCopyItem>(
            VecCopyItem::ID_DEFAULT.clone(),
            VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
        )
        .await?;

    let _cmd_outcome = StatesDiscoverCmd::<_, NoOpOutput, _>::current_and_goal_with(
        &mut CmdIndependence::SubCmd {
            cmd_view: &mut cmd_ctx.view(),
        },
        false,
    )
    .await?;

    let cmd_progress_tracker = cmd_ctx.cmd_progress_tracker();
    let progress_tracker = cmd_progress_tracker
        .progress_trackers()
        .get(VecCopyItem::ID_DEFAULT)
        .unwrap_or_else(|| {
            panic!(
                "Expected `progress_tracker` to exist for {}",
                VecCopyItem::ID_DEFAULT
            )
        });
    let progress_status = progress_tracker.progress_status();
    assert!(
        matches!(
            progress_status,
            ProgressStatus::Initialized | ProgressStatus::Running,
        ),
        "expected `progress_status` to be `Initialized` or `Pending`, but was {progress_status:?}"
    );

    Ok(())
}

#[test]
fn debug() {
    let debug_str = format!(
        "{:?}",
        StatesDiscoverCmd::<VecCopyError, NoOpOutput, ()>::default()
    );
    assert!(
        debug_str
            == r#"StatesDiscoverCmd(PhantomData<(workspace_tests::vec_copy_item::VecCopyError, workspace_tests::no_op_output::NoOpOutput, ())>)"#
            || debug_str == r#"StatesDiscoverCmd(PhantomData)"#
    );
}
