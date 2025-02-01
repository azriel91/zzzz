use std::path::PathBuf;

use peace_flow_model::FlowId;

use crate::paths::ProfileDir;

/// Directory to store all data produced by the current flow's execution.
///
/// Typically `$workspace_dir/.peace/$app/$profile/$flow_id`.
///
/// This is the directory that contains the information produced and used during
/// a `peace` tool invocation for a particular flow.
///
/// See `FlowDir::from<(&ProfileDir, &FlowId)>` if you want to construct a
/// `FlowDir` with the conventional `$profile_dir/$flow_id` path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlowDir(PathBuf);

crate::paths::pathbuf_newtype!(FlowDir);

impl From<(&ProfileDir, &FlowId)> for FlowDir {
    fn from((peace_dir, flow_id): (&ProfileDir, &FlowId)) -> Self {
        let path = peace_dir.join(flow_id.as_ref());

        Self(path)
    }
}
