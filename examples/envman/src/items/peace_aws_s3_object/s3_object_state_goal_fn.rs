use std::{fmt::Write, marker::PhantomData, path::Path};

use peace::{
    cfg::{state::Generated, FnCtx},
    params::Params,
};

use crate::items::peace_aws_s3_object::{
    S3ObjectData, S3ObjectError, S3ObjectParams, S3ObjectState,
};

#[cfg(feature = "output_progress")]
use peace::progress_model::ProgressMsgUpdate;

/// Reads the goal state of the S3 object state.
#[derive(Debug)]
pub struct S3ObjectStateGoalFn<Id>(PhantomData<Id>);

impl<Id> S3ObjectStateGoalFn<Id>
where
    Id: Send + Sync,
{
    pub async fn try_state_goal(
        fn_ctx: FnCtx<'_>,
        params_partial: &<S3ObjectParams<Id> as Params>::Partial,
        _data: S3ObjectData<'_, Id>,
    ) -> Result<Option<S3ObjectState>, S3ObjectError> {
        let file_path = params_partial.file_path();
        let bucket_name = params_partial.bucket_name();
        let object_key = params_partial.object_key();
        if let Some(((file_path, bucket_name), object_key)) =
            file_path.zip(bucket_name).zip(object_key)
        {
            #[cfg(not(target_arch = "wasm32"))]
            {
                if !tokio::fs::try_exists(file_path).await.map_err(|error| {
                    S3ObjectError::ObjectFileExists {
                        file_path: file_path.to_path_buf(),
                        bucket_name: bucket_name.to_string(),
                        object_key: object_key.to_string(),
                        error,
                    }
                })? {
                    return Ok(None);
                }
            }
            Self::state_goal_internal(
                fn_ctx,
                file_path,
                bucket_name.to_string(),
                object_key.to_string(),
            )
            .await
            .map(Some)
        } else {
            Ok(None)
        }
    }

    pub async fn state_goal(
        fn_ctx: FnCtx<'_>,
        params: &S3ObjectParams<Id>,
        _data: S3ObjectData<'_, Id>,
    ) -> Result<S3ObjectState, S3ObjectError> {
        let file_path = params.file_path();
        let bucket_name = params.bucket_name().to_string();
        let object_key = params.object_key().to_string();
        Self::state_goal_internal(fn_ctx, file_path, bucket_name, object_key).await
    }

    async fn state_goal_internal(
        fn_ctx: FnCtx<'_>,
        file_path: &Path,
        bucket_name: String,
        object_key: String,
    ) -> Result<S3ObjectState, S3ObjectError> {
        #[cfg(not(feature = "output_progress"))]
        let _fn_ctx = fn_ctx;
        #[cfg(feature = "output_progress")]
        let progress_sender = &fn_ctx.progress_sender;
        #[cfg(feature = "output_progress")]
        progress_sender.tick(ProgressMsgUpdate::Set(String::from("computing MD5 sum")));

        #[cfg(not(target_arch = "wasm32"))]
        let content_md5_bytes = {
            use tokio::{fs::File, io::AsyncReadExt};

            let mut md5_ctx = md5_rs::Context::new();
            let mut file =
                File::open(file_path)
                    .await
                    .map_err(|error| S3ObjectError::ObjectFileOpen {
                        file_path: file_path.to_path_buf(),
                        bucket_name: bucket_name.clone(),
                        object_key: object_key.clone(),
                        error,
                    })?;

            let mut bytes_buffer = [0u8; 1024];
            loop {
                match file.read(&mut bytes_buffer).await.map_err(|error| {
                    S3ObjectError::ObjectFileRead {
                        file_path: file_path.to_path_buf(),
                        bucket_name: bucket_name.clone(),
                        object_key: object_key.clone(),
                        error,
                    }
                })? {
                    0 => break md5_ctx.finish(),
                    n => {
                        md5_ctx.read(&bytes_buffer[..n]);
                    }
                }
            }
        };
        #[cfg(feature = "output_progress")]
        progress_sender.tick(ProgressMsgUpdate::Set(String::from("MD5 sum computed")));

        let content_md5_hexstr = content_md5_bytes
            .iter()
            .try_fold(
                String::with_capacity(content_md5_bytes.len() * 2),
                |mut hexstr, x| {
                    write!(&mut hexstr, "{:02x}", x)?;
                    Result::<_, std::fmt::Error>::Ok(hexstr)
                },
            )
            .expect("Failed to construct hexstring from S3 object MD5.");

        Ok(S3ObjectState::Some {
            bucket_name,
            object_key,
            content_md5_hexstr: Some(content_md5_hexstr),
            e_tag: Generated::Tbd,
        })
    }
}
