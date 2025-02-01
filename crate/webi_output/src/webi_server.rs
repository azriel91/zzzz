use std::{net::SocketAddr, path::Path};

use axum::Router;
use futures::stream::{self, StreamExt, TryStreamExt};
use leptos::view;
use leptos_axum::LeptosRoutes;
use peace_cmd_model::CmdExecutionId;
use peace_flow_model::FlowId;
use peace_webi_components::{App, ChildrenFn, Shell};
use peace_webi_model::{WebUiUpdate, WebiError};
use tokio::{io::AsyncWriteExt, sync::mpsc};
use tower_http::services::ServeDir;

use crate::{CmdExecSpawnCtx, CmdExecToLeptosCtx, FlowWebiFns, WebiOutput};

#[cfg(feature = "item_interactions")]
use crate::OutcomeInfoGraphCalculator;
#[cfg(feature = "item_interactions")]
use peace_webi_model::OutcomeInfoGraphVariant;

#[cfg(feature = "output_progress")]
use std::collections::HashMap;

#[cfg(feature = "output_progress")]
use peace_progress_model::CmdBlockItemInteractionType;

#[cfg(feature = "output_progress")]
use crate::ProgressInfoGraphCalculator;

/// Maximum number of `CmdExecReqT`s to queue up.
const CMD_EXEC_REQUEST_CHANNEL_LIMIT: usize = 1024;

/// Web server that runs the following work:
///
/// * UI rendering with `leptos`.
/// * `CmdExecution` through receiving requests from leptos.
/// * Updating `leptos` context data for components to render.
#[derive(Debug)]
pub struct WebiServer;

impl WebiServer {
    /// Starts the web server.
    ///
    /// ## Parameters
    ///
    /// * `socker_addr`: IP address and port to listen on.
    pub async fn start<E, CmdExecReqT>(
        app_name: String,
        socket_addr: Option<SocketAddr>,
        app_home: ChildrenFn,
        flow_webi_fns: FlowWebiFns<E, CmdExecReqT>,
    ) -> Result<(), WebiError>
    where
        E: 'static,
        CmdExecReqT: Send + 'static,
    {
        let cmd_exec_to_leptos_ctx = CmdExecToLeptosCtx::default();
        let (cmd_exec_request_tx, cmd_exec_request_rx) =
            mpsc::channel::<CmdExecReqT>(CMD_EXEC_REQUEST_CHANNEL_LIMIT);

        let flow_id = flow_webi_fns.flow.flow_id().clone();
        let webi_server_task = Self::leptos_server_start(
            app_name,
            socket_addr,
            app_home,
            cmd_exec_request_tx,
            cmd_exec_to_leptos_ctx.clone(),
            flow_id,
        );
        let cmd_execution_listener_task = Self::cmd_execution_listener(
            cmd_exec_request_rx,
            cmd_exec_to_leptos_ctx,
            flow_webi_fns,
        );

        tokio::try_join!(webi_server_task, cmd_execution_listener_task).map(|((), ())| ())
    }

    async fn cmd_execution_listener<E, CmdExecReqT>(
        mut cmd_exec_request_rx: mpsc::Receiver<CmdExecReqT>,
        cmd_exec_to_leptos_ctx: CmdExecToLeptosCtx,
        flow_webi_fns: FlowWebiFns<E, CmdExecReqT>,
    ) -> Result<(), WebiError>
    where
        E: 'static,
        CmdExecReqT: Send + 'static,
    {
        // TODO:
        //
        // 1. Listen for params specs
        // 2. Instantiate `CmdCtx`
        // 3. Calculate example `info_graph`s
        // 4. Insert into `FlowInfoGraphs`.
        let FlowWebiFns {
            flow,
            outcome_info_graph_fn,
            cmd_exec_spawn_fn,
        } = flow_webi_fns;
        let outcome_info_graph_fn = &outcome_info_graph_fn;
        #[cfg(feature = "output_progress")]
        let item_count = flow.graph().node_count();

        let CmdExecToLeptosCtx {
            flow_progress_example_info_graphs,
            flow_progress_actual_info_graphs,
            flow_outcome_example_info_graphs,
            flow_outcome_actual_info_graphs,
            mut cmd_exec_interrupt_txs,
            cmd_execution_id: cmd_execution_id_arc,
        } = cmd_exec_to_leptos_ctx;

        // TODO: remove this mock?
        // Should we have one WebiOutput for the whole server? doesn't seem right.
        let (web_ui_update_tx, _web_ui_update_rx) = mpsc::channel(128);
        let mut webi_output_mock = WebiOutput::new(web_ui_update_tx);
        let flow_spec_info = flow.flow_spec_info();
        let flow_progress_example_info_graph = flow_spec_info.to_progress_info_graph();
        let flow_outcome_example_info_graph = outcome_info_graph_fn(
            &mut webi_output_mock,
            Box::new(|flow, params_specs, resources| {
                #[cfg(all(feature = "item_interactions", feature = "item_state_example"))]
                {
                    OutcomeInfoGraphCalculator::calculate::<E>(
                        flow,
                        params_specs,
                        resources,
                        OutcomeInfoGraphVariant::Example,
                    )
                }

                #[cfg(not(all(feature = "item_interactions", feature = "item_state_example")))]
                {
                    use dot_ix_model::info_graph::InfoGraph;

                    let _flow = flow;
                    let _params_specs = params_specs;
                    let _resources = resources;

                    InfoGraph::default()
                }
            }),
        )
        .await;

        let flow_id = flow.flow_id();
        if let Ok(mut flow_progress_example_info_graphs) = flow_progress_example_info_graphs.lock()
        {
            flow_progress_example_info_graphs
                .insert(flow_id.clone(), flow_progress_example_info_graph);
        }
        if let Ok(mut flow_outcome_example_info_graphs) = flow_outcome_example_info_graphs.lock() {
            flow_outcome_example_info_graphs
                .insert(flow_id.clone(), flow_outcome_example_info_graph);
        }

        let (cmd_exec_join_handle_tx, mut cmd_exec_join_handle_rx) = mpsc::channel(128);

        let cmd_execution_starter_task = async move {
            let mut cmd_execution_id_next = CmdExecutionId::new(0u64);
            while let Some(cmd_exec_request) = cmd_exec_request_rx.recv().await {
                // Note: If we don't have a large enough buffer, we might drop updates,
                // which may mean a node appears to still be in progress when it has completed.
                let (web_ui_update_tx, web_ui_update_rx) = mpsc::channel(1024);
                let webi_output = WebiOutput::new(web_ui_update_tx);

                let webi_output_clone = webi_output.clone_without_tx();
                let CmdExecSpawnCtx {
                    interrupt_tx,
                    cmd_exec_task,
                } = cmd_exec_spawn_fn(webi_output, cmd_exec_request);

                let cmd_execution_id = cmd_execution_id_next;
                cmd_execution_id_next = CmdExecutionId::new(*cmd_execution_id + 1);

                cmd_exec_join_handle_tx
                    .send((cmd_execution_id, webi_output_clone, web_ui_update_rx))
                    .await
                    .expect("Expected `cmd_execution_receiver_task` to be running.");

                if let Some(interrupt_tx) = interrupt_tx {
                    cmd_exec_interrupt_txs.insert(cmd_execution_id, interrupt_tx);
                }

                let local_set = tokio::task::LocalSet::new();
                local_set
                    .run_until(async move {
                        let cmd_exec_join_handle = tokio::task::spawn_local(cmd_exec_task);

                        match cmd_exec_join_handle.await {
                            Ok(()) => {
                                eprintln!("`cmd_execution` completed.")
                            }
                            Err(join_error) => {
                                eprintln!(
                                    "Failed to wait for `cmd_execution` to complete. {join_error}"
                                );
                                // TODO: insert CmdExecution failed status
                            }
                        }
                    })
                    .await;
            }
        };

        let cmd_execution_receiver_task = async move {
            while let Some((cmd_execution_id, mut webi_output, mut web_ui_update_rx)) =
                cmd_exec_join_handle_rx.recv().await
            {
                if let Ok(mut cmd_execution_id_guard) = cmd_execution_id_arc.lock() {
                    *cmd_execution_id_guard = Some(cmd_execution_id);
                } else {
                    eprintln!("Unable to insert cmd_execution_id to run: {cmd_execution_id:?}");
                }

                let flow_progress_actual_info_graphs = flow_progress_actual_info_graphs.clone();
                let flow_outcome_actual_info_graphs = flow_outcome_actual_info_graphs.clone();

                #[cfg(not(feature = "output_progress"))]
                let flow_spec_info = flow_spec_info.clone();
                #[cfg(feature = "output_progress")]
                let flow_ref = &flow;

                // Update `InfoGraph`s every time `progress_update` is sent.
                let web_ui_update_task = async move {
                    // Keep track of item execution progress.
                    #[cfg(feature = "output_progress")]
                    let mut cmd_block_item_interaction_type = CmdBlockItemInteractionType::Local;
                    #[cfg(feature = "output_progress")]
                    let mut item_location_states = HashMap::with_capacity(item_count);
                    #[cfg(feature = "output_progress")]
                    let mut item_progress_statuses = HashMap::with_capacity(item_count);

                    while let Some(web_ui_update) = web_ui_update_rx.recv().await {
                        match web_ui_update {
                            #[cfg(feature = "output_progress")]
                            WebUiUpdate::CmdBlockStart {
                                cmd_block_item_interaction_type:
                                    cmd_block_item_interaction_type_next,
                            } => {
                                cmd_block_item_interaction_type =
                                    cmd_block_item_interaction_type_next;
                            }
                            #[cfg(feature = "output_progress")]
                            WebUiUpdate::ItemLocationState {
                                item_id,
                                item_location_state,
                            } => {
                                item_location_states.insert(item_id, item_location_state);
                            }
                            #[cfg(feature = "output_progress")]
                            WebUiUpdate::ItemProgressStatus {
                                item_id,
                                progress_status,
                                progress_limit: _,
                                message: _,
                            } => {
                                item_progress_statuses.insert(item_id, progress_status);
                            }
                            WebUiUpdate::Markdown { markdown_src: _ } => {
                                // TODO: render markdown on server side?
                            }
                        }

                        #[cfg(not(feature = "output_progress"))]
                        let flow_progress_actual_info_graph =
                            flow_spec_info.to_progress_info_graph();

                        #[cfg(feature = "output_progress")]
                        let flow_progress_actual_info_graph =
                            ProgressInfoGraphCalculator::calculate(
                                flow_ref,
                                &item_progress_statuses,
                            );

                        if let Ok(mut flow_progress_actual_info_graphs) =
                            flow_progress_actual_info_graphs.lock()
                        {
                            flow_progress_actual_info_graphs
                                .insert(cmd_execution_id, flow_progress_actual_info_graph);
                        }

                        #[cfg(feature = "output_progress")]
                        let item_location_states_snapshot = item_location_states.clone();
                        #[cfg(feature = "output_progress")]
                        let item_progress_statuses_snapshot = item_progress_statuses.clone();

                        let flow_outcome_actual_info_graph = outcome_info_graph_fn(
                            &mut webi_output,
                            Box::new(move |flow, params_specs, resources| {
                                #[cfg(feature = "output_progress")]
                                let item_location_states = item_location_states_snapshot.clone();
                                #[cfg(feature = "output_progress")]
                                let item_progress_statuses =
                                    item_progress_statuses_snapshot.clone();

                                #[cfg(feature = "item_interactions")]
                                {
                                    OutcomeInfoGraphCalculator::calculate::<E>(
                                        flow,
                                        params_specs,
                                        resources,
                                        OutcomeInfoGraphVariant::Current {
                                            #[cfg(feature = "output_progress")]
                                            cmd_block_item_interaction_type,
                                            #[cfg(feature = "output_progress")]
                                            item_location_states,
                                            #[cfg(feature = "output_progress")]
                                            item_progress_statuses,
                                        },
                                    )
                                }

                                #[cfg(not(feature = "item_interactions"))]
                                {
                                    use dot_ix_model::info_graph::InfoGraph;

                                    let _flow = flow;
                                    let _params_specs = params_specs;
                                    let _resources = resources;

                                    InfoGraph::default()
                                }
                            }),
                        )
                        .await;

                        if let Ok(mut flow_outcome_actual_info_graphs) =
                            flow_outcome_actual_info_graphs.lock()
                        {
                            flow_outcome_actual_info_graphs
                                .insert(cmd_execution_id, flow_outcome_actual_info_graph);
                        }
                    }
                };

                // ```rust,ignore
                // let cmd_exec_join_task = async move {
                //     match cmd_exec_join_handle.await {
                //         Ok(()) => {}
                //         Err(join_error) => {
                //             eprintln!(
                //                 "Failed to wait for `cmd_execution` to complete. {join_error}"
                //             );
                //             // TODO: insert CmdExecution failed status
                //         }
                //     }
                // };
                // ```

                // tokio::join!(web_ui_update_task, cmd_exec_join_task);

                // TODO: spawn task and go back to waiting, instead of waiting for this task, or
                // drop the txes
                web_ui_update_task.await;
            }
        };

        tokio::join!(cmd_execution_starter_task, cmd_execution_receiver_task);

        Ok(())
    }

    /// Analogous to the `main()` function in an SSR app built using
    /// `cargo-leptos`.
    ///
    /// # Parameters
    ///
    /// * `socket_addr`: IP address and port to listen on.
    async fn leptos_server_start<CmdExecReqT>(
        app_name: String,
        socket_addr: Option<SocketAddr>,
        app_home: ChildrenFn,
        cmd_exec_request_tx: mpsc::Sender<CmdExecReqT>,
        cmd_exec_to_leptos_ctx: CmdExecToLeptosCtx,
        flow_id: FlowId,
    ) -> Result<(), WebiError>
    where
        CmdExecReqT: Send + 'static,
    {
        // Setting this to None means we'll be using cargo-leptos and its env vars
        let conf = leptos::prelude::get_configuration(None)
            .map_err(|error| WebiError::LeptosConfigRead { error })?;
        let leptos_options = conf.leptos_options;
        let socket_addr = socket_addr.unwrap_or(leptos_options.site_addr);
        let routes = leptos_axum::generate_route_list({
            let app_home = app_home.clone();
            move || {
                let app_home = app_home.clone();
                view! { <App app_home /> }
            }
        });

        stream::iter(crate::assets::ASSETS.iter())
            .map(Result::<_, WebiError>::Ok)
            .try_for_each(|(path_str, contents)| async move {
                let asset_path = Path::new(path_str);
                if let Some(parent_dir) = asset_path.parent() {
                    tokio::fs::create_dir_all(parent_dir)
                        .await
                        .map_err(|error| WebiError::AssetDirCreate {
                            asset_dir: parent_dir.to_path_buf(),
                            error,
                        })?;
                }

                tokio::fs::write(asset_path, contents)
                    .await
                    .map_err(|error| WebiError::AssetWrite {
                        asset_path: asset_path.to_path_buf(),
                        error,
                    })?;

                Ok(())
            })
            .await?;

        let leptos_options_for_shell = leptos_options.clone();
        let router = Router::new()
            // serve the pkg directory
            .nest_service(
                "/pkg",
                ServeDir::new(Path::new(&*leptos_options.site_pkg_dir)),
            )
            // serve the `webi` directory
            .nest_service("/webi", ServeDir::new(Path::new("webi")))
            // serve the SSR rendered homepage
            .leptos_routes_with_context(
                &leptos_options,
                routes,
                move || {
                    // Add global state here if necessary
                    let CmdExecToLeptosCtx {
                        flow_progress_example_info_graphs,
                        flow_progress_actual_info_graphs,
                        flow_outcome_example_info_graphs,
                        flow_outcome_actual_info_graphs,
                        cmd_exec_interrupt_txs,
                        cmd_execution_id,
                    } = cmd_exec_to_leptos_ctx.clone();

                    let (flow_id, flow_id_set) = leptos::prelude::signal(flow_id.clone());

                    leptos::context::provide_context(flow_id);
                    leptos::context::provide_context(flow_id_set);
                    leptos::context::provide_context(flow_progress_example_info_graphs.clone());
                    leptos::context::provide_context(flow_progress_actual_info_graphs.clone());
                    leptos::context::provide_context(flow_outcome_example_info_graphs.clone());
                    leptos::context::provide_context(flow_outcome_actual_info_graphs.clone());
                    leptos::context::provide_context(cmd_exec_interrupt_txs.clone());
                    leptos::context::provide_context(cmd_execution_id.clone());
                    leptos::context::provide_context(cmd_exec_request_tx.clone());
                },
                move || {
                    let app_home = app_home.clone();
                    Shell(app_name.clone(), leptos_options_for_shell.clone(), app_home)
                },
            )
            .with_state(leptos_options);

        let listener = tokio::net::TcpListener::bind(socket_addr)
            .await
            .unwrap_or_else(|e| panic!("Failed to listen on {socket_addr}. Error: {e}"));
        let (Ok(()) | Err(_)) = tokio::io::stderr()
            .write_all(format!("listening on http://{}\n", socket_addr).as_bytes())
            .await;
        let (Ok(()) | Err(_)) = tokio::io::stderr()
            .write_all(
                format!(
                    "working dir: {}\n",
                    std::env::current_dir().unwrap().display()
                )
                .as_bytes(),
            )
            .await;
        axum::serve(listener, router)
            .await
            .map_err(|error| WebiError::ServerServe { socket_addr, error })
    }
}
