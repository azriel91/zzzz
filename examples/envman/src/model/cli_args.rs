#[cfg(feature = "web_server")]
use std::net::IpAddr;

use clap::{Parser, Subcommand, ValueHint};
use peace::{cli::output::CliColorizeOpt, cli_model::OutputFormat, profile_model::Profile};
use semver::Version;
use url::Url;

use crate::model::{EnvManFlow, EnvType, RepoSlug};

#[derive(Parser)]
#[clap(
    author,
    version,
    about = "Manages an application's lifecycle.",
    long_about = "Manages deployment, configuration, upgrading, and cleaning up of an application."
)]
pub struct CliArgs {
    /// Command to run.
    #[command(subcommand)]
    pub command: EnvManCommand,
    /// Whether to run with multiple threads.
    #[arg(long, default_value = "false", global(true))]
    pub fast: bool,
    /// The format of the command output.
    ///
    /// At this level, this needs to be specified before the subcommand.
    /// <https://github.com/clap-rs/clap/issues/3002> needs to be implemented
    /// for the argument to be passed in after the subcommand.
    #[arg(long, global(true))]
    pub format: Option<OutputFormat>,
    /// Whether output should be colorized.
    ///
    /// * "auto" (default): Colorize when used interactively.
    /// * "always": Always colorize output.
    /// * "never": Never colorize output.
    #[arg(long, default_value = "auto", global(true))]
    pub color: CliColorizeOpt,
    /// Whether to show debug information.
    #[arg(long, default_value = "false", global(true))]
    pub debug: bool,
}

#[derive(Subcommand)]
pub enum EnvManCommand {
    /// Initializes a profile to deploy a web application.
    Init {
        /// Name to use for the profile.
        profile: Profile,
        /// Type of the profile's deployed environment.
        #[arg(short, long)]
        r#type: EnvType,
        /// Which flow to use: `"upload"` or `"deploy"`.
        #[arg(long, default_value = "deploy")]
        flow: EnvManFlow,
        /// Username and repository of the application to download.
        slug: RepoSlug,
        /// Version of the application to download.
        version: Version,
        /// URL to override the default download URL.
        #[arg(long, value_hint(ValueHint::Url))]
        url: Option<Url>,
    },
    /// Shows or initializes the current profile.
    Profile {
        /// Profile command to run.
        #[command(subcommand)]
        command: Option<ProfileCommand>,
    },
    /// Switches the current profile.
    ///
    /// Similar to changing the branch in git.
    Switch {
        /// Profile name to switch to.
        profile: Profile,
        /// Whether or not to create the profile.
        ///
        /// * If this flag is specified, and the profile already exists, then
        ///   the switch does not happen.
        /// * If this flag is not specified, and the profile does not exist,
        ///   then the switch does not happen.
        #[arg(short, long)]
        create: bool,
        /// Type of the profile's deployed environment.
        #[arg(short, long, required_if_eq("create", "true"))]
        r#type: Option<EnvType>,
        /// Username and repository of the application to download.
        #[arg(required_if_eq("create", "true"))]
        slug: Option<RepoSlug>,
        /// Version of the application to download.
        #[arg(required_if_eq("create", "true"))]
        version: Option<Version>,
        /// URL to override the default download URL.
        #[arg(short, long, value_hint(ValueHint::Url))]
        url: Option<Url>,
    },
    /// Discovers the state of the environment.
    Discover,
    /// Shows the state of the environment.
    Status,
    /// Shows the goal state of the environment.
    Goal,
    /// Shows the diff between states of the environment.
    ///
    /// By default, this compares the current and goal states of the active
    /// profile.
    ///
    /// Users may pass in two profiles to compare the current states of both
    /// profiles.
    Diff {
        /// First profile in the comparison.
        #[arg(requires("profile_b"))]
        profile_a: Option<Profile>,
        /// Second profile in the comparison.
        profile_b: Option<Profile>,
    },
    /// Deploys / updates the environment.
    Deploy,
    /// Cleans the current environment.
    Clean,
    /// Runs `envman` as a web server.
    #[cfg(feature = "web_server")]
    Web {
        /// The address to bind to, defaults to `127.0.0.1`.
        ///
        /// If you want to listen on all interfaces, use `0.0.0.0`.
        #[arg(long, default_value = "127.0.0.1", value_hint(ValueHint::Other))]
        address: IpAddr,
        /// Port to listen on, defaults to `7890`.
        #[arg(short, long, default_value = "7890", value_hint(ValueHint::Other))]
        port: u16,
    },
}

#[derive(Subcommand)]
pub enum ProfileCommand {
    /// Lists available profiles.
    List,
    /// Shows the current profile, if any.
    Show,
}
