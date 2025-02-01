use std::path::PathBuf;

use peace::{
    cfg::app_name,
    flow_model::flow_id,
    flow_rt::{Flow, ItemGraphBuilder},
    item_model::item_id,
    params::{Params, ParamsSpec},
    profile_model::Profile,
};
use peace_items::file_download::{FileDownloadItem, FileDownloadParams};
use semver::Version;
use url::Url;

use crate::{
    items::{
        peace_aws_iam_policy::{IamPolicyItem, IamPolicyParams, IamPolicyState},
        peace_aws_iam_role::{IamRoleItem, IamRoleParams},
        peace_aws_instance_profile::{InstanceProfileItem, InstanceProfileParams},
        peace_aws_s3_bucket::{S3BucketItem, S3BucketParams},
        peace_aws_s3_object::{S3ObjectItem, S3ObjectParams},
    },
    model::{EnvManError, RepoSlug, WebApp},
};

/// Flow to deploy a web application.
#[derive(Debug)]
pub struct EnvDeployFlow;

impl EnvDeployFlow {
    /// Returns the `Flow` graph.
    pub async fn flow() -> Result<Flow<EnvManError>, EnvManError> {
        let flow = {
            let graph = {
                let mut graph_builder = ItemGraphBuilder::<EnvManError>::new();

                let [app_download_id, iam_policy_item_id, iam_role_item_id, instance_profile_item_id, s3_bucket_id, s3_object_id] =
                    graph_builder.add_fns([
                        FileDownloadItem::<WebApp>::new(item_id!("app_download")).into(),
                        IamPolicyItem::<WebApp>::new(item_id!("iam_policy")).into(),
                        IamRoleItem::<WebApp>::new(item_id!("iam_role")).into(),
                        InstanceProfileItem::<WebApp>::new(item_id!("instance_profile")).into(),
                        S3BucketItem::<WebApp>::new(item_id!("s3_bucket")).into(),
                        S3ObjectItem::<WebApp>::new(item_id!("s3_object")).into(),
                    ]);

                graph_builder.add_logic_edges([
                    (iam_policy_item_id, iam_role_item_id),
                    (iam_role_item_id, instance_profile_item_id),
                    // Download the file before uploading it.
                    (app_download_id, s3_object_id),
                ])?;
                // Create the bucket before uploading to it.
                graph_builder.add_contains_edge(s3_bucket_id, s3_object_id)?;
                graph_builder.build()
            };

            Flow::new(flow_id!("env_deploy"), graph)
        };

        Ok(flow)
    }

    /// Returns the params needed for this flow.
    pub fn params(
        profile: &Profile,
        slug: &RepoSlug,
        version: &Version,
        url: Option<Url>,
    ) -> Result<EnvDeployFlowParamsSpecs, EnvManError> {
        let account = slug.account();
        let repo_name = slug.repo_name();
        let app_download_dir = PathBuf::from_iter([account, repo_name, &format!("{version}")]);

        // https://github.com/azriel91/web_app/releases/download/0.1.0/web_app.tar
        let file_ext = "tar";
        let web_app_file_url = url.map(Result::Ok).unwrap_or_else(|| {
            let url_candidate = format!(
                "https://github.com/{account}/{repo_name}/releases/download/{version}/{repo_name}.{file_ext}"
            );
            Url::parse(&url_candidate).map_err(|error| EnvManError::EnvManUrlBuild {
                url_candidate,
                error,
            })
        })?;
        let web_app_path_local = app_download_dir.join(format!("{repo_name}.{file_ext}"));
        let app_download_params_spec = FileDownloadParams::new(
            web_app_file_url,
            web_app_path_local.clone(),
            #[cfg(target_arch = "wasm32")]
            peace_items::file_download::StorageForm::Base64,
        )
        .into();

        let iam_policy_name = profile.to_string();
        let iam_role_name = profile.to_string();
        let instance_profile_name = profile.to_string();
        let bucket_name = {
            let username = whoami::username();
            let app_name = app_name!();
            format!("{username}-peace-{app_name}-{profile}").replace('_', "-")
        };
        let path = String::from("/");

        let iam_policy_params_spec = IamPolicyParams::<WebApp>::field_wise_spec()
            .with_name(iam_policy_name)
            .with_path(path.clone())
            .with_policy_document(format!(
                include_str!("ec2_to_s3_bucket_policy.json"),
                bucket_name = bucket_name
            ))
            .build();

        let iam_role_params_spec = IamRoleParams::<WebApp>::field_wise_spec()
            .with_name(iam_role_name)
            .with_path(path.clone())
            .with_managed_policy_arn_from_map(IamPolicyState::policy_id_arn_version)
            .build();
        let instance_profile_params_spec = InstanceProfileParams::<WebApp>::field_wise_spec()
            .with_name(instance_profile_name)
            .with_path(path)
            .with_role_associate(true)
            .build();
        let object_key = web_app_path_local
            .file_name()
            .expect("Expected web app file name to exist.")
            .to_string_lossy()
            .to_string();
        let s3_bucket_params_spec = S3BucketParams::<WebApp>::field_wise_spec()
            .with_name(bucket_name.clone())
            .build();
        let s3_object_params_spec = S3ObjectParams::<WebApp>::field_wise_spec()
            .with_file_path(web_app_path_local)
            .with_bucket_name(bucket_name)
            .with_object_key(object_key)
            .build();

        Ok(EnvDeployFlowParamsSpecs {
            app_download_params_spec,
            iam_policy_params_spec,
            iam_role_params_spec,
            instance_profile_params_spec,
            s3_bucket_params_spec,
            s3_object_params_spec,
        })
    }
}

#[derive(Debug)]
pub struct EnvDeployFlowParamsSpecs {
    pub app_download_params_spec: ParamsSpec<FileDownloadParams<WebApp>>,
    pub iam_policy_params_spec: ParamsSpec<IamPolicyParams<WebApp>>,
    pub iam_role_params_spec: ParamsSpec<IamRoleParams<WebApp>>,
    pub instance_profile_params_spec: ParamsSpec<InstanceProfileParams<WebApp>>,
    pub s3_bucket_params_spec: ParamsSpec<S3BucketParams<WebApp>>,
    pub s3_object_params_spec: ParamsSpec<S3ObjectParams<WebApp>>,
}
