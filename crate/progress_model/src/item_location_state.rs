use serde::{Deserialize, Serialize};

/// A low-resolution representation of the state of an [`ItemLocation`].
///
/// Combined with [`ProgressStatus`], [`ItemLocationStateInProgress`] can be
/// computed, to determine how an [`ItemLocation`] should be rendered.
///
/// [`ItemLocation`]: crate::ItemLocation
/// [`ItemLocationStateInProgress`]: crate::ItemLocationStateInProgress
/// [`ProgressStatus`]: crate::ProgressStatus
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum ItemLocationState {
    /// [`ItemLocation`] does not exist.
    ///
    /// This means it should be rendered invisible / low opacity.
    NotExists,
    /// [`ItemLocation`] exists.
    ///
    /// This means it should be rendered with full opacity.
    ///
    /// [`ItemLocation`]: crate::ItemLocation
    Exists,
}
