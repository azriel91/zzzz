use peace::{
    cfg::{app_name, profile, AppName},
    cmd::ctx::CmdCtxBuilder,
    flow_model::flow_id,
    flow_rt::{Flow, ItemGraphBuilder},
    rt_model::Workspace,
};

use crate::{no_op_output::NoOpOutput, PeaceTestError};

#[tokio::test]
async fn single_profile_single_flow_getters() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = workspace(tempdir, app_name!("test_single_profile_single_flow"))?;
    let profile = profile!("test_profile");
    let flow_id = flow_id!("test_flow_id");
    let flow = Flow::<PeaceTestError>::new(flow_id.clone(), ItemGraphBuilder::new().build());

    let output = NoOpOutput;
    let cmd_ctx = CmdCtxBuilder::single_profile_single_flow(output.into(), (&workspace).into())
        .with_profile(profile.clone())
        .with_flow((&flow).into())
        .build()
        .await?;

    let scope = cmd_ctx.scope();
    assert!(std::ptr::eq(&workspace, cmd_ctx.workspace()));
    assert_eq!(workspace.dirs().workspace_dir(), cmd_ctx.workspace_dir());
    assert_eq!(workspace.dirs().peace_dir(), cmd_ctx.peace_dir());
    assert_eq!(workspace.dirs().peace_app_dir(), cmd_ctx.peace_app_dir());
    assert_eq!(scope.profile(), cmd_ctx.profile());
    assert_eq!(scope.profile_dir(), cmd_ctx.profile_dir());
    assert_eq!(scope.flow().flow_id(), cmd_ctx.flow().flow_id());
    assert_eq!(scope.flow_dir(), cmd_ctx.flow_dir());
    Ok(())
}

fn workspace(
    tempdir: tempfile::TempDir,
    app_name: AppName,
) -> Result<Workspace, Box<dyn std::error::Error>> {
    let workspace = {
        let workspace_spec = peace::rt_model::WorkspaceSpec::Path(tempdir.path().to_path_buf());
        Workspace::new(app_name, workspace_spec)?
    };
    Ok(workspace)
}
