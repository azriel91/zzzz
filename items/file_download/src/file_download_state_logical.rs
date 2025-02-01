use std::{fmt, path::PathBuf};

use serde::{Deserialize, Serialize};

#[cfg(feature = "output_progress")]
use peace::item_interaction_model::ItemLocationState;

/// State of the contents of the file to download.
///
/// This is used to represent the state of the source file, as well as the
/// destination file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FileDownloadStateLogical {
    /// File does not exist.
    None {
        /// Path to the tracked file, if any.
        path: Option<PathBuf>,
    },
    /// String contents of the file.
    ///
    /// Use this when:
    ///
    /// * File contents is text.
    /// * File is small enough to load in memory.
    StringContents {
        /// Path to the file.
        path: PathBuf,
        /// Contents of the file.
        contents: String,
    },
    /// Length of the file.
    ///
    /// Use this when:
    ///
    /// * File is not practical to load in memory.
    Length {
        /// Path to the file.
        path: PathBuf,
        /// Number of bytes.
        byte_count: u64,
    },
    /// Cannot determine file state.
    ///
    /// May be used for the goal state
    Unknown {
        /// Path to the file.
        path: PathBuf,
    },
}

impl fmt::Display for FileDownloadStateLogical {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None { path } => {
                if let Some(path) = path {
                    let path = path.display();
                    write!(f, "`{path}` non-existent")
                } else {
                    write!(f, "non-existent")
                }
            }
            Self::StringContents { path, contents } => {
                let path = path.display();
                write!(f, "`{path}` containing \"{contents}\"")
            }
            Self::Length { path, byte_count } => {
                let path = path.display();
                write!(f, "`{path}` containing {byte_count} bytes")
            }
            Self::Unknown { path } => {
                let path = path.display();
                write!(f, "`{path}` (contents not tracked)")
            }
        }
    }
}

#[cfg(feature = "output_progress")]
impl<'state> From<&'state FileDownloadStateLogical> for ItemLocationState {
    fn from(file_download_state: &'state FileDownloadStateLogical) -> ItemLocationState {
        match file_download_state {
            FileDownloadStateLogical::None { .. } => ItemLocationState::NotExists,
            FileDownloadStateLogical::StringContents { .. }
            | FileDownloadStateLogical::Length { .. }
            | FileDownloadStateLogical::Unknown { .. } => todo!(),
        }
    }
}

impl PartialEq for FileDownloadStateLogical {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                FileDownloadStateLogical::None { path: path_self },
                FileDownloadStateLogical::None { path: path_other },
            ) => path_self == path_other,
            (
                FileDownloadStateLogical::Unknown {
                    path: path_self, ..
                },
                FileDownloadStateLogical::StringContents {
                    path: path_other, ..
                },
            )
            | (
                FileDownloadStateLogical::Unknown {
                    path: path_self, ..
                },
                FileDownloadStateLogical::Length {
                    path: path_other, ..
                },
            )
            | (
                FileDownloadStateLogical::StringContents {
                    path: path_self, ..
                },
                FileDownloadStateLogical::Unknown {
                    path: path_other, ..
                },
            )
            | (
                FileDownloadStateLogical::Length {
                    path: path_self, ..
                },
                FileDownloadStateLogical::Unknown {
                    path: path_other, ..
                },
            )
            | (
                FileDownloadStateLogical::Unknown { path: path_self },
                FileDownloadStateLogical::Unknown { path: path_other },
            ) => path_self == path_other,

            (FileDownloadStateLogical::Unknown { .. }, FileDownloadStateLogical::None { .. })
            | (FileDownloadStateLogical::None { .. }, FileDownloadStateLogical::Unknown { .. })
            | (
                FileDownloadStateLogical::None { .. },
                FileDownloadStateLogical::StringContents { .. },
            )
            | (
                FileDownloadStateLogical::StringContents { .. },
                FileDownloadStateLogical::None { .. },
            )
            | (
                FileDownloadStateLogical::StringContents { .. },
                FileDownloadStateLogical::Length { .. },
            )
            | (FileDownloadStateLogical::Length { .. }, FileDownloadStateLogical::None { .. })
            | (
                FileDownloadStateLogical::Length { .. },
                FileDownloadStateLogical::StringContents { .. },
            )
            | (FileDownloadStateLogical::None { .. }, FileDownloadStateLogical::Length { .. }) => {
                false
            }
            (
                FileDownloadStateLogical::StringContents {
                    path: path_self,
                    contents: contents_self,
                },
                FileDownloadStateLogical::StringContents {
                    path: path_other,
                    contents: contents_other,
                },
            ) => path_self == path_other && contents_self == contents_other,
            (
                FileDownloadStateLogical::Length {
                    path: path_self,
                    byte_count: byte_count_self,
                },
                FileDownloadStateLogical::Length {
                    path: path_other,
                    byte_count: byte_count_other,
                },
            ) => path_self == path_other && byte_count_self == byte_count_other,
        }
    }
}
