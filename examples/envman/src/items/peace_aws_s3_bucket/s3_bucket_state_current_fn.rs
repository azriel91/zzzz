use std::marker::PhantomData;

use chrono::DateTime;
use peace::{
    cfg::{state::Timestamped, FnCtx},
    params::Params,
};

use crate::items::peace_aws_s3_bucket::{
    S3BucketData, S3BucketError, S3BucketParams, S3BucketState,
};

#[cfg(feature = "output_progress")]
use peace::progress_model::ProgressMsgUpdate;

/// Reads the current state of the S3 bucket state.
#[derive(Debug)]
pub struct S3BucketStateCurrentFn<Id>(PhantomData<Id>);

impl<Id> S3BucketStateCurrentFn<Id>
where
    Id: Send + Sync,
{
    pub async fn try_state_current(
        fn_ctx: FnCtx<'_>,
        params_partial: &<S3BucketParams<Id> as Params>::Partial,
        data: S3BucketData<'_, Id>,
    ) -> Result<Option<S3BucketState>, S3BucketError> {
        if let Some(name) = params_partial.name() {
            Self::state_current_internal(fn_ctx, data, name)
                .await
                .map(Some)
        } else {
            Ok(None)
        }
    }

    pub async fn state_current(
        fn_ctx: FnCtx<'_>,
        params: &S3BucketParams<Id>,
        data: S3BucketData<'_, Id>,
    ) -> Result<S3BucketState, S3BucketError> {
        let name = params.name();
        Self::state_current_internal(fn_ctx, data, name).await
    }

    async fn state_current_internal(
        fn_ctx: FnCtx<'_>,
        data: S3BucketData<'_, Id>,
        name: &str,
    ) -> Result<S3BucketState, S3BucketError> {
        let client = data.client();

        #[cfg(not(feature = "output_progress"))]
        let _fn_ctx = fn_ctx;
        #[cfg(feature = "output_progress")]
        let progress_sender = &fn_ctx.progress_sender;
        #[cfg(feature = "output_progress")]
        progress_sender.tick(ProgressMsgUpdate::Set(String::from("listing buckets")));
        let list_buckets_output = client.list_buckets().send().await.map_err(|error| {
            #[cfg(feature = "error_reporting")]
            let (aws_desc, aws_desc_span) = crate::items::aws_error_desc!(&error);

            S3BucketError::S3BucketListError {
                s3_bucket_name: name.to_string(),
                #[cfg(feature = "error_reporting")]
                aws_desc,
                #[cfg(feature = "error_reporting")]
                aws_desc_span,
                error,
            }
        })?;
        #[cfg(feature = "output_progress")]
        progress_sender.tick(ProgressMsgUpdate::Set(String::from("finding bucket")));
        let creation_date = list_buckets_output.buckets().iter().find_map(|bucket| {
            if matches!(bucket.name(), Some(bucket_name_listed) if bucket_name_listed == name) {
                Some(
                    bucket
                        .creation_date()
                        .cloned()
                        .expect("Expected bucket creation date to be Some."),
                )
            } else {
                None
            }
        });
        #[cfg(feature = "output_progress")]
        {
            let message = if creation_date.is_some() {
                "bucket found"
            } else {
                "bucket not found"
            };
            progress_sender.tick(ProgressMsgUpdate::Set(String::from(message)));
        }

        // let head_bucket_result = client.head_bucket().bucket(name).send().await;
        // let s3_bucket_exists = match head_bucket_result {
        //     Ok(_head_bucket_output) => true,
        //     Err(error) => match &error {
        //         SdkError::ServiceError(service_error) => {
        //             dbg!(&service_error);

        //             // If your user does not have permissions, AWS SDK Rust does not
        // return an             // access denied error. It just returns "Error"
        // with no other information.             //
        //             // https://github.com/awslabs/aws-sdk-rust/issues/227

        //             match service_error.err().kind {
        //                 HeadBucketErrorKind::NotFound(_) => false,
        //                 _ => {
        //                     return Err(S3BucketError::S3BucketGetError {
        //                         s3_bucket_name: name.to_string(),
        //                         error,
        //                     });
        //                 }
        //             }
        //         }
        //         _ => {
        //             return Err(S3BucketError::S3BucketGetError {
        //                 s3_bucket_name: name.to_string(),
        //                 error,
        //             });
        //         }
        //     },
        // };

        if let Some(creation_date) = creation_date {
            let state_current = S3BucketState::Some {
                name: name.to_string(),
                creation_date: Timestamped::Value(
                    DateTime::from_timestamp(creation_date.secs(), creation_date.subsec_nanos())
                        .unwrap(),
                ),
            };

            Ok(state_current)
        } else {
            Ok(S3BucketState::None)
        }
    }
}
