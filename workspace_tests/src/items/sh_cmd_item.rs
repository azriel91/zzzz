use peace::{
    cfg::{app_name, profile},
    cmd::ctx::CmdCtx,
    cmd_model::CmdOutcome,
    data::marker::Clean,
    flow_model::FlowId,
    flow_rt::{Flow, ItemGraphBuilder},
    item_model::{item_id, ItemId},
    rt::cmds::{CleanCmd, DiffCmd, EnsureCmd, StatesDiscoverCmd},
    rt_model::{InMemoryTextOutput, Workspace, WorkspaceSpec},
};
use peace_items::sh_cmd::{
    ShCmd, ShCmdError, ShCmdItem, ShCmdParams, ShCmdState, ShCmdStateDiff, ShCmdStateLogical,
};

/// Creates a file.
#[derive(Clone, Copy, Debug)]
pub struct TestFileCreationShCmdItem;

pub type TestFileCreationShCmdState = ShCmdState<TestFileCreationShCmdItem>;

impl TestFileCreationShCmdItem {
    /// ID
    pub const ID: ItemId = item_id!("test_file_creation");

    /// Returns a new `TestFileCreationShCmdItem`.
    pub fn new() -> ShCmdItem<Self> {
        ShCmdItem::new(Self::ID)
    }

    fn params() -> ShCmdParams<TestFileCreationShCmdItem> {
        #[cfg(unix)]
        let sh_cmd_params = {
            #[cfg(feature = "item_state_example")]
            let state_example_sh_cmd = ShCmd::new("bash").arg("-c").arg(include_str!(
                "sh_cmd_item/unix/test_file_creation_state_example.sh"
            ));
            let state_clean_sh_cmd = ShCmd::new("bash").arg("-c").arg(include_str!(
                "sh_cmd_item/unix/test_file_creation_state_clean.sh"
            ));
            let state_current_sh_cmd = ShCmd::new("bash").arg("-c").arg(include_str!(
                "sh_cmd_item/unix/test_file_creation_state_current.sh"
            ));
            let state_goal_sh_cmd = ShCmd::new("bash").arg("-c").arg(include_str!(
                "sh_cmd_item/unix/test_file_creation_state_goal.sh"
            ));
            let state_diff_sh_cmd = ShCmd::new("bash").arg("-c").arg(include_str!(
                "sh_cmd_item/unix/test_file_creation_state_diff.sh"
            ));
            let apply_check_sh_cmd = ShCmd::new("bash").arg("-c").arg(include_str!(
                "sh_cmd_item/unix/test_file_creation_apply_check.sh"
            ));
            let apply_exec_sh_cmd = ShCmd::new("bash").arg("-c").arg(include_str!(
                "sh_cmd_item/unix/test_file_creation_apply_exec.sh"
            ));
            ShCmdParams::<TestFileCreationShCmdItem>::new(
                #[cfg(feature = "item_state_example")]
                state_example_sh_cmd,
                state_clean_sh_cmd,
                state_current_sh_cmd,
                state_goal_sh_cmd,
                state_diff_sh_cmd,
                apply_check_sh_cmd,
                apply_exec_sh_cmd,
            )
        };

        #[cfg(windows)]
        let sh_cmd_params = {
            #[cfg(feature = "item_state_example")]
            let state_example_sh_cmd =
                ShCmd::new("Powershell.exe")
                    .arg("-Command")
                    .arg(include_str!(
                        "sh_cmd_item/windows/test_file_creation_state_example.ps1"
                    ));
            let state_clean_sh_cmd =
                ShCmd::new("Powershell.exe")
                    .arg("-Command")
                    .arg(include_str!(
                        "sh_cmd_item/windows/test_file_creation_state_clean.ps1"
                    ));
            let state_current_sh_cmd =
                ShCmd::new("Powershell.exe")
                    .arg("-Command")
                    .arg(include_str!(
                        "sh_cmd_item/windows/test_file_creation_state_current.ps1"
                    ));
            let state_goal_sh_cmd = ShCmd::new("Powershell.exe")
                .arg("-Command")
                .arg(include_str!(
                    "sh_cmd_item/windows/test_file_creation_state_goal.ps1"
                ));
            let state_diff_sh_cmd = ShCmd::new("Powershell.exe").arg("-Command").arg(concat!(
                "& { ",
                include_str!("sh_cmd_item/windows/test_file_creation_state_diff.ps1"),
                " }"
            ));
            let apply_check_sh_cmd = ShCmd::new("Powershell.exe").arg("-Command").arg(concat!(
                "& { ",
                include_str!("sh_cmd_item/windows/test_file_creation_apply_check.ps1"),
                " }"
            ));
            let apply_exec_sh_cmd = ShCmd::new("Powershell.exe").arg("-Command").arg(concat!(
                "& { ",
                include_str!("sh_cmd_item/windows/test_file_creation_apply_exec.ps1"),
                " }"
            ));
            ShCmdParams::<TestFileCreationShCmdItem>::new(
                #[cfg(feature = "item_state_example")]
                state_example_sh_cmd,
                state_clean_sh_cmd,
                state_current_sh_cmd,
                state_goal_sh_cmd,
                state_diff_sh_cmd,
                apply_check_sh_cmd,
                apply_exec_sh_cmd,
            )
        };

        sh_cmd_params
    }
}

#[test]
fn clone() {
    let _sh_cmd_item = Clone::clone(&TestFileCreationShCmdItem::new());
}

#[test]
fn debug() {
    let sh_cmd_item = TestFileCreationShCmdItem::new();

    assert_eq!(
        "ShCmdItem { \
        item_id: ItemId(\"test_file_creation\"), \
        marker: PhantomData<workspace_tests::items::sh_cmd_item::TestFileCreationShCmdItem> \
    }",
        format!("{sh_cmd_item:?}")
    );
}

#[tokio::test]
async fn state_clean_returns_shell_command_clean_state() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<ShCmdError>::new();
        graph_builder.add_fn(TestFileCreationShCmdItem::new().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let output = InMemoryTextOutput::new();
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(output.into(), workspace.into())
        .with_profile(profile!("test_profile"))
        .with_flow((&flow).into())
        .with_item_params::<ShCmdItem<TestFileCreationShCmdItem>>(
            TestFileCreationShCmdItem::ID,
            TestFileCreationShCmdItem::params().into(),
        )
        .await?;

    StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?;
    CleanCmd::exec_dry(&mut cmd_ctx).await?;
    let state_clean = cmd_ctx
        .resources()
        .borrow::<Clean<TestFileCreationShCmdState>>();
    let Some(state_clean) = state_clean.as_ref() else {
        panic!(
            "Expected `Clean<TestFileCreationShCmdState>` to be Some after `CleanCmd::exec_dry`."
        );
    };
    if let ShCmdStateLogical::Some {
        stdout,
        stderr,
        marker: _,
    } = &state_clean.0.logical
    {
        assert_eq!("not_exists", stdout);
        assert_eq!("`test_file` does not exist", stderr);
    } else {
        panic!("Expected `state_clean` to be `ShCmdState::Some` after `CleanCmd::exec_dry`.");
    }

    Ok(())
}

#[tokio::test]
async fn state_current_returns_shell_command_current_state(
) -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<ShCmdError>::new();
        graph_builder.add_fn(TestFileCreationShCmdItem::new().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let output = InMemoryTextOutput::new();
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(output.into(), workspace.into())
        .with_profile(profile!("test_profile"))
        .with_flow((&flow).into())
        .with_item_params::<ShCmdItem<TestFileCreationShCmdItem>>(
            TestFileCreationShCmdItem::ID,
            TestFileCreationShCmdItem::params().into(),
        )
        .await?;

    let CmdOutcome::Complete {
        value: states_current,
        cmd_blocks_processed: _,
    } = StatesDiscoverCmd::current(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesDiscoverCmd::current` to complete successfully.");
    };
    let state_current = states_current
        .get::<TestFileCreationShCmdState, _>(&TestFileCreationShCmdItem::ID)
        .unwrap();
    if let ShCmdStateLogical::Some {
        stdout,
        stderr,
        marker: _,
    } = &state_current.0.logical
    {
        assert_eq!("not_exists", stdout);
        assert_eq!("`test_file` does not exist", stderr);
    } else {
        panic!(
            "Expected `state_current` to be `ShCmdState::Some` after `StatesCurrent` discovery."
        );
    }

    Ok(())
}

#[tokio::test]
async fn state_goal_returns_shell_command_goal_state() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<ShCmdError>::new();
        graph_builder.add_fn(TestFileCreationShCmdItem::new().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let output = InMemoryTextOutput::new();
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(output.into(), workspace.into())
        .with_profile(profile!("test_profile"))
        .with_flow((&flow).into())
        .with_item_params::<ShCmdItem<TestFileCreationShCmdItem>>(
            TestFileCreationShCmdItem::ID,
            TestFileCreationShCmdItem::params().into(),
        )
        .await?;

    let CmdOutcome::Complete {
        value: states_goal,
        cmd_blocks_processed: _,
    } = StatesDiscoverCmd::goal(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesDiscoverCmd::goal` to complete successfully.");
    };
    let state_goal = states_goal
        .get::<ShCmdState<TestFileCreationShCmdItem>, _>(&TestFileCreationShCmdItem::ID)
        .unwrap();
    if let ShCmdStateLogical::Some {
        stdout,
        stderr,
        marker: _,
    } = &state_goal.0.logical
    {
        assert_eq!("exists", stdout);
        assert_eq!("`test_file` exists", stderr);
    } else {
        panic!("Expected `state_goal` to be `ShCmdState::Some` after `StatesGoal` discovery.");
    }

    Ok(())
}

#[tokio::test]
async fn state_diff_returns_shell_command_state_diff() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<ShCmdError>::new();
        graph_builder.add_fn(TestFileCreationShCmdItem::new().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let output = InMemoryTextOutput::new();
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(output.into(), workspace.into())
        .with_profile(profile!("test_profile"))
        .with_flow((&flow).into())
        .with_item_params::<ShCmdItem<TestFileCreationShCmdItem>>(
            TestFileCreationShCmdItem::ID,
            TestFileCreationShCmdItem::params().into(),
        )
        .await?;

    // Discover current and goal states.
    StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?;

    // Diff current and goal states.
    let CmdOutcome::Complete {
        value: state_diffs,
        cmd_blocks_processed: _,
    } = DiffCmd::diff_stored(&mut cmd_ctx).await?
    else {
        panic!("Expected `DiffCmd::diff_stored` to complete successfully.");
    };

    let state_diff = state_diffs
        .get::<ShCmdStateDiff, _>(&TestFileCreationShCmdItem::ID)
        .unwrap();
    assert_eq!("creation_required", state_diff.stdout());
    assert_eq!("`test_file` will be created", state_diff.stderr());

    Ok(())
}

#[tokio::test]
async fn ensure_when_creation_required_executes_apply_exec_shell_command(
) -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<ShCmdError>::new();
        graph_builder.add_fn(TestFileCreationShCmdItem::new().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let output = InMemoryTextOutput::new();
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(output.into(), workspace.into())
        .with_profile(profile!("test_profile"))
        .with_flow((&flow).into())
        .with_item_params::<ShCmdItem<TestFileCreationShCmdItem>>(
            TestFileCreationShCmdItem::ID,
            TestFileCreationShCmdItem::params().into(),
        )
        .await?;

    // Discover states current and goal
    StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?;

    // Create the file
    let CmdOutcome::Complete {
        value: states_ensured,
        cmd_blocks_processed: _,
    } = EnsureCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `EnsureCmd::exec` to complete successfully.");
    };

    let state_ensured = states_ensured
        .get::<TestFileCreationShCmdState, _>(&TestFileCreationShCmdItem::ID)
        .unwrap();
    if let ShCmdStateLogical::Some {
        stdout,
        stderr,
        marker: _,
    } = &state_ensured.0.logical
    {
        assert_eq!("exists", stdout);
        assert_eq!("`test_file` exists", stderr);
    } else {
        panic!("Expected `state_ensured` to be `ShCmdState::Some` after `EnsureCmd` execution.");
    }

    Ok(())
}

#[tokio::test]
async fn ensure_when_exists_sync_does_not_reexecute_apply_exec_shell_command(
) -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<ShCmdError>::new();
        graph_builder.add_fn(TestFileCreationShCmdItem::new().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let output = InMemoryTextOutput::new();
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(output.into(), workspace.into())
        .with_profile(profile!("test_profile"))
        .with_flow((&flow).into())
        .with_item_params::<ShCmdItem<TestFileCreationShCmdItem>>(
            TestFileCreationShCmdItem::ID,
            TestFileCreationShCmdItem::params().into(),
        )
        .await?;

    // Discover states current and goal
    StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?;

    // Create the file
    EnsureCmd::exec(&mut cmd_ctx).await?;

    // Diff state after creation
    let CmdOutcome::Complete {
        value: state_diffs,
        cmd_blocks_processed: _,
    } = DiffCmd::diff_stored(&mut cmd_ctx).await?
    else {
        panic!("Expected `DiffCmd::diff_stored` to complete successfully.");
    };

    let state_diff = state_diffs
        .get::<ShCmdStateDiff, _>(&TestFileCreationShCmdItem::ID)
        .unwrap();
    assert_eq!("exists_sync", state_diff.stdout());
    assert_eq!("nothing to do", state_diff.stderr());

    // Run again, for idempotence check
    let CmdOutcome::Complete {
        value: states_ensured,
        cmd_blocks_processed: _,
    } = EnsureCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `EnsureCmd::exec` to complete successfully.");
    };

    let state_ensured = states_ensured
        .get::<TestFileCreationShCmdState, _>(&TestFileCreationShCmdItem::ID)
        .unwrap();
    if let ShCmdStateLogical::Some {
        stdout,
        stderr,
        marker: _,
    } = &state_ensured.0.logical
    {
        assert_eq!("exists", stdout);
        assert_eq!("`test_file` exists", stderr);
    } else {
        panic!("Expected `state_ensured` to be `ShCmdState::Some` after `EnsureCmd` execution.");
    }

    Ok(())
}

#[tokio::test]
async fn clean_when_exists_sync_executes_shell_command() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<ShCmdError>::new();
        graph_builder.add_fn(TestFileCreationShCmdItem::new().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let output = InMemoryTextOutput::new();
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(output.into(), workspace.into())
        .with_profile(profile!("test_profile"))
        .with_flow((&flow).into())
        .with_item_params::<ShCmdItem<TestFileCreationShCmdItem>>(
            TestFileCreationShCmdItem::ID,
            TestFileCreationShCmdItem::params().into(),
        )
        .await?;

    // Discover states current and goal
    StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?;

    // Create the file
    EnsureCmd::exec(&mut cmd_ctx).await?;

    assert!(tempdir.path().join("test_file").exists());

    // Clean the file
    CleanCmd::exec(&mut cmd_ctx).await?;

    assert!(!tempdir.path().join("test_file").exists());

    // Run again, for idempotence check
    let CmdOutcome::Complete {
        value: states_cleaned,
        cmd_blocks_processed: _,
    } = CleanCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `CleanCmd::exec` to complete successfully.");
    };

    let state_cleaned = states_cleaned
        .get::<TestFileCreationShCmdState, _>(&TestFileCreationShCmdItem::ID)
        .unwrap();
    if let ShCmdStateLogical::Some {
        stdout,
        stderr,
        marker: _,
    } = &state_cleaned.0.logical
    {
        assert_eq!("not_exists", stdout);
        assert_eq!("`test_file` does not exist", stderr);
    } else {
        panic!("Expected `state_cleaned` to be `ShCmdState::Some` after `CleanCmd` execution.");
    }

    Ok(())
}
