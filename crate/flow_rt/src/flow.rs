use peace_data::fn_graph::GraphInfo;
use peace_flow_model::{FlowId, FlowSpecInfo, ItemSpecInfo};

use crate::ItemGraph;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "item_interactions", feature = "item_state_example"))] {
        use std::collections::{BTreeMap, BTreeSet};

        use indexmap::IndexMap;
        use peace_item_interaction_model::{
            ItemInteraction,
            ItemInteractionsCurrentOrExample,
            ItemLocation,
            ItemLocationsAndInteractions,
            ItemLocationTree,
        };
        use peace_item_model::ItemId;
        use peace_params::ParamsSpecs;
        use peace_resource_rt::{resources::ts::SetUp, Resources};
    }
}

#[cfg(all(
    feature = "item_interactions",
    feature = "item_state_example",
    feature = "output_progress",
))]
use std::collections::{HashMap, HashSet};

/// A flow to manage items.
///
/// A Flow ID is strictly associated with an [`ItemGraph`], as the graph
/// contains the definitions to read and write the items' [`State`]s.
///
/// [`State`]: peace_cfg::Item::State
#[derive(Debug)]
pub struct Flow<E> {
    /// ID of this flow.
    flow_id: FlowId,
    /// Graph of [`Item`]s in this flow.
    ///
    /// [`Item`]: peace_cfg::Item
    graph: ItemGraph<E>,
}

impl<E> PartialEq for Flow<E>
where
    E: 'static,
{
    fn eq(&self, other: &Flow<E>) -> bool {
        self.flow_id == other.flow_id && self.graph == other.graph
    }
}

impl<E> Clone for Flow<E> {
    fn clone(&self) -> Self {
        Self {
            flow_id: self.flow_id.clone(),
            graph: self.graph.clone(),
        }
    }
}

impl<E> Eq for Flow<E> where E: 'static {}

impl<E> Flow<E> {
    /// Returns a new `Flow`.
    pub fn new(flow_id: FlowId, graph: ItemGraph<E>) -> Self {
        Self { flow_id, graph }
    }

    /// Returns the flow ID.
    pub fn flow_id(&self) -> &FlowId {
        &self.flow_id
    }

    /// Returns the item graph.
    pub fn graph(&self) -> &ItemGraph<E> {
        &self.graph
    }

    /// Returns a mutable reference to the item graph.
    pub fn graph_mut(&self) -> &ItemGraph<E> {
        &self.graph
    }

    /// Generates a `FlowSpecInfo` from this `Flow`'s information.
    pub fn flow_spec_info(&self) -> FlowSpecInfo
    where
        E: 'static,
    {
        let flow_id = self.flow_id.clone();
        let graph_info = GraphInfo::from_graph(&self.graph, |item_boxed| {
            let item_id = item_boxed.id().clone();
            ItemSpecInfo { item_id }
        });

        FlowSpecInfo::new(flow_id, graph_info)
    }

    // TODO: Refactor -- there is a lot of duplication between this method and
    // `item_locations_and_interactions_current`
    #[cfg(all(feature = "item_interactions", feature = "item_state_example"))]
    pub fn item_locations_and_interactions_example(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
    ) -> ItemLocationsAndInteractions
    where
        E: 'static,
    {
        // Build the flattened hierarchy.
        //
        // Regardless of how nested each `ItemLocation` is, the map will have an entry
        // for it.
        //
        // The entry key is the `ItemLocation`, and the values are a list of its direct
        // descendent `ItemLocation`s.
        //
        // This means a lot of cloning of `ItemLocation`s.

        let item_interactions_ctx = ItemInteractionsCtx {
            item_location_direct_descendents: BTreeMap::new(),
            item_to_item_interactions: IndexMap::with_capacity(self.graph().node_count()),
            // Rough estimate that each item has about 4 item locations
            //
            // * 2 `ItemLocationAncestors`s for from, 2 for to.
            // * Some may have more, but we also combine `ItemLocation`s.
            //
            // After the `ItemLocationTree`s are constructed, we'll have an accurate
            // number, but they are being constructed at the same time as this map.
            #[cfg(feature = "output_progress")]
            item_location_to_item_id_sets: HashMap::with_capacity(self.graph().node_count() * 4),
        };
        let item_interactions_ctx = self
            .graph()
            .iter()
            // Note: This will silently drop the item locations if `interactions_example` fails to
            // return.
            .filter_map(|item| {
                item.interactions_example(params_specs, resources)
                    .ok()
                    .map(|item_interactions_example| (item.id(), item_interactions_example))
            })
            .fold(
                item_interactions_ctx,
                |item_interactions_ctx, (item_id, item_interactions_example)| {
                    let ItemInteractionsCtx {
                        mut item_location_direct_descendents,
                        mut item_to_item_interactions,
                        #[cfg(feature = "output_progress")]
                        mut item_location_to_item_id_sets,
                    } = item_interactions_ctx;

                    item_location_descendents_populate(
                        &item_interactions_example,
                        &mut item_location_direct_descendents,
                    );

                    #[cfg(feature = "output_progress")]
                    item_interactions_example
                        .iter()
                        .for_each(|item_interaction| match &item_interaction {
                            ItemInteraction::Push(item_interaction_push) => item_interaction_push
                                .location_from()
                                .iter()
                                .last()
                                .into_iter()
                                .chain(item_interaction_push.location_to().iter().last())
                                .for_each(|item_location| {
                                    item_location_to_item_id_sets_insert(
                                        &mut item_location_to_item_id_sets,
                                        item_location,
                                        item_id,
                                    )
                                }),
                            ItemInteraction::Pull(item_interaction_pull) => item_interaction_pull
                                .location_client()
                                .iter()
                                .last()
                                .into_iter()
                                .chain(item_interaction_pull.location_server().iter().last())
                                .for_each(|item_location| {
                                    item_location_to_item_id_sets_insert(
                                        &mut item_location_to_item_id_sets,
                                        item_location,
                                        item_id,
                                    )
                                }),
                            ItemInteraction::Within(item_interaction_within) => {
                                item_interaction_within
                                    .location()
                                    .iter()
                                    .last()
                                    .into_iter()
                                    .for_each(|item_location| {
                                        item_location_to_item_id_sets_insert(
                                            &mut item_location_to_item_id_sets,
                                            item_location,
                                            item_id,
                                        )
                                    })
                            }
                        });

                    item_to_item_interactions
                        .insert(item_id.clone(), item_interactions_example.into_inner());

                    ItemInteractionsCtx {
                        item_location_direct_descendents,
                        item_to_item_interactions,
                        #[cfg(feature = "output_progress")]
                        item_location_to_item_id_sets,
                    }
                },
            );

        let ItemInteractionsCtx {
            item_location_direct_descendents,
            item_to_item_interactions,
            #[cfg(feature = "output_progress")]
            item_location_to_item_id_sets,
        } = item_interactions_ctx;

        let item_locations_top_level = item_location_direct_descendents
            .keys()
            .filter(|item_location| {
                // this item_location is not in any descendents
                !item_location_direct_descendents
                    .values()
                    .any(|item_location_descendents| {
                        item_location_descendents.contains(item_location)
                    })
            })
            .cloned()
            .collect::<Vec<ItemLocation>>();

        let item_locations_top_level_len = item_locations_top_level.len();
        let (_item_location_direct_descendents, item_location_trees) =
            item_locations_top_level.into_iter().fold(
                (
                    item_location_direct_descendents,
                    Vec::with_capacity(item_locations_top_level_len),
                ),
                |(mut item_location_direct_descendents, mut item_location_trees), item_location| {
                    let item_location_tree = item_location_tree_collect(
                        &mut item_location_direct_descendents,
                        item_location,
                    );
                    item_location_trees.push(item_location_tree);

                    (item_location_direct_descendents, item_location_trees)
                },
            );

        let item_location_count = item_location_trees.iter().fold(
            item_location_trees.len(),
            |item_location_count_acc, item_location_tree| {
                item_location_count_acc + item_location_tree.item_location_count()
            },
        );

        ItemLocationsAndInteractions::new(
            item_location_trees,
            item_to_item_interactions,
            item_location_count,
            #[cfg(feature = "output_progress")]
            item_location_to_item_id_sets,
        )
    }

    // TODO: Refactor -- there is a lot of duplication between this method and
    // `item_locations_and_interactions_example`
    #[cfg(all(feature = "item_interactions", feature = "item_state_example"))]
    pub fn item_locations_and_interactions_current(
        &self,
        params_specs: &ParamsSpecs,
        resources: &Resources<SetUp>,
    ) -> ItemLocationsAndInteractions
    where
        E: 'static,
    {
        // Build the flattened hierarchy.
        //
        // Regardless of how nested each `ItemLocation` is, the map will have an entry
        // for it.
        //
        // The entry key is the `ItemLocation`, and the values are a list of its direct
        // descendent `ItemLocation`s.
        //
        // This means a lot of cloning of `ItemLocation`s.

        let item_interactions_ctx = ItemInteractionsCtx {
            item_location_direct_descendents: BTreeMap::new(),
            item_to_item_interactions: IndexMap::with_capacity(self.graph().node_count()),
            // Rough estimate that each item has about 4 item locations
            //
            // * 2 `ItemLocationAncestors`s for from, 2 for to.
            // * Some may have more, but we also combine `ItemLocation`s.
            //
            // After the `ItemLocationTree`s are constructed, we'll have an accurate
            // number, but they are being constructed at the same time as this map.
            #[cfg(feature = "output_progress")]
            item_location_to_item_id_sets: HashMap::with_capacity(self.graph().node_count() * 4),
        };
        let item_interactions_ctx = self
            .graph()
            .iter()
            // Note: This will silently drop the item locations if `interactions_try_current` fails
            // to return.
            .filter_map(|item| {
                item.interactions_try_current(params_specs, resources)
                    .ok()
                    .map(|item_interactions_current_or_example| {
                        (item.id(), item_interactions_current_or_example)
                    })
            })
            .fold(
                item_interactions_ctx,
                |item_interactions_ctx, (item_id, item_interactions_current_or_example)| {
                    let ItemInteractionsCtx {
                        mut item_location_direct_descendents,
                        mut item_to_item_interactions,
                        #[cfg(feature = "output_progress")]
                        mut item_location_to_item_id_sets,
                    } = item_interactions_ctx;

                    // TODO: we need to hide the nodes if they came from `Example`.
                    let item_interactions_current_or_example =
                        match item_interactions_current_or_example {
                            ItemInteractionsCurrentOrExample::Current(
                                item_interactions_current,
                            ) => item_interactions_current.into_inner(),
                            ItemInteractionsCurrentOrExample::Example(
                                item_interactions_example,
                            ) => item_interactions_example.into_inner(),
                        };

                    item_location_descendents_populate(
                        &item_interactions_current_or_example,
                        &mut item_location_direct_descendents,
                    );

                    #[cfg(feature = "output_progress")]
                    item_interactions_current_or_example
                        .iter()
                        .for_each(|item_interaction| match &item_interaction {
                            ItemInteraction::Push(item_interaction_push) => item_interaction_push
                                .location_from()
                                .iter()
                                .last()
                                .into_iter()
                                .chain(item_interaction_push.location_to().iter().last())
                                .for_each(|item_location| {
                                    item_location_to_item_id_sets_insert(
                                        &mut item_location_to_item_id_sets,
                                        item_location,
                                        item_id,
                                    )
                                }),
                            ItemInteraction::Pull(item_interaction_pull) => item_interaction_pull
                                .location_client()
                                .iter()
                                .last()
                                .into_iter()
                                .chain(item_interaction_pull.location_server().iter().last())
                                .for_each(|item_location| {
                                    item_location_to_item_id_sets_insert(
                                        &mut item_location_to_item_id_sets,
                                        item_location,
                                        item_id,
                                    )
                                }),
                            ItemInteraction::Within(item_interaction_within) => {
                                item_interaction_within
                                    .location()
                                    .iter()
                                    .last()
                                    .into_iter()
                                    .for_each(|item_location| {
                                        item_location_to_item_id_sets_insert(
                                            &mut item_location_to_item_id_sets,
                                            item_location,
                                            item_id,
                                        )
                                    })
                            }
                        });

                    item_to_item_interactions
                        .insert(item_id.clone(), item_interactions_current_or_example);

                    ItemInteractionsCtx {
                        item_location_direct_descendents,
                        item_to_item_interactions,
                        #[cfg(feature = "output_progress")]
                        item_location_to_item_id_sets,
                    }
                },
            );

        let ItemInteractionsCtx {
            item_location_direct_descendents,
            item_to_item_interactions,
            #[cfg(feature = "output_progress")]
            item_location_to_item_id_sets,
        } = item_interactions_ctx;

        let item_locations_top_level = item_location_direct_descendents
            .keys()
            .filter(|item_location| {
                // this item_location is not in any descendents
                !item_location_direct_descendents
                    .values()
                    .any(|item_location_descendents| {
                        item_location_descendents.contains(item_location)
                    })
            })
            .cloned()
            .collect::<Vec<ItemLocation>>();

        let item_locations_top_level_len = item_locations_top_level.len();
        let (_item_location_direct_descendents, item_location_trees) =
            item_locations_top_level.into_iter().fold(
                (
                    item_location_direct_descendents,
                    Vec::with_capacity(item_locations_top_level_len),
                ),
                |(mut item_location_direct_descendents, mut item_location_trees), item_location| {
                    let item_location_tree = item_location_tree_collect(
                        &mut item_location_direct_descendents,
                        item_location,
                    );
                    item_location_trees.push(item_location_tree);

                    (item_location_direct_descendents, item_location_trees)
                },
            );

        let item_location_count = item_location_trees.iter().fold(
            item_location_trees.len(),
            |item_location_count_acc, item_location_tree| {
                item_location_count_acc + item_location_tree.item_location_count()
            },
        );

        ItemLocationsAndInteractions::new(
            item_location_trees,
            item_to_item_interactions,
            item_location_count,
            #[cfg(feature = "output_progress")]
            item_location_to_item_id_sets,
        )
    }
}

#[cfg(all(
    feature = "item_interactions",
    feature = "item_state_example",
    feature = "output_progress",
))]
fn item_location_to_item_id_sets_insert(
    item_location_to_item_id_sets: &mut HashMap<ItemLocation, HashSet<ItemId>>,
    item_location: &ItemLocation,
    item_id: &ItemId,
) {
    if let Some(item_id_set) = item_location_to_item_id_sets.get_mut(item_location) {
        item_id_set.insert(item_id.clone());
    } else {
        let mut item_id_set = HashSet::new();
        item_id_set.insert(item_id.clone());
        item_location_to_item_id_sets.insert(item_location.clone(), item_id_set);
    }
}

#[cfg(all(feature = "item_interactions", feature = "item_state_example",))]
fn item_location_descendents_populate(
    item_interactions_current_or_example: &[ItemInteraction],
    item_location_direct_descendents: &mut BTreeMap<ItemLocation, BTreeSet<ItemLocation>>,
) {
    item_interactions_current_or_example.iter().for_each(
        |item_interaction| match &item_interaction {
            ItemInteraction::Push(item_interaction_push) => {
                item_location_descendents_insert(
                    item_location_direct_descendents,
                    item_interaction_push.location_from(),
                );
                item_location_descendents_insert(
                    item_location_direct_descendents,
                    item_interaction_push.location_to(),
                );
            }
            ItemInteraction::Pull(item_interaction_pull) => {
                item_location_descendents_insert(
                    item_location_direct_descendents,
                    item_interaction_pull.location_client(),
                );
                item_location_descendents_insert(
                    item_location_direct_descendents,
                    item_interaction_pull.location_server(),
                );
            }
            ItemInteraction::Within(item_interaction_within) => {
                item_location_descendents_insert(
                    item_location_direct_descendents,
                    item_interaction_within.location(),
                );
            }
        },
    );
}

/// Recursively constructs an `ItemLocationTree`.
#[cfg(all(feature = "item_interactions", feature = "item_state_example"))]
fn item_location_tree_collect(
    item_location_direct_descendents: &mut BTreeMap<ItemLocation, BTreeSet<ItemLocation>>,
    item_location_parent: ItemLocation,
) -> ItemLocationTree {
    match item_location_direct_descendents.remove_entry(&item_location_parent) {
        Some((item_location, item_location_children)) => {
            let children = item_location_children
                .into_iter()
                .map(|item_location_child| {
                    item_location_tree_collect(
                        item_location_direct_descendents,
                        item_location_child,
                    )
                })
                .collect::<Vec<ItemLocationTree>>();
            ItemLocationTree::new(item_location, children)
        }

        // Should never be reached.
        None => ItemLocationTree::new(item_location_parent, Vec::new()),
    }
}

/// Inserts / extends the `item_location_direct_descendents` with an entry for
/// each `ItemLocation` and its direct `ItemLocation` descendents.
#[cfg(all(feature = "item_interactions", feature = "item_state_example"))]
fn item_location_descendents_insert(
    item_location_direct_descendents: &mut BTreeMap<ItemLocation, BTreeSet<ItemLocation>>,
    item_location_ancestors: &[ItemLocation],
) {
    // Each subsequent `ItemLocation` in `location_from` is a child of the previous
    // `ItemLocation`.
    let item_location_iter = item_location_ancestors.iter();
    let item_location_child_iter = item_location_ancestors.iter().skip(1);

    item_location_iter.zip(item_location_child_iter).for_each(
        |(item_location, item_location_child)| {
            // Save one clone by not using `BTreeSet::entry`
            if let Some(item_location_children) =
                item_location_direct_descendents.get_mut(item_location)
            {
                if !item_location_children.contains(item_location_child) {
                    item_location_children.insert(item_location_child.clone());
                }
            } else {
                let mut item_location_children = BTreeSet::new();
                item_location_children.insert(item_location_child.clone());
                item_location_direct_descendents
                    .insert(item_location.clone(), item_location_children);
            }

            // Add an empty set for the child `ItemLocation`.
            if !item_location_direct_descendents.contains_key(item_location_child) {
                item_location_direct_descendents
                    .insert(item_location_child.clone(), BTreeSet::new());
            }
        },
    );
}

/// Accumulates the links between
#[cfg(all(feature = "item_interactions", feature = "item_state_example"))]
struct ItemInteractionsCtx {
    /// Map from each `ItemLocation` to all of its direct descendents collected
    /// from all items.
    item_location_direct_descendents: BTreeMap<ItemLocation, BTreeSet<ItemLocation>>,
    /// Map from each item to each of its `ItemInteractions`.
    item_to_item_interactions: IndexMap<ItemId, Vec<ItemInteraction>>,
    /// Tracks the items that referred to this item location.
    #[cfg(feature = "output_progress")]
    item_location_to_item_id_sets: HashMap<ItemLocation, HashSet<ItemId>>,
}
