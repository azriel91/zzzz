use std::fmt;

use peace::cfg::{state::FetchedOpt, State};
use serde::{Deserialize, Serialize};

use crate::{ETag, FileDownloadStateLogical};

#[cfg(feature = "output_progress")]
use peace::item_interaction_model::ItemLocationState;

/// Newtype wrapper for `State<FileDownloadStatePhysical, FetchedOpt<ETag>>`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileDownloadState(pub State<FileDownloadStateLogical, FetchedOpt<ETag>>);

impl FileDownloadState {
    /// Returns a new `FileDownloadState`.
    pub fn new(
        file_download_state_logical: FileDownloadStateLogical,
        etag: FetchedOpt<ETag>,
    ) -> Self {
        Self(State::new(file_download_state_logical, etag))
    }
}

impl From<State<FileDownloadStateLogical, FetchedOpt<ETag>>> for FileDownloadState {
    fn from(state: State<FileDownloadStateLogical, FetchedOpt<ETag>>) -> Self {
        Self(state)
    }
}

impl fmt::Display for FileDownloadState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "output_progress")]
impl<'state> From<&'state FileDownloadState> for ItemLocationState {
    fn from(state: &'state FileDownloadState) -> ItemLocationState {
        match &state.0.logical {
            FileDownloadStateLogical::None { .. } => ItemLocationState::NotExists,
            FileDownloadStateLogical::StringContents { .. }
            | FileDownloadStateLogical::Length { .. }
            | FileDownloadStateLogical::Unknown { .. } => ItemLocationState::Exists,
        }
    }
}
