use std::marker::PhantomData;

use peace::cfg::{state::Generated, ApplyCheck, FnCtx};
#[cfg(feature = "output_progress")]
use peace::progress_model::{ProgressLimit, ProgressMsgUpdate, ProgressSender};

use crate::items::peace_aws_instance_profile::{
    model::InstanceProfileIdAndArn, InstanceProfileData, InstanceProfileError,
    InstanceProfileParams, InstanceProfileState, InstanceProfileStateDiff,
};

/// ApplyFns for the instance profile state.
#[derive(Debug)]
pub struct InstanceProfileApplyFns<Id>(PhantomData<Id>);

impl<Id> InstanceProfileApplyFns<Id> {
    async fn role_associate(
        #[cfg(feature = "output_progress")] progress_sender: &ProgressSender<'_>,
        client: &aws_sdk_iam::Client,
        name: &str,
        path: &str,
    ) -> Result<(), InstanceProfileError> {
        #[cfg(feature = "output_progress")]
        progress_sender.tick(ProgressMsgUpdate::Set(String::from("associating role")));
        let _instance_profile_role_add_output = client
            .add_role_to_instance_profile()
            .role_name(name)
            .instance_profile_name(name)
            .send()
            .await
            .map_err(|error| {
                let instance_profile_name = name.to_string();
                let instance_profile_path = path.to_string();
                let role_name = name.to_string();

                #[cfg(feature = "error_reporting")]
                let (aws_desc, aws_desc_span) = crate::items::aws_error_desc!(&error);

                InstanceProfileError::InstanceProfileRoleAddError {
                    instance_profile_name,
                    instance_profile_path,
                    role_name,
                    #[cfg(feature = "error_reporting")]
                    aws_desc,
                    #[cfg(feature = "error_reporting")]
                    aws_desc_span,
                    error,
                }
            })?;
        #[cfg(feature = "output_progress")]
        progress_sender.inc(1, ProgressMsgUpdate::Set(String::from("role associated")));

        Ok(())
    }

    pub(crate) async fn role_disassociate(
        #[cfg(feature = "output_progress")] progress_sender: &ProgressSender<'_>,
        client: &aws_sdk_iam::Client,
        name: &str,
        path: &str,
    ) -> Result<(), InstanceProfileError> {
        #[cfg(feature = "output_progress")]
        progress_sender.tick(ProgressMsgUpdate::Set(String::from("disassociating role")));
        client
            .remove_role_from_instance_profile()
            .instance_profile_name(name)
            .role_name(name)
            .send()
            .await
            .map_err(|error| {
                let instance_profile_name = name.to_string();
                let instance_profile_path = path.to_string();

                #[cfg(feature = "error_reporting")]
                let (aws_desc, aws_desc_span) = crate::items::aws_error_desc!(&error);

                InstanceProfileError::InstanceProfileRoleRemoveError {
                    instance_profile_name,
                    instance_profile_path,
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
            ProgressMsgUpdate::Set(String::from("role disassociated")),
        );

        Ok(())
    }
}

impl<Id> InstanceProfileApplyFns<Id>
where
    Id: Send + Sync + 'static,
{
    pub async fn apply_check(
        _params: &InstanceProfileParams<Id>,
        _data: InstanceProfileData<'_, Id>,
        state_current: &InstanceProfileState,
        _state_goal: &InstanceProfileState,
        diff: &InstanceProfileStateDiff,
    ) -> Result<ApplyCheck, InstanceProfileError> {
        match diff {
            InstanceProfileStateDiff::Added
            | InstanceProfileStateDiff::RoleAssociatedModified { .. } => {
                let apply_check = {
                    #[cfg(not(feature = "output_progress"))]
                    {
                        ApplyCheck::ExecRequired
                    }
                    #[cfg(feature = "output_progress")]
                    {
                        // Create instance profile, associate role
                        let progress_limit = ProgressLimit::Steps(2);
                        ApplyCheck::ExecRequired { progress_limit }
                    }
                };

                Ok(apply_check)
            }
            InstanceProfileStateDiff::Removed => {
                let apply_check = match state_current {
                    InstanceProfileState::None => ApplyCheck::ExecNotRequired,
                    InstanceProfileState::Some {
                        name: _,
                        path: _,
                        instance_profile_id_and_arn,
                        role_associated,
                    } => {
                        let mut steps_required = 0;
                        if *role_associated {
                            steps_required += 1;
                        }
                        if matches!(instance_profile_id_and_arn, Generated::Value(_)) {
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
            InstanceProfileStateDiff::NameOrPathModified {
                name_diff,
                path_diff,
            } => Err(
                InstanceProfileError::InstanceProfileModificationNotSupported {
                    name_diff: name_diff.clone(),
                    path_diff: path_diff.clone(),
                },
            ),
            InstanceProfileStateDiff::InSyncExists
            | InstanceProfileStateDiff::InSyncDoesNotExist => Ok(ApplyCheck::ExecNotRequired),
        }
    }

    pub async fn apply_dry(
        _fn_ctx: FnCtx<'_>,
        _params: &InstanceProfileParams<Id>,
        _data: InstanceProfileData<'_, Id>,
        _state_current: &InstanceProfileState,
        state_goal: &InstanceProfileState,
        _diff: &InstanceProfileStateDiff,
    ) -> Result<InstanceProfileState, InstanceProfileError> {
        Ok(state_goal.clone())
    }

    pub async fn apply(
        #[cfg(not(feature = "output_progress"))] _fn_ctx: FnCtx<'_>,
        #[cfg(feature = "output_progress")] fn_ctx: FnCtx<'_>,
        _params: &InstanceProfileParams<Id>,
        data: InstanceProfileData<'_, Id>,
        state_current: &InstanceProfileState,
        state_goal: &InstanceProfileState,
        diff: &InstanceProfileStateDiff,
    ) -> Result<InstanceProfileState, InstanceProfileError> {
        #[cfg(feature = "output_progress")]
        let progress_sender = &fn_ctx.progress_sender;

        match diff {
            InstanceProfileStateDiff::Added => match state_goal {
                InstanceProfileState::None => {
                    panic!("`InstanceProfileApplyFns::exec` called with state_goal being None.");
                }
                InstanceProfileState::Some {
                    name,
                    path,
                    instance_profile_id_and_arn: _,
                    role_associated: _,
                } => {
                    let client = data.client();

                    #[cfg(feature = "output_progress")]
                    progress_sender.tick(ProgressMsgUpdate::Set(String::from(
                        "creating instance profile",
                    )));
                    let create_instance_profile_output = client
                        .create_instance_profile()
                        .instance_profile_name(name)
                        .path(path)
                        .send()
                        .await
                        .map_err(|error| {
                            let instance_profile_name = name.to_string();
                            let instance_profile_path = path.to_string();

                            #[cfg(feature = "error_reporting")]
                            let (aws_desc, aws_desc_span) = crate::items::aws_error_desc!(&error);

                            InstanceProfileError::InstanceProfileCreateError {
                                instance_profile_name,
                                instance_profile_path,
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
                        ProgressMsgUpdate::Set(String::from("instance profile created")),
                    );

                    let instance_profile = create_instance_profile_output
                        .instance_profile()
                        .expect("Expected instance_profile to be Some when create_instance_profile is successful.");
                    let instance_profile_id = instance_profile.instance_profile_id().to_string();
                    let instance_profile_arn = instance_profile.arn().to_string();
                    let instance_profile_id_and_arn =
                        InstanceProfileIdAndArn::new(instance_profile_id, instance_profile_arn);

                    Self::role_associate(
                        #[cfg(feature = "output_progress")]
                        progress_sender,
                        client,
                        name,
                        path,
                    )
                    .await?;

                    let state_applied = InstanceProfileState::Some {
                        name: name.to_string(),
                        path: path.clone(),
                        instance_profile_id_and_arn: Generated::Value(instance_profile_id_and_arn),
                        role_associated: true,
                    };

                    Ok(state_applied)
                }
            },
            InstanceProfileStateDiff::Removed => match state_current {
                InstanceProfileState::None => {
                    unreachable!("Instance profile must be Some when it is to be removed.")
                }
                InstanceProfileState::Some {
                    name,
                    path,
                    instance_profile_id_and_arn,
                    role_associated,
                } => {
                    let client = data.client();
                    if *role_associated {
                        Self::role_disassociate(
                            #[cfg(feature = "output_progress")]
                            progress_sender,
                            client,
                            name,
                            path,
                        )
                        .await?;
                    }
                    if let Generated::Value(instance_profile_id_and_arn) =
                        instance_profile_id_and_arn
                    {
                        #[cfg(feature = "output_progress")]
                        progress_sender.tick(ProgressMsgUpdate::Set(String::from(
                            "deleting instance profile",
                        )));
                        client
                            .delete_instance_profile()
                            .instance_profile_name(name)
                            .send()
                            .await
                            .map_err(|error| {
                                let instance_profile_name = name.to_string();
                                let instance_profile_path = path.to_string();
                                let instance_profile_id =
                                    instance_profile_id_and_arn.id().to_string();
                                let instance_profile_arn =
                                    instance_profile_id_and_arn.arn().to_string();

                                #[cfg(feature = "error_reporting")]
                                let (aws_desc, aws_desc_span) =
                                    crate::items::aws_error_desc!(&error);

                                InstanceProfileError::InstanceProfileDeleteError {
                                    instance_profile_name,
                                    instance_profile_path,
                                    instance_profile_id,
                                    instance_profile_arn,
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
                            ProgressMsgUpdate::Set(String::from("instance profile deleted")),
                        );
                    }

                    let state_applied = state_goal.clone();
                    Ok(state_applied)
                }
            },
            InstanceProfileStateDiff::InSyncExists
            | InstanceProfileStateDiff::InSyncDoesNotExist => {
                unreachable!(
                    "`InstanceProfileApplyFns::exec` should never be called when state is in sync."
                );
            }
            InstanceProfileStateDiff::NameOrPathModified {
                name_diff,
                path_diff,
            } => Err(InstanceProfileError::NameOrPathModificationNotSupported {
                name_diff: name_diff.clone(),
                path_diff: path_diff.clone(),
            }),
            InstanceProfileStateDiff::RoleAssociatedModified {
                role_associated_current,
                role_associated_goal: _,
            } => {
                let (name, path) = match state_goal {
                    InstanceProfileState::None => {
                        panic!(
                            "`InstanceProfileApplyFns::exec` called with state_goal being None."
                        );
                    }
                    InstanceProfileState::Some {
                        name,
                        path,
                        instance_profile_id_and_arn: _,
                        role_associated: _,
                    } => (name, path),
                };

                let client = data.client();
                if *role_associated_current {
                    // Remove the association.
                    Self::role_disassociate(
                        #[cfg(feature = "output_progress")]
                        progress_sender,
                        client,
                        name,
                        path,
                    )
                    .await?;
                } else {
                    // Associate the role.
                    Self::role_associate(
                        #[cfg(feature = "output_progress")]
                        progress_sender,
                        client,
                        name,
                        path,
                    )
                    .await?;
                }
                let state_applied = state_goal.clone();
                Ok(state_applied)
            }
        }
    }
}
