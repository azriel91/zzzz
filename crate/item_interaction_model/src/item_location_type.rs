use serde::{Deserialize, Serialize};

/// The type of resource location.
///
/// This affects how the [`ItemLocation`] is rendered.
///
/// [`ItemLocation`]: crate::ItemLocation
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Deserialize, Serialize)]
pub enum ItemLocationType {
    /// Rendered with dashed lines.
    ///
    /// Suitable for concepts like:
    ///
    /// * Cloud provider
    /// * Network / subnet
    Group,
    /// Rendered with solid lines.
    ///
    /// Suitable for concepts like:
    ///
    /// * Localhost
    /// * Server
    Host,
    /// Rendered with solid lines.
    ///
    /// Suitable for concepts like:
    ///
    /// * File path
    /// * URL
    Path,
}
