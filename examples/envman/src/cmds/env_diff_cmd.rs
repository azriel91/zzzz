use futures::FutureExt;
use peace::{
    cmd::scopes::{
        MultiProfileSingleFlowView, SingleProfileSingleFlowView,
        SingleProfileSingleFlowViewAndOutput,
    },
    flow_rt::Flow,
    fmt::{
        presentable::{Heading, HeadingLevel, ListNumberedAligned},
        PresentableExt,
    },
    profile_model::Profile,
    resource_rt::states::StateDiffs,
    rt::cmds::DiffCmd,
    rt_model::output::OutputWrite,
};

use crate::{
    cmds::{
        common::{env_man_flow, workspace},
        AppUploadCmd, CmdOpts, EnvCmd,
    },
    model::{EnvDiffSelection, EnvManError, EnvManFlow},
};

/// Shows the diff between current and goal states of the environment.
#[derive(Debug)]
pub struct EnvDiffCmd;

impl EnvDiffCmd {
    /// Shows the diff between current and goal states of the environment.
    ///
    /// # Parameters
    ///
    /// * `output`: Output to write the execution outcome.
    /// * `env_diff_selection`: Profiles to compare.
    pub async fn run<O>(
        output: &mut O,
        env_diff_selection: EnvDiffSelection,
    ) -> Result<(), EnvManError>
    where
        O: OutputWrite<EnvManError> + Send,
    {
        let workspace = workspace()?;
        let env_man_flow = env_man_flow(output, &workspace).await?;
        match env_diff_selection {
            EnvDiffSelection::CurrentAndGoal => {
                Self::active_profile_current_vs_goal(output, env_man_flow).await
            }
            EnvDiffSelection::DiffProfilesCurrent {
                profile_a,
                profile_b,
            } => Self::diff_stored(output, env_man_flow, profile_a, profile_b).await,
        }
    }

    async fn active_profile_current_vs_goal<O>(
        output: &mut O,
        env_man_flow: EnvManFlow,
    ) -> Result<(), EnvManError>
    where
        O: OutputWrite<EnvManError> + Send,
    {
        match env_man_flow {
            EnvManFlow::AppUpload => run!(output, AppUploadCmd),
            EnvManFlow::EnvDeploy => run!(output, EnvCmd),
        }
    }

    async fn diff_stored<O>(
        output: &mut O,
        env_man_flow: EnvManFlow,
        profile_a: Profile,
        profile_b: Profile,
    ) -> Result<(), EnvManError>
    where
        O: OutputWrite<EnvManError> + Send,
    {
        match env_man_flow {
            EnvManFlow::AppUpload => {
                run_multi!(output, AppUploadCmd, profile_a, profile_b)
            }
            EnvManFlow::EnvDeploy => run_multi!(output, EnvCmd, profile_a, profile_b),
        };

        Ok(())
    }

    async fn state_diffs_present<O>(
        output: &mut O,
        flow: &Flow<EnvManError>,
        state_diffs: &StateDiffs,
    ) -> Result<(), EnvManError>
    where
        O: OutputWrite<EnvManError> + Send,
    {
        let state_diffs_raw_map = &***state_diffs;

        let states_diffs_presentables: ListNumberedAligned<_, _> = flow
            .graph()
            .iter_insertion()
            .map(|item| {
                let item_id = item.id();

                let state_diff_presentable = match state_diffs_raw_map.get(item_id) {
                    Some(state_diff) => format!("{state_diff}").left_presentable(),
                    None => "<unknown>".right_presentable(),
                };

                (item_id, state_diff_presentable)
            })
            .collect::<Vec<_>>()
            .into();

        output
            .present(&(
                Heading::new(HeadingLevel::Level1, "State Diffs"),
                states_diffs_presentables,
                "\n",
            ))
            .await?;

        Ok(())
    }
}

macro_rules! run {
    ($output:ident, $flow_cmd:ident) => {{
        $flow_cmd::run($output, CmdOpts::default(), |ctx| {
            async {
                let state_diffs_outcome = DiffCmd::diff_stored(ctx).await?;

                let SingleProfileSingleFlowViewAndOutput {
                    output,
                    cmd_view: SingleProfileSingleFlowView { flow, .. },
                    ..
                } = ctx.view_and_output();

                if let Some(state_diffs) = state_diffs_outcome.value() {
                    Self::state_diffs_present(output, flow, &state_diffs).await?;
                }

                Ok(())
            }
            .boxed_local()
        })
        .await
    }};
}

macro_rules! run_multi {
    ($output:ident, $flow_cmd:ident, $profile_a:ident, $profile_b:ident) => {{
        $flow_cmd::multi_profile($output, move |ctx| {
            async move {
                let state_diffs =
                    DiffCmd::diff_current_stored(ctx, &$profile_a, &$profile_b).await?;
                let MultiProfileSingleFlowView { output, flow, .. } = ctx.view();

                Self::state_diffs_present(output, flow, &state_diffs).await?;

                Ok(())
            }
            .boxed_local()
        })
        .await?;
    }};
}

use run;
use run_multi;
