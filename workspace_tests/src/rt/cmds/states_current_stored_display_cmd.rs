use peace::{
    cfg::{app_name, profile},
    cmd::ctx::CmdCtx,
    cmd_model::CmdOutcome,
    flow_model::FlowId,
    flow_rt::{Flow, ItemGraphBuilder},
    rt::cmds::{StatesCurrentStoredDisplayCmd, StatesDiscoverCmd},
    rt_model::{Error, Workspace, WorkspaceSpec},
};

use crate::{
    peace_cmd_ctx_types::PeaceCmdCtxTypes, FnInvocation, FnTrackerOutput, NoOpOutput,
    PeaceTestError, VecA, VecCopyItem, VecCopyState,
};

#[tokio::test]
async fn reads_states_current_stored_from_disk_when_present(
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
    let mut fn_tracker_output = FnTrackerOutput::new();

    // Write current states to disk.
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
        value: states_current_stored_from_discover,
        cmd_blocks_processed: _,
    } = StatesDiscoverCmd::current(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesDiscoverCmd::current` to complete successfully.");
    };

    // Re-read states from disk in a new set of resources.
    let mut cmd_ctx =
        CmdCtx::builder_single_profile_single_flow::<PeaceTestError, FnTrackerOutput>(
            (&mut fn_tracker_output).into(),
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
        value: states_current_stored_from_read,
        cmd_blocks_processed: _,
    } = StatesCurrentStoredDisplayCmd::exec(&mut cmd_ctx).await?
    else {
        panic!("Expected `StatesCurrentStoredDisplayCmd::exec` to complete successfully.");
    };
    let fn_tracker_output = cmd_ctx.output();

    let vec_copy_state_from_discover =
        states_current_stored_from_discover.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    let states_current_stored_from_read = &*states_current_stored_from_read;
    let vec_copy_state_from_read =
        states_current_stored_from_read.get::<VecCopyState, _>(VecCopyItem::ID_DEFAULT);
    assert_eq!(vec_copy_state_from_discover, vec_copy_state_from_read);
    assert_eq!(
        vec![FnInvocation::new(
            "present",
            vec![Some(serde_yaml::to_string(
                states_current_stored_from_read
            )?)],
        )],
        fn_tracker_output.fn_invocations()
    );
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
    let mut fn_tracker_output = FnTrackerOutput::new();

    // Try and display states from disk.
    let mut cmd_ctx =
        CmdCtx::builder_single_profile_single_flow::<PeaceTestError, FnTrackerOutput>(
            (&mut fn_tracker_output).into(),
            (&workspace).into(),
        )
        .with_profile(profile!("test_profile"))
        .with_flow((&flow).into())
        .with_item_params::<VecCopyItem>(
            VecCopyItem::ID_DEFAULT.clone(),
            VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
        )
        .await?;
    let exec_result = StatesCurrentStoredDisplayCmd::exec(&mut cmd_ctx).await;

    assert!(matches!(
        exec_result,
        Err(PeaceTestError::PeaceRt(
            Error::StatesCurrentDiscoverRequired
        ))
    ));
    let err = PeaceTestError::PeaceRt(Error::StatesCurrentDiscoverRequired);
    assert_eq!(
        vec![FnInvocation::new(
            "write_err",
            vec![Some(format!("{err:?}"))],
        )],
        fn_tracker_output.fn_invocations()
    );
    Ok(())
}

#[test]
fn debug() {
    let debug_str = format!(
        "{:?}",
        StatesCurrentStoredDisplayCmd::<PeaceCmdCtxTypes>::default()
    );
    assert_eq!(
        r#"StatesCurrentStoredDisplayCmd(PhantomData<workspace_tests::peace_cmd_ctx_types::PeaceCmdCtxTypes>)"#,
        debug_str,
    );
}
