use peace::profile_model::Profile;
use semver::Version;
use url::Url;

use crate::model::{EnvType, RepoSlug};

/// Whether to switch to an existing profile or to a new one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProfileSwitch {
    /// Switch to an existing profile.
    ToExisting {
        /// The existing profile to switch to.
        profile: Profile,
    },
    /// Switch to a newly created profile.
    CreateNew {
        /// The existing profile to switch to.
        profile: Profile,
        /// Type of environment: development or production.
        env_type: EnvType,
        /// Username and repository of the application to download.
        slug: RepoSlug,
        /// Version of the application to download.
        version: Version,
        /// URL to override where to download the application from.
        url: Option<Url>,
    },
}
