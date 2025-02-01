use std::borrow::Cow;

use serde::{Deserialize, Serialize};

/// Identifier or namespace to distinguish execution environments.
///
/// Example suitable identifiers are:
///
/// * `"dev_user1"`
/// * `"dev_user2"`
/// * `"prod_customer1"`
///
/// Must begin with a letter or underscore, and contain only letters, numbers,
/// and underscores.
///
/// # Examples
///
/// The following are all examples of valid `Profile`s:
///
/// ```rust
/// # use peace_profile_model::{profile, Profile};
/// #
/// let _snake = profile!("snake_case");
/// let _camel = profile!("camelCase");
/// let _pascal = profile!("PascalCase");
/// ```
#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize, PartialOrd, Ord)]
pub struct Profile(Cow<'static, str>);

peace_core::id_newtype!(Profile, ProfileInvalidFmt, profile, tag);
