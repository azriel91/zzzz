use peace::{
    cfg::app_name,
    cmd::{ctx::CmdCtx, scopes::SingleProfileSingleFlow},
    cmd_model::CmdOutcome,
    flow_model::FlowId,
    flow_rt::{Flow, ItemGraphBuilder},
    item_model::{item_id, ItemId},
    profile_model::Profile,
    rt::cmds::{
        CleanCmd, DiffCmd, EnsureCmd, StatesCurrentStoredDisplayCmd, StatesDiscoverCmd,
        StatesGoalDisplayCmd,
    },
    rt_model::{output::OutputWrite, Workspace, WorkspaceSpec},
};
use peace_items::file_download::{FileDownloadItem, FileDownloadParams};

#[cfg(not(target_arch = "wasm32"))]
pub use crate::download_args::{DownloadArgs, DownloadCommand};
pub use crate::{
    download_cmd_ctx_types::DownloadCmdCtxTypes, download_error::DownloadError, file_id::FileId,
};

#[cfg(not(target_arch = "wasm32"))]
mod download_args;
mod download_cmd_ctx_types;
mod download_error;
mod file_id;

#[cfg(target_arch = "wasm32")]
mod wasm;

#[derive(Debug)]
pub struct WorkspaceAndFlow {
    workspace: Workspace,
    flow: Flow<DownloadError>,
}

const FILE_ITEM_SPEC_ID: ItemId = item_id!("file");

/// Returns a default workspace and the Download item graph.
#[cfg(not(target_arch = "wasm32"))]
pub async fn workspace_and_flow_setup(
    workspace_spec: WorkspaceSpec,
    flow_id: FlowId,
) -> Result<WorkspaceAndFlow, DownloadError> {
    let workspace = Workspace::new(app_name!(), workspace_spec)?;

    let item_graph = {
        let mut item_graph_builder = ItemGraphBuilder::<DownloadError>::new();
        item_graph_builder.add_fn(FileDownloadItem::<FileId>::new(FILE_ITEM_SPEC_ID).into());
        item_graph_builder.build()
    };
    let flow = Flow::new(flow_id, item_graph);

    let workspace_and_flow = WorkspaceAndFlow { workspace, flow };
    Ok(workspace_and_flow)
}

/// Returns a default workspace and the Download item graph.
#[cfg(target_arch = "wasm32")]
pub async fn workspace_and_flow_setup(
    workspace_spec: WorkspaceSpec,
    flow_id: FlowId,
) -> Result<WorkspaceAndFlow, DownloadError> {
    let workspace = Workspace::new(app_name!(), workspace_spec)?;
    let item_graph = {
        let mut item_graph_builder = ItemGraphBuilder::<DownloadError>::new();
        item_graph_builder.add_fn(FileDownloadItem::<FileId>::new(item_id!("file")).into());
        item_graph_builder.build()
    };
    let flow = Flow::new(flow_id, item_graph);

    let workspace_and_flow = WorkspaceAndFlow { workspace, flow };
    Ok(workspace_and_flow)
}

pub type DownloadCmdCtx<'ctx, O> = CmdCtx<SingleProfileSingleFlow<'ctx, DownloadCmdCtxTypes<O>>>;

/// Returns a `CmdCtx` initialized from the workspace and item graph
pub async fn cmd_ctx<'ctx, O>(
    workspace_and_flow: &'ctx WorkspaceAndFlow,
    profile: Profile,
    output: &'ctx mut O,
    file_download_params: Option<FileDownloadParams<FileId>>,
) -> Result<DownloadCmdCtx<'ctx, O>, DownloadError>
where
    O: OutputWrite<DownloadError>,
{
    let WorkspaceAndFlow { workspace, flow } = workspace_and_flow;
    let mut cmd_ctx_builder =
        CmdCtx::builder_single_profile_single_flow(output.into(), workspace.into())
            .with_profile(profile)
            .with_flow(flow.into());

    if let Some(file_download_params) = file_download_params {
        cmd_ctx_builder = cmd_ctx_builder.with_item_params::<FileDownloadItem<FileId>>(
            FILE_ITEM_SPEC_ID,
            file_download_params.into(),
        );
    }

    cmd_ctx_builder.await
}

pub async fn fetch<O>(cmd_ctx: &mut DownloadCmdCtx<'_, O>) -> Result<(), DownloadError>
where
    O: OutputWrite<DownloadError>,
{
    let CmdOutcome::Complete {
        value: (_states_current, _states_goal),
        cmd_blocks_processed: _,
    } = StatesDiscoverCmd::current_and_goal(cmd_ctx).await?
    else {
        panic!("Expected `StatesDiscoverCmd::current_and_goal` to complete successfully.");
    };
    Ok(())
}

pub async fn status<O>(cmd_ctx: &mut DownloadCmdCtx<'_, O>) -> Result<(), DownloadError>
where
    O: OutputWrite<DownloadError>,
{
    // Already displayed by the command
    let _states_current_stored = StatesCurrentStoredDisplayCmd::exec(cmd_ctx).await?;
    Ok(())
}

pub async fn goal<O>(cmd_ctx: &mut DownloadCmdCtx<'_, O>) -> Result<(), DownloadError>
where
    O: OutputWrite<DownloadError>,
{
    // Already displayed by the command
    let _states_goal = StatesGoalDisplayCmd::exec(cmd_ctx).await?;
    Ok(())
}

pub async fn diff<O>(cmd_ctx: &mut DownloadCmdCtx<'_, O>) -> Result<(), DownloadError>
where
    O: OutputWrite<DownloadError>,
{
    let cmd_outcome = DiffCmd::diff_stored(cmd_ctx).await?;
    let states_diff = cmd_outcome
        .value()
        .expect("Expected `states_diff` to exist.");
    cmd_ctx.output_mut().present(states_diff).await?;
    Ok(())
}

pub async fn ensure_dry<O>(cmd_ctx: &mut DownloadCmdCtx<'_, O>) -> Result<(), DownloadError>
where
    O: OutputWrite<DownloadError>,
{
    let states_ensured_dry_outcome = EnsureCmd::exec_dry(cmd_ctx).await?;
    let states_ensured_dry = states_ensured_dry_outcome
        .value()
        .expect("Expected `states_ensured_dry` to exist.");
    cmd_ctx.output_mut().present(states_ensured_dry).await?;
    Ok(())
}

pub async fn ensure<O>(cmd_ctx: &mut DownloadCmdCtx<'_, O>) -> Result<(), DownloadError>
where
    O: OutputWrite<DownloadError>,
{
    let states_ensured_outcome = EnsureCmd::exec(cmd_ctx).await?;
    let states_ensured = states_ensured_outcome
        .value()
        .expect("Expected `states_ensured` to exist.");
    cmd_ctx.output_mut().present(states_ensured).await?;
    Ok(())
}

pub async fn clean_dry<O>(cmd_ctx: &mut DownloadCmdCtx<'_, O>) -> Result<(), DownloadError>
where
    O: OutputWrite<DownloadError>,
{
    let states_cleaned_dry_outcome = CleanCmd::exec_dry(cmd_ctx).await?;
    let states_cleaned_dry = states_cleaned_dry_outcome
        .value()
        .expect("Expected `states_cleaned_dry` to exist.");
    cmd_ctx.output_mut().present(states_cleaned_dry).await?;
    Ok(())
}

pub async fn clean<O>(cmd_ctx: &mut DownloadCmdCtx<'_, O>) -> Result<(), DownloadError>
where
    O: OutputWrite<DownloadError>,
{
    let states_cleaned_outcome = CleanCmd::exec(cmd_ctx).await?;
    let states_cleaned = states_cleaned_outcome
        .value()
        .expect("Expected `states_cleaned` to exist.");
    cmd_ctx.output_mut().present(states_cleaned).await?;
    Ok(())
}
