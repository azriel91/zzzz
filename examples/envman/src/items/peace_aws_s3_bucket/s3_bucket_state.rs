use std::fmt;

use chrono::{DateTime, Utc};
use peace::cfg::state::Timestamped;
use serde::{Deserialize, Serialize};

#[cfg(feature = "output_progress")]
use peace::item_interaction_model::ItemLocationState;

/// S3 bucket state.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum S3BucketState {
    /// S3 bucket does not exist.
    None,
    /// S3 bucket exists.
    Some {
        /// S3 bucket name.
        ///
        /// Alphanumeric characters and `_+=,.@-` are allowed.
        ///
        /// TODO: newtype + proc macro.
        name: String,
        /// Timestamp that the S3Bucket was created.
        creation_date: Timestamped<DateTime<Utc>>,
    },
}

impl S3BucketState {
    /// Returns the bucket name if the bucket exists.
    pub fn bucket_name(&self) -> Option<String> {
        match self {
            S3BucketState::None => None,
            S3BucketState::Some {
                name,
                creation_date: _,
            } => Some(name.clone()),
        }
    }
}

impl fmt::Display for S3BucketState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::None => "does not exist".fmt(f),
            // https://s3.console.aws.amazon.com/s3/buckets/azriel-peace-envman-demo
            Self::Some {
                name,
                creation_date,
            } => match creation_date {
                Timestamped::Tbd => write!(f, "`{name}` should exist"),
                Timestamped::Value(_) => write!(
                    f,
                    "exists at https://s3.console.aws.amazon.com/s3/buckets/{name}"
                ),
            },
        }
    }
}

#[cfg(feature = "output_progress")]
impl<'state> From<&'state S3BucketState> for ItemLocationState {
    fn from(s3_bucket_state: &'state S3BucketState) -> ItemLocationState {
        match s3_bucket_state {
            S3BucketState::Some { .. } => ItemLocationState::Exists,
            S3BucketState::None => ItemLocationState::NotExists,
        }
    }
}
