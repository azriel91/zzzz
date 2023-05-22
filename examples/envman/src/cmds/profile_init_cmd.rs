use peace::{
    cfg::{app_name, item_id, AppName, ItemId, Profile},
    cmd::{
        ctx::CmdCtx,
        scopes::{MultiProfileNoFlowView, SingleProfileSingleFlowView},
    },
    fmt::{presentable::CodeInline, presentln},
    rt::cmds::StatesDiscoverCmd,
    rt_model::{output::OutputWrite, Workspace, WorkspaceSpec},
};
use peace_items::{file_download::FileDownloadItem, tar_x::TarXItem};
use semver::Version;
use url::Url;

use crate::{
    flows::{EnvDeployFlow, EnvDeployFlowParamsSpecs},
    items::{
        peace_aws_iam_policy::IamPolicyItem, peace_aws_iam_role::IamRoleItem,
        peace_aws_instance_profile::InstanceProfileItem, peace_aws_s3_bucket::S3BucketItem,
        peace_aws_s3_object::S3ObjectItem,
    },
    model::{EnvManError, EnvType, ProfileParamsKey, RepoSlug, WebAppFileId, WorkspaceParamsKey},
};

/// Flow to initialize and set the default profile.
#[derive(Debug)]
pub struct ProfileInitCmd;

impl ProfileInitCmd {
    /// Stores profile init parameters.
    ///
    /// # Parameters
    ///
    /// * `output`: Output to write the execution outcome.
    /// * `profile`: Name of the profile to create.
    /// * `type`: Type of the environment.
    pub async fn run<O>(
        output: &mut O,
        profile_to_create: Profile,
        env_type: EnvType,
        slug: &RepoSlug,
        version: &Version,
        url: Option<Url>,
    ) -> Result<(), EnvManError>
    where
        O: OutputWrite<EnvManError>,
    {
        let app_name = app_name!();
        let workspace = Workspace::new(
            app_name.clone(),
            #[cfg(not(target_arch = "wasm32"))]
            WorkspaceSpec::WorkingDir,
            #[cfg(target_arch = "wasm32")]
            WorkspaceSpec::SessionStorage,
        )?;

        let profile_workspace_init = Profile::workspace_init();
        let cmd_ctx_builder =
            CmdCtx::builder_multi_profile_no_flow::<EnvManError, _>(output, &workspace);
        crate::cmds::ws_and_profile_params_augment!(cmd_ctx_builder);

        let cmd_ctx_result = cmd_ctx_builder
            .with_profile_filter(|profile| profile != &profile_workspace_init)
            .await;
        match cmd_ctx_result {
            Ok(mut cmd_ctx) => {
                let MultiProfileNoFlowView { profiles, .. } = cmd_ctx.view();

                if profiles.contains(&profile_to_create) {
                    return Err(EnvManError::ProfileToCreateExists {
                        profile_to_create,
                        app_name,
                    });
                }
            }
            Err(_e) => {
                // On first invocation, the `.peace` app dir will not exist, so
                // we won't be able to list any profiles.
            }
        }

        let cmd_ctx_builder =
            CmdCtx::builder_single_profile_no_flow::<EnvManError, _>(output, &workspace);
        crate::cmds::ws_and_profile_params_augment!(cmd_ctx_builder);

        // Creating the `CmdCtx` writes the workspace and profile params.
        // We don't need to run any flows with it.
        let _cmd_ctx = cmd_ctx_builder
            .with_workspace_param_value(
                WorkspaceParamsKey::Profile,
                Some(profile_to_create.clone()),
            )
            .with_profile_param_value(ProfileParamsKey::EnvType, Some(env_type))
            .with_profile(profile_to_create.clone())
            .await?;

        // --- //

        let EnvDeployFlowParamsSpecs {
            app_download_params_spec,
            app_extract_params_spec,
            iam_policy_params_spec,
            iam_role_params_spec,
            instance_profile_params_spec,
            s3_bucket_params_spec,
            s3_object_params_spec,
        } = EnvDeployFlow::params(&profile_to_create, slug, version, url)?;
        let flow = EnvDeployFlow::flow().await?;
        let profile_key = WorkspaceParamsKey::Profile;

        let mut cmd_ctx = {
            let cmd_ctx_builder =
                CmdCtx::builder_single_profile_single_flow::<EnvManError, _>(output, &workspace);
            crate::cmds::ws_and_profile_params_augment!(cmd_ctx_builder);

            cmd_ctx_builder
                .with_profile_from_workspace_param(&profile_key)
                .with_flow(&flow)
                .with_item_params::<FileDownloadItem<WebAppFileId>>(
                    item_id!("app_download"),
                    app_download_params_spec,
                )
                .with_item_params::<TarXItem<WebAppFileId>>(
                    item_id!("app_extract"),
                    app_extract_params_spec,
                )
                .with_item_params::<IamPolicyItem<WebAppFileId>>(
                    item_id!("iam_policy"),
                    iam_policy_params_spec,
                )
                .with_item_params::<IamRoleItem<WebAppFileId>>(
                    item_id!("iam_role"),
                    iam_role_params_spec,
                )
                .with_item_params::<InstanceProfileItem<WebAppFileId>>(
                    item_id!("instance_profile"),
                    instance_profile_params_spec,
                )
                .with_item_params::<S3BucketItem<WebAppFileId>>(
                    item_id!("s3_bucket"),
                    s3_bucket_params_spec,
                )
                .with_item_params::<S3ObjectItem<WebAppFileId>>(
                    item_id!("s3_object"),
                    s3_object_params_spec,
                )
                .await?
        };

        let states_discover_outcome = StatesDiscoverCmd::current_and_desired(&mut cmd_ctx).await?;
        let SingleProfileSingleFlowView { output, .. } = cmd_ctx.view();

        if states_discover_outcome.is_ok() {
            presentln!(
                output,
                [
                    "Initialized profile ",
                    &profile_to_create,
                    " using ",
                    &CodeInline::new(format!("{slug}@{version}").into()),
                    "."
                ]
            );
        } else {
            crate::output::item_errors_present(output, &states_discover_outcome.errors).await?;
        }

        Ok(())
    }
}
