use std::path::PathBuf;

use peace_profile_model::Profile;

use crate::paths::PeaceAppDir;

/// Directory to store all data produced by the current profile's execution.
///
/// Typically `$workspace_dir/.peace/$app/$profile`.
///
/// This is the directory that contains all information produced and used during
/// a `peace` tool invocation. Exceptions include authentication information
/// stored in their respective directories on the file system, such as
/// application credentials stored in `~/${app}/credentials`.
///
/// See `ProfileDir::from<(&PeaceAppDir, &Profile)>` if you want to construct a
/// `ProfileDir` with the conventional `$peace_dir/.peace/$app/$profile` path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileDir(PathBuf);

crate::paths::pathbuf_newtype!(ProfileDir);

impl From<(&PeaceAppDir, &Profile)> for ProfileDir {
    fn from((peace_app_dir, profile): (&PeaceAppDir, &Profile)) -> Self {
        let path = peace_app_dir.join(profile.as_ref());

        Self(path)
    }
}
