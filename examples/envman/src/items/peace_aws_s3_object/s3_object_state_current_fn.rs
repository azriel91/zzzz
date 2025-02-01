use std::marker::PhantomData;

use aws_sdk_iam::error::SdkError;
use aws_sdk_s3::operation::head_object::HeadObjectError;
use peace::{
    cfg::{state::Generated, FnCtx},
    params::Params,
};

use crate::items::peace_aws_s3_object::{
    S3ObjectData, S3ObjectError, S3ObjectParams, S3ObjectState,
};

#[cfg(feature = "output_progress")]
use peace::progress_model::ProgressMsgUpdate;

/// Reads the current state of the S3 object state.
#[derive(Debug)]
pub struct S3ObjectStateCurrentFn<Id>(PhantomData<Id>);

impl<Id> S3ObjectStateCurrentFn<Id>
where
    Id: Send + Sync,
{
    pub async fn try_state_current(
        fn_ctx: FnCtx<'_>,
        params_partial: &<S3ObjectParams<Id> as Params>::Partial,
        data: S3ObjectData<'_, Id>,
    ) -> Result<Option<S3ObjectState>, S3ObjectError> {
        let bucket_name = params_partial.bucket_name();
        let object_key = params_partial.object_key();
        if let Some((bucket_name, object_key)) = bucket_name.zip(object_key) {
            Self::state_current_internal(fn_ctx, data, bucket_name, object_key)
                .await
                .map(Some)
        } else {
            Ok(Some(S3ObjectState::None))
        }
    }

    pub async fn state_current(
        fn_ctx: FnCtx<'_>,
        params: &S3ObjectParams<Id>,
        data: S3ObjectData<'_, Id>,
    ) -> Result<S3ObjectState, S3ObjectError> {
        let bucket_name = params.bucket_name();
        let object_key = params.object_key();

        Self::state_current_internal(fn_ctx, data, bucket_name, object_key).await
    }

    async fn state_current_internal(
        fn_ctx: FnCtx<'_>,
        data: S3ObjectData<'_, Id>,
        bucket_name: &str,
        object_key: &str,
    ) -> Result<S3ObjectState, S3ObjectError> {
        let client = data.client();

        #[cfg(not(feature = "output_progress"))]
        let _fn_ctx = fn_ctx;
        #[cfg(feature = "output_progress")]
        let progress_sender = &fn_ctx.progress_sender;
        #[cfg(feature = "output_progress")]
        progress_sender.tick(ProgressMsgUpdate::Set(String::from(
            "fetching object metadata",
        )));
        let head_object_result = client
            .head_object()
            .bucket(bucket_name)
            .key(object_key)
            .send()
            .await;
        let content_md5_and_e_tag = match head_object_result {
            Ok(head_object_output) => {
                #[cfg(feature = "output_progress")]
                progress_sender.tick(ProgressMsgUpdate::Set(String::from(
                    "object metadata fetched",
                )));
                let content_md5_hexstr = head_object_output
                    .metadata()
                    .and_then(|metadata| metadata.get("content_md5_hexstr"))
                    .cloned();

                let e_tag = head_object_output
                    .e_tag()
                    .expect("Expected S3 object e_tag to be Some when head_object is successful.")
                    .to_string();

                Some((content_md5_hexstr, e_tag))
            }
            Err(error) => {
                #[cfg(feature = "output_progress")]
                progress_sender.tick(ProgressMsgUpdate::Set(String::from(
                    "object metadata not fetched",
                )));

                #[cfg(feature = "error_reporting")]
                let (aws_desc, aws_desc_span) = crate::items::aws_error_desc!(&error);

                match &error {
                    SdkError::ServiceError(service_error) => match service_error.err() {
                        HeadObjectError::NotFound(_) => None,
                        _ => {
                            return Err(S3ObjectError::S3ObjectGetError {
                                object_key: object_key.to_string(),
                                #[cfg(feature = "error_reporting")]
                                aws_desc,
                                #[cfg(feature = "error_reporting")]
                                aws_desc_span,
                                error,
                            });
                        }
                    },
                    _ => {
                        return Err(S3ObjectError::S3ObjectGetError {
                            object_key: object_key.to_string(),
                            #[cfg(feature = "error_reporting")]
                            aws_desc,
                            #[cfg(feature = "error_reporting")]
                            aws_desc_span,
                            error,
                        });
                    }
                }
            }
        };

        if let Some((content_md5_hexstr, e_tag)) = content_md5_and_e_tag {
            let state_current = S3ObjectState::Some {
                bucket_name: bucket_name.to_string(),
                object_key: object_key.to_string(),
                content_md5_hexstr,
                e_tag: Generated::Value(e_tag),
            };

            Ok(state_current)
        } else {
            Ok(S3ObjectState::None)
        }
    }
}
