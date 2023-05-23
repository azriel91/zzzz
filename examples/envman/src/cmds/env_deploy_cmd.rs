use futures::FutureExt;
use peace::{
    cmd::scopes::SingleProfileSingleFlowView,
    fmt::presentable::{Heading, HeadingLevel, ListNumbered},
    rt::cmds::{sub::StatesSavedReadCmd, EnsureCmd},
    rt_model::{outcomes::CmdOutcome, output::OutputWrite},
};

use crate::{
    cmds::{
        common::{env_man_flow, workspace},
        AppUploadCmd, EnvCmd,
    },
    model::{EnvManError, EnvManFlow},
};

/// Deploys or updates the environment.
#[derive(Debug)]
pub struct EnvDeployCmd;

impl EnvDeployCmd {
    /// Deploys or updates the environment.
    ///
    /// # Parameters
    ///
    /// * `output`: Output to write the execution outcome.
    pub async fn run<O>(output: &mut O) -> Result<(), EnvManError>
    where
        O: OutputWrite<EnvManError> + Send,
    {
        let workspace = workspace()?;
        let env_man_flow = env_man_flow(output, &workspace).await?;
        match env_man_flow {
            EnvManFlow::AppUpload => run!(output, AppUploadCmd, 3usize),
            EnvManFlow::EnvDeploy => run!(output, EnvCmd, 18usize),
        };

        Ok(())
    }
}

macro_rules! run {
    ($output:ident, $flow_cmd:ident, $padding:expr) => {{
        let states_saved = $flow_cmd::run($output, true, |ctx| {
            StatesSavedReadCmd::exec(ctx).boxed_local()
        })
        .await?;
        $flow_cmd::run($output, false, |ctx| {
            async move {
                let states_saved_ref = &states_saved;
                let states_ensured_outcome = EnsureCmd::exec(ctx, states_saved_ref).await?;
                let CmdOutcome {
                    value: states_ensured,
                    errors,
                } = &states_ensured_outcome;
                let SingleProfileSingleFlowView {
                    output,
                    flow,
                    resources,
                    ..
                } = ctx.view();

                if states_ensured_outcome.is_ok() {
                    let states_ensured_raw_map = &***states_ensured;

                    let states_ensured_presentables = {
                        let states_ensured_presentables = flow
                            .graph()
                            .iter_insertion()
                            .map(|item| {
                                let item_id = item.id();
                                // Hack: for alignment
                                let padding = " ".repeat(
                                    $padding.saturating_sub(format!("{item_id}").len() + 2),
                                );
                                match states_ensured_raw_map.get(item_id) {
                                    Some(state_ensured) => {
                                        (item_id, format!("{padding}: {state_ensured}"))
                                    }
                                    None => (item_id, format!("{padding}: <unknown>")),
                                }
                            })
                            .collect::<Vec<_>>();

                        ListNumbered::new(states_ensured_presentables)
                    };

                    output
                        .present(&(
                            Heading::new(HeadingLevel::Level1, "States Ensured"),
                            states_ensured_presentables,
                            "\n",
                        ))
                        .await?;
                } else {
                    crate::output::item_errors_present(output, errors).await?;
                    let _ = tokio::fs::write("resources.ron", format!("{resources:#?}")).await;
                }

                Ok(())
            }
            .boxed_local()
        })
        .await?;
    }};
}

use run;
