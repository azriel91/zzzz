use clap::Parser;
use peace::{cfg::profile, cli::output::CliOutput, flow_model::flow_id, rt_model::WorkspaceSpec};
use peace_items::file_download::FileDownloadParams;

use download::{
    clean, clean_dry, cmd_ctx, diff, ensure, ensure_dry, fetch, goal, status,
    workspace_and_flow_setup, DownloadArgs, DownloadCommand, DownloadError,
};

#[cfg(not(feature = "error_reporting"))]
pub fn main() {
    run().unwrap();
}

#[cfg(feature = "error_reporting")]
pub fn main() -> peace::miette::Result<(), peace::miette::Report> {
    // Important to return `peace::miette::Report` instead of calling
    // `IntoDiagnostic::intoDiagnostic` on the `Error`, as that does not present the
    // diagnostic contextual information to the user.
    //
    // See <https://docs.rs/miette/latest/miette/trait.IntoDiagnostic.html#warning>.

    // The explicit mapping for `PeaceRtError` appears to be necessary to display
    // the diagnostic information. i.e. `miette` does not automatically delegate to
    // the #[diagnostic_source].
    //
    // This is fixed by <https://github.com/zkat/miette/pull/170>.

    run().map_err(|file_download_error| match file_download_error {
        DownloadError::PeaceItemFileDownload(err) => peace::miette::Report::from(err),
        DownloadError::PeaceRtError(err) => peace::miette::Report::from(err),
        other => peace::miette::Report::from(other),
    })
}

pub fn run() -> Result<(), DownloadError> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .thread_name("main")
        .thread_stack_size(3 * 1024 * 1024)
        .enable_io()
        .enable_time()
        .build()
        .map_err(DownloadError::TokioRuntimeInit)?;

    let DownloadArgs {
        command,
        verbose,
        format,
    } = DownloadArgs::parse();

    if !verbose {
        #[cfg(feature = "error_reporting")]
        peace::miette::set_hook(Box::new(|_| {
            Box::new(
                peace::miette::MietteHandlerOpts::new()
                    .without_cause_chain()
                    .build(),
            )
        }))
        .expect("Failed to configure miette hook.");
    }

    runtime.block_on(async {
        let workspace_spec = WorkspaceSpec::WorkingDir;
        let profile = profile!("default");
        let flow_id = flow_id!("file");
        let mut cli_output = {
            let mut builder = CliOutput::builder();
            if let Some(format) = format {
                builder = builder.with_outcome_format(format);
            }
            builder.build()
        };

        match command {
            DownloadCommand::Init { url, dest } => {
                let workspace_and_flow = workspace_and_flow_setup(workspace_spec, flow_id).await?;
                let mut cmd_ctx = cmd_ctx(
                    &workspace_and_flow,
                    profile,
                    &mut cli_output,
                    Some(FileDownloadParams::new(url, dest)),
                )
                .await?;
                fetch(&mut cmd_ctx).await?;
            }
            DownloadCommand::Fetch => {
                let workspace_and_flow = workspace_and_flow_setup(workspace_spec, flow_id).await?;
                let mut cmd_ctx =
                    cmd_ctx(&workspace_and_flow, profile, &mut cli_output, None).await?;
                fetch(&mut cmd_ctx).await?;
            }
            DownloadCommand::Status => {
                let workspace_and_flow = workspace_and_flow_setup(workspace_spec, flow_id).await?;
                let mut cmd_ctx =
                    cmd_ctx(&workspace_and_flow, profile, &mut cli_output, None).await?;
                status(&mut cmd_ctx).await?;
            }
            DownloadCommand::Goal => {
                let workspace_and_flow = workspace_and_flow_setup(workspace_spec, flow_id).await?;
                let mut cmd_ctx =
                    cmd_ctx(&workspace_and_flow, profile, &mut cli_output, None).await?;
                goal(&mut cmd_ctx).await?;
            }
            DownloadCommand::Diff => {
                let workspace_and_flow = workspace_and_flow_setup(workspace_spec, flow_id).await?;
                let mut cmd_ctx =
                    cmd_ctx(&workspace_and_flow, profile, &mut cli_output, None).await?;
                diff(&mut cmd_ctx).await?;
            }
            DownloadCommand::EnsureDry => {
                let workspace_and_flow = workspace_and_flow_setup(workspace_spec, flow_id).await?;
                let mut cmd_ctx =
                    cmd_ctx(&workspace_and_flow, profile, &mut cli_output, None).await?;
                ensure_dry(&mut cmd_ctx).await?;
            }
            DownloadCommand::Ensure => {
                let workspace_and_flow = workspace_and_flow_setup(workspace_spec, flow_id).await?;
                let mut cmd_ctx =
                    cmd_ctx(&workspace_and_flow, profile, &mut cli_output, None).await?;
                ensure(&mut cmd_ctx).await?;
            }
            DownloadCommand::CleanDry => {
                let workspace_and_flow = workspace_and_flow_setup(workspace_spec, flow_id).await?;
                let mut cmd_ctx =
                    cmd_ctx(&workspace_and_flow, profile, &mut cli_output, None).await?;
                clean_dry(&mut cmd_ctx).await?;
            }
            DownloadCommand::Clean => {
                let workspace_and_flow = workspace_and_flow_setup(workspace_spec, flow_id).await?;
                let mut cmd_ctx =
                    cmd_ctx(&workspace_and_flow, profile, &mut cli_output, None).await?;
                clean(&mut cmd_ctx).await?;
            }
        }

        Ok::<_, DownloadError>(())
    })
}
