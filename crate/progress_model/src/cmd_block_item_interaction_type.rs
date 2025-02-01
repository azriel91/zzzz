use serde::{Deserialize, Serialize};

/// Type of interactions that a `CmdBlock`s has with `ItemLocation`s.
///
/// # Design
///
/// Together with `ProgressStatus` and `ItemLocationState`, this is used to
/// compute how an `ItemLocation` should be rendered to a user.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum CmdBlockItemInteractionType {
    /// Creates / modifies / deletes the item.
    ///
    /// Makes write calls to `ItemLocation`s.
    Write,
    /// Makes read-only calls to `ItemLocation`s.
    Read,
    /// Local logic that does not interact with `ItemLocation`s / external
    /// services.
    Local,
}
