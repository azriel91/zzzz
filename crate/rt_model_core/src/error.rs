use std::path::PathBuf;

use peace_cmd_model::CmdExecutionError;
use peace_flow_model::FlowId;
use peace_item_model::ItemId;
use peace_params::{ParamsResolveError, ParamsSpecs};
use peace_profile_model::Profile;
use peace_resource_rt::paths::ParamsSpecsFile;

pub use self::{apply_cmd_error::ApplyCmdError, state_downcast_error::StateDowncastError};

mod apply_cmd_error;
mod state_downcast_error;

cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        pub use self::native_error::NativeError;

        mod native_error;
    } else {
        pub use self::web_error::WebError;

        mod web_error;
    }
}

/// Peace runtime errors.
#[cfg_attr(feature = "error_reporting", derive(miette::Diagnostic))]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to apply changes.
    #[error("Failed to apply changes.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::apply_error))
    )]
    ApplyCmdError(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        ApplyCmdError,
    ),

    /// Error in `CmdExecution` or `CmdBlock` logic, usually due to incorrect
    /// `Resource` insertion or removal.
    #[error(
        "Error in `CmdExecution` or `CmdBlock` logic, usually due to incorrect `Resource` insertion or removal."
    )]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(help("Make sure that the value is populated by a predecessor."))
    )]
    CmdExecution(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        CmdExecutionError,
    ),

    /// Failed to serialize error.
    #[error("Failed to serialize error.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::error_serialize))
    )]
    ErrorSerialize(#[source] serde_yaml::Error),

    /// Failed to resolve values for a `Params` object from `resources`.
    ///
    /// This possibly indicates the user has provided a `Params::Spec` with
    /// `InMemory` or `MappingFn`, but no predecessor populates that type.
    #[error("Failed to resolve values for a `Params` object from `resources`.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(
            code(peace_rt_model::params_resolve_error),
            help("Make sure that the value is populated by a predecessor.")
        )
    )]
    ParamsResolveError(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        ParamsResolveError,
    ),

    /// A `Params::Spec` was not present for a given item ID.
    ///
    /// If this happens, this is a bug in the Peace framework.
    #[error("A `Params::Spec` was not present for item: {item_id}")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(
            code(peace_rt_model::params_spec_not_found),
            help(
                "If you are an end user, please ask for help from the providers of your automation tool.\n\
                \n\
                If you are developing a tool with the Peace framework,\n\
                please open an issue in the Peace repository:\n\
                \n\
                https://github.com/azriel91/peace/"
            )
        )
    )]
    ParamsSpecNotFound {
        /// Item ID for which the params spec was not found.
        item_id: ItemId,
    },

    /// Item params specs do not match with the items in the flow.
    ///
    /// # Symptoms
    ///
    /// * Provided params specs for an item ID has no corresponding item ID in
    ///   the flow.
    /// * Stored params specs for an item ID has no corresponding item ID in the
    ///   flow.
    /// * ID of an item in the flow does not have a corresponding provided
    ///   params spec.
    /// * ID of an item in the flow does not have a corresponding stored params
    ///   spec.
    ///
    /// # Causes
    ///
    /// These can happen when:
    ///
    /// * An item is added.
    ///
    ///    - No corresponding provided params spec.
    ///    - No corresponding stored params spec.
    ///
    /// * An item ID is renamed.
    ///
    ///    - Provided params spec ID mismatch.
    ///    - Stored params spec ID mismatch.
    ///    - No corresponding provided params spec.
    ///
    /// * An item is removed.
    ///
    ///    - Provided params spec ID mismatch.
    ///    - Stored params spec ID mismatch.
    #[error("Item params specs do not match with the items in the flow.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(
            code(peace_rt_model::params_specs_mismatch),
            help("{}", params_specs_mismatch_display(
                item_ids_with_no_params_specs,
                params_specs_provided_mismatches,
                params_specs_stored_mismatches.as_ref().as_ref(),
                params_specs_not_usable,
            ))
        )
    )]
    ParamsSpecsMismatch {
        /// Item IDs for which there are no provided or stored params spec.
        item_ids_with_no_params_specs: Vec<ItemId>,
        /// Provided params specs with no matching item ID in the flow.
        params_specs_provided_mismatches: ParamsSpecs,
        /// Stored params specs with no matching item ID in the flow.
        //
        // Boxed so that this enum variant is not so large compared to other variants
        // to address `clippy::large_enum_variant`.
        params_specs_stored_mismatches: Box<Option<ParamsSpecs>>,
        /// Item IDs which had a mapping function previously provided in
        /// its params spec, but on a subsequent invocation nothing was
        /// provided.
        params_specs_not_usable: Vec<ItemId>,
    },

    /// In a `MultiProfileSingleFlow` diff, neither profile had `Params::Specs`
    /// defined.
    #[error("Params specifications not defined for `{profile_a}` or `{profile_b}`.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(
            code(peace_rt_model::params_specs_not_defined_for_diff),
            help(
                "Make sure at least one of the flows has `.with_items_params(..)`\n\
                defined for every item in the flow."
            )
        )
    )]
    ParamsSpecsNotDefinedForDiff {
        /// First profile looked up for params specs.
        profile_a: Profile,
        /// Second profile looked up for params specs.
        profile_b: Profile,
    },

    /// Failed to serialize a presentable type.
    #[error("Failed to serialize a presentable type.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::presentable_serialize))
    )]
    PresentableSerialize(#[source] serde_yaml::Error),

    /// Failed to serialize progress update.
    #[error("Failed to serialize progress update.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::progress_update_serialize))
    )]
    ProgressUpdateSerialize(#[source] serde_yaml::Error),
    /// Failed to serialize progress update as JSON.
    #[error("Failed to serialize progress update.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::progress_update_serialize_json))
    )]
    ProgressUpdateSerializeJson(#[source] serde_json::Error),

    /// Failed to deserialize states.
    #[error("Failed to deserialize states for flow: `{flow_id}`.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(
            code(peace_rt_model::states_deserialize),
            help(
                "Make sure that all commands using the `{flow_id}` flow, also use the same item graph.\n\
                This is because all Items are used to deserialize state.\n\
                \n\
                If the item graph is different, it may make sense to use a different flow ID."
            )
        )
    )]
    StatesDeserialize {
        /// Flow ID whose states are being deserialized.
        flow_id: FlowId,
        /// Source text to be deserialized.
        #[cfg(feature = "error_reporting")]
        #[source_code]
        states_file_source: miette::NamedSource<String>,
        /// Offset within the source text that the error occurred.
        #[cfg(feature = "error_reporting")]
        #[label("{}", error_message)]
        error_span: Option<miette::SourceOffset>,
        /// Message explaining the error.
        #[cfg(feature = "error_reporting")]
        error_message: String,
        /// Offset within the source text surrounding the error.
        #[cfg(feature = "error_reporting")]
        #[label]
        context_span: Option<miette::SourceOffset>,
        /// Underlying error.
        #[source]
        error: serde_yaml::Error,
    },

    /// Failed to serialize states.
    #[error("Failed to serialize states.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::states_serialize))
    )]
    StatesSerialize(#[source] serde_yaml::Error),

    /// Failed to deserialize params specs.
    #[error("Failed to deserialize params specs for `{profile}/{flow_id}`.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(
            code(peace_rt_model::params_specs_deserialize),
            help(
                "Make sure that all commands using the `{flow_id}` flow, also use the same item graph.\n\
                This is because all Items are used to deserialize state.\n\
                \n\
                If the item graph is different, it may make sense to use a different flow ID."
            )
        )
    )]
    ParamsSpecsDeserialize {
        /// Profile of the flow.
        profile: Profile,
        /// Flow ID whose params specs are being deserialized.
        flow_id: FlowId,
        /// Source text to be deserialized.
        #[cfg(feature = "error_reporting")]
        #[source_code]
        params_specs_file_source: miette::NamedSource<String>,
        /// Offset within the source text that the error occurred.
        #[cfg(feature = "error_reporting")]
        #[label("{}", error_message)]
        error_span: Option<miette::SourceOffset>,
        /// Message explaining the error.
        #[cfg(feature = "error_reporting")]
        error_message: String,
        /// Offset within the source text surrounding the error.
        #[cfg(feature = "error_reporting")]
        #[label]
        context_span: Option<miette::SourceOffset>,
        /// Underlying error.
        #[source]
        error: serde_yaml::Error,
    },

    /// Failed to serialize params specs.
    #[error("Failed to serialize params specs.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::params_specs_serialize))
    )]
    ParamsSpecsSerialize(#[source] serde_yaml::Error),

    /// Params specs file does not exist.
    ///
    /// This is returned when `ParamsSpecs` is attempted to be
    /// deserialized but the file does not exist.
    ///
    /// The automation tool implementor needs to ensure the
    /// `SingleProfileSingleFlow` command context has been initialized for that
    /// flow previously.
    #[error("Params specs file does not exist for `{profile}/{flow_id}`.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(
            code(peace_rt_model::params_specs_file_not_exists),
            help(
                "Ensure that a `SingleProfileSingleFlow` command context has previously been built."
            )
        )
    )]
    ParamsSpecsFileNotExists {
        /// Profile of the flow.
        profile: Profile,
        /// Flow ID whose params are being deserialized.
        flow_id: FlowId,
        /// Path of the params specs file.
        params_specs_file: ParamsSpecsFile,
    },

    /// Current states have not been discovered.
    ///
    /// This is returned when `StatesCurrentFile` is attempted to be
    /// deserialized but does not exist.
    #[error("Current states have not been discovered.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(
            code(peace_rt_model::states_current_discover_required),
            help("Ensure that `StatesDiscoverCmd::current` has been called.")
        )
    )]
    StatesCurrentDiscoverRequired,

    /// Goal states have not been written to disk.
    ///
    /// This is returned when `StatesGoalFile` is attempted to be
    /// deserialized but does not exist.
    #[error("Goal states have not been written to disk.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(
            code(peace_rt_model::states_goal_discover_required),
            help("Ensure that `StatesDiscoverCmd::goal` has been called.")
        )
    )]
    StatesGoalDiscoverRequired,

    /// Failed to serialize state diffs.
    #[error("Failed to serialize state diffs.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::state_diffs_serialize))
    )]
    StateDiffsSerialize(#[source] serde_yaml::Error),

    /// Failed to serialize error as JSON.
    #[error("Failed to serialize error as JSON.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::error_serialize_json))
    )]
    ErrorSerializeJson(#[source] serde_json::Error),

    /// Failed to serialize states as JSON.
    #[error("Failed to serialize states as JSON.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::states_current_serialize_json))
    )]
    StatesSerializeJson(#[source] serde_json::Error),

    /// Failed to serialize state diffs as JSON.
    #[error("Failed to serialize state diffs as JSON.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::state_diffs_serialize_json))
    )]
    StateDiffsSerializeJson(#[source] serde_json::Error),

    /// Failed to serialize workspace init params.
    #[error("Failed to serialize workspace init params.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::workspace_init_params_serialize))
    )]
    WorkspaceParamsSerialize(#[source] serde_yaml::Error),

    /// Failed to deserialize workspace init params.
    #[error("Failed to deserialize workspace init params.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::workspace_init_params_deserialize))
    )]
    WorkspaceParamsDeserialize(#[source] serde_yaml::Error),

    /// Workspace params does not exist, so cannot look up `Profile`.
    #[error("Workspace params does not exist, so cannot look up `Profile`.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::workspace_params_none_for_profile))
    )]
    WorkspaceParamsNoneForProfile,

    /// Workspace param for `Profile` does not exist.
    #[error("Workspace param for `Profile` does not exist.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::workspace_params_profile_none))
    )]
    WorkspaceParamsProfileNone,

    /// Profile to diff does not exist in `MultiProfileSingleFlow` scope.
    ///
    /// This could mean the caller provided a profile that does not exist, or
    /// the profile filter function filtered out the profile from the list of
    /// profiles.
    #[error("Profile `{profile}` not in scope, make sure it exists in `.peace/*/{profile}`.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(
            code(peace_rt_model::profile_not_in_scope),
            help(
                "Make sure the profile is spelt correctly.\n\
                Available profiles are: [{profiles_in_scope}]",
                profiles_in_scope = profiles_in_scope
                    .iter()
                    .map(|profile| format!("{profile}"))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        )
    )]
    ProfileNotInScope {
        /// The profile that was not in scope.
        profile: Profile,
        /// The profiles that are in scope.
        profiles_in_scope: Vec<Profile>,
    },

    /// Profile to diff has not had its states current discovered.
    #[error("Profile `{profile}`'s states have not been discovered.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(
            code(peace_rt_model::profile_states_current_not_discovered),
            help("Switch to the profile and run the states discover command.")
        )
    )]
    ProfileStatesCurrentNotDiscovered {
        /// The profile that was not in scope.
        profile: Profile,
    },

    /// Failed to serialize profile init params.
    #[error("Failed to serialize profile init params.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::profile_init_params_serialize))
    )]
    ProfileParamsSerialize(#[source] serde_yaml::Error),

    /// Failed to deserialize profile init params.
    #[error("Failed to deserialize profile init params.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::profile_init_params_deserialize))
    )]
    ProfileParamsDeserialize(#[source] serde_yaml::Error),

    /// Failed to serialize flow init params.
    #[error("Failed to serialize flow init params.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::flow_init_params_serialize))
    )]
    FlowParamsSerialize(#[source] serde_yaml::Error),

    /// Failed to deserialize flow init params.
    #[error("Failed to deserialize flow init params.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::flow_init_params_deserialize))
    )]
    FlowParamsDeserialize(#[source] serde_yaml::Error),

    /// Item does not exist in storage.
    #[error("Item does not exist in storage: `{}`.", path.display())]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model::item_not_exists))
    )]
    ItemNotExists {
        /// Path to the file.
        path: PathBuf,
    },

    /// Error downcasting a `BoxDtDisplay` into an item's concrete state type.
    #[error("Error downcasting a `BoxDtDisplay` into an item's concrete state type.")]
    StateDowncastError(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        StateDowncastError,
    ),

    /// Native application error occurred.
    #[error("Native application error occurred.")]
    #[cfg(not(target_arch = "wasm32"))]
    Native(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        NativeError,
    ),

    /// Web application error occurred.
    #[error("Web application error occurred.")]
    #[cfg(target_arch = "wasm32")]
    Web(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        WebError,
    ),
}

#[cfg(feature = "error_reporting")]
fn params_specs_mismatch_display(
    item_ids_with_no_params: &[ItemId],
    params_specs_provided_mismatches: &ParamsSpecs,
    params_specs_stored_mismatches: Option<&ParamsSpecs>,
    params_specs_not_usable: &[ItemId],
) -> String {
    let mut items = Vec::<String>::new();

    if !item_ids_with_no_params.is_empty() {
        items.push(format!(
            "The following items do not have parameters provided:\n\
            \n\
            {}\n",
            item_ids_with_no_params
                .iter()
                .map(|item_id| format!("* {item_id}"))
                .collect::<Vec<String>>()
                .join("\n")
        ));
    }

    if !params_specs_provided_mismatches.is_empty() {
        let params_specs_provided_mismatches_list = params_specs_provided_mismatches
            .keys()
            .map(|item_id| format!("* {item_id}"))
            .collect::<Vec<String>>()
            .join("\n");
        items.push(format!(
            "The following provided params specs do not correspond to any items in the flow:\n\
            \n\
            {params_specs_provided_mismatches_list}\n",
        ))
    }

    if let Some(params_specs_stored_mismatches) = params_specs_stored_mismatches {
        if !params_specs_stored_mismatches.is_empty() {
            let params_specs_stored_mismatches_list = params_specs_stored_mismatches
                .keys()
                .map(|item_id| format!("* {item_id}"))
                .collect::<Vec<String>>()
                .join("\n");
            items.push(format!(
                "The following stored params specs do not correspond to any items in the flow:\n\
                \n\
                {params_specs_stored_mismatches_list}\n",
            ));
        }
    }

    if !params_specs_not_usable.is_empty() {
        items.push(format!(
            "The following items either have not had a params spec provided previously,\n\
            or had contained a mapping function, which cannot be loaded from disk.\n\
            \n\
            So the params spec needs to be provided to the command context for:\n\
            \n\
            {}\n",
            params_specs_not_usable
                .iter()
                .map(|item_id| format!("* {item_id}"))
                .collect::<Vec<String>>()
                .join("\n")
        ));
    }

    items.join("\n")
}
