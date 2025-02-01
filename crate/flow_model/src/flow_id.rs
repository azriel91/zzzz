use std::borrow::Cow;

use serde::{Deserialize, Serialize};

/// Identifier or name of a process flow.
///
/// Examples are `"dev_env"` and `"artifact"` in the following snippet:
///
/// ```bash
/// peace dev_env discover # StatesDiscoverCmd
/// peace dev_env status   # StatesCurrentStoredDisplayCmd
/// peace dev_env deploy   # EnsureCmd
/// peace dev_env clean    # CleanCmd
///
/// peace artifact discover # StatesDiscoverCmd
/// peace artifact status   # StatesCurrentStoredDisplayCmd
/// peace artifact publish  # EnsureCmd
/// ```
///
/// Must begin with a letter or underscore, and contain only letters, numbers,
/// and underscores.
///
/// # Examples
///
/// The following are all examples of valid `FlowId`s:
///
/// ```rust
/// # use peace_flow_model::{flow_id, FlowId};
/// #
/// let _snake = flow_id!("snake_case");
/// let _camel = flow_id!("camelCase");
/// let _pascal = flow_id!("PascalCase");
/// ```
#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct FlowId(Cow<'static, str>);

peace_core::id_newtype!(FlowId, FlowIdInvalidFmt, flow_id, code_inline);
