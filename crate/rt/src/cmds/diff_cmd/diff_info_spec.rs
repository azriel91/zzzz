use peace_profile_model::Profile;

use crate::cmds::diff_cmd::DiffStateSpec;

/// Indicates where to source information to diff.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiffInfoSpec<'diff> {
    /// Profile to read parameters and state from.
    pub profile: &'diff Profile,
    /// Whether to read stored state or discover state.
    pub diff_state_spec: DiffStateSpec,
}

impl<'diff> DiffInfoSpec<'diff> {
    /// Returns a new `DiffInfoSpec`.
    pub fn new(profile: &'diff Profile, diff_state_spec: DiffStateSpec) -> Self {
        Self {
            profile,
            diff_state_spec,
        }
    }

    /// Returns the profile to read parameters and state from.
    pub fn profile(&self) -> &Profile {
        self.profile
    }

    /// Returns whether to read stored state or discover state.
    pub fn diff_state_spec(&self) -> DiffStateSpec {
        self.diff_state_spec
    }
}
