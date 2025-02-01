use std::fmt;

use peace::cfg::state::Generated;
use serde::{Deserialize, Serialize};

use crate::items::peace_aws_iam_role::model::{ManagedPolicyAttachment, RoleIdAndArn};

#[cfg(feature = "output_progress")]
use peace::item_interaction_model::ItemLocationState;

/// IAM role state.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum IamRoleState {
    /// Instance profile does not exist.
    None,
    /// Instance profile exists.
    Some {
        /// Instance profile name.
        ///
        /// Alphanumeric characters and `_+=,.@-` are allowed.
        ///
        /// TODO: newtype + proc macro.
        name: String,
        /// String that begins and ends with a forward slash.
        ///
        /// Defaults to `/`.
        ///
        /// e.g. `/demo/`
        #[serde(default = "path_default")]
        path: String,
        /// The stable and unique IDs identifying the role.
        role_id_and_arn: Generated<RoleIdAndArn>,
        /// Managed policy to attach to the role.
        managed_policy_attachment: ManagedPolicyAttachment,
    },
}

fn path_default() -> String {
    String::from("/")
}

impl fmt::Display for IamRoleState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::None => "does not exist".fmt(f),
            Self::Some {
                name,
                path,
                role_id_and_arn,
                managed_policy_attachment,
            } => {
                match role_id_and_arn {
                    Generated::Tbd => write!(f, "`{path}{name}` should exist")?,
                    Generated::Value(_role_id_and_arn) => {
                        // https://console.aws.amazon.com/iamv2/home#/roles/details/demo
                        write!(
                            f,
                            "exists at https://console.aws.amazon.com/iamv2/home#/roles/details/{name}"
                        )?;
                    }
                }
                if managed_policy_attachment.attached() {
                    write!(f, " with policy attached")?;
                } else {
                    write!(f, ", but managed policy not attached")?;
                };

                Ok(())
            }
        }
    }
}

#[cfg(feature = "output_progress")]
impl<'state> From<&'state IamRoleState> for ItemLocationState {
    fn from(iam_role_state: &'state IamRoleState) -> ItemLocationState {
        match iam_role_state {
            IamRoleState::Some { .. } => ItemLocationState::Exists,
            IamRoleState::None => ItemLocationState::NotExists,
        }
    }
}
