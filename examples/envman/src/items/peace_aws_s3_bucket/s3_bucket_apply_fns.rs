use std::marker::PhantomData;

use aws_sdk_iam::error::{ProvideErrorMetadata, SdkError};
use aws_sdk_s3::{
    operation::create_bucket::CreateBucketError,
    types::{BucketLocationConstraint, CreateBucketConfiguration},
};
use peace::cfg::{ApplyCheck, FnCtx};
#[cfg(feature = "output_progress")]
use peace::progress_model::{ProgressLimit, ProgressMsgUpdate};

use crate::items::peace_aws_s3_bucket::{
    S3BucketData, S3BucketError, S3BucketParams, S3BucketState, S3BucketStateDiff,
};

use super::S3BucketStateCurrentFn;

/// ApplyFns for the S3 bucket state.
#[derive(Debug)]
pub struct S3BucketApplyFns<Id>(PhantomData<Id>);

impl<Id> S3BucketApplyFns<Id>
where
    Id: Send + Sync + 'static,
{
    pub async fn apply_check(
        _params: &S3BucketParams<Id>,
        _data: S3BucketData<'_, Id>,
        state_current: &S3BucketState,
        _state_goal: &S3BucketState,
        diff: &S3BucketStateDiff,
    ) -> Result<ApplyCheck, S3BucketError> {
        match diff {
            S3BucketStateDiff::Added { .. } => {
                let apply_check = {
                    #[cfg(not(feature = "output_progress"))]
                    {
                        ApplyCheck::ExecRequired
                    }
                    #[cfg(feature = "output_progress")]
                    {
                        let progress_limit = ProgressLimit::Steps(1);
                        ApplyCheck::ExecRequired { progress_limit }
                    }
                };

                Ok(apply_check)
            }
            S3BucketStateDiff::Removed => {
                let apply_check = match state_current {
                    S3BucketState::None => ApplyCheck::ExecNotRequired,
                    S3BucketState::Some {
                        name: _,
                        creation_date: _,
                    } => {
                        #[cfg(not(feature = "output_progress"))]
                        {
                            ApplyCheck::ExecRequired
                        }
                        #[cfg(feature = "output_progress")]
                        {
                            let steps_required = 1;
                            let progress_limit = ProgressLimit::Steps(steps_required);
                            ApplyCheck::ExecRequired { progress_limit }
                        }
                    }
                };

                Ok(apply_check)
            }
            S3BucketStateDiff::NameModified {
                s3_bucket_name_current,
                s3_bucket_name_goal,
            } => Err(S3BucketError::S3BucketModificationNotSupported {
                s3_bucket_name_current: s3_bucket_name_current.clone(),
                s3_bucket_name_goal: s3_bucket_name_goal.clone(),
            }),
            S3BucketStateDiff::InSyncExists | S3BucketStateDiff::InSyncDoesNotExist => {
                Ok(ApplyCheck::ExecNotRequired)
            }
        }
    }

    pub async fn apply_dry(
        _fn_ctx: FnCtx<'_>,
        _params: &S3BucketParams<Id>,
        _data: S3BucketData<'_, Id>,
        _state_current: &S3BucketState,
        state_goal: &S3BucketState,
        _diff: &S3BucketStateDiff,
    ) -> Result<S3BucketState, S3BucketError> {
        Ok(state_goal.clone())
    }

    pub async fn apply(
        fn_ctx: FnCtx<'_>,
        params: &S3BucketParams<Id>,
        data: S3BucketData<'_, Id>,
        state_current: &S3BucketState,
        state_goal: &S3BucketState,
        diff: &S3BucketStateDiff,
    ) -> Result<S3BucketState, S3BucketError> {
        #[cfg(feature = "output_progress")]
        let progress_sender = &fn_ctx.progress_sender;

        match diff {
            S3BucketStateDiff::Added => match state_goal {
                S3BucketState::None => {
                    panic!("`S3BucketApplyFns::exec` called with state_goal being None.");
                }
                S3BucketState::Some {
                    name,
                    creation_date: _,
                } => {
                    let client = data.client();

                    #[cfg(feature = "output_progress")]
                    progress_sender.tick(ProgressMsgUpdate::Set(String::from("creating bucket")));
                    let mut create_bucket = client.create_bucket().bucket(name);
                    if let Some(region) = data.region().as_ref() {
                        create_bucket = create_bucket.create_bucket_configuration(
                            CreateBucketConfiguration::builder()
                                .location_constraint(BucketLocationConstraint::from(
                                    region.as_ref(),
                                ))
                                .build(),
                        );
                    }
                    let _create_bucket_output = create_bucket.send().await.map_err(|error| {
                        let s3_bucket_name = name.to_string();

                        #[cfg(feature = "error_reporting")]
                        let (aws_desc, aws_desc_span) = crate::items::aws_error_desc!(&error);

                        match &error {
                            SdkError::ServiceError(service_error) => match &service_error.err() {
                                CreateBucketError::BucketAlreadyExists(error) => {
                                    S3BucketError::S3BucketOwnedBySomeoneElseError {
                                        s3_bucket_name,
                                        error: error.clone(),
                                    }
                                }
                                CreateBucketError::BucketAlreadyOwnedByYou(error) => {
                                    S3BucketError::S3BucketOwnedByYouError {
                                        s3_bucket_name,
                                        error: error.clone(),
                                    }
                                }
                                _ => S3BucketError::S3BucketCreateError {
                                    s3_bucket_name,
                                    #[cfg(feature = "error_reporting")]
                                    aws_desc,
                                    #[cfg(feature = "error_reporting")]
                                    aws_desc_span,
                                    error,
                                },
                            },
                            _ => S3BucketError::S3BucketCreateError {
                                s3_bucket_name,
                                #[cfg(feature = "error_reporting")]
                                aws_desc,
                                #[cfg(feature = "error_reporting")]
                                aws_desc_span,
                                error,
                            },
                        }
                    })?;
                    #[cfg(feature = "output_progress")]
                    progress_sender.inc(1, ProgressMsgUpdate::Set(String::from("bucket created")));

                    let state_applied =
                        S3BucketStateCurrentFn::<Id>::state_current(fn_ctx, params, data).await?;

                    Ok(state_applied)
                }
            },
            S3BucketStateDiff::Removed => {
                match state_current {
                    S3BucketState::None => {}
                    S3BucketState::Some {
                        name,
                        creation_date: _,
                    } => {
                        let client = data.client();

                        #[cfg(feature = "output_progress")]
                        progress_sender
                            .tick(ProgressMsgUpdate::Set(String::from("deleting bucket")));
                        let delete_bucket_result = client.delete_bucket().bucket(name).send().await;

                        // Sometimes AWS returns this error:
                        //
                        // ```xml
                        // <Code>NoSuchBucket</Code>
                        // <Message>The specified bucket does not exist</Message>
                        // <BucketName>the-bucket-name</BucketName>
                        // ```
                        //
                        // This is really an issue with AWS, where they still show the
                        // bucket even though it *has* been deleted. See:
                        //
                        // <https://serverfault.com/questions/969962/how-to-remove-orphaned-s3-bucket>
                        delete_bucket_result
                            .map(|_delete_bucket_output| ())
                            .or_else(|error| {
                                if let SdkError::ServiceError(service_error) = &error {
                                    if let Some("NoSuchBucket") = service_error.err().code() {
                                        return Ok(());
                                    }
                                }

                                #[cfg(feature = "error_reporting")]
                                let (aws_desc, aws_desc_span) =
                                    crate::items::aws_error_desc!(&error);

                                let s3_bucket_name = name.to_string();
                                Err(S3BucketError::S3BucketDeleteError {
                                    s3_bucket_name,
                                    #[cfg(feature = "error_reporting")]
                                    aws_desc,
                                    #[cfg(feature = "error_reporting")]
                                    aws_desc_span,
                                    error,
                                })
                            })?;
                        #[cfg(feature = "output_progress")]
                        progress_sender
                            .inc(1, ProgressMsgUpdate::Set(String::from("bucket deleted")));
                    }
                }

                let state_applied = state_goal.clone();
                Ok(state_applied)
            }
            S3BucketStateDiff::InSyncExists | S3BucketStateDiff::InSyncDoesNotExist => {
                unreachable!(
                    "`S3BucketApplyFns::exec` should never be called when state is in sync."
                );
            }
            S3BucketStateDiff::NameModified {
                s3_bucket_name_current,
                s3_bucket_name_goal,
            } => Err(S3BucketError::NameModificationNotSupported {
                s3_bucket_name_current: s3_bucket_name_current.clone(),
                s3_bucket_name_goal: s3_bucket_name_goal.clone(),
            }),
        }
    }
}
