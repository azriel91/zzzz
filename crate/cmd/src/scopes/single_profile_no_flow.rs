use std::{fmt::Debug, hash::Hash};

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

/// A command that works with a single profile, not scoped to a flow.
///
/// ```bash
/// path/to/repo/.peace/envman
/// |- 📝 workspace_params.yaml    # ✅ can read or write `WorkspaceParams`
/// |
/// |- 🌏 internal_dev_a           # ✅ can read `Profile`
/// |   |- 📝 profile_params.yaml  # ✅ can read or write `ProfileParams`
/// |   |
/// |   |- 🌊 ..                   # ❌ cannot read or write Flow information
/// |
/// |- 🌏 ..                       # ❌ cannot read or write other `Profile` information
/// ```
///
/// ## Capabilities
///
/// This kind of command can:
///
/// * Read or write workspace parameters.
/// * Read or write a single profile's parameters. For multiple profiles, see
///   `MultiProfileNoFlow`.
///
/// This kind of command cannot:
///
/// * Read or write flow parameters -- see `SingleProfileNoFlow` or
///   `MultiProfileSingleFlow`.
/// * Read or write flow state -- see `SingleProfileNoFlow` or
///   `MultiProfileSingleFlow`.
#[derive(Debug)]
pub struct SingleProfileNoFlow<'ctx, CmdCtxTypesT>
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
    /// The profile this command operates on.
    profile: Profile,
    /// Profile directory that stores params and flows.
    profile_dir: ProfileDir,
    /// Directory to store profile executions' summaries.
    profile_history_dir: ProfileHistoryDir,
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
    profile_params: ProfileParams<
        <<CmdCtxTypesT::ParamsKeys as ParamsKeys>::ProfileParamsKMaybe as KeyMaybe>::Key,
    >,
}

/// A command that works with a single profile, not scoped to a flow.
///
/// ```bash
/// path/to/repo/.peace/envman
/// |- 📝 workspace_params.yaml    # ✅ can read or write `WorkspaceParams`
/// |
/// |- 🌏 internal_dev_a           # ✅ can read `Profile`
/// |   |- 📝 profile_params.yaml  # ✅ can read or write `ProfileParams`
/// |   |
/// |   |- 🌊 ..                   # ❌ cannot read or write Flow information
/// |
/// |- 🌏 ..                       # ❌ cannot read or write other `Profile` information
/// ```
///
/// ## Capabilities
///
/// This kind of command can:
///
/// * Read or write workspace parameters.
/// * Read or write a single profile's parameters. For multiple profiles, see
///   `MultiProfileNoFlow`.
///
/// This kind of command cannot:
///
/// * Read or write flow parameters -- see `SingleProfileNoFlow` or
///   `MultiProfileSingleFlow`.
/// * Read or write flow state -- see `SingleProfileNoFlow` or
///   `MultiProfileSingleFlow`.
#[derive(Debug)]
pub struct SingleProfileNoFlowView<'view, CmdCtxTypesT>
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
    /// The profile this command operates on.
    pub profile: &'view Profile,
    /// Profile directory that stores params and flows.
    pub profile_dir: &'view ProfileDir,
    /// Directory to store profile executions' summaries.
    pub profile_history_dir: &'view ProfileHistoryDir,
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
    pub profile_params: &'view ProfileParams<
        <<CmdCtxTypesT::ParamsKeys as ParamsKeys>::ProfileParamsKMaybe as KeyMaybe>::Key,
    >,
}

impl<'ctx, CmdCtxTypesT> SingleProfileNoFlow<'ctx, CmdCtxTypesT>
where
    CmdCtxTypesT: CmdCtxTypes,
{
    /// Returns a new `SingleProfileNoFlow` scope.
    #[allow(clippy::too_many_arguments)] // Constructed by proc macro
    pub(crate) fn new(
        output: OwnedOrMutRef<'ctx, CmdCtxTypesT::Output>,
        interruptibility_state: InterruptibilityState<'static, 'static>,
        workspace: OwnedOrRef<'ctx, Workspace>,
        profile: Profile,
        profile_dir: ProfileDir,
        profile_history_dir: ProfileHistoryDir,
        params_type_regs: ParamsTypeRegs<CmdCtxTypesT::ParamsKeys>,
        workspace_params: WorkspaceParams<
            <<CmdCtxTypesT::ParamsKeys as ParamsKeys>::WorkspaceParamsKMaybe as KeyMaybe>::Key,
        >,
        profile_params: ProfileParams<
            <<CmdCtxTypesT::ParamsKeys as ParamsKeys>::ProfileParamsKMaybe as KeyMaybe>::Key,
        >,
    ) -> Self {
        Self {
            output,
            interruptibility_state,
            workspace,
            profile,
            profile_dir,
            profile_history_dir,
            params_type_regs,
            workspace_params,
            profile_params,
        }
    }

    /// Returns a view struct of this scope.
    pub fn view(&mut self) -> SingleProfileNoFlowView<'_, CmdCtxTypesT> {
        let Self {
            output,
            interruptibility_state,
            workspace,
            profile,
            profile_dir,
            profile_history_dir,
            params_type_regs,
            workspace_params,
            profile_params,
        } = self;

        let interruptibility_state = interruptibility_state.reborrow();

        SingleProfileNoFlowView {
            output,
            interruptibility_state,
            workspace,
            profile,
            profile_dir,
            profile_history_dir,
            params_type_regs,
            workspace_params,
            profile_params,
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

    /// Returns a reference to the profile.
    pub fn profile(&self) -> &Profile {
        &self.profile
    }

    /// Returns a reference to the profile directory.
    pub fn profile_dir(&self) -> &ProfileDir {
        &self.profile_dir
    }

    /// Returns a reference to the profile history directory.
    pub fn profile_history_dir(&self) -> &ProfileHistoryDir {
        &self.profile_history_dir
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
    SingleProfileNoFlow<'_, CmdCtxTypesT>
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
    SingleProfileNoFlow<'_, CmdCtxTypesT>
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
    /// Returns the profile params.
    pub fn profile_params(&self) -> &ProfileParams<ProfileParamsK> {
        &self.profile_params
    }
}
