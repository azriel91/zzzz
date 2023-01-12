use std::{io::Cursor, path::PathBuf};

use peace::{
    cfg::{
        item_spec_id, profile,
        state::{External, Nothing},
        CleanOpSpec, EnsureOpSpec, FlowId, ItemSpecId, OpCheckStatus, Profile, State,
    },
    data::Data,
    resources::states::{StateDiffs, StatesCleaned, StatesCurrent, StatesDesired, StatesEnsured},
    rt::cmds::{
        sub::{StatesCurrentDiscoverCmd, StatesDesiredDiscoverCmd},
        CleanCmd, DiffCmd, EnsureCmd, StatesDiscoverCmd,
    },
    rt_model::{
        CmdContext, InMemoryTextOutput, ItemSpecGraph, ItemSpecGraphBuilder, Workspace,
        WorkspaceSpec,
    },
};
use peace_item_specs::tar_x::{
    FileMetadata, FileMetadatas, TarXCleanOpSpec, TarXData, TarXEnsureOpSpec, TarXError,
    TarXItemSpec, TarXParams, TarXStateDiff,
};
use pretty_assertions::assert_eq;
use tempfile::TempDir;

#[derive(Clone, Copy, Debug, PartialEq)]
struct TarXTest;

impl TarXTest {
    const ID: ItemSpecId = item_spec_id!("tar_x_test");
}

/// Contains two files: `a` and `sub/c`.
const TAR_X1_TAR: &[u8] = include_bytes!("tar_x_item_spec/tar_x1.tar");
/// Time that the `a` and `sub/c` files in `tar_x_1.tar` were modified.
const TAR_X1_MTIME: u64 = 1671674955;

/// Contains two files: `b` and `sub/d`.
const TAR_X2_TAR: &[u8] = include_bytes!("tar_x_item_spec/tar_x2.tar");
/// Time that the `b` and `sub/a` files in `tar_x.tar` were modified.
const TAR_X2_MTIME: u64 = 1671675052;

#[tokio::test]
async fn state_current_returns_empty_file_metadatas_when_extraction_folder_not_exists()
-> Result<(), Box<dyn std::error::Error>> {
    let TestEnv {
        tempdir: _tempdir,
        workspace,
        graph,
        mut output,
        tar_path,
        dest,
    } = test_env(TAR_X2_TAR).await?;

    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_flow_param(
            "param".to_string(),
            Some(TarXParams::<TarXTest>::new(tar_path, dest)),
        )
        .await?;

    let CmdContext { resources, .. } = StatesCurrentDiscoverCmd::exec(cmd_context).await?;
    let states_current = resources.borrow::<StatesCurrent>();
    let state_current = states_current
        .get::<State<FileMetadatas, Nothing>, _>(&TarXTest::ID)
        .unwrap();

    assert_eq!(FileMetadatas::default(), state_current.logical);

    Ok(())
}

#[tokio::test]
async fn state_current_returns_file_metadatas_when_extraction_folder_contains_file()
-> Result<(), Box<dyn std::error::Error>> {
    let TestEnv {
        tempdir: _tempdir,
        workspace,
        graph,
        mut output,
        tar_path,
        dest,
    } = test_env(TAR_X2_TAR).await?;
    let b_path = PathBuf::from("b");
    let d_path = PathBuf::from("sub").join("d");

    // Create files in the destination.
    tokio::fs::create_dir(&dest).await?;
    tar::Archive::new(Cursor::new(TAR_X2_TAR)).unpack(&dest)?;

    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_flow_param(
            "param".to_string(),
            Some(TarXParams::<TarXTest>::new(tar_path, dest)),
        )
        .await?;

    let CmdContext { resources, .. } = StatesCurrentDiscoverCmd::exec(cmd_context).await?;
    let states_current = resources.borrow::<StatesCurrent>();
    let state_current = states_current
        .get::<State<FileMetadatas, Nothing>, _>(&TarXTest::ID)
        .unwrap();

    assert_eq!(
        FileMetadatas::from(vec![
            FileMetadata::new(b_path, TAR_X2_MTIME),
            FileMetadata::new(d_path, TAR_X2_MTIME),
        ]),
        state_current.logical
    );

    Ok(())
}

#[tokio::test]
async fn state_desired_returns_file_metadatas_from_tar() -> Result<(), Box<dyn std::error::Error>> {
    let TestEnv {
        tempdir: _tempdir,
        workspace,
        graph,
        mut output,
        tar_path,
        dest,
    } = test_env(TAR_X2_TAR).await?;
    let b_path = PathBuf::from("b");
    let d_path = PathBuf::from("sub").join("d");

    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_flow_param(
            "param".to_string(),
            Some(TarXParams::<TarXTest>::new(tar_path, dest)),
        )
        .await?;

    let CmdContext { resources, .. } = StatesDesiredDiscoverCmd::exec(cmd_context).await?;
    let states_desired = resources.borrow::<StatesDesired>();
    let state_desired = states_desired
        .get::<State<FileMetadatas, External>, _>(&TarXTest::ID)
        .unwrap();

    assert_eq!(
        FileMetadatas::from(vec![
            FileMetadata::new(b_path, TAR_X2_MTIME),
            FileMetadata::new(d_path, TAR_X2_MTIME),
        ]),
        state_desired.logical
    );

    Ok(())
}

#[tokio::test]
async fn state_diff_includes_added_when_file_in_tar_is_not_in_dest()
-> Result<(), Box<dyn std::error::Error>> {
    let TestEnv {
        tempdir: _tempdir,
        workspace,
        graph,
        mut output,
        tar_path,
        dest,
    } = test_env(TAR_X2_TAR).await?;
    let b_path = PathBuf::from("b");
    let d_path = PathBuf::from("sub").join("d");

    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_flow_param(
            "param".to_string(),
            Some(TarXParams::<TarXTest>::new(tar_path, dest)),
        )
        .await?;
    StatesDiscoverCmd::exec(cmd_context).await?;

    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_flow_param("param".to_string(), None::<TarXParams<TarXTest>>)
        .await?;
    let CmdContext { resources, .. } = DiffCmd::exec(cmd_context).await?;
    let state_diffs = resources.borrow::<StateDiffs>();
    let state_diff = state_diffs.get::<TarXStateDiff, _>(&TarXTest::ID).unwrap();

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
async fn state_diff_includes_added_when_file_in_tar_is_not_in_dest_and_dest_file_name_greater()
-> Result<(), Box<dyn std::error::Error>> {
    let TestEnv {
        tempdir: _tempdir,
        workspace,
        graph,
        mut output,
        tar_path,
        dest,
    } = test_env(TAR_X2_TAR).await?;
    let a_path = PathBuf::from("a");
    let b_path = PathBuf::from("b");
    let c_path = PathBuf::from("sub").join("c");
    let d_path = PathBuf::from("sub").join("d");

    // Create files in the destination.
    tokio::fs::create_dir(&dest).await?;
    tar::Archive::new(Cursor::new(TAR_X1_TAR)).unpack(&dest)?;

    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_flow_param(
            "param".to_string(),
            Some(TarXParams::<TarXTest>::new(tar_path, dest)),
        )
        .await?;
    StatesDiscoverCmd::exec(cmd_context).await?;

    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_flow_param("param".to_string(), None::<TarXParams<TarXTest>>)
        .await?;
    let CmdContext { resources, .. } = DiffCmd::exec(cmd_context).await?;
    let state_diffs = resources.borrow::<StateDiffs>();
    let state_diff = state_diffs.get::<TarXStateDiff, _>(&TarXTest::ID).unwrap();

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
async fn state_diff_includes_removed_when_file_in_dest_is_not_in_tar_and_tar_file_name_greater()
-> Result<(), Box<dyn std::error::Error>> {
    let TestEnv {
        tempdir: _tempdir,
        workspace,
        graph,
        mut output,
        tar_path,
        dest,
    } = test_env(TAR_X2_TAR).await?;
    let a_path = PathBuf::from("a");
    let c_path = PathBuf::from("sub").join("c");

    // Create files in the destination.
    tokio::fs::create_dir(&dest).await?;
    tar::Archive::new(Cursor::new(TAR_X1_TAR)).unpack(&dest)?;
    tar::Archive::new(Cursor::new(TAR_X2_TAR)).unpack(&dest)?;

    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_flow_param(
            "param".to_string(),
            Some(TarXParams::<TarXTest>::new(tar_path, dest)),
        )
        .await?;
    StatesDiscoverCmd::exec(cmd_context).await?;

    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_flow_param("param".to_string(), None::<TarXParams<TarXTest>>)
        .await?;
    let CmdContext { resources, .. } = DiffCmd::exec(cmd_context).await?;
    let state_diffs = resources.borrow::<StateDiffs>();
    let state_diff = state_diffs.get::<TarXStateDiff, _>(&TarXTest::ID).unwrap();

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
async fn state_diff_includes_removed_when_file_in_dest_is_not_in_tar_and_tar_file_name_lesser()
-> Result<(), Box<dyn std::error::Error>> {
    let TestEnv {
        tempdir: _tempdir,
        workspace,
        graph,
        mut output,
        tar_path,
        dest,
    } = test_env(TAR_X1_TAR).await?;
    let b_path = PathBuf::from("b");
    let d_path = PathBuf::from("sub").join("d");

    // Create files in the destination.
    tokio::fs::create_dir(&dest).await?;
    tar::Archive::new(Cursor::new(TAR_X1_TAR)).unpack(&dest)?;
    tar::Archive::new(Cursor::new(TAR_X2_TAR)).unpack(&dest)?;

    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_flow_param(
            "param".to_string(),
            Some(TarXParams::<TarXTest>::new(tar_path, dest)),
        )
        .await?;
    StatesDiscoverCmd::exec(cmd_context).await?;

    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_flow_param("param".to_string(), None::<TarXParams<TarXTest>>)
        .await?;
    let CmdContext { resources, .. } = DiffCmd::exec(cmd_context).await?;
    let state_diffs = resources.borrow::<StateDiffs>();
    let state_diff = state_diffs.get::<TarXStateDiff, _>(&TarXTest::ID).unwrap();

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
async fn state_diff_includes_modified_when_dest_mtime_is_different()
-> Result<(), Box<dyn std::error::Error>> {
    let TestEnv {
        tempdir: _tempdir,
        workspace,
        graph,
        mut output,
        tar_path,
        dest,
    } = test_env(TAR_X2_TAR).await?;

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

    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_flow_param(
            "param".to_string(),
            Some(TarXParams::<TarXTest>::new(tar_path, dest)),
        )
        .await?;
    StatesDiscoverCmd::exec(cmd_context).await?;

    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_flow_param("param".to_string(), None::<TarXParams<TarXTest>>)
        .await?;
    let CmdContext { resources, .. } = DiffCmd::exec(cmd_context).await?;
    let state_diffs = resources.borrow::<StateDiffs>();
    let state_diff = state_diffs.get::<TarXStateDiff, _>(&TarXTest::ID).unwrap();

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
async fn state_diff_returns_extraction_in_sync_when_tar_and_dest_in_sync()
-> Result<(), Box<dyn std::error::Error>> {
    let TestEnv {
        tempdir: _tempdir,
        workspace,
        graph,
        mut output,
        tar_path,
        dest,
    } = test_env(TAR_X2_TAR).await?;

    // Create files in the destination.
    tokio::fs::create_dir(&dest).await?;
    tar::Archive::new(Cursor::new(TAR_X2_TAR)).unpack(&dest)?;

    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_flow_param(
            "param".to_string(),
            Some(TarXParams::<TarXTest>::new(tar_path, dest)),
        )
        .await?;
    StatesDiscoverCmd::exec(cmd_context).await?;

    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_flow_param("param".to_string(), None::<TarXParams<TarXTest>>)
        .await?;
    let CmdContext { resources, .. } = DiffCmd::exec(cmd_context).await?;
    let state_diffs = resources.borrow::<StateDiffs>();
    let state_diff = state_diffs.get::<TarXStateDiff, _>(&TarXTest::ID).unwrap();

    assert_eq!(&TarXStateDiff::ExtractionInSync, state_diff);

    Ok(())
}

#[tokio::test]
async fn ensure_check_returns_exec_not_required_when_tar_and_dest_in_sync()
-> Result<(), Box<dyn std::error::Error>> {
    let TestEnv {
        tempdir: _tempdir,
        workspace,
        graph,
        mut output,
        tar_path,
        dest,
    } = test_env(TAR_X2_TAR).await?;

    // Create files in the destination.
    tokio::fs::create_dir(&dest).await?;
    tar::Archive::new(Cursor::new(TAR_X2_TAR)).unpack(&dest)?;

    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_flow_param(
            "param".to_string(),
            Some(TarXParams::<TarXTest>::new(tar_path, dest)),
        )
        .await?;
    let CmdContext { resources, .. } = StatesDiscoverCmd::exec(cmd_context).await?;
    let states_current = resources.borrow::<StatesCurrent>();
    let state_current = states_current
        .get::<State<FileMetadatas, Nothing>, _>(&TarXTest::ID)
        .unwrap();

    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_flow_param("param".to_string(), None::<TarXParams<TarXTest>>)
        .await?;
    let CmdContext { resources, .. } = DiffCmd::exec(cmd_context).await?;
    let states_desired = resources.borrow::<StatesDesired>();
    let state_desired = states_desired
        .get::<State<FileMetadatas, External>, _>(&TarXTest::ID)
        .unwrap();
    let state_diffs = resources.borrow::<StateDiffs>();
    let state_diff = state_diffs.get::<TarXStateDiff, _>(&TarXTest::ID).unwrap();

    assert_eq!(
        OpCheckStatus::ExecNotRequired,
        <TarXEnsureOpSpec::<TarXTest> as EnsureOpSpec>::check(
            <TarXData<TarXTest> as Data>::borrow(&resources),
            state_current,
            &state_desired.logical,
            state_diff
        )
        .await?
    );

    Ok(())
}

#[tokio::test]
async fn ensure_unpacks_tar_when_files_not_exists() -> Result<(), Box<dyn std::error::Error>> {
    let TestEnv {
        tempdir: _tempdir,
        workspace,
        graph,
        mut output,
        tar_path,
        dest,
    } = test_env(TAR_X2_TAR).await?;

    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_flow_param(
            "param".to_string(),
            Some(TarXParams::<TarXTest>::new(tar_path, dest)),
        )
        .await?;
    StatesDiscoverCmd::exec(cmd_context).await?;

    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_flow_param("param".to_string(), None::<TarXParams<TarXTest>>)
        .await?;
    let CmdContext { resources, .. } = EnsureCmd::exec(cmd_context).await?;

    let states_ensured = resources.borrow::<StatesEnsured>();
    let state_ensured = states_ensured
        .get::<State<FileMetadatas, Nothing>, _>(&TarXTest::ID)
        .unwrap();

    let b_path = PathBuf::from("b");
    let d_path = PathBuf::from("sub").join("d");
    assert_eq!(
        FileMetadatas::from(vec![
            FileMetadata::new(b_path, TAR_X2_MTIME),
            FileMetadata::new(d_path, TAR_X2_MTIME),
        ]),
        state_ensured.logical
    );

    Ok(())
}

#[tokio::test]
async fn ensure_removes_other_files_and_is_idempotent() -> Result<(), Box<dyn std::error::Error>> {
    let TestEnv {
        tempdir: _tempdir,
        workspace,
        graph,
        mut output,
        tar_path,
        dest,
    } = test_env(TAR_X2_TAR).await?;

    // Create files in the destination.
    let sub_path = dest.join("sub");
    tokio::fs::create_dir_all(sub_path).await?;
    tar::Archive::new(Cursor::new(TAR_X1_TAR)).unpack(&dest)?;
    tokio::fs::write(&dest.join("b"), []).await?;
    tokio::fs::write(&dest.join("sub").join("d"), []).await?;

    let b_path = PathBuf::from("b");
    let d_path = PathBuf::from("sub").join("d");

    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_flow_param(
            "param".to_string(),
            Some(TarXParams::<TarXTest>::new(tar_path, dest)),
        )
        .await?;
    StatesDiscoverCmd::exec(cmd_context).await?;

    // Overwrite changed files and remove extra files
    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_flow_param("param".to_string(), None::<TarXParams<TarXTest>>)
        .await?;
    let CmdContext { resources, .. } = EnsureCmd::exec(cmd_context).await?;

    let states_ensured = resources.borrow::<StatesEnsured>();
    let state_ensured = states_ensured
        .get::<State<FileMetadatas, Nothing>, _>(&TarXTest::ID)
        .unwrap();

    assert_eq!(
        FileMetadatas::from(vec![
            FileMetadata::new(b_path.clone(), TAR_X2_MTIME),
            FileMetadata::new(d_path.clone(), TAR_X2_MTIME),
        ]),
        state_ensured.logical
    );

    // Execute again to check idempotence
    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_flow_param("param".to_string(), None::<TarXParams<TarXTest>>)
        .await?;
    let CmdContext { resources, .. } = EnsureCmd::exec(cmd_context).await?;

    let states_ensured = resources.borrow::<StatesEnsured>();
    let state_ensured = states_ensured
        .get::<State<FileMetadatas, Nothing>, _>(&TarXTest::ID)
        .unwrap();

    assert_eq!(
        FileMetadatas::from(vec![
            FileMetadata::new(b_path, TAR_X2_MTIME),
            FileMetadata::new(d_path, TAR_X2_MTIME),
        ]),
        state_ensured.logical
    );

    Ok(())
}

#[tokio::test]
async fn clean_check_returns_exec_not_required_when_dest_empty()
-> Result<(), Box<dyn std::error::Error>> {
    let TestEnv {
        tempdir: _tempdir,
        workspace,
        graph,
        mut output,
        tar_path,
        dest,
    } = test_env(TAR_X2_TAR).await?;

    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_flow_param(
            "param".to_string(),
            Some(TarXParams::<TarXTest>::new(tar_path, dest)),
        )
        .await?;
    let CmdContext { resources, .. } = StatesDiscoverCmd::exec(cmd_context).await?;
    let states_current = resources.borrow::<StatesCurrent>();
    let state_current = states_current
        .get::<State<FileMetadatas, Nothing>, _>(&TarXTest::ID)
        .unwrap();

    assert_eq!(
        OpCheckStatus::ExecNotRequired,
        <TarXCleanOpSpec::<TarXTest> as CleanOpSpec>::check(
            <TarXData<TarXTest> as Data>::borrow(&resources),
            state_current,
        )
        .await?
    );

    Ok(())
}

#[tokio::test]
async fn clean_removes_files_in_dest_directory() -> Result<(), Box<dyn std::error::Error>> {
    let TestEnv {
        tempdir: _tempdir,
        workspace,
        graph,
        mut output,
        tar_path,
        dest,
    } = test_env(TAR_X2_TAR).await?;

    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_flow_param(
            "param".to_string(),
            Some(TarXParams::<TarXTest>::new(tar_path, dest.clone())),
        )
        .await?;
    StatesDiscoverCmd::exec(cmd_context).await?;

    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_flow_param("param".to_string(), None::<TarXParams<TarXTest>>)
        .await?;
    let CmdContext { resources, .. } = CleanCmd::exec(cmd_context).await?;

    let states_cleaned = resources.borrow::<StatesCleaned>();
    let state_cleaned = states_cleaned
        .get::<State<FileMetadatas, Nothing>, _>(&TarXTest::ID)
        .unwrap();

    assert_eq!(FileMetadatas::default(), state_cleaned.logical);
    assert!(!dest.join("b").exists());
    assert!(!dest.join("sub").join("d").exists());

    Ok(())
}

async fn test_env(tar_bytes: &[u8]) -> Result<TestEnv, Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
        profile!("test_profile"),
        FlowId::new(crate::fn_name_short!())?,
    )?;
    let flow_dir = workspace.dirs().flow_dir();
    let graph = {
        let mut graph_builder = ItemSpecGraphBuilder::<TarXError>::new();
        graph_builder.add_fn(TarXItemSpec::<TarXTest>::new(TarXTest::ID).into());
        graph_builder.build()
    };
    let output = InMemoryTextOutput::new();
    let tar_path = {
        let tar_path = flow_dir.join("tar_x.tar");
        tokio::fs::create_dir_all(flow_dir).await?;
        tokio::fs::write(&tar_path, tar_bytes).await?;
        tar_path
    };
    let dest = flow_dir.join("tar_dest");

    Ok(TestEnv {
        tempdir,
        workspace,
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
    graph: ItemSpecGraph<TarXError>,
    output: InMemoryTextOutput,
    tar_path: PathBuf,
    dest: PathBuf,
}
