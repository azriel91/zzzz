use std::{fmt, marker::PhantomData};

use derivative::Derivative;
use serde::{Deserialize, Serialize};

#[cfg(feature = "output_progress")]
use peace::item_interaction_model::ItemLocationState;

/// State of the shell command execution.
///
/// * If the command has never been executed, this will be `None`.
/// * If it has been executed, this is `Some(String)` captured from stdout.
#[derive(Derivative, Serialize, Deserialize, Eq)]
#[derivative(Clone, Debug, PartialEq)]
pub enum ShCmdStateLogical<Id> {
    /// The command is not executed.
    ///
    /// Represents when the command has either never been executed, or has been
    /// cleaned up.
    None,
    /// Command has not been executed since the source files have been updated.
    Some {
        /// stdout output.
        stdout: String,
        /// stderr output.
        stderr: String,
        /// Marker.
        marker: PhantomData<Id>,
    },
}

impl<Id> fmt::Display for ShCmdStateLogical<Id> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "<none>"),
            Self::Some { stderr, .. } => stderr.fmt(f),
        }
    }
}

#[cfg(feature = "output_progress")]
impl<'state, Id> From<&'state ShCmdStateLogical<Id>> for ItemLocationState {
    fn from(sh_cmd_state: &'state ShCmdStateLogical<Id>) -> ItemLocationState {
        match sh_cmd_state {
            ShCmdStateLogical::Some { .. } => ItemLocationState::Exists,
            ShCmdStateLogical::None => ItemLocationState::NotExists,
        }
    }
}
