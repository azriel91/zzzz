use peace::{
    cmd_model::CmdOutcome,
    fmt::{
        presentable::{Heading, HeadingLevel},
        presentln,
    },
    item_model::ItemId,
    resource_rt::{resources::ts::SetUp, Resources},
    rt_model::{output::OutputWrite, IndexMap},
};

use crate::model::EnvManError;

/// Presents errors.
pub async fn errors_present<O>(output: &mut O, errors: &[EnvManError]) -> Result<(), EnvManError>
where
    O: OutputWrite<EnvManError>,
{
    output
        .present(Heading::new(HeadingLevel::Level1, "Errors"))
        .await?;

    #[cfg(feature = "error_reporting")]
    {
        use peace::miette::{Diagnostic, GraphicalReportHandler};

        let report_handler = GraphicalReportHandler::new().without_cause_chain();
        let mut err_buffer = String::new();
        for error in errors.iter() {
            let mut diagnostic_opt: Option<&dyn Diagnostic> = Some(error);
            while let Some(diagnostic) = diagnostic_opt {
                if diagnostic.help().is_some()
                    || diagnostic.labels().is_some()
                    || diagnostic.diagnostic_source().is_none()
                {
                    // Ignore failures when writing errors
                    let (Ok(()) | Err(_)) =
                        report_handler.render_report(&mut err_buffer, diagnostic);
                    let (Ok(()) | Err(_)) = output.present(&err_buffer).await;
                    let (Ok(()) | Err(_)) = output.present("\n").await;
                }

                diagnostic_opt = diagnostic.diagnostic_source();
            }

            err_buffer.clear();
        }
    }

    #[cfg(not(feature = "error_reporting"))]
    {
        use std::error::Error;

        errors.iter().for_each(|error| {
            eprintln!("\n{error}");
            let mut error = error.source();
            while let Some(source) = error {
                eprintln!("  caused by: {source}");
                error = source.source();
            }
        });
    }

    Ok(())
}

/// Presents item errors.
pub async fn item_errors_present<O>(
    output: &mut O,
    errors: &IndexMap<ItemId, EnvManError>,
) -> Result<(), EnvManError>
where
    O: OutputWrite<EnvManError>,
{
    output
        .present(Heading::new(HeadingLevel::Level1, "Errors"))
        .await?;

    #[cfg(feature = "error_reporting")]
    {
        use peace::miette::{Diagnostic, GraphicalReportHandler};

        let report_handler = GraphicalReportHandler::new().without_cause_chain();
        let mut err_buffer = String::new();
        for (item_id, error) in errors.iter() {
            // Ignore failures when writing errors
            let (Ok(()) | Err(_)) = output.present(item_id).await;
            let (Ok(()) | Err(_)) = output.present(":\n").await;

            let mut diagnostic_opt: Option<&dyn Diagnostic> = Some(error);
            while let Some(diagnostic) = diagnostic_opt {
                if diagnostic.help().is_some()
                    || diagnostic.labels().is_some()
                    || diagnostic.diagnostic_source().is_none()
                {
                    // Ignore failures when writing errors
                    let (Ok(()) | Err(_)) =
                        report_handler.render_report(&mut err_buffer, diagnostic);
                    let (Ok(()) | Err(_)) = output.present(&err_buffer).await;
                    let (Ok(()) | Err(_)) = output.present("\n").await;
                }

                diagnostic_opt = diagnostic.diagnostic_source();
            }

            err_buffer.clear();
        }
    }

    #[cfg(not(feature = "error_reporting"))]
    {
        use std::error::Error;

        errors.iter().for_each(|(item_id, error)| {
            eprintln!("\n{item_id}: {error}");
            let mut error = error.source();
            while let Some(source) = error {
                eprintln!("  caused by: {source}");
                error = source.source();
            }
        });
    }

    Ok(())
}

/// Presents interruption or error information of a `CmdOutcome`.
pub async fn cmd_outcome_completion_present<O, T>(
    output: &mut O,
    resources: &Resources<SetUp>,
    cmd_outcome: CmdOutcome<T, EnvManError>,
) -> Result<(), EnvManError>
where
    O: OutputWrite<EnvManError>,
{
    match &cmd_outcome {
        CmdOutcome::Complete {
            value: _,
            cmd_blocks_processed: _,
        } => {
            // Nothing to do.
        }
        CmdOutcome::BlockInterrupted {
            item_stream_outcome,
            cmd_blocks_processed,
            cmd_blocks_not_processed,
        } => {
            let cmd_blocks_complete = cmd_blocks_processed
                .iter()
                .map(|cmd_block_desc| cmd_block_desc.cmd_block_name())
                .collect::<Vec<_>>();
            let cmd_blocks_incomplete = cmd_blocks_not_processed
                .iter()
                .map(|cmd_block_desc| cmd_block_desc.cmd_block_name())
                .collect::<Vec<_>>();
            let item_ids_processed = item_stream_outcome
                .item_ids_processed()
                .iter()
                .collect::<Vec<_>>();
            let item_ids_not_processed = item_stream_outcome
                .item_ids_not_processed()
                .iter()
                .collect::<Vec<_>>();

            presentln!(output, ["Execution was interrupted."]);
            presentln!(output, [""]);

            presentln!(output, ["`CmdBlock`s completed:"]);
            presentln!(output, [""]);
            presentln!(output, [&cmd_blocks_complete]);

            presentln!(output, ["`CmdBlock`s not completed:"]);
            presentln!(output, [""]);
            presentln!(output, [&cmd_blocks_incomplete]);

            presentln!(output, ["Items completed:"]);
            presentln!(output, [""]);
            presentln!(output, [&item_ids_processed]);

            presentln!(output, ["Items not completed:"]);
            presentln!(output, [""]);
            presentln!(output, [&item_ids_not_processed]);
        }
        CmdOutcome::ExecutionInterrupted {
            value: _,
            cmd_blocks_processed,
            cmd_blocks_not_processed,
        } => {
            let cmd_blocks_complete = cmd_blocks_processed
                .iter()
                .map(|cmd_block_desc| cmd_block_desc.cmd_block_name())
                .collect::<Vec<_>>();
            let cmd_blocks_incomplete = cmd_blocks_not_processed
                .iter()
                .map(|cmd_block_desc| cmd_block_desc.cmd_block_name())
                .collect::<Vec<_>>();

            presentln!(output, ["Execution was interrupted."]);
            presentln!(output, [""]);

            presentln!(output, ["`CmdBlock`s completed:"]);
            presentln!(output, [""]);
            presentln!(output, [&cmd_blocks_complete]);

            presentln!(output, ["`CmdBlock`s not completed:"]);
            presentln!(output, [""]);
            presentln!(output, [&cmd_blocks_incomplete]);
        }
        CmdOutcome::ItemError {
            item_stream_outcome: _,
            cmd_blocks_processed: _,
            cmd_blocks_not_processed: _,
            errors,
        } => {
            item_errors_present(output, errors).await?;
            let _ = tokio::fs::write("resources.ron", format!("{resources:#?}")).await;
        }
    }

    Ok(())
}
