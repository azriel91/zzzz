use std::{collections::BTreeMap, fmt::Debug, hash::Hash};

use interruptible::InterruptibilityState;
use own::{OwnedOrMutRef, OwnedOrRef};
use peace_flow_rt::Flow;
use peace_params::ParamsSpecs;
use peace_profile_model::Profile;
use peace_resource_rt::{
    paths::{FlowDir, PeaceAppDir, PeaceDir, ProfileDir, ProfileHistoryDir, WorkspaceDir},
    resources::ts::SetUp,
    states::StatesCurrentStored,
    Resources,
};
use peace_rt_model::{
    params::{
        FlowParams, KeyKnown, KeyMaybe, ParamsKeys, ParamsKeysImpl, ParamsTypeRegs, ProfileParams,
        WorkspaceParams,
    },
    ParamsSpecsTypeReg, StatesTypeReg, Workspace,
};
use serde::{de::DeserializeOwned, Serialize};

use crate::ctx::CmdCtxTypes;

/// A command that works with multiple profiles, and a single flow.
///
/// ```bash
/// path/to/repo/.peace/envman
/// |- 📝 workspace_params.yaml    # ✅ can read or write `WorkspaceParams`
/// |
/// |- 🌏 internal_dev_a           # ✅ can list multiple `Profile`s
/// |   |- 📝 profile_params.yaml  # ✅ can read multiple `ProfileParams`
/// |   |
/// |   |- 🌊 deploy                # ✅ can read `FlowId`
/// |   |   |- 📝 flow_params.yaml  # ✅ can read or write `FlowParams`
/// |   |   |- 📋 states_goal.yaml  # ✅ can read or write `StatesGoal`
/// |   |   |- 📋 states_current.yaml # ✅ can read or write `StatesCurrentStored`
/// |   |
/// |   |- 🌊 ..                       # ❌ cannot read or write other `Flow` information
/// |
/// |- 🌏 customer_a_dev           # ✅
/// |   |- 📝 profile_params.yaml  # ✅
/// |   |
/// |   |- 🌊 deploy                # ✅
/// |       |- 📝 flow_params.yaml  # ✅
/// |       |- 📋 states_goal.yaml  # ✅
/// |       |- 📋 states_current.yaml # ✅
/// |
/// |- 🌏 customer_a_prod          # ✅
/// |   |- 📝 profile_params.yaml  # ✅
/// |   |
/// |   |- 🌊 deploy                # ✅
/// |       |- 📝 flow_params.yaml  # ✅
/// |       |- 📋 states_goal.yaml  # ✅
/// |       |- 📋 states_current.yaml # ✅
/// |
/// |
/// |- 🌏 workspace_init           # ✅ can list multiple `Profile`s
///     |- 📝 profile_params.yaml  # ❌ cannot read profile params of different underlying type
/// |   |- 🌊 workspace_init       # ❌ cannot read unrelated flows
/// ```
///
/// ## Capabilities
///
/// This kind of command can:
///
/// * Read or write workspace parameters.
/// * Read or write multiple profiles' parameters &ndash; as long as they are of
///   the same type (same `struct`).
/// * Read or write flow parameters for the same flow.
/// * Read or write flow state for the same flow.
///
/// This kind of command cannot:
///
/// * Read or write flow parameters for different flows.
/// * Read or write flow state for different flows.
#[derive(Debug)]
pub struct MultiProfileSingleFlow<'ctx, CmdCtxTypesT>
where
    CmdCtxTypesT: CmdCtxTypes,
{
    /// Output endpoint to return values / errors, and write progress
    /// information to.
    ///
    /// See [`OutputWrite`].
    ///
    /// [`OutputWrite`]: peace_rt_model_core::OutputWrite
    output: OwnedOrMutRef<'ctx, CmdCtxTypesT::Output>,
    /// Whether the `CmdExecution` is interruptible.
    ///
    /// If it is, this holds the interrupt channel receiver.
    interruptibility_state: InterruptibilityState<'static, 'static>,
    /// Workspace that the `peace` tool runs in.
    workspace: OwnedOrRef<'ctx, Workspace>,
    /// The profiles that are accessible by this command.
    profiles: Vec<Profile>,
    /// Profile directories that store params and flows.
    profile_dirs: BTreeMap<Profile, ProfileDir>,
    /// Directories of each profile's execution history.
    profile_history_dirs: BTreeMap<Profile, ProfileHistoryDir>,
    /// The chosen process flow.
    flow: OwnedOrRef<'ctx, Flow<CmdCtxTypesT::AppError>>,
    /// Flow directory that stores params and states.
    flow_dirs: BTreeMap<Profile, FlowDir>,
    /// Type registries for [`WorkspaceParams`], [`ProfileParams`], and
    /// [`FlowParams`] deserialization.
    ///
    /// [`WorkspaceParams`]: peace_rt_model::params::WorkspaceParams
    /// [`ProfileParams`]: peace_rt_model::params::ProfileParams
    /// [`FlowParams`]: peace_rt_model::params::FlowParams
    params_type_regs: ParamsTypeRegs<CmdCtxTypesT::ParamsKeys>,
    /// Workspace params.
    workspace_params: WorkspaceParams<
        <<CmdCtxTypesT::ParamsKeys as ParamsKeys>::WorkspaceParamsKMaybe as KeyMaybe>::Key,
    >,
    /// Profile params for the profile.
    profile_to_profile_params: BTreeMap<
        Profile,
        ProfileParams<
            <<CmdCtxTypesT::ParamsKeys as ParamsKeys>::ProfileParamsKMaybe as KeyMaybe>::Key,
        >,
    >,
    /// Flow params for the selected flow.
    profile_to_flow_params: BTreeMap<
        Profile,
        FlowParams<<<CmdCtxTypesT::ParamsKeys as ParamsKeys>::FlowParamsKMaybe as KeyMaybe>::Key>,
    >,
    /// Stored current states for each profile for the selected flow.
    profile_to_states_current_stored: BTreeMap<Profile, Option<StatesCurrentStored>>,
    /// Type registry for each item's [`Params`]`::Spec`.
    ///
    /// This is used to deserialize [`ParamsSpecsFile`].
    ///
    /// [`Params`]: peace_cfg::Item::Params
    /// [`ParamsSpecsFile`]: peace_resource_rt::paths::ParamsSpecsFile
    params_specs_type_reg: ParamsSpecsTypeReg,
    /// Item params specs for each profile for the selected flow.
    profile_to_params_specs: BTreeMap<Profile, Option<ParamsSpecs>>,
    /// Type registry for each item's `State`.
    ///
    /// This is used to deserialize [`StatesCurrentFile`] and
    /// [`StatesGoalFile`].
    ///
    /// [`StatesCurrentFile`]: peace_resource_rt::paths::StatesCurrentFile
    /// [`StatesGoalFile`]: peace_resource_rt::paths::StatesGoalFile
    states_type_reg: StatesTypeReg,
    /// `Resources` for flow execution.
    resources: Resources<SetUp>,
}

/// Access to fields in `MultiProfileSingleFlow` so that multiple borrows can
/// happen simultaneously.
#[derive(Debug)]
pub struct MultiProfileSingleFlowView<'view, CmdCtxTypesT>
where
    CmdCtxTypesT: CmdCtxTypes,
{
    /// Output endpoint to return values / errors, and write progress
    /// information to.
    ///
    /// See [`OutputWrite`].
    ///
    /// [`OutputWrite`]: peace_rt_model_core::OutputWrite
    pub output: &'view mut CmdCtxTypesT::Output,
    /// Whether the `CmdExecution` is interruptible.
    ///
    /// If it is, this holds the interrupt channel receiver.
    pub interruptibility_state: InterruptibilityState<'view, 'view>,
    /// Workspace that the `peace` tool runs in.
    pub workspace: &'view Workspace,
    /// The profiles that are accessible by this command.
    pub profiles: &'view [Profile],
    /// Profile directories that store params and flows.
    pub profile_dirs: &'view BTreeMap<Profile, ProfileDir>,
    /// Directories of each profile's execution history.
    pub profile_history_dirs: &'view BTreeMap<Profile, ProfileHistoryDir>,
    /// The chosen process flow.
    pub flow: &'view Flow<CmdCtxTypesT::AppError>,
    /// Flow directory that stores params and states.
    pub flow_dirs: &'view BTreeMap<Profile, FlowDir>,
    /// Type registries for [`WorkspaceParams`], [`ProfileParams`], and
    /// [`FlowParams`] deserialization.
    ///
    /// [`WorkspaceParams`]: peace_rt_model::params::WorkspaceParams
    /// [`ProfileParams`]: peace_rt_model::params::ProfileParams
    /// [`FlowParams`]: peace_rt_model::params::FlowParams
    pub params_type_regs: &'view ParamsTypeRegs<CmdCtxTypesT::ParamsKeys>,
    /// Workspace params.
    pub workspace_params: &'view WorkspaceParams<
        <<CmdCtxTypesT::ParamsKeys as ParamsKeys>::WorkspaceParamsKMaybe as KeyMaybe>::Key,
    >,
    /// Profile params for the profile.
    pub profile_to_profile_params: &'view BTreeMap<
        Profile,
        ProfileParams<
            <<CmdCtxTypesT::ParamsKeys as ParamsKeys>::ProfileParamsKMaybe as KeyMaybe>::Key,
        >,
    >,
    /// Flow params for the selected flow.
    pub profile_to_flow_params: &'view BTreeMap<
        Profile,
        FlowParams<<<CmdCtxTypesT::ParamsKeys as ParamsKeys>::FlowParamsKMaybe as KeyMaybe>::Key>,
    >,
    /// Stored current states for each profile for the selected flow.
    pub profile_to_states_current_stored: &'view BTreeMap<Profile, Option<StatesCurrentStored>>,
    /// Type registry for each item's [`Params`]`::Spec`.
    ///
    /// This is used to deserialize [`ParamsSpecsFile`].
    ///
    /// [`Params`]: peace_cfg::Item::Params
    /// [`ParamsSpecsFile`]: peace_resource_rt::paths::ParamsSpecsFile
    pub params_specs_type_reg: &'view ParamsSpecsTypeReg,
    /// Item params specs for each profile for the selected flow.
    pub profile_to_params_specs: &'view BTreeMap<Profile, Option<ParamsSpecs>>,
    /// Type registry for each item's `State`.
    ///
    /// This is used to deserialize [`StatesCurrentFile`] and
    /// [`StatesGoalFile`].
    ///
    /// [`StatesCurrentFile`]: peace_resource_rt::paths::StatesCurrentFile
    /// [`StatesGoalFile`]: peace_resource_rt::paths::StatesGoalFile
    pub states_type_reg: &'view StatesTypeReg,
    /// `Resources` for flow execution.
    pub resources: &'view mut Resources<SetUp>,
}

impl<'ctx, CmdCtxTypesT> MultiProfileSingleFlow<'ctx, CmdCtxTypesT>
where
    CmdCtxTypesT: CmdCtxTypes,
{
    /// Returns a new `MultiProfileSingleFlow` scope.
    #[allow(clippy::too_many_arguments)] // Constructed by proc macro
    pub(crate) fn new(
        output: OwnedOrMutRef<'ctx, CmdCtxTypesT::Output>,
        interruptibility_state: InterruptibilityState<'static, 'static>,
        workspace: OwnedOrRef<'ctx, Workspace>,
        profiles: Vec<Profile>,
        profile_dirs: BTreeMap<Profile, ProfileDir>,
        profile_history_dirs: BTreeMap<Profile, ProfileHistoryDir>,
        flow: OwnedOrRef<'ctx, Flow<CmdCtxTypesT::AppError>>,
        flow_dirs: BTreeMap<Profile, FlowDir>,
        params_type_regs: ParamsTypeRegs<CmdCtxTypesT::ParamsKeys>,
        workspace_params: WorkspaceParams<
            <<CmdCtxTypesT::ParamsKeys as ParamsKeys>::WorkspaceParamsKMaybe as KeyMaybe>::Key,
        >,
        profile_to_profile_params: BTreeMap<
            Profile,
            ProfileParams<
                <<CmdCtxTypesT::ParamsKeys as ParamsKeys>::ProfileParamsKMaybe as KeyMaybe>::Key,
            >,
        >,
        profile_to_flow_params: BTreeMap<
            Profile,
            FlowParams<
                <<CmdCtxTypesT::ParamsKeys as ParamsKeys>::FlowParamsKMaybe as KeyMaybe>::Key,
            >,
        >,
        profile_to_states_current_stored: BTreeMap<Profile, Option<StatesCurrentStored>>,
        params_specs_type_reg: ParamsSpecsTypeReg,
        profile_to_params_specs: BTreeMap<Profile, Option<ParamsSpecs>>,
        states_type_reg: StatesTypeReg,
        resources: Resources<SetUp>,
    ) -> Self {
        Self {
            output,
            interruptibility_state,
            workspace,
            profiles,
            profile_dirs,
            profile_history_dirs,
            flow,
            flow_dirs,
            params_type_regs,
            workspace_params,
            profile_to_profile_params,
            profile_to_flow_params,
            profile_to_states_current_stored,

            params_specs_type_reg,
            profile_to_params_specs,
            states_type_reg,
            resources,
        }
    }
}

impl<CmdCtxTypesT> MultiProfileSingleFlow<'_, CmdCtxTypesT>
where
    CmdCtxTypesT: CmdCtxTypes,
{
    /// Returns a view struct of this scope.
    ///
    /// This allows the flow and resources to be borrowed concurrently.
    pub fn view(&mut self) -> MultiProfileSingleFlowView<'_, CmdCtxTypesT> {
        let Self {
            output,
            interruptibility_state,
            workspace,
            profiles,
            profile_dirs,
            profile_history_dirs,
            flow,
            flow_dirs,
            params_type_regs,
            workspace_params,
            profile_to_profile_params,
            profile_to_flow_params,
            profile_to_states_current_stored,

            params_specs_type_reg,
            profile_to_params_specs,
            states_type_reg,
            resources,
        } = self;

        let interruptibility_state = interruptibility_state.reborrow();

        MultiProfileSingleFlowView {
            output,
            interruptibility_state,
            workspace,
            profiles,
            profile_dirs,
            profile_history_dirs,
            flow,
            flow_dirs,
            params_type_regs,
            workspace_params,
            profile_to_profile_params,
            profile_to_flow_params,
            profile_to_states_current_stored,

            params_specs_type_reg,
            profile_to_params_specs,
            states_type_reg,
            resources,
        }
    }

    /// Returns a reference to the output.
    pub fn output(&self) -> &CmdCtxTypesT::Output {
        &self.output
    }

    /// Returns a mutable reference to the output.
    pub fn output_mut(&mut self) -> &mut CmdCtxTypesT::Output {
        &mut self.output
    }

    /// Returns the interruptibility capability.
    pub fn interruptibility_state(&mut self) -> InterruptibilityState<'_, '_> {
        self.interruptibility_state.reborrow()
    }

    /// Returns the workspace that the `peace` tool runs in.
    pub fn workspace(&self) -> &Workspace {
        &self.workspace
    }

    /// Returns a reference to the workspace directory.
    pub fn workspace_dir(&self) -> &WorkspaceDir {
        self.workspace.dirs().workspace_dir()
    }

    /// Returns a reference to the `.peace` directory.
    pub fn peace_dir(&self) -> &PeaceDir {
        self.workspace.dirs().peace_dir()
    }

    /// Returns a reference to the `.peace/$app` directory.
    pub fn peace_app_dir(&self) -> &PeaceAppDir {
        self.workspace.dirs().peace_app_dir()
    }

    /// Returns the accessible profiles.
    ///
    /// These are the profiles that are filtered by the filter function, if
    /// provided.
    pub fn profiles(&self) -> &[Profile] {
        self.profiles.as_ref()
    }

    /// Returns the profile directories keyed by each profile.
    pub fn profile_dirs(&self) -> &BTreeMap<Profile, ProfileDir> {
        &self.profile_dirs
    }

    /// Returns the profile history directories keyed by each profile.
    pub fn profile_history_dirs(&self) -> &BTreeMap<Profile, ProfileHistoryDir> {
        &self.profile_history_dirs
    }

    /// Returns the flow.
    pub fn flow(&self) -> &Flow<CmdCtxTypesT::AppError> {
        &self.flow
    }

    /// Returns the flow directories keyed by each profile.
    pub fn flow_dirs(&self) -> &BTreeMap<Profile, FlowDir> {
        &self.flow_dirs
    }

    /// Returns the type registries for [`WorkspaceParams`], [`ProfileParams`],
    /// and [`FlowParams`] deserialization.
    ///
    /// [`FlowParams`]: peace_rt_model::params::FlowParams
    /// [`ItemParams`]: peace_rt_model::ItemParams
    /// [`ProfileParams`]: peace_rt_model::params::ProfileParams
    /// [`WorkspaceParams`]: peace_rt_model::params::WorkspaceParams
    pub fn params_type_regs(&self) -> &ParamsTypeRegs<CmdCtxTypesT::ParamsKeys> {
        &self.params_type_regs
    }

    /// Returns the stored current states for each profile for the selected
    /// flow.
    pub fn profile_to_states_current_stored(
        &self,
    ) -> &BTreeMap<Profile, Option<StatesCurrentStored>> {
        &self.profile_to_states_current_stored
    }

    /// Returns the type registry for each item's [`Params`]`::Spec`.
    ///
    /// This is used to deserialize [`ParamsSpecsFile`].
    ///
    /// [`Params`]: peace_cfg::Item::Params
    /// [`ParamsSpecsFile`]: peace_resource_rt::paths::ParamsSpecsFile
    pub fn params_specs_type_reg(&self) -> &ParamsSpecsTypeReg {
        &self.params_specs_type_reg
    }

    /// Returns the item params specs for each profile for the selected
    /// flow.
    pub fn profile_to_params_specs(&self) -> &BTreeMap<Profile, Option<ParamsSpecs>> {
        &self.profile_to_params_specs
    }

    /// Returns the type registry for each item's `State`.
    ///
    /// This is used to deserialize [`StatesCurrentFile`] and
    /// [`StatesGoalFile`].
    ///
    /// [`StatesCurrentFile`]: peace_resource_rt::paths::StatesCurrentFile
    /// [`StatesGoalFile`]: peace_resource_rt::paths::StatesGoalFile
    pub fn states_type_reg(&self) -> &StatesTypeReg {
        &self.states_type_reg
    }

    /// Returns a reference to the `Resources` for flow execution.
    pub fn resources(&self) -> &Resources<SetUp> {
        &self.resources
    }

    /// Returns a reference to the `Resources` for flow execution.
    pub fn resources_mut(&mut self) -> &mut Resources<SetUp> {
        &mut self.resources
    }
}

impl<CmdCtxTypesT, WorkspaceParamsK, ProfileParamsKMaybe, FlowParamsKMaybe>
    MultiProfileSingleFlow<'_, CmdCtxTypesT>
where
    CmdCtxTypesT: CmdCtxTypes<
        ParamsKeys = ParamsKeysImpl<
            KeyKnown<WorkspaceParamsK>,
            ProfileParamsKMaybe,
            FlowParamsKMaybe,
        >,
    >,
    WorkspaceParamsK:
        Clone + Debug + Eq + Hash + DeserializeOwned + Serialize + Send + Sync + Unpin + 'static,
    ProfileParamsKMaybe: KeyMaybe,
    FlowParamsKMaybe: KeyMaybe,
{
    /// Returns the workspace params.
    pub fn workspace_params(&self) -> &WorkspaceParams<WorkspaceParamsK> {
        &self.workspace_params
    }
}

impl<CmdCtxTypesT, WorkspaceParamsKMaybe, ProfileParamsK, FlowParamsKMaybe>
    MultiProfileSingleFlow<'_, CmdCtxTypesT>
where
    CmdCtxTypesT: CmdCtxTypes<
        ParamsKeys = ParamsKeysImpl<
            WorkspaceParamsKMaybe,
            KeyKnown<ProfileParamsK>,
            FlowParamsKMaybe,
        >,
    >,
    WorkspaceParamsKMaybe: KeyMaybe,
    ProfileParamsK:
        Clone + Debug + Eq + Hash + DeserializeOwned + Serialize + Send + Sync + Unpin + 'static,
    FlowParamsKMaybe: KeyMaybe,
{
    /// Returns the profile params for each profile.
    pub fn profile_to_profile_params(&self) -> &BTreeMap<Profile, ProfileParams<ProfileParamsK>> {
        &self.profile_to_profile_params
    }
}

impl<CmdCtxTypesT, WorkspaceParamsKMaybe, ProfileParamsKMaybe, FlowParamsK>
    MultiProfileSingleFlow<'_, CmdCtxTypesT>
where
    CmdCtxTypesT: CmdCtxTypes<
        ParamsKeys = ParamsKeysImpl<
            WorkspaceParamsKMaybe,
            ProfileParamsKMaybe,
            KeyKnown<FlowParamsK>,
        >,
    >,
    WorkspaceParamsKMaybe: KeyMaybe,
    ProfileParamsKMaybe: KeyMaybe,
    FlowParamsK:
        Clone + Debug + Eq + Hash + DeserializeOwned + Serialize + Send + Sync + Unpin + 'static,
{
    /// Returns the flow params for the selected flow for each profile.
    pub fn profile_to_flow_params(&self) -> &BTreeMap<Profile, FlowParams<FlowParamsK>> {
        &self.profile_to_flow_params
    }
}
