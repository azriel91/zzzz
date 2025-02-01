use std::{io::Cursor, path::PathBuf};

use peace::{
    cfg::{app_name, ApplyCheck, Item},
    cmd::{ctx::CmdCtx, scopes::SingleProfileSingleFlowView},
    cmd_model::CmdOutcome,
    data::Data,
    flow_model::FlowId,
    flow_rt::{Flow, ItemGraph, ItemGraphBuilder},
    item_model::{item_id, ItemId},
    params::{ParamsSpec, ValueResolutionCtx, ValueResolutionMode},
    profile_model::{profile, Profile},
    resource_rt::paths::{FlowDir, ProfileDir},
    rt::cmds::{CleanCmd, DiffCmd, EnsureCmd, StatesDiscoverCmd},
    rt_model::{InMemoryTextOutput, Workspace, WorkspaceSpec},
};
use peace_items::tar_x::{
    FileMetadata, FileMetadatas, TarXData, TarXError, TarXItem, TarXParams, TarXStateDiff,
};
use pretty_assertions::assert_eq;
use tempfile::TempDir;

#[derive(Clone, Copy, Debug, PartialEq)]
struct TarXTest;

impl TarXTest {
    const ID: &'static ItemId = &item_id!("tar_x_test");
}

/// Contains two files: `a` and `sub/c`.
const TAR_X1_TAR: &[u8] = include_bytes!("tar_x_item/tar_x1.tar");
/// Time that the `a` and `sub/c` files in `tar_x_1.tar` were modified.
const TAR_X1_MTIME: u64 = 1671674955;

/// Contains two files: `b` and `sub/d`.
const TAR_X2_TAR: &[u8] = include_bytes!("tar_x_item/tar_x2.tar");
/// Time that the `b` and `sub/a` files in `tar_x.tar` were modified.
const TAR_X2_MTIME: u64 = 1671675052;

#[test]
fn clone() {
    let _item = Clone::clone(&TarXItem::<()>::new(TarXTest::ID.clone()));
}

#[tokio::test]
async fn state_current_returns_empty_file_metadatas_when_extraction_folder_not_exists(
) -> Result<(), Box<dyn std::error::Error>> {
    let flow_id = FlowId::new(crate::fn_name_short!())?;
    let TestEnv {
        tempdir: _tempdir,
        workspace,
        profile,
        graph,
        output,
        tar_path,
        dest,
    } = test_env(&flow_id, TAR_X2_TAR).await?;
    let flow = Flow::new(flow_id, graph);

    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(output.into(), workspace.into())
        .with_profile(profile.clone())
        .with_flow((&flow).into())
        .with_item_params::<TarXItem<TarXTest>>(
            TarXTest::ID.clone(),
            TarXParams::<TarXTest>::new(tar_path, dest).into(),
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
        .get::<FileMetadatas, _>(TarXTest::ID)
        .unwrap();

    assert_eq!(&FileMetadatas::default(), state_current);

    Ok(())
}

#[tokio::test]
async fn state_current_returns_file_metadatas_when_extraction_folder_contains_file(
) -> Result<(), Box<dyn std::error::Error>> {
    let flow_id = FlowId::new(crate::fn_name_short!())?;
    let TestEnv {
        tempdir: _tempdir,
        workspace,
        profile,
        graph,
        output,
        tar_path,
        dest,
    } = test_env(&flow_id, TAR_X2_TAR).await?;
    let flow = Flow::new(flow_id, graph);
    let b_path = PathBuf::from("b");
    let d_path = PathBuf::from("sub").join("d");

    // Create files in the destination.
    tokio::fs::create_dir(&dest).await?;
    tar::Archive::new(Cursor::new(TAR_X2_TAR)).unpack(&dest)?;

    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(output.into(), workspace.into())
        .with_profile(profile.clone())
        .with_flow((&flow).into())
        .with_item_params::<TarXItem<TarXTest>>(
            TarXTest::ID.clone(),
            TarXParams::<TarXTest>::new(tar_path, dest).into(),
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
        .get::<FileMetadatas, _>(TarXTest::ID)
        .unwrap();

    assert_eq!(
        &FileMetadatas::from(vec![
            FileMetadata::new(b_path, TAR_X2_MTIME),
            FileMetadata::new(d_path, TAR_X2_MTIME),
        ]),
        state_current
    );

    Ok(())
}

#[tokio::test]
async fn state_goal_returns_file_metadatas_from_tar() -> Result<(), Box<dyn std::error::Error>> {
    let flow_id = FlowId::new(crate::fn_name_short!())?;
    let TestEnv {
        tempdir: _tempdir,
        workspace,
        profile,
        graph,
        output,
        tar_path,
        dest,
    } = test_env(&flow_id, TAR_X2_TAR).await?;
    let flow = Flow::new(flow_id, graph);
    let b_path = PathBuf::from("b");
    let d_path = PathBuf::from("sub").join("d");

    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(output.into(), workspace.into())
        .with_profile(profile.clone())
        .with_flow((&flow).into())
        .with_item_params::<TarXItem<TarXTest>>(
            TarXTest::ID.clone(),
            TarXParams::<TarXTest>::new(tar_path, dest).into(),
        )
        .await?;

    let CmdOutcome::Complete {
        value: states_goal,
        cmd_blocks_processed: _,
    } = StatesDiscoverCmd::goal(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesDiscoverCmd::goal` to complete successfully.");
    };
    let state_goal = states_goal.get::<FileMetadatas, _>(TarXTest::ID).unwrap();

    assert_eq!(
        &FileMetadatas::from(vec![
            FileMetadata::new(b_path, TAR_X2_MTIME),
            FileMetadata::new(d_path, TAR_X2_MTIME),
        ]),
        state_goal
    );

    Ok(())
}

#[tokio::test]
async fn state_diff_includes_added_when_file_in_tar_is_not_in_dest(
) -> Result<(), Box<dyn std::error::Error>> {
    let flow_id = FlowId::new(crate::fn_name_short!())?;
    let TestEnv {
        tempdir: _tempdir,
        workspace,
        profile,
        graph,
        output,
        tar_path,
        dest,
    } = test_env(&flow_id, TAR_X2_TAR).await?;
    let flow = Flow::new(flow_id, graph);
    let b_path = PathBuf::from("b");
    let d_path = PathBuf::from("sub").join("d");

    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(output.into(), workspace.into())
        .with_profile(profile.clone())
        .with_flow((&flow).into())
        .with_item_params::<TarXItem<TarXTest>>(
            TarXTest::ID.clone(),
            TarXParams::<TarXTest>::new(tar_path, dest).into(),
        )
        .await?;
    StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?;

    // Diff current and goal states.
    let CmdOutcome::Complete {
        value: state_diffs,
        cmd_blocks_processed: _,
    } = DiffCmd::diff_stored(&mut cmd_ctx).await?
    else {
        panic!("Expected `DiffCmd::diff_stored` to complete successfully.");
    };
    let state_diff = state_diffs.get::<TarXStateDiff, _>(TarXTest::ID).unwrap();

    assert_eq!(
        &TarXStateDiff::ExtractionOutOfSync {
            added: FileMetadatas::from(vec![
                FileMetadata::new(b_path, TAR_X2_MTIME),
                FileMetadata::new(d_path, TAR_X2_MTIME),
            ]),
            modified: FileMetadatas::default(),
            removed: FileMetadatas::default()
        },
        state_diff
    );

    Ok(())
}

#[tokio::test]
async fn state_diff_includes_added_when_file_in_tar_is_not_in_dest_and_dest_file_name_greater(
) -> Result<(), Box<dyn std::error::Error>> {
    let flow_id = FlowId::new(crate::fn_name_short!())?;
    let TestEnv {
        tempdir: _tempdir,
        workspace,
        profile,
        graph,
        output,
        tar_path,
        dest,
    } = test_env(&flow_id, TAR_X2_TAR).await?;
    let flow = Flow::new(flow_id, graph);
    let a_path = PathBuf::from("a");
    let b_path = PathBuf::from("b");
    let c_path = PathBuf::from("sub").join("c");
    let d_path = PathBuf::from("sub").join("d");

    // Create files in the destination.
    tokio::fs::create_dir(&dest).await?;
    tar::Archive::new(Cursor::new(TAR_X1_TAR)).unpack(&dest)?;

    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(output.into(), workspace.into())
        .with_profile(profile.clone())
        .with_flow((&flow).into())
        .with_item_params::<TarXItem<TarXTest>>(
            TarXTest::ID.clone(),
            TarXParams::<TarXTest>::new(tar_path, dest).into(),
        )
        .await?;
    StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?;

    // Diff current and goal states.
    let CmdOutcome::Complete {
        value: state_diffs,
        cmd_blocks_processed: _,
    } = DiffCmd::diff_stored(&mut cmd_ctx).await?
    else {
        panic!("Expected `DiffCmd::diff_stored` to complete successfully.");
    };
    let state_diff = state_diffs.get::<TarXStateDiff, _>(TarXTest::ID).unwrap();

    assert_eq!(
        &TarXStateDiff::ExtractionOutOfSync {
            added: FileMetadatas::from(vec![
                FileMetadata::new(b_path, TAR_X2_MTIME),
                FileMetadata::new(d_path, TAR_X2_MTIME),
            ]),
            modified: FileMetadatas::default(),
            removed: FileMetadatas::from(vec![
                FileMetadata::new(a_path, TAR_X1_MTIME),
                FileMetadata::new(c_path, TAR_X1_MTIME),
            ])
        },
        state_diff
    );

    Ok(())
}

#[tokio::test]
async fn state_diff_includes_removed_when_file_in_dest_is_not_in_tar_and_tar_file_name_greater(
) -> Result<(), Box<dyn std::error::Error>> {
    let flow_id = FlowId::new(crate::fn_name_short!())?;
    let TestEnv {
        tempdir: _tempdir,
        workspace,
        profile,
        graph,
        output,
        tar_path,
        dest,
    } = test_env(&flow_id, TAR_X2_TAR).await?;
    let flow = Flow::new(flow_id, graph);
    let a_path = PathBuf::from("a");
    let c_path = PathBuf::from("sub").join("c");

    // Create files in the destination.
    tokio::fs::create_dir(&dest).await?;
    tar::Archive::new(Cursor::new(TAR_X1_TAR)).unpack(&dest)?;
    tar::Archive::new(Cursor::new(TAR_X2_TAR)).unpack(&dest)?;

    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(output.into(), workspace.into())
        .with_profile(profile.clone())
        .with_flow((&flow).into())
        .with_item_params::<TarXItem<TarXTest>>(
            TarXTest::ID.clone(),
            TarXParams::<TarXTest>::new(tar_path, dest).into(),
        )
        .await?;
    StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?;

    // Diff current and goal states.
    let CmdOutcome::Complete {
        value: state_diffs,
        cmd_blocks_processed: _,
    } = DiffCmd::diff_stored(&mut cmd_ctx).await?
    else {
        panic!("Expected `DiffCmd::diff_stored` to complete successfully.");
    };
    let state_diff = state_diffs.get::<TarXStateDiff, _>(TarXTest::ID).unwrap();

    // `b` and `d` are not included in the diff
    assert_eq!(
        &TarXStateDiff::ExtractionOutOfSync {
            added: FileMetadatas::default(),
            modified: FileMetadatas::default(),
            removed: FileMetadatas::from(vec![
                FileMetadata::new(a_path, TAR_X1_MTIME),
                FileMetadata::new(c_path, TAR_X1_MTIME),
            ])
        },
        state_diff
    );

    Ok(())
}

#[tokio::test]
async fn state_diff_includes_removed_when_file_in_dest_is_not_in_tar_and_tar_file_name_lesser(
) -> Result<(), Box<dyn std::error::Error>> {
    let flow_id = FlowId::new(crate::fn_name_short!())?;
    let TestEnv {
        tempdir: _tempdir,
        workspace,
        profile,
        graph,
        output,
        tar_path,
        dest,
    } = test_env(&flow_id, TAR_X1_TAR).await?;
    let flow = Flow::new(flow_id, graph);
    let b_path = PathBuf::from("b");
    let d_path = PathBuf::from("sub").join("d");

    // Create files in the destination.
    tokio::fs::create_dir(&dest).await?;
    tar::Archive::new(Cursor::new(TAR_X1_TAR)).unpack(&dest)?;
    tar::Archive::new(Cursor::new(TAR_X2_TAR)).unpack(&dest)?;

    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(output.into(), workspace.into())
        .with_profile(profile.clone())
        .with_flow((&flow).into())
        .with_item_params::<TarXItem<TarXTest>>(
            TarXTest::ID.clone(),
            TarXParams::<TarXTest>::new(tar_path, dest).into(),
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
    let state_diff = state_diffs.get::<TarXStateDiff, _>(TarXTest::ID).unwrap();

    // `b` and `d` are not included in the diff
    assert_eq!(
        &TarXStateDiff::ExtractionOutOfSync {
            added: FileMetadatas::default(),
            modified: FileMetadatas::default(),
            removed: FileMetadatas::from(vec![
                FileMetadata::new(b_path, TAR_X2_MTIME),
                FileMetadata::new(d_path, TAR_X2_MTIME),
            ])
        },
        state_diff
    );

    Ok(())
}

#[tokio::test]
async fn state_diff_includes_modified_when_dest_mtime_is_different(
) -> Result<(), Box<dyn std::error::Error>> {
    let flow_id = FlowId::new(crate::fn_name_short!())?;
    let TestEnv {
        tempdir: _tempdir,
        workspace,
        profile,
        graph,
        output,
        tar_path,
        dest,
    } = test_env(&flow_id, TAR_X2_TAR).await?;
    let flow = Flow::new(flow_id, graph);

    // Create files in the destination.
    let sub_path = dest.join("sub");
    tokio::fs::create_dir_all(sub_path).await?;
    tar::Archive::new(Cursor::new(TAR_X1_TAR)).unpack(&dest)?;
    tokio::fs::write(&dest.join("b"), []).await?;
    tokio::fs::write(&dest.join("sub").join("d"), []).await?;

    let a_path = PathBuf::from("a");
    let c_path = PathBuf::from("sub").join("c");
    let b_path = PathBuf::from("b");
    let d_path = PathBuf::from("sub").join("d");

    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(output.into(), workspace.into())
        .with_profile(profile.clone())
        .with_flow((&flow).into())
        .with_item_params::<TarXItem<TarXTest>>(
            TarXTest::ID.clone(),
            TarXParams::<TarXTest>::new(tar_path, dest).into(),
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
    let state_diff = state_diffs.get::<TarXStateDiff, _>(TarXTest::ID).unwrap();

    assert_eq!(
        &TarXStateDiff::ExtractionOutOfSync {
            added: FileMetadatas::default(),
            modified: FileMetadatas::from(vec![
                FileMetadata::new(b_path, TAR_X2_MTIME),
                FileMetadata::new(d_path, TAR_X2_MTIME),
            ]),
            removed: FileMetadatas::from(vec![
                FileMetadata::new(a_path, TAR_X1_MTIME),
                FileMetadata::new(c_path, TAR_X1_MTIME),
            ])
        },
        state_diff
    );

    Ok(())
}

#[tokio::test]
async fn state_diff_returns_extraction_in_sync_when_tar_and_dest_in_sync(
) -> Result<(), Box<dyn std::error::Error>> {
    let flow_id = FlowId::new(crate::fn_name_short!())?;
    let TestEnv {
        tempdir: _tempdir,
        workspace,
        profile,
        graph,
        output,
        tar_path,
        dest,
    } = test_env(&flow_id, TAR_X2_TAR).await?;
    let flow = Flow::new(flow_id, graph);

    // Create files in the destination.
    tokio::fs::create_dir(&dest).await?;
    tar::Archive::new(Cursor::new(TAR_X2_TAR)).unpack(&dest)?;

    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(output.into(), workspace.into())
        .with_profile(profile.clone())
        .with_flow((&flow).into())
        .with_item_params::<TarXItem<TarXTest>>(
            TarXTest::ID.clone(),
            TarXParams::<TarXTest>::new(tar_path, dest).into(),
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
    let state_diff = state_diffs.get::<TarXStateDiff, _>(TarXTest::ID).unwrap();

    assert_eq!(&TarXStateDiff::ExtractionInSync, state_diff);

    Ok(())
}

#[tokio::test]
async fn ensure_check_returns_exec_not_required_when_tar_and_dest_in_sync(
) -> Result<(), Box<dyn std::error::Error>> {
    let flow_id = FlowId::new(crate::fn_name_short!())?;
    let TestEnv {
        tempdir: _tempdir,
        workspace,
        profile,
        graph,
        output,
        tar_path,
        dest,
    } = test_env(&flow_id, TAR_X2_TAR).await?;
    let flow = Flow::new(flow_id, graph);

    // Create files in the destination.
    tokio::fs::create_dir(&dest).await?;
    tar::Archive::new(Cursor::new(TAR_X2_TAR)).unpack(&dest)?;

    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(output.into(), workspace.into())
        .with_profile(profile.clone())
        .with_flow((&flow).into())
        .with_item_params::<TarXItem<TarXTest>>(
            TarXTest::ID.clone(),
            TarXParams::<TarXTest>::new(tar_path, dest).into(),
        )
        .await?;
    let CmdOutcome::Complete {
        value: (states_current, states_goal),
        cmd_blocks_processed: _,
    } = StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesDiscoverCmd::current_and_goal` to complete successfully.");
    };
    let state_current = states_current
        .get::<FileMetadatas, _>(TarXTest::ID)
        .unwrap();

    let CmdOutcome::Complete {
        value: state_diffs,
        cmd_blocks_processed: _,
    } = DiffCmd::diff_stored(&mut cmd_ctx).await?
    else {
        panic!("Expected `DiffCmd::diff_stored` to complete successfully.");
    };
    let state_goal = states_goal.get::<FileMetadatas, _>(TarXTest::ID).unwrap();
    let state_diff = state_diffs.get::<TarXStateDiff, _>(TarXTest::ID).unwrap();

    let SingleProfileSingleFlowView {
        params_specs,
        resources,
        ..
    } = cmd_ctx.view();
    let tar_x_params_spec = params_specs
        .get::<ParamsSpec<TarXParams<TarXTest>>, _>(TarXTest::ID)
        .unwrap();
    let mut value_resolution_ctx = ValueResolutionCtx::new(
        ValueResolutionMode::Current,
        TarXTest::ID.clone(),
        tynm::type_name::<TarXParams<TarXTest>>(),
    );
    let tar_x_params = tar_x_params_spec
        .resolve(resources, &mut value_resolution_ctx)
        .unwrap();
    assert_eq!(
        ApplyCheck::ExecNotRequired,
        <TarXItem::<TarXTest> as Item>::apply_check(
            &tar_x_params,
            <TarXData<TarXTest> as Data>::borrow(TarXTest::ID, resources),
            state_current,
            state_goal,
            state_diff
        )
        .await?
    );

    Ok(())
}

#[tokio::test]
async fn ensure_unpacks_tar_when_files_not_exists() -> Result<(), Box<dyn std::error::Error>> {
    let flow_id = FlowId::new(crate::fn_name_short!())?;
    let TestEnv {
        tempdir: _tempdir,
        workspace,
        profile,
        graph,
        output,
        tar_path,
        dest,
    } = test_env(&flow_id, TAR_X2_TAR).await?;
    let flow = Flow::new(flow_id, graph);

    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(output.into(), workspace.into())
        .with_profile(profile.clone())
        .with_flow((&flow).into())
        .with_item_params::<TarXItem<TarXTest>>(
            TarXTest::ID.clone(),
            TarXParams::<TarXTest>::new(tar_path, dest).into(),
        )
        .await?;
    StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?;

    let CmdOutcome::Complete {
        value: states_ensured,
        cmd_blocks_processed: _,
    } = EnsureCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `EnsureCmd::exec` to complete successfully.");
    };

    let state_ensured = states_ensured
        .get::<FileMetadatas, _>(TarXTest::ID)
        .unwrap();

    let b_path = PathBuf::from("b");
    let d_path = PathBuf::from("sub").join("d");
    assert_eq!(
        &FileMetadatas::from(vec![
            FileMetadata::new(b_path, TAR_X2_MTIME),
            FileMetadata::new(d_path, TAR_X2_MTIME),
        ]),
        state_ensured
    );

    Ok(())
}

#[tokio::test]
async fn ensure_removes_other_files_and_is_idempotent() -> Result<(), Box<dyn std::error::Error>> {
    let flow_id = FlowId::new(crate::fn_name_short!())?;
    let TestEnv {
        tempdir: _tempdir,
        workspace,
        profile,
        graph,
        output,
        tar_path,
        dest,
    } = test_env(&flow_id, TAR_X2_TAR).await?;
    let flow = Flow::new(flow_id, graph);

    // Create files in the destination.
    let sub_path = dest.join("sub");
    tokio::fs::create_dir_all(sub_path).await?;
    tar::Archive::new(Cursor::new(TAR_X1_TAR)).unpack(&dest)?;
    tokio::fs::write(&dest.join("b"), []).await?;
    tokio::fs::write(&dest.join("sub").join("d"), []).await?;

    let b_path = PathBuf::from("b");
    let d_path = PathBuf::from("sub").join("d");

    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(output.into(), workspace.into())
        .with_profile(profile.clone())
        .with_flow((&flow).into())
        .with_item_params::<TarXItem<TarXTest>>(
            TarXTest::ID.clone(),
            TarXParams::<TarXTest>::new(tar_path, dest).into(),
        )
        .await?;
    StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?;

    // Overwrite changed files and remove extra files
    let CmdOutcome::Complete {
        value: states_ensured,
        cmd_blocks_processed: _,
    } = EnsureCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `EnsureCmd::exec` to complete successfully.");
    };

    let state_ensured = states_ensured
        .get::<FileMetadatas, _>(TarXTest::ID)
        .unwrap();

    assert_eq!(
        &FileMetadatas::from(vec![
            FileMetadata::new(b_path.clone(), TAR_X2_MTIME),
            FileMetadata::new(d_path.clone(), TAR_X2_MTIME),
        ]),
        state_ensured
    );

    // Execute again to check idempotence
    let CmdOutcome::Complete {
        value: states_ensured,
        cmd_blocks_processed: _,
    } = EnsureCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `EnsureCmd::exec` to complete successfully.");
    };

    let state_ensured = states_ensured
        .get::<FileMetadatas, _>(TarXTest::ID)
        .unwrap();

    assert_eq!(
        &FileMetadatas::from(vec![
            FileMetadata::new(b_path, TAR_X2_MTIME),
            FileMetadata::new(d_path, TAR_X2_MTIME),
        ]),
        state_ensured
    );

    Ok(())
}

#[tokio::test]
async fn clean_removes_files_in_dest_directory() -> Result<(), Box<dyn std::error::Error>> {
    let flow_id = FlowId::new(crate::fn_name_short!())?;
    let TestEnv {
        tempdir: _tempdir,
        workspace,
        profile,
        graph,
        output,
        tar_path,
        dest,
    } = test_env(&flow_id, TAR_X2_TAR).await?;
    let flow = Flow::new(flow_id, graph);

    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(output.into(), workspace.into())
        .with_profile(profile.clone())
        .with_flow((&flow).into())
        .with_item_params::<TarXItem<TarXTest>>(
            TarXTest::ID.clone(),
            TarXParams::<TarXTest>::new(tar_path, dest.clone()).into(),
        )
        .await?;
    StatesDiscoverCmd::current_and_goal(&mut cmd_ctx).await?;

    let CmdOutcome::Complete {
        value: states_cleaned,
        cmd_blocks_processed: _,
    } = CleanCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `CleanCmd::exec` to complete successfully.");
    };

    let state_cleaned = states_cleaned
        .get::<FileMetadatas, _>(TarXTest::ID)
        .unwrap();

    assert_eq!(&FileMetadatas::default(), state_cleaned);
    assert!(!dest.join("b").exists());
    assert!(!dest.join("sub").join("d").exists());

    Ok(())
}

async fn test_env(
    flow_id: &FlowId,
    tar_bytes: &[u8],
) -> Result<TestEnv, Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let profile = profile!("test_profile");
    let flow_dir = {
        let profile_dir = ProfileDir::from((workspace.dirs().peace_app_dir(), &profile));
        FlowDir::from((&profile_dir, flow_id))
    };
    let graph = {
        let mut graph_builder = ItemGraphBuilder::<TarXError>::new();
        graph_builder.add_fn(TarXItem::<TarXTest>::new(TarXTest::ID.clone()).into());
        graph_builder.build()
    };
    let output = InMemoryTextOutput::new();
    let tar_path = {
        let tar_path = flow_dir.join("tar_x.tar");
        tokio::fs::create_dir_all(&flow_dir).await?;
        tokio::fs::write(&tar_path, tar_bytes).await?;
        tar_path
    };
    let dest = flow_dir.join("tar_dest");

    Ok(TestEnv {
        tempdir,
        workspace,
        profile,
        graph,
        output,
        tar_path,
        dest,
    })
}

#[derive(Debug)]
struct TestEnv {
    tempdir: TempDir,
    workspace: Workspace,
    profile: Profile,
    graph: ItemGraph<TarXError>,
    output: InMemoryTextOutput,
    tar_path: PathBuf,
    dest: PathBuf,
}
