use std::marker::PhantomData;

cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        use bytes::Bytes;
        use futures::{Stream, StreamExt, TryStreamExt};
        use tokio::io::AsyncWriteExt;
        use tokio::{fs::File, io::BufWriter};
    } else if #[cfg(target_arch = "wasm32")] {
        use std::path::Path;

        use peace::rt_model::Storage;
    }
}

use peace::cfg::{state::FetchedOpt, ApplyCheck, FnCtx, State};
use reqwest::header::ETAG;

use crate::{
    ETag, FileDownloadData, FileDownloadError, FileDownloadParams, FileDownloadState,
    FileDownloadStateDiff, FileDownloadStateLogical,
};

#[cfg(feature = "output_progress")]
use peace::{
    diff::Tracked,
    progress_model::{ProgressLimit, ProgressMsgUpdate},
};

/// ApplyFns for the file to download.
#[derive(Debug)]
pub struct FileDownloadApplyFns<Id>(PhantomData<Id>);

impl<Id> FileDownloadApplyFns<Id>
where
    Id: Send + Sync + 'static,
{
    async fn file_download(
        #[cfg(not(feature = "output_progress"))] _fn_ctx: FnCtx<'_>,
        #[cfg(feature = "output_progress")] fn_ctx: FnCtx<'_>,
        params: &FileDownloadParams<Id>,
        data: FileDownloadData<'_, Id>,
    ) -> Result<FetchedOpt<ETag>, FileDownloadError> {
        let client = data.client();
        let src_url = params.src();

        #[cfg(feature = "output_progress")]
        fn_ctx
            .progress_sender
            .tick(ProgressMsgUpdate::Set(String::from("starting download")));

        let response = client
            .get(src_url.clone())
            .send()
            .await
            .map_err(|error| FileDownloadError::src_get(src_url.clone(), error))?;

        let e_tag = response
            .headers()
            .get(ETAG)
            .and_then(|header| header.to_str().ok())
            .map(|header| ETag::new(header.to_string()))
            .map(FetchedOpt::Value)
            .unwrap_or(FetchedOpt::None);

        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::stream_write(
                #[cfg(feature = "output_progress")]
                fn_ctx,
                params,
                response.bytes_stream(),
            )
            .await?;
        }

        // reqwest in wasm doesn't support streams
        // https://github.com/seanmonstar/reqwest/issues/1424
        #[cfg(target_arch = "wasm32")]
        {
            Self::stream_write(
                #[cfg(feature = "output_progress")]
                fn_ctx,
                params.dest(),
                data.storage(),
                params.storage_form(),
                response,
            )
            .await?;
        }

        Ok(e_tag)
    }

    /// Streams the content to disk.
    #[cfg(not(target_arch = "wasm32"))]
    async fn stream_write(
        #[cfg(feature = "output_progress")] fn_ctx: FnCtx<'_>,
        file_download_params: &FileDownloadParams<Id>,
        byte_stream: impl Stream<Item = reqwest::Result<Bytes>>,
    ) -> Result<(), FileDownloadError> {
        use std::{fmt::Write, path::Component};

        #[cfg(feature = "error_reporting")]
        use peace::miette::{SourceOffset, SourceSpan};

        let dest_path = file_download_params.dest();
        if let Some(dest_parent) = dest_path.parent() {
            // Ensure all parent directories are created
            tokio::fs::create_dir_all(dest_parent)
                .await
                .map_err(|error| {
                    #[cfg(feature = "error_reporting")]
                    let dest_display = format!("{}", dest_path.display());
                    #[cfg(feature = "error_reporting")]
                    let parent_dirs_span = {
                        let start = 1usize;
                        SourceSpan::from((start, dest_display.len()))
                    };

                    FileDownloadError::DestParentDirsCreate {
                        dest: dest_path.to_path_buf(),
                        dest_parent: dest_parent.to_path_buf(),
                        #[cfg(feature = "error_reporting")]
                        dest_display,
                        #[cfg(feature = "error_reporting")]
                        parent_dirs_span,
                        error,
                    }
                })?;
        }
        let dest_file = File::create(dest_path).await.or_else(|error| {
            let mut init_command_approx = String::with_capacity(256);
            let exe_path = std::env::current_exe().map_err(FileDownloadError::CurrentExeRead)?;
            let exe_name =
                if let Some(Component::Normal(exe_name)) = exe_path.components().next_back() {
                    exe_name
                } else {
                    return Err(FileDownloadError::CurrentExeNameRead);
                };

            let exe_name = exe_name.to_string_lossy();
            let src = file_download_params.src();
            let dest = dest_path.to_path_buf();
            let dest_display = dest.display();

            write!(&mut init_command_approx, "{exe_name} init {src} ")
                .map_err(FileDownloadError::FormatString)?;
            #[cfg(feature = "error_reporting")]
            let dest_offset_col = init_command_approx.len();
            write!(&mut init_command_approx, "{dest_display}")
                .map_err(FileDownloadError::FormatString)?;

            #[cfg(feature = "error_reporting")]
            let dest_span = {
                let loc_line = 1;
                // Add one to offset because we are 1-based, not 0-based?
                let start = SourceOffset::from_location(
                    &init_command_approx,
                    loc_line,
                    dest_offset_col + 1,
                );
                // Add one to length because we are 1-based, not 0-based?
                let length = init_command_approx.len() - dest_offset_col + 1;
                SourceSpan::new(start, length)
            };
            Err(FileDownloadError::DestFileCreate {
                init_command_approx,
                #[cfg(feature = "error_reporting")]
                dest_span,
                dest,
                error,
            })
        })?;

        let buffer = BufWriter::new(dest_file);
        #[cfg(feature = "output_progress")]
        let progress_sender = &fn_ctx.progress_sender;
        let mut buffer = byte_stream
            .map(|bytes_result| bytes_result.map_err(FileDownloadError::ResponseBytesStream))
            .try_fold(buffer, |mut buffer, bytes| async move {
                buffer
                    .write_all(&bytes)
                    .await
                    .map_err(FileDownloadError::ResponseFileWrite)?;

                #[cfg(feature = "output_progress")]
                if let Ok(progress_inc) = u64::try_from(bytes.len()) {
                    progress_sender.inc(progress_inc, ProgressMsgUpdate::NoChange)
                } else {
                    progress_sender.tick(ProgressMsgUpdate::NoChange)
                };

                Ok(buffer)
            })
            .await?;
        buffer
            .flush()
            .await
            .map_err(FileDownloadError::ResponseFileWrite)?;
        Ok(())
    }

    /// Streams the content to disk.
    #[cfg(target_arch = "wasm32")]
    async fn stream_write(
        #[cfg(feature = "output_progress")] _fn_ctx: FnCtx<'_>,
        dest_path: &Path,
        storage: &Storage,
        storage_form: crate::StorageForm,
        response: reqwest::Response,
    ) -> Result<(), FileDownloadError> {
        use crate::StorageForm;

        match storage_form {
            StorageForm::Text => {
                let value = response
                    .text()
                    .await
                    .map_err(FileDownloadError::ResponseTextRead)?;
                storage.set_item(dest_path, &value)?;
            }
            StorageForm::Base64 => {
                let bytes = response
                    .bytes()
                    .await
                    .map_err(FileDownloadError::ResponseBytesRead)?;
                storage.set_item_b64(dest_path, &bytes)?;
            }
        }

        Ok(())
    }
}

impl<Id> FileDownloadApplyFns<Id>
where
    Id: Send + Sync + 'static,
{
    pub async fn apply_check(
        _params: &FileDownloadParams<Id>,
        _data: FileDownloadData<'_, Id>,
        FileDownloadState(State {
            logical: file_state_current,
            physical: _e_tag,
        }): &FileDownloadState,
        _file_download_state_goal: &FileDownloadState,
        diff: &FileDownloadStateDiff,
    ) -> Result<ApplyCheck, FileDownloadError> {
        let apply_check = match diff {
            FileDownloadStateDiff::Change {
                #[cfg(feature = "output_progress")]
                byte_len,
                ..
            } => {
                #[cfg(not(feature = "output_progress"))]
                {
                    ApplyCheck::ExecRequired
                }

                #[cfg(feature = "output_progress")]
                {
                    let progress_limit = match byte_len.to {
                        Tracked::None => ProgressLimit::Unknown,
                        Tracked::Known(len) => len
                            .try_into()
                            .map(ProgressLimit::Bytes)
                            .unwrap_or(ProgressLimit::Unknown),
                        Tracked::Unknown => ProgressLimit::Unknown,
                    };

                    ApplyCheck::ExecRequired { progress_limit }
                }
            }
            FileDownloadStateDiff::Deleted { .. } => match file_state_current {
                FileDownloadStateLogical::None { .. } => ApplyCheck::ExecNotRequired,
                FileDownloadStateLogical::StringContents {
                    path: _,
                    #[cfg(not(feature = "output_progress"))]
                        contents: _,
                    #[cfg(feature = "output_progress")]
                    contents,
                } => {
                    #[cfg(not(feature = "output_progress"))]
                    {
                        ApplyCheck::ExecRequired
                    }
                    #[cfg(feature = "output_progress")]
                    {
                        ApplyCheck::ExecRequired {
                            progress_limit: ProgressLimit::Bytes(
                                contents.as_bytes().len().try_into().unwrap(),
                            ),
                        }
                    }
                }
                FileDownloadStateLogical::Length {
                    path: _,
                    #[cfg(not(feature = "output_progress"))]
                        byte_count: _,
                    #[cfg(feature = "output_progress")]
                    byte_count,
                } => {
                    #[cfg(not(feature = "output_progress"))]
                    {
                        ApplyCheck::ExecRequired
                    }

                    #[cfg(feature = "output_progress")]
                    ApplyCheck::ExecRequired {
                        progress_limit: ProgressLimit::Bytes(*byte_count),
                    }
                }
                FileDownloadStateLogical::Unknown { path: _ } => {
                    #[cfg(not(feature = "output_progress"))]
                    {
                        ApplyCheck::ExecRequired
                    }

                    #[cfg(feature = "output_progress")]
                    ApplyCheck::ExecRequired {
                        progress_limit: ProgressLimit::Unknown,
                    }
                }
            },
            FileDownloadStateDiff::NoChangeNotExists { .. }
            | FileDownloadStateDiff::NoChangeSync { .. } => ApplyCheck::ExecNotRequired,
        };
        Ok(apply_check)
    }

    pub async fn apply_dry(
        _fn_ctx: FnCtx<'_>,
        _params: &FileDownloadParams<Id>,
        _data: FileDownloadData<'_, Id>,
        _file_download_state_current: &FileDownloadState,
        file_download_state_goal: &FileDownloadState,
        _diff: &FileDownloadStateDiff,
    ) -> Result<FileDownloadState, FileDownloadError> {
        // TODO: fetch headers but don't write to file.

        Ok(file_download_state_goal.clone())
    }

    pub async fn apply(
        fn_ctx: FnCtx<'_>,
        params: &FileDownloadParams<Id>,
        data: FileDownloadData<'_, Id>,
        _file_download_state_current: &FileDownloadState,
        file_download_state_goal: &FileDownloadState,
        diff: &FileDownloadStateDiff,
    ) -> Result<FileDownloadState, FileDownloadError> {
        match diff {
            FileDownloadStateDiff::Deleted { path } => {
                #[cfg(feature = "output_progress")]
                fn_ctx
                    .progress_sender
                    .tick(ProgressMsgUpdate::Set(String::from("removing file")));

                #[cfg(not(target_arch = "wasm32"))]
                tokio::fs::remove_file(path)
                    .await
                    .map_err(FileDownloadError::DestFileRemove)?;

                #[cfg(target_arch = "wasm32")]
                data.storage().remove_item(path)?;

                Ok(file_download_state_goal.clone())
            }
            FileDownloadStateDiff::Change { .. } => {
                let e_tag = Self::file_download(fn_ctx, params, data).await?;

                let mut file_download_state_ensured = file_download_state_goal.clone();
                file_download_state_ensured.0.physical = e_tag;

                Ok(file_download_state_ensured)
            }
            FileDownloadStateDiff::NoChangeNotExists { .. }
            | FileDownloadStateDiff::NoChangeSync { .. } => {
                unreachable!("exec is never called when file is in sync.")
            }
        }
    }
}
