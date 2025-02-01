use std::{collections::BTreeMap, fmt::Debug, hash::Hash};

use interruptible::InterruptibilityState;
use own::{OwnedOrMutRef, OwnedOrRef};
use peace_profile_model::Profile;
use peace_resource_rt::paths::{
    PeaceAppDir, PeaceDir, ProfileDir, ProfileHistoryDir, WorkspaceDir,
};
use peace_rt_model::{
    params::{
        KeyKnown, KeyMaybe, ParamsKeys, ParamsKeysImpl, ParamsTypeRegs, ProfileParams,
        WorkspaceParams,
    },
    Workspace,
};
use serde::{de::DeserializeOwned, Serialize};

use crate::ctx::CmdCtxTypes;

/// A command that works with multiple profiles, not scoped to a flow.
///
/// ```bash
/// path/to/repo/.peace/envman
/// |- 📝 workspace_params.yaml    # ✅ can read or write `WorkspaceParams`
/// |
/// |- 🌏 internal_dev_a           # ✅ can list multiple `Profile`s
/// |   |- 📝 profile_params.yaml  # ✅ can read multiple `ProfileParams`
/// |   |
/// |   |- ..                      # ❌ cannot read or write `Flow` information
/// |
/// |- 🌏 customer_a_dev           # ✅
/// |   |- 📝 profile_params.yaml  # ✅
/// |
/// |- 🌏 customer_a_prod          # ✅
/// |   |- 📝 profile_params.yaml  # ✅
/// |
/// |- 🌏 workspace_init           # ✅ can list multiple `Profile`s
///     |- 📝 profile_params.yaml  # ❌ cannot read profile params of different underlying type
/// ```
///
/// ## Capabilities
///
/// This kind of command can:
///
/// * Read or write workspace parameters.
/// * Read or write multiple profiles' parameters &ndash; as long as they are of
///   the same type (same `struct`).
///
/// This kind of command cannot:
///
/// * Read or write flow parameters -- see `MultiProfileNoFlow` or
///   `MultiProfileSingleFlow`.
/// * Read or write flow state -- see `MultiProfileNoFlow` or
///   `MultiProfileSingleFlow`.
#[derive(Debug)]
pub struct MultiProfileNoFlow<'ctx, CmdCtxTypesT>
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
}

/// A command that works with multiple profiles, not scoped to a flow.
///
/// ```bash
/// path/to/repo/.peace/envman
/// |- 📝 workspace_params.yaml    # ✅ can read or write `WorkspaceParams`
/// |
/// |- 🌏 internal_dev_a           # ✅ can list multiple `Profile`s
/// |   |- 📝 profile_params.yaml  # ✅ can read multiple `ProfileParams`
/// |   |
/// |   |- ..                      # ❌ cannot read or write `Flow` information
/// |
/// |- 🌏 customer_a_dev           # ✅
/// |   |- 📝 profile_params.yaml  # ✅
/// |
/// |- 🌏 customer_a_prod          # ✅
/// |   |- 📝 profile_params.yaml  # ✅
/// |
/// |- 🌏 workspace_init           # ✅ can list multiple `Profile`s
///     |- 📝 profile_params.yaml  # ❌ cannot read profile params of different underlying type
/// ```
///
/// ## Capabilities
///
/// This kind of command can:
///
/// * Read or write workspace parameters.
/// * Read or write multiple profiles' parameters &ndash; as long as they are of
///   the same type (same `struct`).
///
/// This kind of command cannot:
///
/// * Read or write flow parameters -- see `MultiProfileNoFlow` or
///   `MultiProfileSingleFlow`.
/// * Read or write flow state -- see `MultiProfileNoFlow` or
///   `MultiProfileSingleFlow`.
#[derive(Debug)]
pub struct MultiProfileNoFlowView<'view, CmdCtxTypesT>
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
    pub profiles: &'view Vec<Profile>,
    /// Profile directories that store params and flows.
    pub profile_dirs: &'view BTreeMap<Profile, ProfileDir>,
    /// Directories of each profile's execution history.
    pub profile_history_dirs: &'view BTreeMap<Profile, ProfileHistoryDir>,
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
}

impl<'ctx, CmdCtxTypesT> MultiProfileNoFlow<'ctx, CmdCtxTypesT>
where
    CmdCtxTypesT: CmdCtxTypes,
{
    /// Returns a new `MultiProfileNoFlow` scope.
    #[allow(clippy::too_many_arguments)] // Constructed by proc macro
    pub(crate) fn new(
        output: OwnedOrMutRef<'ctx, CmdCtxTypesT::Output>,
        interruptibility_state: InterruptibilityState<'static, 'static>,
        workspace: OwnedOrRef<'ctx, Workspace>,
        profiles: Vec<Profile>,
        profile_dirs: BTreeMap<Profile, ProfileDir>,
        profile_history_dirs: BTreeMap<Profile, ProfileHistoryDir>,
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
    ) -> Self {
        Self {
            output,
            interruptibility_state,
            workspace,
            profiles,
            profile_dirs,
            profile_history_dirs,
            params_type_regs,
            workspace_params,
            profile_to_profile_params,
        }
    }

    /// Returns a view struct of this scope.
    pub fn view(&mut self) -> MultiProfileNoFlowView<'_, CmdCtxTypesT> {
        let Self {
            output,
            interruptibility_state,
            workspace,
            profiles,
            profile_dirs,
            profile_history_dirs,
            params_type_regs,
            workspace_params,
            profile_to_profile_params,
        } = self;

        let interruptibility_state = interruptibility_state.reborrow();

        MultiProfileNoFlowView {
            output,
            interruptibility_state,
            workspace,
            profiles,
            profile_dirs,
            profile_history_dirs,
            params_type_regs,
            workspace_params,
            profile_to_profile_params,
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

    /// Returns the type registries for [`WorkspaceParams`], [`ProfileParams`],
    /// and [`FlowParams`] deserialization.
    ///
    /// [`WorkspaceParams`]: peace_rt_model::params::WorkspaceParams
    /// [`ProfileParams`]: peace_rt_model::params::ProfileParams
    /// [`FlowParams`]: peace_rt_model::params::FlowParams
    pub fn params_type_regs(&self) -> &ParamsTypeRegs<CmdCtxTypesT::ParamsKeys> {
        &self.params_type_regs
    }
}

impl<CmdCtxTypesT, WorkspaceParamsK, ProfileParamsKMaybe, FlowParamsKMaybe>
    MultiProfileNoFlow<'_, CmdCtxTypesT>
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
    MultiProfileNoFlow<'_, CmdCtxTypesT>
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
