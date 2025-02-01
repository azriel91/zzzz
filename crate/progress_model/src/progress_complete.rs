use serde::{Deserialize, Serialize};

/// Progress completion outcome.
///
/// # Implementation Note
///
/// Should we split `Fail` to be `Fail` and `Error`?
///
/// The distinction will allow determining if:
///
/// * The end user is using invalid values (need for education).
/// * The environment is flakey (need for stabilization).
///
/// Ideally [rust#84277] is implemented so `Item` implementations can use
/// `?` when returning each variant.
///
/// [rust#84277]: https://github.com/rust-lang/rust/issues/84277
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum ProgressComplete {
    /// Execution completed successfully.
    Success,
    /// Execution did not complete.
    Fail,
}

impl ProgressComplete {
    /// Returns whether this is a successful outcome.
    pub fn is_successful(&self) -> bool {
        match self {
            Self::Success => true,
            Self::Fail => false,
        }
    }

    /// Returns whether this is a failure outcome.
    pub fn is_failure(&self) -> bool {
        match self {
            Self::Success => false,
            Self::Fail => true,
        }
    }
}
