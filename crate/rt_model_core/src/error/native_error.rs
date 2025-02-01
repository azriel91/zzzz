use std::{ffi::OsString, path::PathBuf, sync::Mutex};

use peace_profile_model::ProfileInvalidFmt;
use peace_resource_rt::paths::WorkspaceDir;

/// Peace runtime errors.
#[cfg_attr(feature = "error_reporting", derive(miette::Diagnostic))]
#[derive(Debug, thiserror::Error)]
pub enum NativeError {
    /// Failed to present data.
    #[error("Failed to present data.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model_native::cli_output_present))
    )]
    CliOutputPresent(#[source] std::io::Error),

    #[error("Failed to set current dir to workspace directory: `{}`", workspace_dir.display())]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model_native::current_dir_set))
    )]
    CurrentDirSet {
        /// The workspace directory.
        workspace_dir: WorkspaceDir,
        /// Underlying IO error
        #[source]
        error: std::io::Error,
    },

    /// Failed to create file for writing.
    #[error("Failed to create file for writing: `{path}`")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model_native::file_create))
    )]
    FileCreate {
        /// Path to the file.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        error: std::io::Error,
    },

    /// Failed to open file for reading.
    #[error("Failed to open file for reading: `{path}`")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model_native::file_open))
    )]
    FileOpen {
        /// Path to the file.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        error: std::io::Error,
    },

    /// Failed to read from file.
    #[error("Failed to read from file: `{path}`")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model_native::file_read))
    )]
    FileRead {
        /// Path to the file.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        error: std::io::Error,
    },

    /// Failed to write to file.
    #[error("Failed to write to file: `{path}`")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model_native::file_write))
    )]
    FileWrite {
        /// Path to the file.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        error: std::io::Error,
    },

    /// Failed to list entries in `PeaceAppDir`.
    #[error("Failed to list entries in `PeaceAppDir`: {}", peace_app_dir.display())]
    PeaceAppDirRead {
        /// Path to the `PeaceAppDir`.
        peace_app_dir: PathBuf,
        /// Underlying IO error.
        #[source]
        error: std::io::Error,
    },

    /// Failed to read entry in `PeaceAppDir`.
    #[error("Failed to read entry in `PeaceAppDir`: {}", peace_app_dir.display())]
    PeaceAppDirEntryRead {
        /// Path to the `PeaceAppDir`.
        peace_app_dir: PathBuf,
        /// Underlying IO error.
        #[source]
        error: std::io::Error,
    },

    /// Failed to read entry file type in `PeaceAppDir`.
    #[error("Failed to read entry file type in `PeaceAppDir`: {}", path.display())]
    PeaceAppDirEntryFileTypeRead {
        /// Path to the entry within `PeaceAppDir`.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        error: std::io::Error,
    },

    /// Profile directory name is not a valid profile name.
    #[error("Profile directory name is not a valid profile name: {}, path: {}", dir_name, path.display())]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model_native::profile_dir_invalid_name))
    )]
    ProfileDirInvalidName {
        /// Name of the directory attempted to be parsed as a `Profile`.
        dir_name: String,
        /// Path to the profile directory.
        path: PathBuf,
        /// Underlying error,
        error: ProfileInvalidFmt<'static>,
    },

    /// Failed to write to stdout.
    #[error("Failed to write to stdout.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model_native::stdout_write))
    )]
    StdoutWrite(#[source] std::io::Error),

    /// Storage synchronous thread failed to be joined.
    ///
    /// This variant is used for thread spawning errors for both reads and
    /// writes.
    #[error("Storage synchronous thread failed to be joined.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model_native::storage_sync_thread_spawn))
    )]
    StorageSyncThreadSpawn(#[source] std::io::Error),

    /// Storage synchronous thread failed to be joined.
    ///
    /// This variant is used for thread spawning errors for both reads and
    /// writes.
    ///
    /// Note: The underlying thread join error does not implement
    /// `std::error::Error`. See
    /// <https://doc.rust-lang.org/std/thread/type.Result.html>.
    ///
    /// The `Mutex` is needed to allow `Error` to be `Sync`.
    #[error("Storage synchronous thread failed to be joined.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model_native::storage_sync_thread_join))
    )]
    StorageSyncThreadJoin(Mutex<Box<dyn std::any::Any + Send + 'static>>),

    /// Failed to read current directory to discover workspace directory.
    #[error("Failed to read current directory to discover workspace directory.")]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model_native::working_dir_read))
    )]
    WorkingDirRead(#[source] std::io::Error),

    /// Failed to create a workspace directory.
    #[error("Failed to create workspace directory: `{path}`.", path = path.display())]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model_native::workspace_dir_create))
    )]
    WorkspaceDirCreate {
        /// The directory that was attempted to be created.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        error: std::io::Error,
    },

    /// Failed to determine workspace directory.
    #[error(
        "Failed to determine workspace directory as could not find `{file_name}` \
            in `{working_dir}` or any parent directories.",
        file_name = file_name.to_string_lossy(),
        working_dir = working_dir.display())]
    #[cfg_attr(
        feature = "error_reporting",
        diagnostic(code(peace_rt_model_native::workspace_file_not_found))
    )]
    WorkspaceFileNotFound {
        /// Beginning directory of traversal.
        working_dir: PathBuf,
        /// File or directory name searched for.
        file_name: OsString,
    },
}
