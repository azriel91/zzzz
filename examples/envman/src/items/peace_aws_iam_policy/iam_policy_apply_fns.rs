use std::marker::PhantomData;

use aws_sdk_iam::{error::SdkError, operation::list_policy_versions::ListPolicyVersionsError};
use peace::cfg::{state::Generated, ApplyCheck, FnCtx};
#[cfg(feature = "output_progress")]
use peace::progress_model::{ProgressLimit, ProgressMsgUpdate};

use crate::items::peace_aws_iam_policy::{
    model::PolicyIdArnVersion, IamPolicyData, IamPolicyError, IamPolicyParams, IamPolicyState,
    IamPolicyStateDiff,
};

/// ApplyFns for the instance profile state.
#[derive(Debug)]
pub struct IamPolicyApplyFns<Id>(PhantomData<Id>);

impl<Id> IamPolicyApplyFns<Id>
where
    Id: Send + Sync + 'static,
{
    pub async fn apply_check(
        _params: &IamPolicyParams<Id>,
        _data: IamPolicyData<'_, Id>,
        state_current: &IamPolicyState,
        _state_goal: &IamPolicyState,
        diff: &IamPolicyStateDiff,
    ) -> Result<ApplyCheck, IamPolicyError> {
        match diff {
            IamPolicyStateDiff::Added | IamPolicyStateDiff::DocumentModified { .. } => {
                let apply_check = {
                    #[cfg(not(feature = "output_progress"))]
                    {
                        ApplyCheck::ExecRequired
                    }
                    #[cfg(feature = "output_progress")]
                    {
                        let progress_limit = ProgressLimit::Steps(3);
                        ApplyCheck::ExecRequired { progress_limit }
                    }
                };

                Ok(apply_check)
            }
            IamPolicyStateDiff::Removed => {
                let apply_check = match state_current {
                    IamPolicyState::None => ApplyCheck::ExecNotRequired,
                    IamPolicyState::Some {
                        name: _,
                        path: _,
                        policy_document: _,
                        policy_id_arn_version,
                    } => {
                        let mut steps_required = 0;
                        if matches!(policy_id_arn_version, Generated::Value(_)) {
                            steps_required += 1;
                        }

                        if steps_required == 0 {
                            ApplyCheck::ExecNotRequired
                        } else {
                            #[cfg(not(feature = "output_progress"))]
                            {
                                ApplyCheck::ExecRequired
                            }
                            #[cfg(feature = "output_progress")]
                            {
                                let progress_limit = ProgressLimit::Steps(steps_required);
                                ApplyCheck::ExecRequired { progress_limit }
                            }
                        }
                    }
                };

                Ok(apply_check)
            }
            IamPolicyStateDiff::NameOrPathModified {
                name_diff,
                path_diff,
            } => Err(IamPolicyError::PolicyModificationNotSupported {
                name_diff: name_diff.clone(),
                path_diff: path_diff.clone(),
            }),
            IamPolicyStateDiff::InSyncExists => {
                // Hack: Remove this when referential param values is implemented.
                let IamPolicyState::Some {
                    policy_id_arn_version,
                    ..
                } = state_current
                else {
                    unreachable!()
                };
                let Generated::Value(_policy_id_version_arn) = policy_id_arn_version else {
                    unreachable!()
                };

                Ok(ApplyCheck::ExecNotRequired)
            }
            IamPolicyStateDiff::InSyncDoesNotExist => Ok(ApplyCheck::ExecNotRequired),
        }
    }

    pub async fn apply_dry(
        _fn_ctx: FnCtx<'_>,
        _params: &IamPolicyParams<Id>,
        _iam_policy_data: IamPolicyData<'_, Id>,
        _state_current: &IamPolicyState,
        state_goal: &IamPolicyState,
        _diff: &IamPolicyStateDiff,
    ) -> Result<IamPolicyState, IamPolicyError> {
        Ok(state_goal.clone())
    }

    pub async fn apply(
        #[cfg(not(feature = "output_progress"))] _fn_ctx: FnCtx<'_>,
        #[cfg(feature = "output_progress")] fn_ctx: FnCtx<'_>,
        _params: &IamPolicyParams<Id>,
        data: IamPolicyData<'_, Id>,
        state_current: &IamPolicyState,
        state_goal: &IamPolicyState,
        diff: &IamPolicyStateDiff,
    ) -> Result<IamPolicyState, IamPolicyError> {
        #[cfg(feature = "output_progress")]
        let progress_sender = &fn_ctx.progress_sender;

        match diff {
            IamPolicyStateDiff::Added => match state_goal {
                IamPolicyState::None => {
                    panic!("`IamPolicyApplyFns::exec` called with state_goal being None.");
                }
                IamPolicyState::Some {
                    name,
                    path,
                    policy_document,
                    policy_id_arn_version: _,
                } => {
                    #[cfg(feature = "output_progress")]
                    progress_sender.tick(ProgressMsgUpdate::Set(String::from("creating policy")));
                    let create_policy_output = data
                        .client()
                        .create_policy()
                        .policy_name(name)
                        .path(path)
                        .policy_document(policy_document)
                        .send()
                        .await
                        .map_err(|error| {
                            let policy_name = name.to_string();
                            let policy_path = path.to_string();

                            #[cfg(feature = "error_reporting")]
                            let (aws_desc, aws_desc_span) = crate::items::aws_error_desc!(&error);

                            IamPolicyError::PolicyCreateError {
                                policy_name,
                                policy_path,
                                #[cfg(feature = "error_reporting")]
                                aws_desc,
                                #[cfg(feature = "error_reporting")]
                                aws_desc_span,
                                error,
                            }
                        })?;
                    #[cfg(feature = "output_progress")]
                    progress_sender.inc(1, ProgressMsgUpdate::Set(String::from("policy created")));

                    let policy = create_policy_output
                        .policy()
                        .expect("Expected policy to be Some when create_policy is successful.");
                    let policy_id = policy
                        .policy_id()
                        .expect("Expected policy id to be Some when create_policy is successful.")
                        .to_string();
                    let policy_arn = policy
                        .arn()
                        .expect("Expected policy ARN to be Some when create_policy is successful.")
                        .to_string();
                    let policy_version = policy
                        .default_version_id()
                        .expect(
                            "Expected policy default version ID to be Some \
                            when create_policy is successful.",
                        )
                        .to_string();

                    let policy_id_arn_version =
                        PolicyIdArnVersion::new(policy_id, policy_arn, policy_version);

                    let state_applied = IamPolicyState::Some {
                        name: name.to_string(),
                        path: path.clone(),
                        policy_document: policy_document.clone(),
                        policy_id_arn_version: Generated::Value(policy_id_arn_version),
                    };

                    Ok(state_applied)
                }
            },
            IamPolicyStateDiff::Removed => {
                match state_current {
                    IamPolicyState::None => {}
                    IamPolicyState::Some {
                        name,
                        path,
                        policy_document: _,
                        policy_id_arn_version,
                    } => {
                        if let Generated::Value(policy_id_arn_version) = policy_id_arn_version {
                            let client = data.client();

                            #[cfg(feature = "output_progress")]
                            progress_sender.tick(ProgressMsgUpdate::Set(String::from(
                                "discovering policy versions",
                            )));
                            let list_policy_versions_result = client
                                .list_policy_versions()
                                .policy_arn(policy_id_arn_version.arn())
                                .send()
                                .await;
                            #[cfg(feature = "output_progress")]
                            progress_sender.inc(
                                1,
                                ProgressMsgUpdate::Set(String::from("policy versions discovered")),
                            );

                            // Need to delete all of the non-default versions individually.
                            match list_policy_versions_result {
                                Ok(list_policy_versions_output) => {
                                    let policy_versions = list_policy_versions_output.versions();

                                    let non_default_policy_versions =
                                        policy_versions.iter().filter(|policy_version| {
                                            !policy_version.is_default_version()
                                        });
                                    for policy_version in non_default_policy_versions {
                                        let version_id = policy_version.version_id().expect(
                                            "Expected policy version version ID to be Some.",
                                        );
                                        #[cfg(feature = "output_progress")]
                                        progress_sender.tick(ProgressMsgUpdate::Set(String::from(
                                            "deleting policy versions",
                                        )));
                                        client
                                            .delete_policy_version()
                                            .policy_arn(policy_id_arn_version.arn())
                                            .version_id(version_id)
                                            .send()
                                            .await
                                            .map_err(|error| {
                                                #[cfg(feature = "error_reporting")]
                                                let (aws_desc, aws_desc_span) =
                                                    crate::items::aws_error_desc!(&error);

                                                IamPolicyError::PolicyVersionDeleteError {
                                                    policy_name: name.to_string(),
                                                    policy_path: path.to_string(),
                                                    version: version_id.to_string(),
                                                    #[cfg(feature = "error_reporting")]
                                                    aws_desc,
                                                    #[cfg(feature = "error_reporting")]
                                                    aws_desc_span,
                                                    error,
                                                }
                                            })?;
                                        #[cfg(feature = "output_progress")]
                                        progress_sender.inc(
                                            1,
                                            ProgressMsgUpdate::Set(String::from(
                                                "policy versions deleted",
                                            )),
                                        );
                                    }
                                }
                                Err(error) => {
                                    #[cfg(feature = "error_reporting")]
                                    let (aws_desc, aws_desc_span) =
                                        crate::items::aws_error_desc!(&error);
                                    match &error {
                                        SdkError::ServiceError(service_error) => {
                                            match service_error.err() {
                                                ListPolicyVersionsError::NoSuchEntityException(
                                                    _,
                                                ) => {
                                                    return Err(
                                                        IamPolicyError::PolicyNotFoundAfterList {
                                                            policy_name: name.to_string(),
                                                            policy_path: path.to_string(),
                                                            policy_id: policy_id_arn_version
                                                                .id()
                                                                .to_string(),
                                                            policy_arn: policy_id_arn_version
                                                                .arn()
                                                                .to_string(),
                                                            #[cfg(feature = "error_reporting")]
                                                            aws_desc,
                                                            #[cfg(feature = "error_reporting")]
                                                            aws_desc_span,
                                                        },
                                                    );
                                                }
                                                _ => {
                                                    return Err(
                                                        IamPolicyError::PolicyVersionsListError {
                                                            policy_name: name.to_string(),
                                                            policy_path: path.to_string(),
                                                            #[cfg(feature = "error_reporting")]
                                                            aws_desc,
                                                            #[cfg(feature = "error_reporting")]
                                                            aws_desc_span,
                                                            error,
                                                        },
                                                    );
                                                }
                                            }
                                        }
                                        _ => {
                                            return Err(IamPolicyError::PolicyVersionsListError {
                                                policy_name: name.to_string(),
                                                policy_path: path.to_string(),
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

                            // The default version is deleted along with the policy.
                            #[cfg(feature = "output_progress")]
                            progress_sender
                                .tick(ProgressMsgUpdate::Set(String::from("deleting policy")));
                            client
                                .delete_policy()
                                .policy_arn(policy_id_arn_version.arn())
                                .send()
                                .await
                                .map_err(|error| {
                                    let policy_name = name.to_string();
                                    let policy_path = path.to_string();
                                    let policy_id = policy_id_arn_version.id().to_string();
                                    let policy_arn = policy_id_arn_version.arn().to_string();

                                    #[cfg(feature = "error_reporting")]
                                    let (aws_desc, aws_desc_span) =
                                        crate::items::aws_error_desc!(&error);

                                    IamPolicyError::PolicyDeleteError {
                                        policy_name,
                                        policy_path,
                                        policy_id,
                                        policy_arn,
                                        #[cfg(feature = "error_reporting")]
                                        aws_desc,
                                        #[cfg(feature = "error_reporting")]
                                        aws_desc_span,
                                        error,
                                    }
                                })?;
                            #[cfg(feature = "output_progress")]
                            progress_sender
                                .inc(1, ProgressMsgUpdate::Set(String::from("policy deleted")));
                        }
                    }
                }

                let state_applied = state_goal.clone();
                Ok(state_applied)
            }
            IamPolicyStateDiff::DocumentModified { .. } => match state_goal {
                IamPolicyState::None => {
                    panic!("`IamPolicyApplyFns::exec` called with state_goal being None.");
                }
                IamPolicyState::Some {
                    name,
                    path,
                    policy_document,
                    policy_id_arn_version: _,
                } => {
                    let IamPolicyState::Some {
                        policy_id_arn_version: Generated::Value(policy_id_arn_version),
                        ..
                    } = state_current
                    else {
                        panic!("Expected policy ID and ARN to exist when diff is modified.");
                    };
                    let policy_arn = policy_id_arn_version.arn();
                    #[cfg(feature = "output_progress")]
                    progress_sender.tick(ProgressMsgUpdate::Set(String::from(
                        "creating policy version",
                    )));
                    let create_policy_output = data
                        .client()
                        .create_policy_version()
                        .policy_arn(policy_arn)
                        .policy_document(policy_document)
                        .send()
                        .await
                        .map_err(|error| {
                            let policy_name = name.to_string();
                            let policy_path = path.to_string();

                            #[cfg(feature = "error_reporting")]
                            let (aws_desc, aws_desc_span) = crate::items::aws_error_desc!(&error);

                            IamPolicyError::PolicyVersionCreateError {
                                policy_name,
                                policy_path,
                                #[cfg(feature = "error_reporting")]
                                aws_desc,
                                #[cfg(feature = "error_reporting")]
                                aws_desc_span,
                                error,
                            }
                        })?;
                    #[cfg(feature = "output_progress")]
                    progress_sender.inc(
                        1,
                        ProgressMsgUpdate::Set(String::from("policy version created")),
                    );
                    let policy_version = create_policy_output.policy_version().expect(
                        "Expected policy_version to be Some when create_policy is successful.",
                    );
                    let policy_id = policy_id_arn_version.id().to_string();
                    let policy_arn = policy_id_arn_version.arn().to_string();
                    let policy_version_id = policy_version
                        .version_id()
                        .expect("Expected policy_version version_id to be Some when create_policy is successful.")
                        .to_string();

                    let policy_id_arn_version =
                        PolicyIdArnVersion::new(policy_id, policy_arn, policy_version_id);

                    let state_applied = IamPolicyState::Some {
                        name: name.to_string(),
                        path: path.clone(),
                        policy_document: policy_document.clone(),
                        policy_id_arn_version: Generated::Value(policy_id_arn_version),
                    };

                    Ok(state_applied)
                }
            },
            IamPolicyStateDiff::InSyncExists | IamPolicyStateDiff::InSyncDoesNotExist => {
                unreachable!(
                    "`IamPolicyApplyFns::exec` should never be called when state is in sync."
                );
            }
            IamPolicyStateDiff::NameOrPathModified {
                name_diff,
                path_diff,
            } => Err(IamPolicyError::NameOrPathModificationNotSupported {
                name_diff: name_diff.clone(),
                path_diff: path_diff.clone(),
            }),
        }
    }
}
