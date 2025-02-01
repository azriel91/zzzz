use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use interruptible::InterruptSignal;
use peace_cmd_model::CmdExecutionId;
use peace_flow_model::FlowId;
use tokio::sync::mpsc;

use peace_webi_model::{FlowOutcomeInfoGraphs, FlowProgressInfoGraphs};

/// The shared memory to write to to communicate between the `CmdExecution`s and
/// `leptos`.
#[derive(Clone, Debug, Default)]
pub struct CmdExecToLeptosCtx {
    /// The example progress `InfoGraph` for all `CmdExecution`s.
    ///
    /// Shared memory for `Map<CmdExecutionId, InfoGraph>`.
    pub flow_progress_example_info_graphs: FlowProgressInfoGraphs<FlowId>,
    /// The actual progress `InfoGraph` for all `CmdExecution`s.
    ///
    /// Shared memory for `Map<CmdExecutionId, InfoGraph>`.
    pub flow_progress_actual_info_graphs: FlowProgressInfoGraphs<CmdExecutionId>,
    /// The example outcome `InfoGraph` for all `CmdExecution`s.
    ///
    /// Shared memory for `Map<CmdExecutionId, InfoGraph>`.
    pub flow_outcome_example_info_graphs: FlowOutcomeInfoGraphs<FlowId>,
    /// The actual outcome `InfoGraph` for all `CmdExecution`s.
    ///
    /// Shared memory for `Map<CmdExecutionId, InfoGraph>`.
    pub flow_outcome_actual_info_graphs: FlowOutcomeInfoGraphs<CmdExecutionId>,
    /// The interrupt channel sender for each `CmdExecution`.
    pub cmd_exec_interrupt_txs: HashMap<CmdExecutionId, mpsc::Sender<InterruptSignal>>,
    /// The `cmd_execution_id` of the active `CmdExecution`.
    ///
    /// # Design
    ///
    /// This should go away, and instead be a value returned to the client and
    /// stored in the URL.
    pub cmd_execution_id: Arc<Mutex<Option<CmdExecutionId>>>,
}

impl CmdExecToLeptosCtx {
    /// Returns a new `CmdExecToLeptosCtx`.
    pub fn new(
        flow_progress_example_info_graphs: FlowProgressInfoGraphs<FlowId>,
        flow_progress_actual_info_graphs: FlowProgressInfoGraphs<CmdExecutionId>,
        flow_outcome_example_info_graphs: FlowOutcomeInfoGraphs<FlowId>,
        flow_outcome_actual_info_graphs: FlowOutcomeInfoGraphs<CmdExecutionId>,
        cmd_exec_interrupt_txs: HashMap<CmdExecutionId, mpsc::Sender<InterruptSignal>>,
        cmd_execution_id: Arc<Mutex<Option<CmdExecutionId>>>,
    ) -> Self {
        Self {
            flow_progress_example_info_graphs,
            flow_progress_actual_info_graphs,
            flow_outcome_example_info_graphs,
            flow_outcome_actual_info_graphs,
            cmd_exec_interrupt_txs,
            cmd_execution_id,
        }
    }
}
