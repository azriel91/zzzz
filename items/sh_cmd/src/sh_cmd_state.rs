use std::fmt;

use derivative::Derivative;
use peace::cfg::State;
use serde::{Deserialize, Serialize};

use crate::{ShCmdExecutionRecord, ShCmdStateLogical};

#[cfg(feature = "output_progress")]
use peace::item_interaction_model::ItemLocationState;

/// Newtype wrapper for `State<ShCmdStatePhysical<Id>, ShCmdExecutionRecord>`.
#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Clone(bound = ""), Debug(bound = ""), PartialEq(bound = ""))]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct ShCmdState<Id>(pub State<ShCmdStateLogical<Id>, ShCmdExecutionRecord>);

impl<Id> ShCmdState<Id> {
    /// Returns a new `ShCmdState<Id>`.
    pub fn new(
        sh_cmd_state_physical: ShCmdStateLogical<Id>,
        execution_record: ShCmdExecutionRecord,
    ) -> Self {
        Self(State::new(sh_cmd_state_physical, execution_record))
    }
}

impl<Id> From<State<ShCmdStateLogical<Id>, ShCmdExecutionRecord>> for ShCmdState<Id> {
    fn from(state: State<ShCmdStateLogical<Id>, ShCmdExecutionRecord>) -> Self {
        Self(state)
    }
}

impl<Id> fmt::Display for ShCmdState<Id> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "output_progress")]
impl<'state, Id> From<&'state ShCmdState<Id>> for ItemLocationState {
    fn from(state: &'state ShCmdState<Id>) -> ItemLocationState {
        match &state.0.logical {
            ShCmdStateLogical::Some { .. } => ItemLocationState::Exists,
            ShCmdStateLogical::None => ItemLocationState::NotExists,
        }
    }
}
