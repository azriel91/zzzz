#![allow(clippy::type_complexity)]

use std::{fmt::Debug, hash::Hash};

use futures::stream::{StreamExt, TryStreamExt};
use interruptible::Interruptibility;
use own::{OwnedOrMutRef, OwnedOrRef};
use peace_flow_rt::{Flow, ItemGraph};
use peace_item_model::ItemId;
use peace_params::ParamsSpecs;
use peace_resource_rt::{
    internal::{FlowParamsFile, ProfileParamsFile, WorkspaceParamsFile},
    paths::ParamsSpecsFile,
    resources::ts::{Empty, SetUp},
    Resources,
};
use peace_rt_model::{
    fn_graph::resman::Resource,
    params::{FlowParams, ProfileParams, WorkspaceParams},
    ParamsSpecsSerializer, ParamsSpecsTypeReg, StatesTypeReg, Storage, Workspace,
    WorkspaceInitializer,
};
use serde::{de::DeserializeOwned, Serialize};

pub use self::{
    multi_profile_no_flow_builder::MultiProfileNoFlowBuilder,
    multi_profile_single_flow_builder::MultiProfileSingleFlowBuilder,
    no_profile_no_flow_builder::NoProfileNoFlowBuilder,
    single_profile_no_flow_builder::SingleProfileNoFlowBuilder,
    single_profile_single_flow_builder::SingleProfileSingleFlowBuilder,
};

use crate::ctx::CmdCtxBuilderTypes;

mod multi_profile_no_flow_builder;
mod multi_profile_single_flow_builder;
mod no_profile_no_flow_builder;
mod single_profile_no_flow_builder;
mod single_profile_single_flow_builder;

/// Collects parameters and initializes values relevant to the built [`CmdCtx`].
#[derive(Debug)]
pub struct CmdCtxBuilder<'ctx, CmdCtxBuilderTypesT, ScopeBuilder>
where
    CmdCtxBuilderTypesT: CmdCtxBuilderTypes,
{
    /// Output endpoint to return values / errors, and write progress
    /// information to.
    ///
    /// See [`OutputWrite`].
    ///
    /// [`OutputWrite`]: peace_rt_model_core::OutputWrite
    output: OwnedOrMutRef<'ctx, CmdCtxBuilderTypesT::Output>,
    /// The interrupt channel receiver if this `CmdExecution` is interruptible.
    interruptibility: Interruptibility<'static>,
    /// Workspace that the `peace` tool runs in.
    workspace: OwnedOrRef<'ctx, Workspace>,
    /// Runtime borrow-checked typemap of data available to the command context.
    resources: Resources<Empty>,
    /// Data held while building `CmdCtx`.
    scope_builder: ScopeBuilder,
}

/// Serializes workspace params to storage.
async fn workspace_params_serialize<WorkspaceParamsK>(
    workspace_params: &WorkspaceParams<WorkspaceParamsK>,
    storage: &Storage,
    workspace_params_file: &WorkspaceParamsFile,
) -> Result<(), peace_rt_model::Error>
where
    WorkspaceParamsK:
        Clone + Debug + Eq + Hash + DeserializeOwned + Serialize + Send + Sync + 'static,
{
    WorkspaceInitializer::workspace_params_serialize(
        storage,
        workspace_params,
        workspace_params_file,
    )
    .await?;

    Ok(())
}

/// Inserts workspace params into the `Resources` map.
fn workspace_params_insert<WorkspaceParamsK>(
    mut workspace_params: WorkspaceParams<WorkspaceParamsK>,
    resources: &mut Resources<Empty>,
) where
    WorkspaceParamsK:
        Clone + Debug + Eq + Hash + DeserializeOwned + Serialize + Send + Sync + 'static,
{
    workspace_params
        .drain(..)
        .for_each(|(_key, workspace_param)| {
            let workspace_param = workspace_param.into_inner().upcast();
            let type_id = Resource::type_id(&*workspace_param);
            resources.insert_raw(type_id, workspace_param);
        });
}

/// Serializes profile params to storage.
async fn profile_params_serialize<ProfileParamsK>(
    profile_params: &ProfileParams<ProfileParamsK>,
    storage: &Storage,
    profile_params_file: &ProfileParamsFile,
) -> Result<(), peace_rt_model::Error>
where
    ProfileParamsK:
        Clone + Debug + Eq + Hash + DeserializeOwned + Serialize + Send + Sync + 'static,
{
    WorkspaceInitializer::profile_params_serialize(storage, profile_params, profile_params_file)
        .await?;

    Ok(())
}

/// Inserts profile params into the `Resources` map.
fn profile_params_insert<ProfileParamsK>(
    mut profile_params: ProfileParams<ProfileParamsK>,
    resources: &mut Resources<Empty>,
) where
    ProfileParamsK:
        Clone + Debug + Eq + Hash + DeserializeOwned + Serialize + Send + Sync + 'static,
{
    profile_params.drain(..).for_each(|(_key, profile_param)| {
        let profile_param = profile_param.into_inner().upcast();
        let type_id = Resource::type_id(&*profile_param);
        resources.insert_raw(type_id, profile_param);
    });
}

/// Serializes flow params to storage.
async fn flow_params_serialize<FlowParamsK>(
    flow_params: &FlowParams<FlowParamsK>,
    storage: &Storage,
    flow_params_file: &FlowParamsFile,
) -> Result<(), peace_rt_model::Error>
where
    FlowParamsK: Clone + Debug + Eq + Hash + DeserializeOwned + Serialize + Send + Sync + 'static,
{
    WorkspaceInitializer::flow_params_serialize(storage, flow_params, flow_params_file).await?;

    Ok(())
}

/// Serializes item params to storage.
async fn params_specs_serialize(
    params_specs: &ParamsSpecs,
    storage: &Storage,
    params_specs_file: &ParamsSpecsFile,
) -> Result<(), peace_rt_model::Error> {
    ParamsSpecsSerializer::serialize(storage, params_specs, params_specs_file).await
}

/// Inserts flow params into the `Resources` map.
fn flow_params_insert<FlowParamsK>(
    mut flow_params: FlowParams<FlowParamsK>,
    resources: &mut Resources<Empty>,
) where
    FlowParamsK: Clone + Debug + Eq + Hash + DeserializeOwned + Serialize + Send + Sync + 'static,
{
    flow_params.drain(..).for_each(|(_key, flow_param)| {
        let flow_param = flow_param.into_inner().upcast();
        let type_id = Resource::type_id(&*flow_param);
        resources.insert_raw(type_id, flow_param);
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn profiles_from_peace_app_dir(
    peace_app_dir: &peace_resource_rt::paths::PeaceAppDir,
    profiles_filter_fn: Option<&dyn Fn(&peace_profile_model::Profile) -> bool>,
) -> Result<Vec<peace_profile_model::Profile>, peace_rt_model::Error> {
    use std::{ffi::OsStr, str::FromStr};

    let mut profiles = Vec::new();
    let mut peace_app_read_dir = tokio::fs::read_dir(peace_app_dir).await.map_err(
        #[cfg_attr(coverage_nightly, coverage(off))]
        |error| {
            peace_rt_model::Error::Native(peace_rt_model::NativeError::PeaceAppDirRead {
                peace_app_dir: peace_app_dir.to_path_buf(),
                error,
            })
        },
    )?;
    while let Some(entry) = peace_app_read_dir.next_entry().await.map_err(
        #[cfg_attr(coverage_nightly, coverage(off))]
        |error| {
            peace_rt_model::Error::Native(peace_rt_model::NativeError::PeaceAppDirEntryRead {
                peace_app_dir: peace_app_dir.to_path_buf(),
                error,
            })
        },
    )? {
        let file_type = entry.file_type().await.map_err(
            #[cfg_attr(coverage_nightly, coverage(off))]
            |error| {
                peace_rt_model::Error::Native(
                    peace_rt_model::NativeError::PeaceAppDirEntryFileTypeRead {
                        path: entry.path(),
                        error,
                    },
                )
            },
        )?;

        if file_type.is_dir() {
            let entry_path = entry.path();
            if let Some(dir_name) = entry_path.file_name().and_then(OsStr::to_str) {
                // Assume this is a profile directory
                let profile =
                    peace_profile_model::Profile::from_str(dir_name).map_err(|error| {
                        peace_rt_model::Error::Native(
                            peace_rt_model::NativeError::ProfileDirInvalidName {
                                dir_name: dir_name.to_string(),
                                path: entry_path.to_path_buf(),
                                error,
                            },
                        )
                    })?;

                if let Some(profiles_filter_fn) = profiles_filter_fn {
                    if !profiles_filter_fn(&profile) {
                        // Exclude any profiles that do not pass the filter
                        continue;
                    }
                }

                profiles.push(profile)
            }

            // Assume non-UTF8 file names are not profile directories
        }
    }

    // Ensure profiles are in a consistent, sensible order.
    profiles.sort();

    Ok(profiles)
}

#[cfg(target_arch = "wasm32")]
pub(crate) async fn profiles_from_peace_app_dir(
    _peace_app_dir: &peace_resource_rt::paths::PeaceAppDir,
    _profiles_filter_fn: Option<&dyn Fn(&peace_profile_model::Profile) -> bool>,
) -> Result<Vec<peace_profile_model::Profile>, peace_rt_model::Error> {
    let profiles = Vec::new();

    // Not supported yet -- needs a `Storage` abstraction over both native an web
    // assembly.

    Ok(profiles)
}

/// Registers each item's `Params` and `State` for stateful
/// deserialization.
fn params_and_states_type_reg<E>(item_graph: &ItemGraph<E>) -> (ParamsSpecsTypeReg, StatesTypeReg)
where
    E: 'static,
{
    item_graph.iter().fold(
        (ParamsSpecsTypeReg::new(), StatesTypeReg::new()),
        |(mut params_specs_type_reg, mut states_type_reg), item| {
            item.params_and_state_register(&mut params_specs_type_reg, &mut states_type_reg);

            (params_specs_type_reg, states_type_reg)
        },
    )
}

/// Merges provided item parameters with previously stored item
/// parameters.
///
/// If an item's parameters are not provided, and nothing was previously
/// stored, then an error is returned.
fn params_specs_merge<E>(
    flow: &Flow<E>,
    mut params_specs_provided: ParamsSpecs,
    params_specs_stored: Option<ParamsSpecs>,
) -> Result<ParamsSpecs, peace_rt_model::Error>
where
    E: From<peace_rt_model::Error> + 'static,
{
    // Combine provided and stored params specs. Provided params specs take
    // precedence.
    //
    // We construct a new TypeMap because we want to make sure params specs are
    // serialized in order of the items in the graph.
    let item_graph = flow.graph();
    let mut params_specs = ParamsSpecs::with_capacity(item_graph.node_count());

    // Collected erroneous data -- parameters may have been valid in the past, but:
    //
    // * item IDs may have changed.
    // * items may have been removed, but params specs remain.
    // * items may have been added, but params specs forgotten to be added.
    let mut item_ids_with_no_params_specs = Vec::<ItemId>::new();
    let mut params_specs_stored_mismatches = None;
    let mut params_specs_not_usable = Vec::<ItemId>::new();

    if let Some(mut params_specs_stored) = params_specs_stored {
        item_graph.iter_insertion().for_each(|item_rt| {
            let item_id = item_rt.id();

            // Removing the entry from stored params specs is deliberate, so filtering for
            // stored params specs that no longer have a corresponding item are
            // detected.
            let params_spec_provided = params_specs_provided.shift_remove_entry(item_id);
            let params_spec_stored = params_specs_stored.shift_remove_entry(item_id);

            // Deep merge params specs.
            let params_spec_to_use = match (params_spec_provided, params_spec_stored) {
                (None, None) => None,
                (None, Some(params_spec_stored)) => Some(params_spec_stored),

                // Newly added item, or potentially renamed.
                (Some(params_spec_provided), None) => Some(params_spec_provided),

                (
                    Some((item_id, mut params_spec_provided)),
                    Some((_item_id, params_spec_stored)),
                ) => {
                    params_spec_provided.merge(&*params_spec_stored);
                    Some((item_id, params_spec_provided))
                }
            };

            if let Some((item_id, params_spec_boxed)) = params_spec_to_use {
                // `*Spec::MappingFn`s will be present in `params_spec_stored`, but will not
                // be valid mapping functions as they cannot be serialized / deserialized.
                //
                // Also, field wise `ParamsSpec`s may contain `ValueSpec::Stored` for fields
                // which never had specifications, which are also unusable.
                if params_spec_boxed.is_usable() {
                    params_specs.insert_raw(item_id, params_spec_boxed);
                } else {
                    params_specs_not_usable.push(item_id);
                }
            } else {
                // Collect items that do not have parameters.
                item_ids_with_no_params_specs.push(item_id.clone());
            }
        });

        // Stored parameters whose IDs do not correspond to any item IDs in the
        // graph. May be empty.
        params_specs_stored_mismatches = Some(params_specs_stored);
    } else {
        item_graph.iter_insertion().for_each(|item_rt| {
            let item_id = item_rt.id();

            if let Some((item_id, params_spec_boxed)) =
                params_specs_provided.shift_remove_entry(item_id)
            {
                params_specs.insert_raw(item_id, params_spec_boxed);
            } else {
                // Collect items that do not have parameters.
                item_ids_with_no_params_specs.push(item_id.clone());
            }
        });
    }

    // Provided parameters whose IDs do not correspond to any item IDs in the
    // graph.
    let params_specs_provided_mismatches = params_specs_provided;

    let params_no_issues = item_ids_with_no_params_specs.is_empty()
        && params_specs_provided_mismatches.is_empty()
        && params_specs_stored_mismatches
            .as_ref()
            .map(|params_specs_stored_mismatches| params_specs_stored_mismatches.is_empty())
            .unwrap_or(true)
        && params_specs_not_usable.is_empty();

    if params_no_issues {
        Ok(params_specs)
    } else {
        let params_specs_stored_mismatches = Box::new(params_specs_stored_mismatches);
        Err(peace_rt_model::Error::ParamsSpecsMismatch {
            item_ids_with_no_params_specs,
            params_specs_provided_mismatches,
            params_specs_stored_mismatches,
            params_specs_not_usable,
        })
    }
}

async fn item_graph_setup<E>(
    item_graph: &ItemGraph<E>,
    resources: Resources<Empty>,
) -> Result<Resources<SetUp>, E>
where
    E: std::error::Error + 'static,
{
    let resources = item_graph
        .stream()
        .map(Ok::<_, E>)
        .try_fold(resources, |mut resources, item| async move {
            item.setup(&mut resources).await?;
            Ok(resources)
        })
        .await?;

    Ok(Resources::<SetUp>::from(resources))
}
