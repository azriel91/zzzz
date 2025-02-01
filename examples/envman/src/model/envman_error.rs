#[cfg(feature = "error_reporting")]
use peace::miette;

use peace::{
    cfg::AppName,
    profile_model::Profile,
    rt_model::fn_graph::{Edge, WouldCycle},
};

/// Error while managing a web application.
#[cfg_attr(feature = "error_reporting", derive(peace::miette::Diagnostic))]
#[derive(Debug, thiserror::Error)]
pub enum EnvManError {
    /// Failed to construct web application download URL.
    #[error("Failed to construct web application download URL.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(
            code(envman::envman_url_build),
            help("If the URL is valid, this may be a bug in the example, or the `url` library.")
        )
    )]
    EnvManUrlBuild {
        /// Computed URL that is attempted to be parsed.
        #[cfg_attr(feature = "error_reporting", source_code)]
        url_candidate: String,
        /// URL parse error.
        #[source]
        error: url::ParseError,
    },
    /// Failed to parse environment type.
    #[error("Failed to parse environment type.")]
    EnvTypeParseError(
        #[source]
        #[from]
        crate::model::EnvTypeParseError,
    ),

    /// User tried to switch to a profile that doesn't exist.
    #[error("Profile to switch to does not exist.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(
            code(envman::profile_switch_to_non_existent),
            help(
                "The `{profile_to_switch_to}` profile does not exist.\n\
                You can create it by passing the `--create --type development` parameters\n\
                or run `{app_name} profile list` to see profiles you can switch to."
            )
        )
    )]
    ProfileSwitchToNonExistent {
        /// The profile that the user tried to switch to.
        profile_to_switch_to: Profile,
        /// Name of this app.
        app_name: AppName,
    },

    /// User tried to create a profile that already exists.
    #[error("Profile to create already exists.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(
            code(envman::profile_to_create_exists),
            help(
                "The `{profile_to_create}` profile already exists.\n\
                You may switch to the profile using `{app_name} switch {profile_to_create}`\n\
                or create a profile with a different name."
            )
        )
    )]
    ProfileToCreateExists {
        /// The profile that the user tried to create.
        profile_to_create: Profile,
        /// Name of this app.
        app_name: AppName,
    },

    // === Item errors === //
    /// A `FileDownload` item error occurred.
    #[error("A `FileDownload` item error occurred.")]
    PeaceItemFileDownload(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        peace_items::file_download::FileDownloadError,
    ),
    /// An `InstanceProfile` item error occurred.
    #[error("An `InstanceProfile` item error occurred.")]
    InstanceProfileItem(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        crate::items::peace_aws_instance_profile::InstanceProfileError,
    ),
    /// An `IamPolicy` item error occurred.
    #[error("An `IamPolicy` item error occurred.")]
    IamPolicyItem(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        crate::items::peace_aws_iam_policy::IamPolicyError,
    ),
    /// An `IamRole` item error occurred.
    #[error("An `IamRole` item error occurred.")]
    IamRoleItem(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        crate::items::peace_aws_iam_role::IamRoleError,
    ),
    /// An `S3Bucket` item error occurred.
    #[error("An `S3Bucket` item error occurred.")]
    S3BucketItem(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        crate::items::peace_aws_s3_bucket::S3BucketError,
    ),
    /// An `S3Object` item error occurred.
    #[error("An `S3Object` item error occurred.")]
    S3ObjectItem(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        crate::items::peace_aws_s3_object::S3ObjectError,
    ),

    // === Framework errors === //
    /// A `peace` runtime error occurred.
    #[error("A `peace` runtime error occurred.")]
    PeaceRtError(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        peace::rt_model::Error,
    ),

    /// A graph `WouldCycle` error occurred.
    #[error("A `peace` runtime error occurred.")]
    WouldCycleError(
        #[source]
        #[from]
        WouldCycle<Edge>,
    ),

    // === Scaffolding errors === //
    #[error("Failed to initialize tokio runtime.")]
    TokioRuntimeInit(#[source] std::io::Error),

    // === Web Server errors === //
    /// Web interface server produced an error.
    #[cfg(feature = "ssr")]
    #[error("Web interface server produced an error.")]
    #[cfg_attr(feature = "error_reporting", diagnostic(code(envman::webi)))]
    Webi {
        /// Underlying error.
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[from]
        #[source]
        error: peace::webi_model::WebiError,
    },

    /// Web server ended due to an error.
    #[cfg(feature = "ssr")]
    #[error("Web server ended due to an error.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(envman::web_server_serve),)
    )]
    WebServerServe {
        /// Underlying error.
        #[source]
        error: std::io::Error,
    },

    /// Failed to join thread that rendered web server home page.
    #[cfg(feature = "ssr")]
    #[error("Failed to join thread that rendered web server home page.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(envman::web_server_render_join))
    )]
    WebServerRenderJoin {
        /// Underlying error.
        #[source]
        error: tokio::task::JoinError,
    },
}
