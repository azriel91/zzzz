use std::fmt;

use serde::{Deserialize, Serialize};

#[cfg(feature = "output_progress")]
use peace::item_interaction_model::ItemLocationState;

/// Logical blank state.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct BlankState(pub Option<u32>);

impl fmt::Display for BlankState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            Some(n) => n.fmt(f),
            None => "<none>".fmt(f),
        }
    }
}

impl std::ops::Deref for BlankState {
    type Target = Option<u32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for BlankState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(feature = "output_progress")]
impl<'state> From<&'state BlankState> for ItemLocationState {
    fn from(blank_state: &'state BlankState) -> ItemLocationState {
        match blank_state.is_some() {
            true => ItemLocationState::Exists,
            false => ItemLocationState::NotExists,
        }
    }
}
