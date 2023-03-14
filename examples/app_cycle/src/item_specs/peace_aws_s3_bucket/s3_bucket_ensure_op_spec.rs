use std::marker::PhantomData;

#[cfg(feature = "output_progress")]
use peace::cfg::progress::ProgressLimit;
use peace::cfg::{async_trait, EnsureOpSpec, OpCheckStatus, OpCtx};

use crate::item_specs::peace_aws_s3_bucket::{
    S3BucketData, S3BucketError, S3BucketState, S3BucketStateDiff,
};

/// Ensure OpSpec for the S3 bucket state.
#[derive(Debug)]
pub struct S3BucketEnsureOpSpec<Id>(PhantomData<Id>);

#[async_trait(?Send)]
impl<Id> EnsureOpSpec for S3BucketEnsureOpSpec<Id>
where
    Id: Send + Sync + 'static,
{
    type Data<'op> = S3BucketData<'op, Id>;
    type Error = S3BucketError;
    type State = S3BucketState;
    type StateDiff = S3BucketStateDiff;

    async fn check(
        _s3_bucket_data: S3BucketData<'_, Id>,
        _state_current: &S3BucketState,
        _state_desired: &S3BucketState,
        diff: &S3BucketStateDiff,
    ) -> Result<OpCheckStatus, S3BucketError> {
        match diff {
            S3BucketStateDiff::Added { .. } => {
                let op_check_status = {
                    #[cfg(not(feature = "output_progress"))]
                    {
                        OpCheckStatus::ExecRequired
                    }
                    #[cfg(feature = "output_progress")]
                    {
                        let progress_limit = ProgressLimit::Steps(1);
                        OpCheckStatus::ExecRequired { progress_limit }
                    }
                };

                Ok(op_check_status)
            }
            S3BucketStateDiff::Removed => {
                panic!(
                    "`S3BucketEnsureOpSpec::check` called with `S3BucketStateDiff::Removed`.\n\
                    An ensure should never remove a bucket."
                );
            }
            S3BucketStateDiff::NameModified {
                s3_bucket_name_current,
                s3_bucket_name_desired,
            } => Err(S3BucketError::S3BucketModificationNotSupported {
                s3_bucket_name_current: s3_bucket_name_current.clone(),
                s3_bucket_name_desired: s3_bucket_name_desired.clone(),
            }),
            S3BucketStateDiff::InSyncExists | S3BucketStateDiff::InSyncDoesNotExist => {
                Ok(OpCheckStatus::ExecNotRequired)
            }
        }
    }

    async fn exec_dry(
        _op_ctx: OpCtx<'_>,
        _s3_bucket_data: S3BucketData<'_, Id>,
        _state_current: &S3BucketState,
        state_desired: &S3BucketState,
        _diff: &S3BucketStateDiff,
    ) -> Result<S3BucketState, S3BucketError> {
        Ok(state_desired.clone())
    }

    async fn exec(
        _op_ctx: OpCtx<'_>,
        data: S3BucketData<'_, Id>,
        _state_current: &S3BucketState,
        state_desired: &S3BucketState,
        diff: &S3BucketStateDiff,
    ) -> Result<S3BucketState, S3BucketError> {
        match diff {
            S3BucketStateDiff::Added => match state_desired {
                S3BucketState::None => {
                    panic!("`S3BucketEnsureOpSpec::exec` called with state_desired being None.");
                }
                S3BucketState::Some { name } => {
                    let client = data.client();
                    let _create_bucket_output = client
                        .create_bucket()
                        .bucket(name)
                        .send()
                        .await
                        .map_err(|error| {
                            let s3_bucket_name = name.to_string();

                            S3BucketError::S3BucketCreateError {
                                s3_bucket_name,
                                error,
                            }
                        })?;

                    let state_ensured = S3BucketState::Some {
                        name: name.to_string(),
                    };

                    Ok(state_ensured)
                }
            },
            S3BucketStateDiff::Removed => {
                panic!(
                    "`S3BucketEnsureOpSpec::exec` called with `S3BucketStateDiff::Removed`.\n\
                    An ensure should never remove a bucket."
                );
            }
            S3BucketStateDiff::InSyncExists | S3BucketStateDiff::InSyncDoesNotExist => {
                unreachable!(
                    "`S3BucketEnsureOpSpec::exec` should never be called when state is in sync."
                );
            }
            S3BucketStateDiff::NameModified {
                s3_bucket_name_current,
                s3_bucket_name_desired,
            } => Err(S3BucketError::NameModificationNotSupported {
                s3_bucket_name_current: s3_bucket_name_current.clone(),
                s3_bucket_name_desired: s3_bucket_name_desired.clone(),
            }),
        }
    }
}
