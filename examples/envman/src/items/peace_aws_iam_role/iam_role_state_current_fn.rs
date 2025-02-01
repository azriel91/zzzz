use std::marker::PhantomData;

use aws_sdk_iam::{error::SdkError, operation::get_role::GetRoleError};
use peace::{
    cfg::{state::Generated, FnCtx},
    params::Params,
};

use crate::items::peace_aws_iam_role::{
    model::{ManagedPolicyAttachment, RoleIdAndArn},
    IamRoleData, IamRoleError, IamRoleParams, IamRoleState,
};

#[cfg(feature = "output_progress")]
use peace::progress_model::ProgressMsgUpdate;

/// Reads the current state of the instance profile state.
#[derive(Debug)]
pub struct IamRoleStateCurrentFn<Id>(PhantomData<Id>);

impl<Id> IamRoleStateCurrentFn<Id>
where
    Id: Send + Sync,
{
    pub async fn try_state_current(
        fn_ctx: FnCtx<'_>,
        params_partial: &<IamRoleParams<Id> as Params>::Partial,
        data: IamRoleData<'_, Id>,
    ) -> Result<Option<IamRoleState>, IamRoleError> {
        let name = params_partial.name();
        let path = params_partial.path();
        if let Some((name, path)) = name.zip(path) {
            Self::state_current_internal(fn_ctx, data, name, path)
                .await
                .map(Some)
        } else {
            Ok(None)
        }
    }

    pub async fn state_current(
        fn_ctx: FnCtx<'_>,
        params: &IamRoleParams<Id>,
        data: IamRoleData<'_, Id>,
    ) -> Result<IamRoleState, IamRoleError> {
        let name = params.name();
        let path = params.path();

        Self::state_current_internal(fn_ctx, data, name, path).await
    }

    async fn state_current_internal(
        fn_ctx: FnCtx<'_>,
        data: IamRoleData<'_, Id>,
        name: &str,
        path: &str,
    ) -> Result<IamRoleState, IamRoleError> {
        let client = data.client();

        #[cfg(not(feature = "output_progress"))]
        let _fn_ctx = fn_ctx;
        #[cfg(feature = "output_progress")]
        let progress_sender = &fn_ctx.progress_sender;
        #[cfg(feature = "output_progress")]
        progress_sender.tick(ProgressMsgUpdate::Set(String::from("fetching role")));
        let get_role_result = client.get_role().role_name(name).send().await;
        let role_opt = match get_role_result {
            Ok(get_role_output) => {
                let role = get_role_output
                    .role()
                    .expect("Expected Role to exist when get_role is successful");

                let role_name = role.role_name().to_string();
                let role_path = role.path().to_string();
                let role_id = role.role_id().to_string();
                let role_arn = role.arn().to_string();

                let role_id_and_arn = RoleIdAndArn::new(role_id, role_arn);

                Some((role_name, role_path, role_id_and_arn))
            }
            Err(error) => {
                #[cfg(feature = "error_reporting")]
                let (aws_desc, aws_desc_span) = crate::items::aws_error_desc!(&error);
                match &error {
                    SdkError::ServiceError(service_error) => match service_error.err() {
                        GetRoleError::NoSuchEntityException(_) => None,
                        _ => {
                            return Err(IamRoleError::RoleGetError {
                                role_name: name.to_string(),
                                #[cfg(feature = "error_reporting")]
                                aws_desc,
                                #[cfg(feature = "error_reporting")]
                                aws_desc_span,
                                error,
                            });
                        }
                    },
                    _ => {
                        return Err(IamRoleError::RoleGetError {
                            role_name: name.to_string(),
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

        match role_opt {
            None => {
                #[cfg(feature = "output_progress")]
                progress_sender.tick(ProgressMsgUpdate::Set(String::from("policy not fetched")));
                Ok(IamRoleState::None)
            }
            Some((role_name, role_path, role_id_and_arn)) => {
                assert_eq!(name, role_name);
                assert_eq!(path, role_path);

                #[cfg(feature = "output_progress")]
                progress_sender.tick(ProgressMsgUpdate::Set(String::from("policy fetched")));

                #[cfg(feature = "output_progress")]
                progress_sender.tick(ProgressMsgUpdate::Set(String::from(
                    "listing attached policies",
                )));
                let list_attached_role_policies_output = client
                    .list_attached_role_policies()
                    .role_name(name)
                    .path_prefix(path)
                    .send()
                    .await
                    .map_err(|error| {
                        #[cfg(feature = "error_reporting")]
                        let (aws_desc, aws_desc_span) = crate::items::aws_error_desc!(&error);

                        IamRoleError::ManagedPoliciesListError {
                            role_name: name.to_string(),
                            role_path: path.to_string(),
                            #[cfg(feature = "error_reporting")]
                            aws_desc,
                            #[cfg(feature = "error_reporting")]
                            aws_desc_span,
                            error,
                        }
                    })?;
                #[cfg(feature = "output_progress")]
                progress_sender.tick(ProgressMsgUpdate::Set(String::from(
                    "filtering attached policies",
                )));
                let managed_policy_attachment = list_attached_role_policies_output
                    .attached_policies()
                    .iter()
                    .find_map(|attached_policy| {
                        let policy_name = attached_policy
                            .policy_name()
                            .expect("Expected policy_name to be Some for any attached policy.");

                        if policy_name == name {
                            Some(
                                attached_policy
                                    .policy_arn()
                                    .expect(
                                        "Expected policy_arn to be Some for \
                                            any attached policy.",
                                    )
                                    .to_string(),
                            )
                        } else {
                            None
                        }
                    })
                    .map(|managed_policy_arn| {
                        ManagedPolicyAttachment::new(Generated::Value(managed_policy_arn), true)
                    })
                    .unwrap_or(ManagedPolicyAttachment::new(Generated::Tbd, false));
                #[cfg(feature = "output_progress")]
                {
                    let message = if managed_policy_attachment.attached() {
                        "policy attached"
                    } else {
                        "policy not attached"
                    };
                    progress_sender.tick(ProgressMsgUpdate::Set(String::from(message)));
                }

                let state_current = IamRoleState::Some {
                    name: name.to_string(),
                    path: path.to_string(),
                    role_id_and_arn: Generated::Value(role_id_and_arn),
                    managed_policy_attachment,
                };

                Ok(state_current)
            }
        }
    }
}
