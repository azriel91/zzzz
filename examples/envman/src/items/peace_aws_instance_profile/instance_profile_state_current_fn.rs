use std::marker::PhantomData;

use aws_sdk_iam::{error::SdkError, operation::get_instance_profile::GetInstanceProfileError};
use peace::{
    cfg::{state::Generated, FnCtx},
    params::Params,
};

use crate::items::peace_aws_instance_profile::{
    model::InstanceProfileIdAndArn, InstanceProfileData, InstanceProfileError,
    InstanceProfileParams, InstanceProfileState,
};

#[cfg(feature = "output_progress")]
use peace::progress_model::ProgressMsgUpdate;

/// Reads the current state of the instance profile state.
#[derive(Debug)]
pub struct InstanceProfileStateCurrentFn<Id>(PhantomData<Id>);

impl<Id> InstanceProfileStateCurrentFn<Id>
where
    Id: Send + Sync,
{
    pub async fn try_state_current(
        fn_ctx: FnCtx<'_>,
        params_partial: &<InstanceProfileParams<Id> as Params>::Partial,
        data: InstanceProfileData<'_, Id>,
    ) -> Result<Option<InstanceProfileState>, InstanceProfileError> {
        if let Ok(params) = params_partial.try_into() {
            Self::state_current(fn_ctx, &params, data).await.map(Some)
        } else {
            Ok(None)
        }
    }

    pub async fn state_current(
        fn_ctx: FnCtx<'_>,
        params: &InstanceProfileParams<Id>,
        data: InstanceProfileData<'_, Id>,
    ) -> Result<InstanceProfileState, InstanceProfileError> {
        let name = params.name();
        let path = params.path();

        Self::state_current_internal(fn_ctx, data, name, path).await
    }

    async fn state_current_internal(
        fn_ctx: FnCtx<'_>,
        data: InstanceProfileData<'_, Id>,
        name: &str,
        path: &str,
    ) -> Result<InstanceProfileState, InstanceProfileError> {
        let client = data.client();

        #[cfg(not(feature = "output_progress"))]
        let _fn_ctx = fn_ctx;
        #[cfg(feature = "output_progress")]
        let progress_sender = &fn_ctx.progress_sender;
        #[cfg(feature = "output_progress")]
        progress_sender.tick(ProgressMsgUpdate::Set(String::from(
            "fetching instance profile",
        )));
        let get_instance_profile_result = client
            .get_instance_profile()
            .instance_profile_name(name)
            .send()
            .await;
        let instance_profile_opt = match get_instance_profile_result {
            Ok(get_instance_profile_output) => {
                #[cfg(feature = "output_progress")]
                progress_sender.tick(ProgressMsgUpdate::Set(String::from(
                    "instance profile fetched",
                )));

                let instance_profile = get_instance_profile_output.instance_profile().expect(
                    "Expected instance profile to be some when get_instance_profile is successful.",
                );

                let instance_profile_name = instance_profile.instance_profile_name().to_string();
                let instance_profile_path = instance_profile.path().to_string();
                let instance_profile_id = instance_profile.instance_profile_id().to_string();
                let instance_profile_arn = instance_profile.arn().to_string();
                let instance_profile_id_and_arn =
                    InstanceProfileIdAndArn::new(instance_profile_id, instance_profile_arn);

                let role_associated = !instance_profile.roles().is_empty();

                Some((
                    instance_profile_name,
                    instance_profile_path,
                    instance_profile_id_and_arn,
                    role_associated,
                ))
            }
            Err(error) => {
                #[cfg(feature = "output_progress")]
                progress_sender.tick(ProgressMsgUpdate::Set(String::from(
                    "instance profile not fetched",
                )));

                #[cfg(feature = "error_reporting")]
                let (aws_desc, aws_desc_span) = crate::items::aws_error_desc!(&error);

                match &error {
                    SdkError::ServiceError(service_error) => match service_error.err() {
                        GetInstanceProfileError::NoSuchEntityException(_) => None,
                        _ => {
                            return Err(InstanceProfileError::InstanceProfileGetError {
                                instance_profile_name: name.to_string(),
                                instance_profile_path: path.to_string(),
                                #[cfg(feature = "error_reporting")]
                                aws_desc,
                                #[cfg(feature = "error_reporting")]
                                aws_desc_span,
                                error,
                            });
                        }
                    },
                    _ => {
                        return Err(InstanceProfileError::InstanceProfileGetError {
                            instance_profile_name: name.to_string(),
                            instance_profile_path: path.to_string(),
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

        if let Some((
            instance_profile_name,
            instance_profile_path,
            instance_profile_id_and_arn,
            role_associated,
        )) = instance_profile_opt
        {
            let state_current = InstanceProfileState::Some {
                name: instance_profile_name,
                path: instance_profile_path,
                instance_profile_id_and_arn: Generated::Value(instance_profile_id_and_arn),
                role_associated,
            };

            Ok(state_current)
        } else {
            Ok(InstanceProfileState::None)
        }
    }
}
