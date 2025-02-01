use std::fmt::{self, Debug};

use dot_ix_model::info_graph::InfoGraph;
use futures::future::LocalBoxFuture;
use peace_flow_rt::Flow;
use peace_params::ParamsSpecs;
use peace_resource_rt::{resources::ts::SetUp, Resources};

use crate::{CmdExecSpawnCtx, WebiOutput};

/// Functions to work with `Flow` from the [`WebiOutput`].
///
/// [`WebiOutput`]: crate::WebiOutput
pub struct FlowWebiFns<E, CmdExecReqT> {
    /// Flow to work with.
    pub flow: Flow<E>,
    /// Function to create an `InfoGraph`.
    ///
    /// # Design
    ///
    /// This circumvents the need to pass around the specific `CmdCtx` type by
    /// getting the tool developer to instantiate the `CmdCtx`, then pass the
    /// relevant parameters to the function that we pass in.
    #[allow(clippy::type_complexity)]
    pub outcome_info_graph_fn: Box<
        dyn Fn(
            &mut WebiOutput,
            Box<dyn Fn(&Flow<E>, &ParamsSpecs, &Resources<SetUp>) -> InfoGraph>,
        ) -> LocalBoxFuture<InfoGraph>,
    >,
    /// Function to spawn a `CmdExecution`.
    ///
    /// # Design
    ///
    /// Because passing around a `CmdCtx` with all its type parameters is
    /// technically high cost, all of the `CmdCtx` instantiation logic, and
    /// `*Cmd::run` invocations are hidden behind a plain function interface.
    ///
    /// Currently we only take in one function. In the future this should take
    /// in a `Map<CmdExecutionRequest, CmdExecutionSpawnFn>`
    pub cmd_exec_spawn_fn: Box<dyn Fn(WebiOutput, CmdExecReqT) -> CmdExecSpawnCtx>,
}

impl<E, CmdExecReqT> fmt::Debug for FlowWebiFns<E, CmdExecReqT>
where
    E: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cmd_exec_req_t_type_name = std::any::type_name::<CmdExecReqT>();

        f.debug_struct("FlowWebiFns")
            .field("flow", &self.flow)
            .field(
                "outcome_info_graph_fn",
                &stringify!(
                    Box<
                        dyn Fn(
                            &mut WebiOutput,
                            Box<dyn Fn(&Flow<E>, &ParamsSpecs, &Resources<SetUp>) -> InfoGraph>,
                        ) -> LocalBoxFuture<InfoGraph>,
                    >
                ),
            )
            .field(
                "cmd_exec_spawn_fn",
                &format!("Box<dyn Fn(WebiOutput, {cmd_exec_req_t_type_name}) -> CmdExecSpawnCtx>"),
            )
            .finish()
    }
}
