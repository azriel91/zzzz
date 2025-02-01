use peace::{
    cfg::{app_name, profile},
    cmd::ctx::CmdCtx,
    cmd_model::CmdExecutionError,
    cmd_rt::{CmdBlockWrapper, CmdExecution},
    flow_model::FlowId,
    flow_rt::{Flow, ItemGraphBuilder},
    resource_rt::states::{
        ts::{Current, Goal},
        StateDiffs,
    },
    rt::cmd_blocks::{DiffCmdBlock, StatesDiscoverCmdBlock},
    rt_model::{self, Workspace, WorkspaceSpec},
};
use tempfile::TempDir;

use crate::{
    mock_item::{MockItem, MockSrc},
    no_op_output::NoOpOutput,
    peace_test_error::PeaceTestError,
    VecA, VecCopyItem,
};

#[tokio::test]
async fn builds_error_for_missing_input_tuple_first_parameter() -> Result<(), PeaceTestError> {
    let mut cmd_execution = CmdExecution::<StateDiffs, _>::builder()
        .with_cmd_block(CmdBlockWrapper::new(
            // Note: deliberately don't discover `Current`, so error will occur in `DiffCmdBlock`.
            StatesDiscoverCmdBlock::goal(),
            // Should we support diffing the accumulated states?
            // Requires passing through `cmd_view` to here.
            |_states_current_and_goal_mut| StateDiffs::new(),
        ))
        .with_cmd_block(CmdBlockWrapper::new(
            DiffCmdBlock::<_, Current, Goal>::new(),
            |_state_diffs_ts0_and_ts1| StateDiffs::new(),
        ))
        .build();

    let TestCtx {
        tempdir: _tempdir,
        workspace,
        flow,
    } = test_ctx_init().await?;

    let output = NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(output.into(), workspace.into())
        .with_profile(profile!("test_profile"))
        .with_flow((&flow).into())
        .with_item_params::<VecCopyItem>(
            VecCopyItem::ID_DEFAULT.clone(),
            VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
        )
        .with_item_params::<MockItem<()>>(MockItem::<()>::ID_DEFAULT.clone(), MockSrc(1).into())
        .await?;

    let error = cmd_execution.exec(&mut cmd_ctx).await.unwrap_err();

    match error {
        PeaceTestError::PeaceRt(rt_model::Error::CmdExecution(CmdExecutionError::InputFetch {
            cmd_block_descs,
            cmd_block_index,
            input_name_short,
            input_name_full,
            #[cfg(feature = "error_reporting")]
            cmd_execution_src,
            #[cfg(feature = "error_reporting")]
            input_span,
            #[cfg(feature = "error_reporting")]
            full_span,
        })) => {
            assert_eq!(2, cmd_block_descs.len());
            assert_eq!(1, cmd_block_index);
            assert_eq!("States<Current>", input_name_short);
            assert_eq!(
                "peace_resource_rt::states::States<peace_resource_rt::states::ts::Current>",
                input_name_full
            );
            #[cfg(feature = "error_reporting")]
            {
                assert_eq!(
                    r#"CmdExecution:
  ExecutionOutcome: StateDiffs
CmdBlocks:
  - StatesDiscoverCmdBlock:
    Input: ()
    Outcome: States<Goal>
  - DiffCmdBlock:
    Input: (States<Current>, States<Goal>)
    Outcome: (StateDiffs, States<Current>, States<Goal>)
"#,
                    cmd_execution_src
                );
                match input_span {
                    Some(input_span) => {
                        assert_eq!(154, input_span.offset());
                        assert_eq!("States<Current>".len(), input_span.len());
                    }
                    None => panic!(
                        "Expected `input_span` to be `Some(SourceSpan::from((154, 15)))`, but was `None`."
                    ),
                }
                assert_eq!(0, full_span.offset());
                assert_eq!(cmd_execution_src.len(), full_span.len());
            }
        }
        _ => panic!("Expected `error` to be a `CmdExecutionError`, but was `{error}`"),
    }

    Ok(())
}

#[tokio::test]
async fn builds_error_for_missing_input_tuple_second_parameter() -> Result<(), PeaceTestError> {
    let mut cmd_execution = CmdExecution::<StateDiffs, _>::builder()
        .with_cmd_block(CmdBlockWrapper::new(
            // Note: deliberately don't discover `Goal`, so error will occur in `DiffCmdBlock`.
            StatesDiscoverCmdBlock::current(),
            // Should we support diffing the accumulated states?
            // Requires passing through `cmd_view` to here.
            |_states_current_and_goal_mut| StateDiffs::new(),
        ))
        .with_cmd_block(CmdBlockWrapper::new(
            DiffCmdBlock::<_, Current, Goal>::new(),
            |_state_diffs_ts0_and_ts1| StateDiffs::new(),
        ))
        .build();

    let TestCtx {
        tempdir: _tempdir,
        workspace,
        flow,
    } = test_ctx_init().await?;

    let output = NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(output.into(), workspace.into())
        .with_profile(profile!("test_profile"))
        .with_flow((&flow).into())
        .with_item_params::<VecCopyItem>(
            VecCopyItem::ID_DEFAULT.clone(),
            VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]).into(),
        )
        .with_item_params::<MockItem<()>>(MockItem::<()>::ID_DEFAULT.clone(), MockSrc(1).into())
        .await?;

    let error = cmd_execution.exec(&mut cmd_ctx).await.unwrap_err();

    match error {
        PeaceTestError::PeaceRt(rt_model::Error::CmdExecution(CmdExecutionError::InputFetch {
            cmd_block_descs,
            cmd_block_index,
            input_name_short,
            input_name_full,
            #[cfg(feature = "error_reporting")]
            cmd_execution_src,
            #[cfg(feature = "error_reporting")]
            input_span,
            #[cfg(feature = "error_reporting")]
            full_span,
        })) => {
            assert_eq!(2, cmd_block_descs.len());
            assert_eq!(1, cmd_block_index);
            assert_eq!("States<Goal>", input_name_short);
            assert_eq!(
                "peace_resource_rt::states::States<peace_resource_rt::states::ts::Goal>",
                input_name_full
            );
            #[cfg(feature = "error_reporting")]
            {
                assert_eq!(
                    r#"CmdExecution:
  ExecutionOutcome: StateDiffs
CmdBlocks:
  - StatesDiscoverCmdBlock:
    Input: ()
    Outcome: States<Current>
  - DiffCmdBlock:
    Input: (States<Current>, States<Goal>)
    Outcome: (StateDiffs, States<Current>, States<Goal>)
"#,
                    cmd_execution_src
                );
                match input_span {
                    Some(input_span) => {
                        assert_eq!(174, input_span.offset());
                        assert_eq!("States<Goal>".len(), input_span.len());
                    }
                    None => panic!(
                        "Expected `input_span` to be `Some(SourceSpan::from((174, 12)))`, but was `None`."
                    ),
                }
                assert_eq!(0, full_span.offset());
                assert_eq!(cmd_execution_src.len(), full_span.len());
            }
        }
        _ => panic!("Expected `error` to be a `CmdExecutionError`, but was `{error}`"),
    }

    Ok(())
}

async fn test_ctx_init() -> Result<TestCtx, PeaceTestError> {
    let tempdir = tempfile::tempdir().map_err(PeaceTestError::TempDir)?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let flow = {
        let graph = {
            let mut graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
            graph_builder.add_fn(VecCopyItem::default().into());
            graph_builder.add_fn(MockItem::<()>::default().into());
            graph_builder.build()
        };
        Flow::new(FlowId::new(crate::fn_name_short!())?, graph)
    };

    Ok(TestCtx {
        tempdir,
        workspace,
        flow,
    })
}

struct TestCtx {
    tempdir: TempDir,
    workspace: Workspace,
    flow: Flow<PeaceTestError>,
}
