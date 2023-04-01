use std::fmt;

use peace::cfg::state::Generated;
use serde::{Deserialize, Serialize};

use crate::item_specs::peace_aws_iam_policy::model::PolicyIdArnVersion;

/// Instance profile state.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum IamPolicyState {
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
        /// Policy document to use.
        policy_document: String,
        /// The stable and unique IDs identifying the policy.
        policy_id_arn_version: Generated<PolicyIdArnVersion>,
    },
}

fn path_default() -> String {
    String::from("/")
}

impl fmt::Display for IamPolicyState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::None => "does not exist".fmt(f),
            Self::Some {
                name: _,
                path: _,
                policy_document: _,
                policy_id_arn_version,
            } => {
                match policy_id_arn_version {
                    Generated::Tbd => write!(f, "should exist"),
                    Generated::Value(policy_id_arn_version) => {
                        let arn = policy_id_arn_version.arn();
                        // https://console.aws.amazon.com/iam/home#/policies/arn:aws:iam::$acc_number:policy/demo
                        write!(
                            f,
                            "exists at https://console.aws.amazon.com/iam/home#/policies/{arn}"
                        )
                    }
                }
            }
        }
    }
}
