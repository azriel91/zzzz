use peace_progress_model::{CmdBlockItemInteractionType, ProgressComplete, ProgressStatus};
use serde::{Deserialize, Serialize};

use crate::ItemLocationState;

/// Represents the state of an [`ItemLocation`].
///
/// This affects how the [`ItemLocation`] is rendered.
///
/// This is analogous to [`ItemLocationState`], with added variants for when the
/// state is being determined.
///
/// [`ItemLocation`]: crate::ItemLocation
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum ItemLocationStateInProgress {
    /// [`ItemLocation`] does not exist.
    ///
    /// This means it should be rendered invisible / low opacity.
    ///
    /// [`ItemLocation`]: crate::ItemLocation
    NotExists,
    /// [`ItemLocation`] should not exist, but does.
    ///
    /// This means it should be rendered red outlined with full opacity.
    ///
    /// [`ItemLocation`]: crate::ItemLocation
    NotExistsError,
    /// [`ItemLocation`] may or may not exist, and we are in the process of
    /// determining that.
    ///
    /// This means it should be rendered pulsing / mid opacity.
    ///
    /// [`ItemLocation`]: crate::ItemLocation
    DiscoverInProgress,
    /// [`ItemLocation`] may or may not exist, and we failed to discover that.
    ///
    /// This means it should be rendered red / mid opacity.
    ///
    /// [`ItemLocation`]: crate::ItemLocation
    DiscoverError,
    /// [`ItemLocation`] is being created.
    ///
    /// This means it should be rendered with full opacity and blue animated
    /// outlines.
    ///
    /// [`ItemLocation`]: crate::ItemLocation
    CreateInProgress,
    /// [`ItemLocation`] is being modified.
    ///
    /// This means it should be rendered with full opacity and blue animated
    /// outlines.
    ///
    /// [`ItemLocation`]: crate::ItemLocation
    ModificationInProgress,
    /// [`ItemLocation`] exists.
    ///
    /// This means it should be rendered with full opacity.
    ///
    /// [`ItemLocation`]: crate::ItemLocation
    ExistsOk,
    /// [`ItemLocation`] exists, but is in an erroneous state.
    ///
    /// This means it should be rendered with full opacity with a red shape
    /// colour.
    ///
    /// [`ItemLocation`]: crate::ItemLocation
    ExistsError,
}

impl ItemLocationStateInProgress {
    #[rustfmt::skip]
    pub fn from(
        cmd_block_item_interaction_type: CmdBlockItemInteractionType,
        item_location_state: ItemLocationState,
        progress_status: ProgressStatus,
    ) -> Self {
        match (
            cmd_block_item_interaction_type,
            item_location_state,
            progress_status,
        ) {
            (CmdBlockItemInteractionType::Write, ItemLocationState::NotExists, ProgressStatus::Initialized) => Self::NotExists,
            (CmdBlockItemInteractionType::Write, ItemLocationState::NotExists, ProgressStatus::Interrupted) => Self::NotExists,
            (CmdBlockItemInteractionType::Write, ItemLocationState::NotExists, ProgressStatus::ExecPending) => Self::NotExists,
            (CmdBlockItemInteractionType::Write, ItemLocationState::NotExists, ProgressStatus::Queued) => Self::NotExists,
            (CmdBlockItemInteractionType::Write, ItemLocationState::NotExists, ProgressStatus::Running) => Self::CreateInProgress,
            (CmdBlockItemInteractionType::Write, ItemLocationState::NotExists, ProgressStatus::RunningStalled) => Self::CreateInProgress,
            (CmdBlockItemInteractionType::Write, ItemLocationState::NotExists, ProgressStatus::UserPending) => Self::CreateInProgress,
            (CmdBlockItemInteractionType::Write, ItemLocationState::NotExists, ProgressStatus::Complete(ProgressComplete::Success)) => Self::NotExists,
            (CmdBlockItemInteractionType::Write, ItemLocationState::NotExists, ProgressStatus::Complete(ProgressComplete::Fail)) => Self::NotExistsError,
            (CmdBlockItemInteractionType::Write, ItemLocationState::Exists, ProgressStatus::Initialized) => Self::ExistsOk,
            (CmdBlockItemInteractionType::Write, ItemLocationState::Exists, ProgressStatus::Interrupted) => Self::ExistsOk,
            (CmdBlockItemInteractionType::Write, ItemLocationState::Exists, ProgressStatus::ExecPending) => Self::ExistsOk,
            (CmdBlockItemInteractionType::Write, ItemLocationState::Exists, ProgressStatus::Queued) => Self::ExistsOk,
            (CmdBlockItemInteractionType::Write, ItemLocationState::Exists, ProgressStatus::Running) => Self::ModificationInProgress,
            (CmdBlockItemInteractionType::Write, ItemLocationState::Exists, ProgressStatus::RunningStalled) => Self::ModificationInProgress,
            (CmdBlockItemInteractionType::Write, ItemLocationState::Exists, ProgressStatus::UserPending) => Self::ModificationInProgress,
            (CmdBlockItemInteractionType::Write, ItemLocationState::Exists, ProgressStatus::Complete(ProgressComplete::Success)) => Self::ExistsOk,
            (CmdBlockItemInteractionType::Write, ItemLocationState::Exists, ProgressStatus::Complete(ProgressComplete::Fail)) => Self::ExistsError,
            (CmdBlockItemInteractionType::Read, ItemLocationState::NotExists, ProgressStatus::Initialized) => Self::NotExists,
            (CmdBlockItemInteractionType::Read, ItemLocationState::NotExists, ProgressStatus::Interrupted) => Self::NotExists,
            (CmdBlockItemInteractionType::Read, ItemLocationState::NotExists, ProgressStatus::ExecPending) => Self::NotExists,
            (CmdBlockItemInteractionType::Read, ItemLocationState::NotExists, ProgressStatus::Queued) => Self::NotExists,
            (CmdBlockItemInteractionType::Read, ItemLocationState::NotExists, ProgressStatus::Running) => Self::DiscoverInProgress,
            (CmdBlockItemInteractionType::Read, ItemLocationState::NotExists, ProgressStatus::RunningStalled) => Self::DiscoverInProgress,
            (CmdBlockItemInteractionType::Read, ItemLocationState::NotExists, ProgressStatus::UserPending) => Self::DiscoverInProgress,
            (CmdBlockItemInteractionType::Read, ItemLocationState::NotExists, ProgressStatus::Complete(ProgressComplete::Success)) => Self::NotExists,
            (CmdBlockItemInteractionType::Read, ItemLocationState::NotExists, ProgressStatus::Complete(ProgressComplete::Fail)) => Self::DiscoverError,
            (CmdBlockItemInteractionType::Read, ItemLocationState::Exists, ProgressStatus::Initialized) => Self::ExistsOk,
            (CmdBlockItemInteractionType::Read, ItemLocationState::Exists, ProgressStatus::Interrupted) => Self::ExistsOk,
            (CmdBlockItemInteractionType::Read, ItemLocationState::Exists, ProgressStatus::ExecPending) => Self::ExistsOk,
            (CmdBlockItemInteractionType::Read, ItemLocationState::Exists, ProgressStatus::Queued) => Self::ExistsOk,
            (CmdBlockItemInteractionType::Read, ItemLocationState::Exists, ProgressStatus::Running) => Self::ModificationInProgress,
            (CmdBlockItemInteractionType::Read, ItemLocationState::Exists, ProgressStatus::RunningStalled) => Self::ModificationInProgress,
            (CmdBlockItemInteractionType::Read, ItemLocationState::Exists, ProgressStatus::UserPending) => Self::ModificationInProgress,
            (CmdBlockItemInteractionType::Read, ItemLocationState::Exists, ProgressStatus::Complete(ProgressComplete::Success)) => Self::ExistsOk,
            (CmdBlockItemInteractionType::Read, ItemLocationState::Exists, ProgressStatus::Complete(ProgressComplete::Fail)) => Self::ExistsError,
            (CmdBlockItemInteractionType::Local, ItemLocationState::NotExists, _) => Self::NotExists,
            (CmdBlockItemInteractionType::Local, ItemLocationState::Exists, _) => Self::ExistsOk,
        }
    }
}
