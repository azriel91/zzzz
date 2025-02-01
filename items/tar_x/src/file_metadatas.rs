use std::fmt;

use serde::{Deserialize, Serialize};

use crate::FileMetadata;

#[cfg(feature = "output_progress")]
use peace::item_interaction_model::ItemLocationState;

/// Metadata of files to extract.
///
/// The `FileMetadata`s are sorted by their path.
///
/// This should be constructed using the `From<Vec<FileMetadata>>` function.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileMetadatas(Vec<FileMetadata>);

impl FileMetadatas {
    /// Returns the inner `Vec<FileMetadata>`.
    pub fn into_inner(self) -> Vec<FileMetadata> {
        self.0
    }

    /// Returns a mutable iterator over the file metadatas.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, FileMetadata> {
        self.0.iter_mut()
    }
}

impl std::ops::Deref for FileMetadatas {
    type Target = Vec<FileMetadata>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for FileMetadatas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let len = self.len();
        let s = if len == 1 { "" } else { "s" };
        write!(f, "{len} file{s}")
    }
}

impl From<Vec<FileMetadata>> for FileMetadatas {
    fn from(mut file_metadatas: Vec<FileMetadata>) -> Self {
        file_metadatas.sort_by(|file_metadata_a, file_metadata_b| {
            file_metadata_a.path().cmp(file_metadata_b.path())
        });

        Self(file_metadatas)
    }
}

#[cfg(feature = "output_progress")]
impl<'state> From<&'state FileMetadatas> for ItemLocationState {
    fn from(file_metadatas: &'state FileMetadatas) -> ItemLocationState {
        match file_metadatas.0.is_empty() {
            true => ItemLocationState::NotExists,
            false => ItemLocationState::Exists,
        }
    }
}
