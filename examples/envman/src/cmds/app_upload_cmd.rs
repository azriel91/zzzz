use futures::future::LocalBoxFuture;
use peace::{
    cfg::app_name,
    cmd::{
        ctx::CmdCtx,
        scopes::{
            MultiProfileSingleFlow, SingleProfileSingleFlowView,
            SingleProfileSingleFlowViewAndOutput,
        },
    },
    fmt::presentln,
    item_model::item_id,
    params::Params,
    profile_model::Profile,
    rt_model::{output::OutputWrite, Workspace, WorkspaceSpec},
};

use crate::{
    cmds::CmdOpts,
    flows::AppUploadFlow,
    items::{
        peace_aws_s3_bucket::S3BucketState,
        peace_aws_s3_object::{S3ObjectItem, S3ObjectParams},
    },
    model::{EnvManError, EnvType, ProfileParamsKey, WebApp, WorkspaceParamsKey},
    rt_model::{EnvManCmdCtx, EnvmanCmdCtxTypes},
};

/// Runs a `*Cmd` that interacts with the application upload.
#[derive(Debug)]
pub struct AppUploadCmd;

impl AppUploadCmd {
    /// Runs a command on the environment with the active profile.
    ///
    /// # Parameters
    ///
    /// * `output`: Output to write the execution outcome.
    /// * `cmd_opts`: Options to configure the `Cmd`'s output.
    /// * `f`: The command to run.
    pub async fn run<O, T, F>(output: &mut O, cmd_opts: CmdOpts, f: F) -> Result<T, EnvManError>
    where
        O: OutputWrite<EnvManError>,
        for<'fn_once> F: FnOnce(
            &'fn_once mut EnvManCmdCtx<'_, O>,
        ) -> LocalBoxFuture<'fn_once, Result<T, EnvManError>>,
    {
        let workspace = Workspace::new(
            app_name!(),
            #[cfg(not(target_arch = "wasm32"))]
            WorkspaceSpec::WorkingDir,
            #[cfg(target_arch = "wasm32")]
            WorkspaceSpec::SessionStorage,
        )?;
        let flow = AppUploadFlow::flow().await?;
        let profile_key = WorkspaceParamsKey::Profile;

        let s3_object_params_spec = S3ObjectParams::<WebApp>::field_wise_spec()
            .with_bucket_name_from_map(S3BucketState::bucket_name)
            .build();

        let mut cmd_ctx = {
            let cmd_ctx_builder = CmdCtx::builder_single_profile_single_flow::<EnvManError, O>(
                output.into(),
                workspace.into(),
            );
            crate::cmds::ws_and_profile_params_augment!(cmd_ctx_builder);

            cmd_ctx_builder
                .with_profile_from_workspace_param(profile_key.into())
                .with_flow((&flow).into())
                .with_item_params::<S3ObjectItem<WebApp>>(
                    item_id!("s3_object"),
                    s3_object_params_spec,
                )
                .await?
        };

        let CmdOpts { profile_print } = cmd_opts;

        if profile_print {
            Self::profile_print(&mut cmd_ctx).await?;
        }

        let t = f(&mut cmd_ctx).await?;

        Ok(t)
    }

    /// Runs a multi-profile command using the `EnvDeploy` flow..
    ///
    /// # Parameters
    ///
    /// * `output`: Output to write the execution outcome.
    /// * `f`: The command to run.
    pub async fn multi_profile<O, T, F>(output: &mut O, f: F) -> Result<T, EnvManError>
    where
        O: OutputWrite<EnvManError>,
        for<'fn_once> F: FnOnce(
            &'fn_once mut CmdCtx<MultiProfileSingleFlow<'_, EnvmanCmdCtxTypes<O>>>,
        ) -> LocalBoxFuture<'fn_once, Result<T, EnvManError>>,
    {
        let workspace = Workspace::new(
            app_name!(),
            #[cfg(not(target_arch = "wasm32"))]
            WorkspaceSpec::WorkingDir,
            #[cfg(target_arch = "wasm32")]
            WorkspaceSpec::SessionStorage,
        )?;
        let flow = AppUploadFlow::flow().await?;

        let s3_object_params_spec = S3ObjectParams::<WebApp>::field_wise_spec()
            .with_bucket_name_from_map(|s3_bucket_state: &S3BucketState| match s3_bucket_state {
                S3BucketState::None => None,
                S3BucketState::Some {
                    name,
                    creation_date: _,
                } => Some(name.clone()),
            })
            .build();

        let mut cmd_ctx = {
            let cmd_ctx_builder = CmdCtx::builder_multi_profile_single_flow::<EnvManError, O>(
                output.into(),
                (&workspace).into(),
            );
            crate::cmds::ws_and_profile_params_augment!(cmd_ctx_builder);

            cmd_ctx_builder
                .with_flow((&flow).into())
                .with_item_params::<S3ObjectItem<WebApp>>(
                    item_id!("s3_object"),
                    s3_object_params_spec,
                )
                .await?
        };

        let t = f(&mut cmd_ctx).await?;

        Ok(t)
    }

    async fn profile_print<O>(cmd_ctx: &mut EnvManCmdCtx<'_, O>) -> Result<(), EnvManError>
    where
        O: OutputWrite<EnvManError>,
    {
        let SingleProfileSingleFlowViewAndOutput {
            output,
            cmd_view:
                SingleProfileSingleFlowView {
                    workspace_params,
                    profile_params,
                    ..
                },
            ..
        } = cmd_ctx.view_and_output();

        let profile = workspace_params.get::<Profile, _>(&WorkspaceParamsKey::Profile);
        let env_type = profile_params.get::<EnvType, _>(&ProfileParamsKey::EnvType);

        if let Some((profile, env_type)) = profile.zip(env_type) {
            presentln!(
                output,
                ["Using profile ", profile, " -- type ", env_type, "\n"]
            );
        }

        Ok(())
    }
}
