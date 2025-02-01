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
        peace_aws_s3_bucket::{S3BucketItem, S3BucketParams},
        peace_aws_s3_object::{S3ObjectItem, S3ObjectParams},
    },
    model::{EnvManError, RepoSlug, WebApp},
};

/// Flow to upload a web application to S3.
#[derive(Debug)]
pub struct AppUploadFlow;

impl AppUploadFlow {
    /// Returns the `Flow` graph.
    pub async fn flow() -> Result<Flow<EnvManError>, EnvManError> {
        let flow = {
            let graph = {
                let mut graph_builder = ItemGraphBuilder::<EnvManError>::new();

                let [a, b, c] = graph_builder.add_fns([
                    FileDownloadItem::<WebApp>::new(item_id!("app_download")).into(),
                    S3BucketItem::<WebApp>::new(item_id!("s3_bucket")).into(),
                    S3ObjectItem::<WebApp>::new(item_id!("s3_object")).into(),
                ]);

                graph_builder.add_logic_edge(a, c)?;
                graph_builder.add_contains_edge(b, c)?;
                graph_builder.build()
            };

            Flow::new(flow_id!("app_upload"), graph)
        };

        Ok(flow)
    }

    /// Returns the params needed for this flow.
    pub fn params(
        profile: &Profile,
        slug: &RepoSlug,
        version: &Version,
        url: Option<Url>,
    ) -> Result<AppUploadFlowParamsSpecs, EnvManError> {
        let account = slug.account();
        let repo_name = slug.repo_name();
        let app_download_dir = PathBuf::from_iter([account, repo_name, &format!("{version}")]);

        #[cfg(target_family = "windows")]
        let file_ext = "zip";
        #[cfg(any(target_family = "unix", target_family = "wasm"))]
        let file_ext = "tar";
        // windows:
        // https://github.com/azriel91/web_app/releases/download/0.1.0/web_app.zip
        //
        // linux:
        // https://github.com/azriel91/web_app/releases/download/0.1.0/web_app.tar
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

        let bucket_name = {
            let username = whoami::username();
            let app_name = app_name!();
            format!("{username}-peace-{app_name}-{profile}").replace('_', "-")
        };
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
            .with_object_key(object_key)
            .with_bucket_name(bucket_name)
            .build();

        Ok(AppUploadFlowParamsSpecs {
            app_download_params_spec,
            s3_bucket_params_spec,
            s3_object_params_spec,
        })
    }
}

#[derive(Debug)]
pub struct AppUploadFlowParamsSpecs {
    pub app_download_params_spec: ParamsSpec<FileDownloadParams<WebApp>>,
    pub s3_bucket_params_spec: ParamsSpec<S3BucketParams<WebApp>>,
    pub s3_object_params_spec: ParamsSpec<S3ObjectParams<WebApp>>,
}
