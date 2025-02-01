use std::fmt;

use peace::cfg::state::Generated;
use serde::{Deserialize, Serialize};

use crate::items::peace_aws_instance_profile::model::InstanceProfileIdAndArn;

#[cfg(feature = "output_progress")]
use peace::item_interaction_model::ItemLocationState;

/// Instance profile state.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum InstanceProfileState {
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
        /// The stable and unique IDs identifying the instance profile.
        instance_profile_id_and_arn: Generated<InstanceProfileIdAndArn>,
        /// Whether the role has been associated with the instance profile.
        role_associated: bool,
    },
}

fn path_default() -> String {
    String::from("/")
}

impl fmt::Display for InstanceProfileState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::None => "does not exist".fmt(f),
            Self::Some {
                name,
                path,
                instance_profile_id_and_arn,
                role_associated,
            } => {
                match instance_profile_id_and_arn {
                    Generated::Tbd => write!(f, "`{path}{name}` should exist ")?,
                    Generated::Value(_instance_profile_id_and_arn) => {
                        // https://console.aws.amazon.com/iamv2/home#/roles/details/demo
                        write!(
                            f,
                            "exists at https://console.aws.amazon.com/iamv2/home#/roles/details/{name} "
                        )?;
                    }
                }
                if *role_associated {
                    write!(f, "associated with same named role")?;
                } else {
                    write!(f, "but role not associated")?;
                }

                Ok(())
            }
        }
    }
}

#[cfg(feature = "output_progress")]
impl<'state> From<&'state InstanceProfileState> for ItemLocationState {
    fn from(instance_profile_state: &'state InstanceProfileState) -> ItemLocationState {
        match instance_profile_state {
            InstanceProfileState::Some { .. } => ItemLocationState::Exists,
            InstanceProfileState::None => ItemLocationState::NotExists,
        }
    }
}
