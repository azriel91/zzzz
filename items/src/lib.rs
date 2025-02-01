//! Collection of items the peace framework.
//!
//! Every item crate needs to be enabled with its own feature. Example:
//!
//! ```toml
//! peace_items = { version = "0.0.3", features = ["file_download"] }
//! ```
//!
//! In code:
//!
//! ```rust
//! #[cfg(feature = "file_download")]
//! # fn main() {
//! use peace::item_model::{item_id, ItemId};
//! use peace_items::file_download::FileDownloadItem;
//!
//! /// Marker type for `FileDownloadParams`.
//! #[derive(Clone, Copy, Debug, PartialEq, Eq)]
//! pub struct MyFileId;
//!
//! let file_download_item =
//!     FileDownloadItem::<MyFileId>::new(item_id!("file_to_download"));
//! # }
//! #
//! #[cfg(not(feature = "file_download"))]
//! # fn main() {}
//! ```

// Re-exports
#[cfg(feature = "blank")]
pub use peace_item_blank as blank;
#[cfg(feature = "file_download")]
pub use peace_item_file_download as file_download;
#[cfg(feature = "sh_cmd")]
pub use peace_item_sh_cmd as sh_cmd;
#[cfg(feature = "tar_x")]
pub use peace_item_tar_x as tar_x;
