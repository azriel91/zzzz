use peace::{
    cfg::{app_name, profile},
    cmd::ctx::CmdCtx,
    cmd_model::CmdOutcome,
    flow_model::FlowId,
    flow_rt::{Flow, ItemGraphBuilder},
    rt::cmds::{StatesDiscoverCmd, StatesGoalReadCmd},
    rt_model::{Error, Workspace, WorkspaceSpec},
};

use crate::{
    peace_cmd_ctx_types::PeaceCmdCtxTypes, NoOpOutput, PeaceTestError, VecA, VecCopyItem,
    VecCopyState,
};

#[tokio::test]
async fn reads_states_goal_from_disk_when_present() -> Result<(), Box<dyn std::error::Error>> {
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

    // Write goal states to disk.
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
        value: states_goal_from_discover,
        cmd_blocks_processed: _,
    } = StatesDiscoverCmd::goal(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesDiscoverCmd::goal` to complete successfully.");
    };

    // Re-read states from disk.
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
        value: states_goal_from_read,
        cmd_blocks_processed: _,
    } = StatesGoalReadCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesGoalReadCmd::exec` to complete successfully.");
    };

    let vec_copy_state_from_discover =
        states_goal_from_discover.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    let vec_copy_state_from_read =
        states_goal_from_read.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    assert_eq!(vec_copy_state_from_discover, vec_copy_state_from_read);
    Ok(())
}

#[tokio::test]
async fn returns_error_when_states_not_on_disk() -> Result<(), Box<dyn std::error::Error>> {
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
    let output = NoOpOutput;

    // Try and read goal states from disk.
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(output.into(), workspace.into())
        .with_profile(profile!("test_profile"))
        .with_flow((&flow).into())
        .with_item_params::<VecCopyItem>(
            VecCopyItem::ID_DEFAULT.clone(),
            VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
        )
        .await?;
    let exec_result = StatesGoalReadCmd::exec(&mut cmd_ctx).await;

    assert!(matches!(
        exec_result,
        Err(PeaceTestError::PeaceRt(Error::StatesGoalDiscoverRequired))
    ));
    Ok(())
}

#[test]
fn debug() {
    let debug_str = format!("{:?}", StatesGoalReadCmd::<PeaceCmdCtxTypes>::default());
    assert_eq!(
        r#"StatesGoalReadCmd(PhantomData<workspace_tests::peace_cmd_ctx_types::PeaceCmdCtxTypes>)"#,
        debug_str,
    );
}
