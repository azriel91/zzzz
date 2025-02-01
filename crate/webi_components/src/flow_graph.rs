use dot_ix::{
    model::{common::GraphvizDotTheme, info_graph::InfoGraph},
    rt::IntoGraphvizDotSrc,
    web_components::DotSvg,
};
use leptos::{
    component,
    prelude::{ClassAttribute, ElementChild, ServerFnError, Set, Transition},
    server, view, IntoView,
};

/// Renders the flow graph.
///
/// # Future
///
/// * Take in whether any execution is running. Use that info to style
///   nodes/edges.
/// * Take in values so they can be rendered, or `WriteSignal`s, to notify the
///   component that will render values about which node is selected.
#[component]
pub fn FlowGraph() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center">
            <ProgressGraph />
            <OutcomeGraph />
        </div>
    }
}

#[server]
async fn progress_info_graph_fetch() -> Result<InfoGraph, ServerFnError> {
    use leptos::prelude::{Get, ReadSignal};
    use peace_flow_model::FlowId;
    use peace_webi_model::FlowProgressInfoGraphs;

    let flow_id = leptos::prelude::use_context::<ReadSignal<FlowId>>();
    let flow_progress_info_graphs =
        leptos::prelude::use_context::<FlowProgressInfoGraphs<FlowId>>();
    let progress_info_graph = if let Some(flow_progress_info_graphs) = flow_progress_info_graphs {
        let flow_progress_info_graphs = flow_progress_info_graphs.lock().ok();

        flow_id
            .as_ref()
            .map(Get::get)
            .zip(flow_progress_info_graphs)
            .and_then(|(flow_id, flow_progress_info_graphs)| {
                flow_progress_info_graphs.get(&flow_id).cloned()
            })
            .unwrap_or_default()
    } else {
        InfoGraph::default()
    };

    Ok(progress_info_graph)
}

#[component]
fn ProgressGraph() -> impl IntoView {
    let (progress_info_graph, progress_info_graph_set) =
        leptos::prelude::signal(InfoGraph::default());
    let (dot_src_and_styles, dot_src_and_styles_set) = leptos::prelude::signal(None);

    leptos::prelude::LocalResource::new(move || async move {
        let progress_info_graph = progress_info_graph_fetch().await.unwrap_or_default();
        let dot_src_and_styles =
            IntoGraphvizDotSrc::into(&progress_info_graph, &GraphvizDotTheme::default());

        if let Ok(progress_info_graph_serialized) = serde_yaml::to_string(&progress_info_graph) {
            leptos::logging::log!("{progress_info_graph_serialized}");
        }

        progress_info_graph_set.set(progress_info_graph);
        dot_src_and_styles_set.set(Some(dot_src_and_styles));
    });

    view! {
        <Transition fallback=move || view! { <p>"Loading graph..."</p> }>
            <DotSvg
                info_graph=progress_info_graph.into()
                dot_src_and_styles=dot_src_and_styles.into()
            />
        </Transition>
    }
}

#[server]
async fn outcome_info_graph_fetch() -> Result<InfoGraph, ServerFnError> {
    use leptos::prelude::{Get, ReadSignal};
    use peace_flow_model::FlowId;
    use peace_webi_model::FlowOutcomeInfoGraphs;

    let flow_id = leptos::prelude::use_context::<ReadSignal<FlowId>>();
    let flow_outcome_info_graphs = leptos::prelude::use_context::<FlowOutcomeInfoGraphs<FlowId>>();
    let outcome_info_graph = if let Some(flow_outcome_info_graphs) = flow_outcome_info_graphs {
        let flow_outcome_info_graphs = flow_outcome_info_graphs.lock().ok();

        flow_id
            .as_ref()
            .map(Get::get)
            .zip(flow_outcome_info_graphs)
            .and_then(|(flow_id, flow_outcome_info_graphs)| {
                flow_outcome_info_graphs.get(&flow_id).cloned()
            })
            .unwrap_or_default()
    } else {
        InfoGraph::default()
    };

    Ok(outcome_info_graph)
}

#[component]
fn OutcomeGraph() -> impl IntoView {
    let (outcome_info_graph, outcome_info_graph_set) =
        leptos::prelude::signal(InfoGraph::default());
    let (dot_src_and_styles, dot_src_and_styles_set) = leptos::prelude::signal(None);

    leptos::prelude::LocalResource::new(move || async move {
        let outcome_info_graph = outcome_info_graph_fetch().await.unwrap_or_default();
        let dot_src_and_styles =
            IntoGraphvizDotSrc::into(&outcome_info_graph, &GraphvizDotTheme::default());

        if let Ok(outcome_info_graph_serialized) = serde_yaml::to_string(&outcome_info_graph) {
            leptos::logging::log!("{outcome_info_graph_serialized}");
        }

        outcome_info_graph_set.set(outcome_info_graph);
        dot_src_and_styles_set.set(Some(dot_src_and_styles));
    });

    view! {
        <Transition fallback=move || view! { <p>"Loading graph..."</p> }>
            <DotSvg
                info_graph=outcome_info_graph.into()
                dot_src_and_styles=dot_src_and_styles.into()
            />
        </Transition>
    }
}
