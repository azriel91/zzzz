use peace::{
    cfg::{app_name, AppName, Profile},
    rt::cmds::{StatesDiscoverCmd, StatesSavedDisplayCmd},
    rt_model::{
        output::OutputWrite, ItemSpecGraph, ItemSpecGraphBuilder, Workspace, WorkspaceSpec,
    },
};

use crate::{
    cmds::CmdCtxBuilder,
    model::{AppCycleError, EnvType},
};

/// Flow to initialize and set the default profile.
#[derive(Debug)]
pub struct ProfileInitFlow;

impl ProfileInitFlow {
    /// Stores profile init parameters.
    ///
    /// # Parameters
    ///
    /// * `output`: Output to write the execution outcome.
    /// * `profile`: Name of the profile to create.
    /// * `type`: Type of the environment.
    pub async fn run<O>(
        output: &mut O,
        profile: Profile,
        env_type: EnvType,
    ) -> Result<(), AppCycleError>
    where
        O: OutputWrite<AppCycleError>,
    {
        let workspace = Workspace::new(
            app_name!(),
            #[cfg(not(target_arch = "wasm32"))]
            WorkspaceSpec::WorkingDir,
            #[cfg(target_arch = "wasm32")]
            WorkspaceSpec::SessionStorage,
        )?;
        let graph = Self::graph()?;

        let cmd_context = CmdCtxBuilder::new(&workspace, &graph, output)
            .with_profile(profile)
            .with_env_type(env_type)
            .await?;
        StatesDiscoverCmd::exec(cmd_context).await?;

        let cmd_context = CmdCtxBuilder::new(&workspace, &graph, output)
            .with_profile_from_last_used()
            .await?;
        StatesSavedDisplayCmd::exec(cmd_context).await?;

        Ok(())
    }

    fn graph() -> Result<ItemSpecGraph<AppCycleError>, AppCycleError> {
        let graph_builder = ItemSpecGraphBuilder::<AppCycleError>::new();

        // No item specs, as we are just storing profile init params.

        Ok(graph_builder.build())
    }
}
