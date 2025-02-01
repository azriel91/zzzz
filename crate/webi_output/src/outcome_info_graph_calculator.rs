use std::{collections::HashMap, marker::PhantomData, str::FromStr};

use dot_ix_model::{
    common::{
        graphviz_attrs::{EdgeDir, PackMode, PackModeFlag},
        AnyId, EdgeId, Edges, GraphvizAttrs, NodeHierarchy, NodeId, NodeNames, TagId, TagItems,
        TagNames, TagStyles,
    },
    info_graph::{GraphDir, InfoGraph},
    theme::{AnyIdOrDefaults, CssClassPartials, Theme, ThemeAttr, ThemeStyles},
};
use indexmap::IndexMap;
use peace_flow_rt::Flow;
use peace_item_interaction_model::{
    ItemInteraction, ItemInteractionPull, ItemInteractionPush, ItemInteractionWithin, ItemLocation,
    ItemLocationTree, ItemLocationType, ItemLocationsAndInteractions,
};
use peace_item_model::ItemId;
use peace_params::ParamsSpecs;
use peace_resource_rt::{resources::ts::SetUp, Resources};
use peace_webi_model::OutcomeInfoGraphVariant;
use smallvec::SmallVec;

#[cfg(feature = "output_progress")]
use std::{collections::HashSet, ops::ControlFlow};

#[cfg(feature = "output_progress")]
use peace_item_interaction_model::{ItemLocationState, ItemLocationStateInProgress};
#[cfg(feature = "output_progress")]
use peace_progress_model::{CmdBlockItemInteractionType, ProgressComplete, ProgressStatus};

/// Calculates the example / actual `InfoGraph` for a flow's outcome.
#[derive(Debug)]
pub struct OutcomeInfoGraphCalculator;

impl OutcomeInfoGraphCalculator {
    /// Returns the calculated `InfoGraph`.
    pub fn calculate<E>(
        flow: &Flow<E>,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
        outcome_info_graph_variant: OutcomeInfoGraphVariant,
    ) -> InfoGraph
    where
        E: 'static,
    {
        let item_locations_and_interactions = match &outcome_info_graph_variant {
            OutcomeInfoGraphVariant::Example => {
                flow.item_locations_and_interactions_example(params_specs, resources)
            }
            OutcomeInfoGraphVariant::Current { .. } => {
                flow.item_locations_and_interactions_current(params_specs, resources)
            }
        };

        calculate_info_graph(
            flow,
            outcome_info_graph_variant,
            item_locations_and_interactions,
        )
    }
}

fn calculate_info_graph<E>(
    flow: &Flow<E>,
    outcome_info_graph_variant: OutcomeInfoGraphVariant,
    item_locations_and_interactions: ItemLocationsAndInteractions,
) -> InfoGraph
where
    E: 'static,
{
    let item_count = flow.graph().node_count();
    let ItemLocationsAndInteractions {
        item_location_trees,
        item_to_item_interactions,
        item_location_count,
        #[cfg(feature = "output_progress")]
        item_location_to_item_id_sets,
    } = item_locations_and_interactions;

    let node_id_mappings_and_hierarchy = node_id_mappings_and_hierarchy(
        &item_location_trees,
        item_location_count,
        #[cfg(feature = "output_progress")]
        &item_location_to_item_id_sets,
    );
    let NodeIdMappingsAndHierarchy {
        node_id_to_item_locations,
        mut item_location_to_node_id_segments,
        node_hierarchy,
        #[cfg(feature = "output_progress")]
        node_id_to_item_id_sets,
    } = node_id_mappings_and_hierarchy;

    let node_names = node_id_to_item_locations.iter().fold(
        NodeNames::with_capacity(item_location_count),
        |mut node_names, (node_id, item_location)| {
            node_names.insert(node_id.clone(), item_location.name().to_string());
            node_names
        },
    );

    let tags = match &outcome_info_graph_variant {
        OutcomeInfoGraphVariant::Example => {
            let tags = flow.graph().iter_insertion().fold(
                TagNames::with_capacity(item_count),
                |mut tags, item| {
                    let tag_name = item.interactions_tag_name();

                    // For some reason using `TagId::new(item.id().as_str())` causes an error to be
                    // highlighted on `flow.graph()`, rather than referring to `item.id()` as the
                    // cause of an extended borrow.
                    let item_id = item.id();
                    let tag_id = TagId::try_from(format!("tag_{item_id}"))
                        .expect("Expected `tag_id` from `item_id` to be valid.");

                    tags.insert(tag_id, tag_name);

                    tags
                },
            );

            Some(tags)
        }
        OutcomeInfoGraphVariant::Current { .. } => None,
    };

    // 1. Each item interaction knows the `ItemLocation`s
    // 2. We need to be able to translate from an `ItemLocation`, to the `NodeId`s
    //    that we need to link as edges.
    // 3. We have a way to map from `ItemLocation` to `NodeId` using the
    //    `node_id_from_item_location` function.
    // 4. So, either we calculate the `NodeId` from each `ItemLocation` in each
    //    interaction again, or `ItemLocation` must implement `Hash` and `Eq`, and
    //    look it up.
    // 5. It already implements `Hash` and `Eq`, so let's construct a
    //    `Map<ItemLocation, NodeId>`.
    // 6. Then we can iterate through `item_to_item_interactions`, and for each
    //    `ItemLocation`, look up the map from 5, and add an edge.
    let item_interactions_process_ctx = ItemInteractionsProcessCtx {
        outcome_info_graph_variant: &outcome_info_graph_variant,
        item_count,
        item_location_count,
        item_to_item_interactions: &item_to_item_interactions,
        node_id_to_item_locations: &node_id_to_item_locations,
        item_location_to_node_id_segments: &mut item_location_to_node_id_segments,
    };
    let item_interactions_processed = process_item_interactions(item_interactions_process_ctx);
    let ItemInteractionsProcessed {
        edges,
        graphviz_attrs,
        mut theme,
        tag_items,
        tag_styles_focus,
    } = item_interactions_processed;

    theme_styles_augment(
        &item_location_trees,
        &node_id_to_item_locations,
        &mut theme,
        #[cfg(feature = "output_progress")]
        &outcome_info_graph_variant,
        #[cfg(feature = "output_progress")]
        &node_id_to_item_id_sets,
    );

    let mut info_graph = InfoGraph::default()
        .with_direction(GraphDir::Vertical)
        .with_hierarchy(node_hierarchy)
        .with_node_names(node_names)
        .with_edges(edges)
        .with_graphviz_attrs(graphviz_attrs)
        .with_theme(theme)
        .with_css(String::from(
            r#"
@keyframes node-stroke-dashoffset-move {
  0%   { stroke-dashoffset: 0; }
  100% { stroke-dashoffset: 30; }
}
@keyframes stroke-dashoffset-move {
  0%   { stroke-dashoffset: 136; }
  100% { stroke-dashoffset: 0; }
}
@keyframes stroke-dashoffset-move-request {
  0%   { stroke-dashoffset: 0; }
  100% { stroke-dashoffset: 198; }
}
@keyframes stroke-dashoffset-move-response {
  0%   { stroke-dashoffset: 0; }
  100% { stroke-dashoffset: -218; }
}
"#,
        ));

    if let Some(tags) = tags {
        info_graph = info_graph.with_tags(tags)
    }
    if let Some(tag_items) = tag_items {
        info_graph = info_graph.with_tag_items(tag_items)
    }
    if let Some(tag_styles_focus) = tag_styles_focus {
        info_graph = info_graph.with_tag_styles_focus(tag_styles_focus)
    }

    info_graph
}

/// Adds styles for nodes based on what kind of [`ItemLocation`] they represent,
/// and their progress status.
fn theme_styles_augment(
    item_location_trees: &[ItemLocationTree],
    node_id_to_item_locations: &IndexMap<NodeId, &ItemLocation>,
    theme: &mut Theme,
    #[cfg(feature = "output_progress")] outcome_info_graph_variant: &OutcomeInfoGraphVariant,
    #[cfg(feature = "output_progress")] node_id_to_item_id_sets: &HashMap<NodeId, HashSet<&ItemId>>,
) {
    // Use light styling for `ItemLocationType::Group` nodes.
    let mut css_class_partials_light = CssClassPartials::with_capacity(10);
    css_class_partials_light.insert(ThemeAttr::StrokeStyle, "dotted".to_string());
    css_class_partials_light.insert(ThemeAttr::StrokeShadeNormal, "300".to_string());
    css_class_partials_light.insert(ThemeAttr::StrokeShadeHover, "300".to_string());
    css_class_partials_light.insert(ThemeAttr::StrokeShadeFocus, "400".to_string());
    css_class_partials_light.insert(ThemeAttr::StrokeShadeActive, "500".to_string());
    css_class_partials_light.insert(ThemeAttr::FillShadeNormal, "50".to_string());
    css_class_partials_light.insert(ThemeAttr::FillShadeHover, "50".to_string());
    css_class_partials_light.insert(ThemeAttr::FillShadeFocus, "100".to_string());
    css_class_partials_light.insert(ThemeAttr::FillShadeActive, "200".to_string());

    node_id_to_item_locations
        .iter()
        .for_each(|(node_id, item_location)| {
            let css_class_partials = match item_location.r#type() {
                ItemLocationType::Host => {
                    // Specially colour some known hosts.
                    match item_location.name() {
                        ItemLocation::LOCALHOST => {
                            let mut css_class_partials = css_class_partials_light.clone();
                            css_class_partials.insert(ThemeAttr::ShapeColor, "blue".to_string());

                            Some(css_class_partials)
                        }
                        "github.com" => {
                            let mut css_class_partials = css_class_partials_light.clone();
                            css_class_partials.insert(ThemeAttr::ShapeColor, "purple".to_string());

                            Some(css_class_partials)
                        }
                        _ => {
                            // Not all hosts should be styled light -- only the ones that are top
                            // level. i.e. if the host is inside a group, then it should likely be
                            // styled darker.
                            if item_location_trees
                                .iter()
                                .map(ItemLocationTree::item_location)
                                .any(|item_location_top_level| {
                                    item_location_top_level == *item_location
                                })
                            {
                                Some(css_class_partials_light.clone())
                            } else {
                                None
                            }
                        }
                    }
                }
                ItemLocationType::Group => Some(css_class_partials_light.clone()),
                ItemLocationType::Path => {
                    #[cfg(not(feature = "output_progress"))]
                    {
                        None
                    }

                    #[cfg(feature = "output_progress")]
                    {
                        if let OutcomeInfoGraphVariant::Current {
                            cmd_block_item_interaction_type,
                            item_location_states,
                            item_progress_statuses,
                        } = outcome_info_graph_variant
                        {
                            let cmd_block_item_interaction_type = *cmd_block_item_interaction_type;
                            // 1. For each of the item IDs that referred to this node
                            node_id_to_item_id_sets
                                .get(node_id)
                                // 2. Look up their statuses
                                .and_then(|referrer_item_ids| {
                                    // When we have multiple referrers referring to the same item
                                    // location, we need to prioritize `ItemLocationState::Exists`
                                    // over `ItemLocationState::NotExists`.
                                    //
                                    // This is because:
                                    //
                                    // * For ensure, a predecessor would have created the item
                                    //   beforehand, so we don't want a successor's `NotExists`
                                    //   state to hide the node. e.g. a file download before
                                    //   uploading it somewhere else.
                                    //
                                    // * For clean, the successor's destination would be removed,
                                    //   but not its source. e.g. the upload would remove the
                                    //   destination file, and not the source, which would later be
                                    //   removed by the predecessor.
                                    //
                                    // Which means we need to prioritize the styles from the most
                                    // recent completed / in-progress `referrer_item_id`.
                                    let (ControlFlow::Continue(item_location_state)
                                    | ControlFlow::Break(item_location_state)) = referrer_item_ids
                                        .iter()
                                        .filter_map(|referrer_item_id| {
                                            item_location_states.get(*referrer_item_id).copied()
                                        })
                                        .try_fold(
                                            ItemLocationState::NotExists,
                                            |_item_location_state_acc, item_location_state| {
                                                match item_location_state {
                                                    ItemLocationState::Exists => {
                                                        ControlFlow::Break(
                                                            ItemLocationState::Exists,
                                                        )
                                                    }
                                                    ItemLocationState::NotExists => {
                                                        ControlFlow::Continue(
                                                            ItemLocationState::NotExists,
                                                        )
                                                    }
                                                }
                                            },
                                        );

                                    let (ControlFlow::Continue(progress_status)
                                    | ControlFlow::Break(progress_status)) = referrer_item_ids
                                        .iter()
                                        .filter_map(|referrer_item_id| {
                                            item_progress_statuses.get(*referrer_item_id).cloned()
                                        })
                                        .try_fold(
                                            ProgressStatus::Initialized,
                                            |_progress_status_acc, progress_status| {
                                                match progress_status {
                                                    ProgressStatus::Initialized => {
                                                        ControlFlow::Continue(progress_status)
                                                    }
                                                    ProgressStatus::Interrupted => {
                                                        ControlFlow::Continue(progress_status)
                                                    }
                                                    ProgressStatus::ExecPending => {
                                                        ControlFlow::Continue(progress_status)
                                                    }
                                                    ProgressStatus::Queued => {
                                                        ControlFlow::Continue(progress_status)
                                                    }
                                                    ProgressStatus::Running
                                                    | ProgressStatus::RunningStalled
                                                    | ProgressStatus::UserPending => {
                                                        ControlFlow::Break(progress_status)
                                                    }
                                                    ProgressStatus::Complete(
                                                        ProgressComplete::Success,
                                                    ) => ControlFlow::Continue(progress_status),
                                                    ProgressStatus::Complete(
                                                        ProgressComplete::Fail,
                                                    ) => ControlFlow::Break(progress_status),
                                                }
                                            },
                                        );

                                    node_css_class_partials(
                                        cmd_block_item_interaction_type,
                                        item_location_state,
                                        progress_status,
                                    )
                                })
                        } else {
                            None
                        }
                    }
                }
            };

            if let Some(css_class_partials) = css_class_partials {
                theme.styles.insert(
                    AnyIdOrDefaults::AnyId(AnyId::from(node_id.clone())),
                    css_class_partials,
                );
            }
        });
}

#[cfg(feature = "output_progress")]
fn node_css_class_partials(
    cmd_block_item_interaction_type: CmdBlockItemInteractionType,
    item_location_state: ItemLocationState,
    progress_status: ProgressStatus,
) -> Option<CssClassPartials> {
    // 3. If any of them are running or complete, then it should be visible.
    let item_location_state_in_progress = ItemLocationStateInProgress::from(
        cmd_block_item_interaction_type,
        item_location_state,
        progress_status,
    );

    match item_location_state_in_progress {
        ItemLocationStateInProgress::NotExists => {
            let mut css_class_partials = CssClassPartials::with_capacity(1);
            css_class_partials.insert(ThemeAttr::Extra, "opacity-[0.15]".to_string());
            Some(css_class_partials)
        }
        ItemLocationStateInProgress::NotExistsError => {
            let mut css_class_partials = CssClassPartials::with_capacity(2);
            css_class_partials.insert(ThemeAttr::ShapeColor, "red".to_string());
            css_class_partials.insert(ThemeAttr::StrokeStyle, "dashed".to_string());
            Some(css_class_partials)
        }
        ItemLocationStateInProgress::DiscoverInProgress => {
            let mut css_class_partials = CssClassPartials::with_capacity(3);
            css_class_partials.insert(ThemeAttr::ShapeColor, "yellow".to_string());
            css_class_partials.insert(ThemeAttr::StrokeStyle, "dashed".to_string());
            css_class_partials.insert(
                ThemeAttr::Animate,
                "[node-stroke-dashoffset-move_1s_linear_infinite]".to_string(),
            );
            Some(css_class_partials)
        }
        ItemLocationStateInProgress::DiscoverError => {
            let mut css_class_partials = CssClassPartials::with_capacity(3);
            css_class_partials.insert(ThemeAttr::ShapeColor, "amber".to_string());
            css_class_partials.insert(ThemeAttr::StrokeStyle, "dashed".to_string());
            css_class_partials.insert(
                ThemeAttr::Animate,
                "[node-stroke-dashoffset-move_1s_linear_infinite]".to_string(),
            );
            Some(css_class_partials)
        }
        ItemLocationStateInProgress::CreateInProgress => {
            let mut css_class_partials = CssClassPartials::with_capacity(3);
            css_class_partials.insert(ThemeAttr::ShapeColor, "blue".to_string());
            css_class_partials.insert(ThemeAttr::StrokeStyle, "dashed".to_string());
            css_class_partials.insert(
                ThemeAttr::Animate,
                "[node-stroke-dashoffset-move_1s_linear_infinite]".to_string(),
            );
            Some(css_class_partials)
        }
        ItemLocationStateInProgress::ModificationInProgress => {
            let mut css_class_partials = CssClassPartials::with_capacity(3);
            css_class_partials.insert(ThemeAttr::ShapeColor, "blue".to_string());
            css_class_partials.insert(ThemeAttr::StrokeStyle, "dashed".to_string());
            css_class_partials.insert(
                ThemeAttr::Animate,
                "[node-stroke-dashoffset-move_1s_linear_infinite]".to_string(),
            );
            Some(css_class_partials)
        }
        ItemLocationStateInProgress::ExistsOk => None,
        ItemLocationStateInProgress::ExistsError => {
            let mut css_class_partials = CssClassPartials::with_capacity(1);
            css_class_partials.insert(ThemeAttr::ShapeColor, "red".to_string());
            Some(css_class_partials)
        }
    }
}

/// Calculates edges and styles from `ItemInteraction`s.
///
/// # Code
///
/// Currently the code goes through the `ItemInteraction`s, and populates the
/// `Edges`, `Theme`, and `GraphvizAttrs`. This isn't as "clean" as iterating
/// over the `ItemInteraction`s per attribute that is to be computed, but
/// perhaps populating the different structures per `ItemInteraction` is more
/// manageable than remembering to update multiple functions.
fn process_item_interactions(
    item_interactions_process_ctx: ItemInteractionsProcessCtx<'_, '_>,
) -> ItemInteractionsProcessed {
    let ItemInteractionsProcessCtx {
        outcome_info_graph_variant,
        item_count,
        item_location_count,
        item_to_item_interactions,
        node_id_to_item_locations,
        item_location_to_node_id_segments,
    } = item_interactions_process_ctx;

    let edges = Edges::with_capacity(item_location_count);
    let mut graphviz_attrs = GraphvizAttrs::new().with_edge_minlen_default(3);
    graphviz_attrs.pack_mode = PackMode::Array {
        flags: vec![PackModeFlag::T],
        number: None,
    };
    let mut theme = Theme::new();
    theme.styles.insert(AnyIdOrDefaults::EdgeDefaults, {
        let mut css_class_partials = CssClassPartials::with_capacity(1);
        css_class_partials.insert(ThemeAttr::Visibility, "invisible".to_string());
        css_class_partials
    });

    match outcome_info_graph_variant {
        OutcomeInfoGraphVariant::Example => {
            let item_interactions_processed_example = ItemInteractionsProcessedExample {
                edges,
                graphviz_attrs,
                tag_items: TagItems::with_capacity(item_count),
                tag_styles_focus: TagStyles::new(),
            };

            let item_interactions_processed_example = process_item_interactions_example(
                item_to_item_interactions,
                item_interactions_processed_example,
                node_id_to_item_locations,
                item_location_to_node_id_segments,
            );
            let ItemInteractionsProcessedExample {
                edges,
                graphviz_attrs,
                tag_items,
                tag_styles_focus,
            } = item_interactions_processed_example;

            ItemInteractionsProcessed {
                edges,
                graphviz_attrs,
                theme,
                tag_items: Some(tag_items),
                tag_styles_focus: Some(tag_styles_focus),
            }
        }
        OutcomeInfoGraphVariant::Current {
            #[cfg(feature = "output_progress")]
                cmd_block_item_interaction_type: _,
            #[cfg(feature = "output_progress")]
                item_location_states: _,
            #[cfg(feature = "output_progress")]
            item_progress_statuses,
        } => {
            let item_interactions_processed_current = ItemInteractionsProcessedCurrent {
                edges,
                graphviz_attrs,
                theme,
                #[cfg(feature = "output_progress")]
                item_progress_statuses,
                marker: PhantomData,
            };

            let item_interactions_processed_current = process_item_interactions_current(
                item_to_item_interactions,
                item_interactions_processed_current,
                node_id_to_item_locations,
                item_location_to_node_id_segments,
            );

            let ItemInteractionsProcessedCurrent {
                edges,
                graphviz_attrs,
                theme,
                #[cfg(feature = "output_progress")]
                    item_progress_statuses: _,
                marker: PhantomData,
            } = item_interactions_processed_current;

            ItemInteractionsProcessed {
                edges,
                graphviz_attrs,
                theme,
                tag_items: None,
                tag_styles_focus: None,
            }
        }
    }
}

/// Processes `ItemInteraction`s from all items for an example `InfoGraph`
/// diagram.
///
/// This means:
///
/// 1. Each node should be fully visible.
/// 2. Edges should be visible when a tag is clicked.
fn process_item_interactions_example<'item_location>(
    item_to_item_interactions: &'item_location IndexMap<ItemId, Vec<ItemInteraction>>,
    item_interactions_processed_example: ItemInteractionsProcessedExample,
    node_id_to_item_locations: &IndexMap<NodeId, &'item_location ItemLocation>,
    item_location_to_node_id_segments: &mut HashMap<&'item_location ItemLocation, String>,
) -> ItemInteractionsProcessedExample {
    item_to_item_interactions
        .iter()
        // The capacity could be worked out through the sum of all `ItemInteraction`s.
        //
        // For now we just use the `item_location_count` as a close approximation.
        .fold(
            item_interactions_processed_example,
            // Use `item_id` to compute `tags` and `tag_items`.
            |item_interactions_processed_example, (item_id, item_interactions)| {
                let ItemInteractionsProcessedExample {
                    mut edges,
                    mut graphviz_attrs,
                    mut tag_items,
                    mut tag_styles_focus,
                } = item_interactions_processed_example;

                let tag_id = TagId::try_from(format!("tag_{item_id}"))
                    .expect("Expected `tag_id` from `item_id` to be valid.");
                let tag_id = &tag_id;

                item_interactions.iter().for_each(|item_interaction| {
                    let item_interactions_processing_ctx = ItemInteractionsProcessingCtxExample {
                        node_id_to_item_locations,
                        item_location_to_node_id_segments,
                        edges: &mut edges,
                        tag_items: &mut tag_items,
                        tag_id,
                        tag_styles_focus: &mut tag_styles_focus,
                    };

                    match item_interaction {
                        ItemInteraction::Push(item_interaction_push) => {
                            process_item_interaction_push_example(
                                item_interactions_processing_ctx,
                                item_interaction_push,
                            );
                        }
                        ItemInteraction::Pull(item_interaction_pull) => {
                            process_item_interaction_pull_example(
                                item_interactions_processing_ctx,
                                &mut graphviz_attrs,
                                item_interaction_pull,
                            );
                        }
                        ItemInteraction::Within(item_interaction_within) => {
                            process_item_interaction_within_example(
                                item_interactions_processing_ctx,
                                item_interaction_within,
                            );
                        }
                    }
                });

                ItemInteractionsProcessedExample {
                    edges,
                    graphviz_attrs,
                    tag_items,
                    tag_styles_focus,
                }
            },
        )
}

/// Inserts an edge between the `from` and `to` nodes of an
/// [`ItemInteractionPush`].
fn process_item_interaction_push_example<'item_location>(
    item_interactions_processing_ctx: ItemInteractionsProcessingCtxExample<'_, 'item_location>,
    item_interaction_push: &'item_location ItemInteractionPush,
) {
    let ItemInteractionsProcessingCtxExample {
        node_id_to_item_locations,
        item_location_to_node_id_segments,
        edges,
        tag_items,
        tag_id,
        tag_styles_focus,
    } = item_interactions_processing_ctx;
    // Use the innermost node from the interaction.
    // The `NodeId` for the item location is the longest node ID that contains all
    // of the `node_id_segment`s of the selected item location's ancestors.
    let node_id_from = {
        let node_id_from = node_id_from_item_location(item_location_to_node_id_segments, || {
            item_interaction_push.location_from().iter()
        });

        node_id_with_ancestor_find(node_id_to_item_locations, node_id_from)
    };

    // Use the innermost node.
    let node_id_to = {
        let node_id_to = node_id_from_item_location(item_location_to_node_id_segments, || {
            item_interaction_push.location_to().iter()
        });

        node_id_with_ancestor_find(node_id_to_item_locations, node_id_to)
    };

    let edge_id = EdgeId::from_str(&format!("{node_id_from}___{node_id_to}"))
        .expect("Expected edge ID from item location ID to be valid for `edge_id`.");
    edges.insert(edge_id.clone(), [node_id_from.clone(), node_id_to.clone()]);

    if let Some(any_ids) = tag_items.get_mut(tag_id) {
        any_ids.push(AnyId::from(node_id_from.clone()));
        any_ids.push(AnyId::from(node_id_to.clone()));
        any_ids.push(AnyId::from(edge_id.clone()));
    } else {
        let any_ids = vec![
            AnyId::from(node_id_from.clone()),
            AnyId::from(node_id_to.clone()),
            AnyId::from(edge_id.clone()),
        ];
        tag_items.insert(tag_id.clone(), any_ids);
    }

    let css_class_partials = item_interaction_push_css_class_partials(true);

    if let Some(theme_styles) = tag_styles_focus.get_mut(tag_id) {
        theme_styles.insert(
            AnyIdOrDefaults::AnyId(AnyId::from(edge_id)),
            css_class_partials,
        );
    } else {
        let mut theme_styles = ThemeStyles::with_capacity(1);
        theme_styles.insert(
            AnyIdOrDefaults::AnyId(AnyId::from(edge_id)),
            css_class_partials,
        );
        tag_styles_focus.insert(tag_id.clone(), theme_styles);
    }
}

/// Inserts an edge between the `client` and `server` nodes of an
/// [`ItemInteractionPull`].
fn process_item_interaction_pull_example<'item_location>(
    item_interactions_processing_ctx: ItemInteractionsProcessingCtxExample<'_, 'item_location>,
    graphviz_attrs: &mut GraphvizAttrs,
    item_interaction_pull: &'item_location ItemInteractionPull,
) {
    let ItemInteractionsProcessingCtxExample {
        node_id_to_item_locations,
        item_location_to_node_id_segments,
        edges,
        tag_items,
        tag_id,
        tag_styles_focus,
    } = item_interactions_processing_ctx;

    // Use the outermost `ItemLocationType::Host` node.
    let node_id_client = {
        let item_location_ancestors_iter = || {
            let mut host_found = false;
            let mut location_from_iter = item_interaction_pull.location_client().iter();
            std::iter::from_fn(move || {
                if host_found {
                    return None;
                }

                let item_location = location_from_iter.next();
                if let Some(item_location) = item_location.as_ref() {
                    host_found = item_location.r#type() == ItemLocationType::Host;
                }
                item_location
            })
            .fuse()
        };

        let node_id_client = node_id_from_item_location(
            item_location_to_node_id_segments,
            item_location_ancestors_iter,
        );

        node_id_with_ancestor_find(node_id_to_item_locations, node_id_client)
    };

    // Use the innermost node, as that's where the file is written to.
    let node_id_client_file = {
        let node_id_client_file =
            node_id_from_item_location(item_location_to_node_id_segments, || {
                item_interaction_pull.location_client().iter()
            });

        node_id_with_ancestor_find(node_id_to_item_locations, node_id_client_file)
    };

    // Use the innermost node.
    let node_id_server = {
        let node_id_server = node_id_from_item_location(item_location_to_node_id_segments, || {
            item_interaction_pull.location_server().iter()
        });

        node_id_with_ancestor_find(node_id_to_item_locations, node_id_server)
    };

    let edge_id_request =
        EdgeId::from_str(&format!("{node_id_client}___{node_id_server}___request"))
            .expect("Expected edge ID from item location ID to be valid for `edge_id_request`.");
    edges.insert(
        edge_id_request.clone(),
        [node_id_server.clone(), node_id_client.clone()],
    );

    let edge_id_response = EdgeId::from_str(&format!(
        "{node_id_client_file}___{node_id_server}___response"
    ))
    .expect("Expected edge ID from item location ID to be valid for `edge_id_response`.");
    edges.insert(
        edge_id_response.clone(),
        [node_id_server.clone(), node_id_client_file.clone()],
    );

    graphviz_attrs
        .edge_dirs
        .insert(edge_id_request.clone(), EdgeDir::Back);

    let css_class_partials_request = item_interaction_pull_request_css_class_partials(true);
    let css_class_partials_response = item_interaction_pull_response_css_class_partials(true);

    if let Some(any_ids) = tag_items.get_mut(tag_id) {
        any_ids.push(AnyId::from(node_id_server.clone()));
        any_ids.push(AnyId::from(node_id_client_file.clone()));
        any_ids.push(AnyId::from(edge_id_request.clone()));
        any_ids.push(AnyId::from(edge_id_response.clone()));
    } else {
        let any_ids = vec![
            AnyId::from(node_id_server.clone()),
            AnyId::from(node_id_client_file.clone()),
            AnyId::from(edge_id_request.clone()),
            AnyId::from(edge_id_response.clone()),
        ];
        tag_items.insert(tag_id.clone(), any_ids);
    }

    if let Some(theme_styles) = tag_styles_focus.get_mut(tag_id) {
        theme_styles.insert(
            AnyIdOrDefaults::AnyId(AnyId::from(edge_id_request)),
            css_class_partials_request,
        );
        theme_styles.insert(
            AnyIdOrDefaults::AnyId(AnyId::from(edge_id_response)),
            css_class_partials_response,
        );
    } else {
        let mut theme_styles = ThemeStyles::with_capacity(2);
        theme_styles.insert(
            AnyIdOrDefaults::AnyId(AnyId::from(edge_id_request)),
            css_class_partials_request,
        );
        theme_styles.insert(
            AnyIdOrDefaults::AnyId(AnyId::from(edge_id_response)),
            css_class_partials_response,
        );
        tag_styles_focus.insert(tag_id.clone(), theme_styles);
    }
}

/// Indicates the nodes that are being waited upon by [`ItemInteractionWithin`].
fn process_item_interaction_within_example<'item_location>(
    item_interactions_processing_ctx: ItemInteractionsProcessingCtxExample<'_, 'item_location>,
    item_interaction_within: &'item_location ItemInteractionWithin,
) {
    let ItemInteractionsProcessingCtxExample {
        node_id_to_item_locations,
        item_location_to_node_id_segments,
        edges: _,
        tag_items,
        tag_id,
        tag_styles_focus,
    } = item_interactions_processing_ctx;

    // Use the outermost `ItemLocationType::Host` node.
    let node_id = {
        let item_location_ancestors_iter = || {
            let mut host_found = false;
            let mut location_from_iter = item_interaction_within.location().iter();
            std::iter::from_fn(move || {
                if host_found {
                    return None;
                }

                let item_location = location_from_iter.next();
                if let Some(item_location) = item_location.as_ref() {
                    host_found = item_location.r#type() == ItemLocationType::Host;
                }
                item_location
            })
            .fuse()
        };

        let node_id_client = node_id_from_item_location(
            item_location_to_node_id_segments,
            item_location_ancestors_iter,
        );

        node_id_with_ancestor_find(node_id_to_item_locations, node_id_client)
    };

    let css_class_partials = item_interaction_within_css_class_partials();

    if let Some(any_ids) = tag_items.get_mut(tag_id) {
        any_ids.push(AnyId::from(node_id.clone()));
    } else {
        let any_ids = vec![AnyId::from(node_id.clone())];
        tag_items.insert(tag_id.clone(), any_ids);
    }

    if let Some(theme_styles) = tag_styles_focus.get_mut(tag_id) {
        theme_styles.insert(
            AnyIdOrDefaults::AnyId(AnyId::from(node_id)),
            css_class_partials,
        );
    } else {
        let mut theme_styles = ThemeStyles::with_capacity(1);
        theme_styles.insert(
            AnyIdOrDefaults::AnyId(AnyId::from(node_id)),
            css_class_partials,
        );
        tag_styles_focus.insert(tag_id.clone(), theme_styles);
    }
}

/// Inserts an edge between the `from` and `to` nodes of an
/// [`ItemInteractionPush`].
fn process_item_interaction_push_current<'item_location>(
    item_interactions_processing_ctx: ItemInteractionsProcessingCtxCurrent<'_, 'item_location>,
    item_interaction_push: &'item_location ItemInteractionPush,
) {
    let ItemInteractionsProcessingCtxCurrent {
        node_id_to_item_locations,
        item_location_to_node_id_segments,
        edges,
        theme,
        #[cfg(feature = "output_progress")]
        progress_status,
    } = item_interactions_processing_ctx;
    // Use the innermost node from the interaction.
    // The `NodeId` for the item location is the longest node ID that contains all
    // of the `node_id_segment`s of the selected item location's ancestors.
    let node_id_from = {
        let node_id_from = node_id_from_item_location(item_location_to_node_id_segments, || {
            item_interaction_push.location_from().iter()
        });

        node_id_with_ancestor_find(node_id_to_item_locations, node_id_from)
    };

    // Use the innermost node.
    let node_id_to = {
        let node_id_to = node_id_from_item_location(item_location_to_node_id_segments, || {
            item_interaction_push.location_to().iter()
        });

        node_id_with_ancestor_find(node_id_to_item_locations, node_id_to)
    };

    let edge_id = EdgeId::from_str(&format!("{node_id_from}___{node_id_to}"))
        .expect("Expected edge ID from item location ID to be valid for `edge_id`.");
    edges.insert(edge_id.clone(), [node_id_from.clone(), node_id_to.clone()]);

    #[cfg(feature = "output_progress")]
    let edge_visible = matches!(
        progress_status,
        ProgressStatus::Running | ProgressStatus::RunningStalled | ProgressStatus::UserPending
    );
    #[cfg(not(feature = "output_progress"))]
    let edge_visible = false;
    let css_class_partials = item_interaction_push_css_class_partials(edge_visible);

    theme.styles.insert(
        AnyIdOrDefaults::AnyId(AnyId::from(edge_id)),
        css_class_partials,
    );
}

/// Inserts an edge between the `client` and `server` nodes of an
/// [`ItemInteractionPull`].
fn process_item_interaction_pull_current<'item_location>(
    item_interactions_processing_ctx: ItemInteractionsProcessingCtxCurrent<'_, 'item_location>,
    graphviz_attrs: &mut GraphvizAttrs,
    item_interaction_pull: &'item_location ItemInteractionPull,
) {
    let ItemInteractionsProcessingCtxCurrent {
        node_id_to_item_locations,
        item_location_to_node_id_segments,
        edges,
        theme,
        #[cfg(feature = "output_progress")]
        progress_status,
    } = item_interactions_processing_ctx;

    // Use the outermost `ItemLocationType::Host` node.
    let node_id_client = {
        let item_location_ancestors_iter = || {
            let mut host_found = false;
            let mut location_from_iter = item_interaction_pull.location_client().iter();
            std::iter::from_fn(move || {
                if host_found {
                    return None;
                }

                let item_location = location_from_iter.next();
                if let Some(item_location) = item_location.as_ref() {
                    host_found = item_location.r#type() == ItemLocationType::Host;
                }
                item_location
            })
            .fuse()
        };

        let node_id_client = node_id_from_item_location(
            item_location_to_node_id_segments,
            item_location_ancestors_iter,
        );

        node_id_with_ancestor_find(node_id_to_item_locations, node_id_client)
    };

    // Use the innermost node, as that's where the file is written to.
    let node_id_client_file = {
        let node_id_client_file =
            node_id_from_item_location(item_location_to_node_id_segments, || {
                item_interaction_pull.location_client().iter()
            });

        node_id_with_ancestor_find(node_id_to_item_locations, node_id_client_file)
    };

    // Use the innermost node.
    let node_id_server = {
        let node_id_server = node_id_from_item_location(item_location_to_node_id_segments, || {
            item_interaction_pull.location_server().iter()
        });

        node_id_with_ancestor_find(node_id_to_item_locations, node_id_server)
    };

    let edge_id_request =
        EdgeId::from_str(&format!("{node_id_client}___{node_id_server}___request"))
            .expect("Expected edge ID from item location ID to be valid for `edge_id_request`.");
    edges.insert(
        edge_id_request.clone(),
        [node_id_server.clone(), node_id_client.clone()],
    );

    let edge_id_response = EdgeId::from_str(&format!(
        "{node_id_client_file}___{node_id_server}___response"
    ))
    .expect("Expected edge ID from item location ID to be valid for `edge_id_response`.");
    edges.insert(
        edge_id_response.clone(),
        [node_id_server.clone(), node_id_client_file.clone()],
    );

    graphviz_attrs
        .edge_dirs
        .insert(edge_id_request.clone(), EdgeDir::Back);

    #[cfg(feature = "output_progress")]
    let edge_visible = matches!(
        progress_status,
        ProgressStatus::Running | ProgressStatus::RunningStalled | ProgressStatus::UserPending
    );
    #[cfg(not(feature = "output_progress"))]
    let edge_visible = false;
    let css_class_partials_request = item_interaction_pull_request_css_class_partials(edge_visible);
    let css_class_partials_response =
        item_interaction_pull_response_css_class_partials(edge_visible);

    theme.styles.insert(
        AnyIdOrDefaults::AnyId(AnyId::from(edge_id_request)),
        css_class_partials_request,
    );
    theme.styles.insert(
        AnyIdOrDefaults::AnyId(AnyId::from(edge_id_response)),
        css_class_partials_response,
    );
}

/// Indicates the nodes that are being waited upon by [`ItemInteractionWithin`].
fn process_item_interaction_within_current<'item_location>(
    item_interactions_processing_ctx: ItemInteractionsProcessingCtxCurrent<'_, 'item_location>,
    item_interaction_within: &'item_location ItemInteractionWithin,
) {
    let ItemInteractionsProcessingCtxCurrent {
        node_id_to_item_locations,
        item_location_to_node_id_segments,
        edges: _,
        theme,
        #[cfg(feature = "output_progress")]
        progress_status,
    } = item_interactions_processing_ctx;

    // Use the outermost `ItemLocationType::Host` node.
    let node_id = {
        let item_location_ancestors_iter = || {
            let mut host_found = false;
            let mut location_from_iter = item_interaction_within.location().iter();
            std::iter::from_fn(move || {
                if host_found {
                    return None;
                }

                let item_location = location_from_iter.next();
                if let Some(item_location) = item_location.as_ref() {
                    host_found = item_location.r#type() == ItemLocationType::Host;
                }
                item_location
            })
            .fuse()
        };

        let node_id_client = node_id_from_item_location(
            item_location_to_node_id_segments,
            item_location_ancestors_iter,
        );

        node_id_with_ancestor_find(node_id_to_item_locations, node_id_client)
    };

    #[cfg(feature = "output_progress")]
    let animate_node = matches!(
        progress_status,
        ProgressStatus::Running | ProgressStatus::RunningStalled | ProgressStatus::UserPending
    );
    #[cfg(not(feature = "output_progress"))]
    let animate_node = false;
    if animate_node {
        let css_class_partials = item_interaction_within_css_class_partials();

        theme.styles.insert(
            AnyIdOrDefaults::AnyId(AnyId::from(node_id)),
            css_class_partials,
        );
    }
}

/// Processes `ItemInteraction`s from all items for an example `InfoGraph`
/// diagram.
///
/// This means:
///
/// 1. Each node should be fully visible.
/// 2. Edges should be visible when a tag is clicked.
fn process_item_interactions_current<'item_state, 'item_location>(
    item_to_item_interactions: &'item_location IndexMap<ItemId, Vec<ItemInteraction>>,
    item_interactions_processed_current: ItemInteractionsProcessedCurrent<'item_state>,
    node_id_to_item_locations: &IndexMap<NodeId, &'item_location ItemLocation>,
    item_location_to_node_id_segments: &mut HashMap<&'item_location ItemLocation, String>,
) -> ItemInteractionsProcessedCurrent<'item_state> {
    item_to_item_interactions
        .iter()
        // The capacity could be worked out through the sum of all `ItemInteraction`s.
        //
        // For now we just use the `item_location_count` as a close approximation.
        .fold(
            item_interactions_processed_current,
            |item_interactions_processed_current, (item_id, item_interactions)| {
                let ItemInteractionsProcessedCurrent {
                    mut edges,
                    mut graphviz_attrs,
                    mut theme,
                    #[cfg(feature = "output_progress")]
                    item_progress_statuses,
                    marker: PhantomData,
                } = item_interactions_processed_current;

                #[cfg(feature = "output_progress")]
                let progress_status = item_progress_statuses
                    .get(item_id)
                    .cloned()
                    .unwrap_or(ProgressStatus::Initialized);

                #[cfg(not(feature = "output_progress"))]
                let _item_id = item_id;

                item_interactions.iter().for_each(|item_interaction| {
                    #[cfg(feature = "output_progress")]
                    let progress_status = progress_status.clone();

                    let item_interactions_processing_ctx = ItemInteractionsProcessingCtxCurrent {
                        node_id_to_item_locations,
                        item_location_to_node_id_segments,
                        edges: &mut edges,
                        theme: &mut theme,
                        #[cfg(feature = "output_progress")]
                        progress_status,
                    };

                    match item_interaction {
                        ItemInteraction::Push(item_interaction_push) => {
                            process_item_interaction_push_current(
                                item_interactions_processing_ctx,
                                item_interaction_push,
                            );
                        }
                        ItemInteraction::Pull(item_interaction_pull) => {
                            process_item_interaction_pull_current(
                                item_interactions_processing_ctx,
                                &mut graphviz_attrs,
                                item_interaction_pull,
                            );
                        }
                        ItemInteraction::Within(item_interaction_within) => {
                            process_item_interaction_within_current(
                                item_interactions_processing_ctx,
                                item_interaction_within,
                            );
                        }
                    }
                });

                ItemInteractionsProcessedCurrent {
                    edges,
                    graphviz_attrs,
                    theme,
                    #[cfg(feature = "output_progress")]
                    item_progress_statuses,
                    marker: PhantomData,
                }
            },
        )
}

/// Returns [`CssClassPartials`] for the edge between the `from` and `to`
/// [`ItemLocation`]s of an [`ItemInteractionPush`].
fn item_interaction_push_css_class_partials(visible: bool) -> CssClassPartials {
    let mut css_class_partials = CssClassPartials::with_capacity(6);
    if visible {
        css_class_partials.insert(ThemeAttr::Visibility, "visible".to_string());
    }
    css_class_partials.insert(
        ThemeAttr::Animate,
        "[stroke-dashoffset-move_1s_linear_infinite]".to_string(),
    );
    css_class_partials.insert(ThemeAttr::ShapeColor, "blue".to_string());
    css_class_partials.insert(
        ThemeAttr::StrokeStyle,
        "dasharray:0,40,1,2,1,2,2,2,4,2,8,2,20,50".to_string(),
    );
    css_class_partials.insert(ThemeAttr::StrokeShadeNormal, "600".to_string());
    css_class_partials.insert(ThemeAttr::FillShadeNormal, "500".to_string());
    css_class_partials
}

/// Returns [`CssClassPartials`] for the edge for the `client` to `server`
/// [`ItemLocation`] of an [`ItemInteractionPull`].
fn item_interaction_pull_request_css_class_partials(visible: bool) -> CssClassPartials {
    let mut css_class_partials_request = CssClassPartials::with_capacity(7);
    if visible {
        css_class_partials_request.insert(ThemeAttr::Visibility, "visible".to_string());
    }
    css_class_partials_request.insert(
        ThemeAttr::Animate,
        "[stroke-dashoffset-move-request_1.5s_linear_infinite]".to_string(),
    );
    css_class_partials_request.insert(ThemeAttr::ShapeColor, "blue".to_string());
    css_class_partials_request.insert(
        ThemeAttr::StrokeStyle,
        "dasharray:0,50,12,2,4,2,2,2,1,2,1,120".to_string(),
    );
    css_class_partials_request.insert(ThemeAttr::StrokeWidth, "[1px]".to_string());
    css_class_partials_request.insert(ThemeAttr::StrokeShadeNormal, "600".to_string());
    css_class_partials_request.insert(ThemeAttr::FillShadeNormal, "500".to_string());
    css_class_partials_request
}

/// Returns [`CssClassPartials`] for the edge for the `server` to `client`
/// [`ItemLocation`] of an [`ItemInteractionPull`].
fn item_interaction_pull_response_css_class_partials(visible: bool) -> CssClassPartials {
    let mut css_class_partials_response = CssClassPartials::with_capacity(7);
    if visible {
        css_class_partials_response.insert(ThemeAttr::Visibility, "visible".to_string());
    }
    css_class_partials_response.insert(
        ThemeAttr::Animate,
        "[stroke-dashoffset-move-response_1.5s_linear_infinite]".to_string(),
    );
    css_class_partials_response.insert(ThemeAttr::ShapeColor, "blue".to_string());
    css_class_partials_response.insert(
        ThemeAttr::StrokeStyle,
        "dasharray:0,120,1,2,1,2,2,2,4,2,8,2,20,50".to_string(),
    );
    css_class_partials_response.insert(ThemeAttr::StrokeWidth, "[2px]".to_string());
    css_class_partials_response.insert(ThemeAttr::StrokeShadeNormal, "600".to_string());
    css_class_partials_response.insert(ThemeAttr::FillShadeNormal, "500".to_string());
    css_class_partials_response
}

/// Returns [`CssClassPartials`] for the node for an [`ItemLocation`] of an
/// [`ItemInteractionWithin`].
fn item_interaction_within_css_class_partials() -> CssClassPartials {
    let mut css_class_partials = CssClassPartials::with_capacity(4);
    css_class_partials.insert(
        ThemeAttr::Animate,
        "[stroke-dashoffset-move_1s_linear_infinite]".to_string(),
    );
    css_class_partials.insert(ThemeAttr::ShapeColor, "blue".to_string());
    css_class_partials.insert(ThemeAttr::StrokeStyle, "dashed".to_string());
    css_class_partials
}

/// Returns the node ID that ends with the calculated node ID, in case another
/// `Item` has provided an ancestor as context.
///
/// Not sure if we need to find the longest node ID (which incurs one more
/// sort), but the current implementation just returns the first match.
fn node_id_with_ancestor_find(
    node_id_to_item_locations: &IndexMap<NodeId, &ItemLocation>,
    node_id_from: NodeId,
) -> NodeId {
    node_id_to_item_locations
        .keys()
        .find(|node_id| node_id.ends_with(node_id_from.as_str()))
        .cloned()
        .unwrap_or(node_id_from)
}

/// Returns a map of `NodeId` to the `ItemLocation` it is associated with, and
/// the `NodeHierarchy` constructed from the `ItemLocationTree`s.
fn node_id_mappings_and_hierarchy<'item_location>(
    item_location_trees: &'item_location [ItemLocationTree],
    item_location_count: usize,
    #[cfg(feature = "output_progress")] item_location_to_item_id_sets: &'item_location HashMap<
        ItemLocation,
        HashSet<ItemId>,
    >,
) -> NodeIdMappingsAndHierarchy<'item_location> {
    let node_id_mappings_and_hierarchy = NodeIdMappingsAndHierarchy {
        node_id_to_item_locations: IndexMap::with_capacity(item_location_count),
        item_location_to_node_id_segments: HashMap::with_capacity(item_location_count),
        node_hierarchy: NodeHierarchy::with_capacity(item_location_trees.len()),
        #[cfg(feature = "output_progress")]
        node_id_to_item_id_sets: HashMap::with_capacity(item_location_count),
    };

    item_location_trees.iter().fold(
        node_id_mappings_and_hierarchy,
        |mut node_id_mappings_and_hierarchy, item_location_tree| {
            let NodeIdMappingsAndHierarchy {
                node_id_to_item_locations,
                item_location_to_node_id_segments,
                node_hierarchy,
                #[cfg(feature = "output_progress")]
                node_id_to_item_id_sets,
            } = &mut node_id_mappings_and_hierarchy;

            let item_location = item_location_tree.item_location();

            // Probably won't go more than 8 deep.
            let mut item_location_ancestors = SmallVec::<[&ItemLocation; 8]>::new();
            item_location_ancestors.push(item_location);

            let node_id = node_id_from_item_location(item_location_to_node_id_segments, || {
                item_location_ancestors.clone().into_iter()
            });

            node_id_to_item_locations.insert(node_id.clone(), item_location);

            // Track the items that this node is associated with.
            #[cfg(feature = "output_progress")]
            {
                let referrer_item_ids = item_location_to_item_id_sets.get(item_location);
                if let Some(referrer_item_ids) = referrer_item_ids {
                    if let Some(node_referrer_item_ids) = node_id_to_item_id_sets.get_mut(&node_id)
                    {
                        node_referrer_item_ids.extend(referrer_item_ids);
                    } else {
                        let mut node_referrer_item_ids =
                            HashSet::with_capacity(referrer_item_ids.len());
                        node_referrer_item_ids.extend(referrer_item_ids.iter());
                        node_id_to_item_id_sets.insert(node_id.clone(), node_referrer_item_ids);
                    }
                }
            }

            let node_hierarchy_top_level = node_hierarchy_build_and_item_location_insert(
                item_location_tree,
                node_id_to_item_locations,
                item_location_to_node_id_segments,
                item_location_ancestors,
                #[cfg(feature = "output_progress")]
                item_location_to_item_id_sets,
                #[cfg(feature = "output_progress")]
                node_id_to_item_id_sets,
            );
            node_hierarchy.insert(node_id, node_hierarchy_top_level);

            node_id_mappings_and_hierarchy
        },
    )
}

/// Returns the [`NodeId`] for the given [`ItemLocation`].
///
/// This is computed from all of the node ID segments from all of the node's
/// ancestors.
fn node_id_from_item_location<'item_location, F, I>(
    item_location_to_node_id_segments: &mut HashMap<&'item_location ItemLocation, String>,
    item_location_ancestors_iter_fn: F,
) -> NodeId
where
    F: Fn() -> I,
    I: Iterator<Item = &'item_location ItemLocation>,
{
    let item_location_ancestors_iter_for_capacity = item_location_ancestors_iter_fn();
    let capacity = item_location_ancestors_iter_for_capacity.fold(
        0usize,
        |capacity_acc, item_location_ancestor| {
            let node_id_segment = item_location_to_node_id_segments
                .entry(item_location_ancestor)
                .or_insert_with(move || node_id_segment_from_item_location(item_location_ancestor));

            capacity_acc + node_id_segment.len() + 3
        },
    );
    let mut node_id = item_location_ancestors_iter_fn()
        .filter_map(|item_location_ancestor| {
            item_location_to_node_id_segments.get(item_location_ancestor)
        })
        .fold(
            String::with_capacity(capacity),
            |mut node_id_buffer, node_id_segment| {
                node_id_buffer.push_str(node_id_segment);
                node_id_buffer.push_str("___");
                node_id_buffer
            },
        );

    node_id.truncate(node_id.len() - "___".len());

    NodeId::try_from(node_id).expect("Expected node ID from item location ID to be valid.")
}

/// Returns a `&str` segment that can be used as part of the `NodeId` for the
/// given [`ItemLocation`].
///
/// An [`ItemLocation`]'s [`NodeId`] needs to be joined with the parent segments
/// from its ancestors, otherwise two different `path__path_to_file`
/// [`ItemLocation`]s may be accidentally merged.
fn node_id_segment_from_item_location(item_location: &ItemLocation) -> String {
    let item_location_type = match item_location.r#type() {
        ItemLocationType::Group => "group",
        ItemLocationType::Host => "host",
        ItemLocationType::Path => "path",
    };
    let name = item_location.name();
    let name_transformed =
        name.chars()
            .fold(String::with_capacity(name.len()), |mut name_acc, c| {
                match c {
                    'a'..='z' | '0'..='9' => name_acc.push(c),
                    'A'..='Z' => c.to_lowercase().for_each(|c| name_acc.push(c)),
                    _ => name_acc.push_str("__"),
                }
                name_acc
            });

    format!("{item_location_type}___{name_transformed}")
}

/// Recursively constructs the `NodeHierarchy` and populates a map to facilitate
/// calculation of `InfoGraph` representing `ItemLocation`s.
///
/// Each `Node` corresponds to one `ItemLocation`.
///
/// Because:
///
/// * Each `ItemInteraction` can include multiple `ItemLocation`s -- both nested
///   and separate, and
/// * We need to style each node
///
/// it is useful to be able to retrieve the `ItemLocation` for each `Node` we
/// are adding attributes for.
fn node_hierarchy_build_and_item_location_insert<'item_location>(
    item_location_tree: &'item_location ItemLocationTree,
    node_id_to_item_locations: &mut IndexMap<NodeId, &'item_location ItemLocation>,
    item_location_to_node_id_segments: &mut HashMap<&'item_location ItemLocation, String>,
    item_location_ancestors: SmallVec<[&'item_location ItemLocation; 8]>,
    #[cfg(feature = "output_progress")] item_location_to_item_id_sets: &'item_location HashMap<
        ItemLocation,
        HashSet<ItemId>,
    >,
    #[cfg(feature = "output_progress")] node_id_to_item_id_sets: &mut HashMap<
        NodeId,
        HashSet<&'item_location ItemId>,
    >,
) -> NodeHierarchy {
    let mut node_hierarchy = NodeHierarchy::with_capacity(item_location_tree.children().len());

    item_location_tree
        .children()
        .iter()
        .for_each(|child_item_location_tree| {
            let child_item_location = child_item_location_tree.item_location();
            let mut child_item_location_ancestors = item_location_ancestors.clone();
            child_item_location_ancestors.push(child_item_location);

            let child_node_id =
                node_id_from_item_location(item_location_to_node_id_segments, || {
                    child_item_location_ancestors.clone().into_iter()
                });
            node_id_to_item_locations.insert(child_node_id.clone(), child_item_location);

            // Track the items that this node is associated with.
            #[cfg(feature = "output_progress")]
            {
                let referrer_item_ids = item_location_to_item_id_sets.get(child_item_location);
                if let Some(referrer_item_ids) = referrer_item_ids {
                    if let Some(node_referrer_item_ids) =
                        node_id_to_item_id_sets.get_mut(&child_node_id)
                    {
                        node_referrer_item_ids.extend(referrer_item_ids);
                    } else {
                        let mut node_referrer_item_ids =
                            HashSet::with_capacity(referrer_item_ids.len());
                        node_referrer_item_ids.extend(referrer_item_ids.iter());
                        node_id_to_item_id_sets
                            .insert(child_node_id.clone(), node_referrer_item_ids);
                    }
                }
            }

            let child_hierarchy = node_hierarchy_build_and_item_location_insert(
                child_item_location_tree,
                node_id_to_item_locations,
                item_location_to_node_id_segments,
                child_item_location_ancestors,
                #[cfg(feature = "output_progress")]
                item_location_to_item_id_sets,
                #[cfg(feature = "output_progress")]
                node_id_to_item_id_sets,
            );
            node_hierarchy.insert(child_node_id, child_hierarchy);
        });

    node_hierarchy
}

struct NodeIdMappingsAndHierarchy<'item_location> {
    node_id_to_item_locations: IndexMap<NodeId, &'item_location ItemLocation>,
    item_location_to_node_id_segments: HashMap<&'item_location ItemLocation, String>,
    node_hierarchy: NodeHierarchy,
    /// Mapping to Item IDs that interact with the `ItemLocation` that the
    /// `NodeId` represents.
    #[cfg(feature = "output_progress")]
    node_id_to_item_id_sets: HashMap<NodeId, HashSet<&'item_location ItemId>>,
}

struct ItemInteractionsProcessCtx<'f, 'item_location> {
    outcome_info_graph_variant: &'f OutcomeInfoGraphVariant,
    item_count: usize,
    item_location_count: usize,
    item_to_item_interactions: &'item_location IndexMap<ItemId, Vec<ItemInteraction>>,
    node_id_to_item_locations: &'f IndexMap<NodeId, &'item_location ItemLocation>,
    item_location_to_node_id_segments: &'f mut HashMap<&'item_location ItemLocation, String>,
}

struct ItemInteractionsProcessingCtxExample<'f, 'item_location> {
    node_id_to_item_locations: &'f IndexMap<NodeId, &'item_location ItemLocation>,
    item_location_to_node_id_segments: &'f mut HashMap<&'item_location ItemLocation, String>,
    edges: &'f mut Edges,
    tag_items: &'f mut TagItems,
    tag_id: &'f TagId,
    tag_styles_focus: &'f mut TagStyles,
}

struct ItemInteractionsProcessingCtxCurrent<'f, 'item_location> {
    node_id_to_item_locations: &'f IndexMap<NodeId, &'item_location ItemLocation>,
    item_location_to_node_id_segments: &'f mut HashMap<&'item_location ItemLocation, String>,
    edges: &'f mut Edges,
    theme: &'f mut Theme,
    #[cfg(feature = "output_progress")]
    progress_status: ProgressStatus,
}

struct ItemInteractionsProcessedExample {
    edges: Edges,
    graphviz_attrs: GraphvizAttrs,
    tag_items: TagItems,
    tag_styles_focus: TagStyles,
}

struct ItemInteractionsProcessedCurrent<'item_state> {
    edges: Edges,
    graphviz_attrs: GraphvizAttrs,
    theme: Theme,
    /// Progress of each item.
    #[cfg(feature = "output_progress")]
    item_progress_statuses: &'item_state HashMap<ItemId, ProgressStatus>,
    marker: PhantomData<&'item_state ()>,
}

struct ItemInteractionsProcessed {
    edges: Edges,
    graphviz_attrs: GraphvizAttrs,
    theme: Theme,
    tag_items: Option<TagItems>,
    tag_styles_focus: Option<TagStyles>,
}
