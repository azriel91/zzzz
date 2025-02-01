use std::fmt;

use peace::cfg::state::Generated;
use serde::{Deserialize, Serialize};

#[cfg(feature = "output_progress")]
use peace::item_interaction_model::ItemLocationState;

/// S3 object state.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum S3ObjectState {
    /// S3 object does not exist.
    None,
    /// S3 object exists.
    Some {
        /// S3 bucket to insert the object into,
        bucket_name: String,
        /// S3 object key.
        object_key: String,
        /// MD5 hex string of the content.
        content_md5_hexstr: Option<String>,
        /// ETag served by S3.
        e_tag: Generated<String>,
    },
}

impl fmt::Display for S3ObjectState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::None => "does not exist".fmt(f),
            Self::Some {
                bucket_name,
                object_key,
                content_md5_hexstr,
                e_tag,
            } => {
                match e_tag {
                    Generated::Tbd => {
                        write!(f, "`{object_key}` should be uploaded to `{bucket_name}`")?
                    }
                    Generated::Value(_) => {
                        // https://s3.console.aws.amazon.com/s3/object/azriel-peace-envman-demo?prefix=web_app.tar
                        write!(
                            f,
                            "uploaded at https://s3.console.aws.amazon.com/s3/object/{bucket_name}?prefix={object_key}"
                        )?;
                    }
                }
                if let Some(content_md5_hexstr) = content_md5_hexstr {
                    write!(f, " (MD5: {content_md5_hexstr})")
                } else {
                    write!(f, " (MD5 unknown)")
                }
            }
        }
    }
}

#[cfg(feature = "output_progress")]
impl<'state> From<&'state S3ObjectState> for ItemLocationState {
    fn from(s3_object_state: &'state S3ObjectState) -> ItemLocationState {
        match s3_object_state {
            S3ObjectState::Some { .. } => ItemLocationState::Exists,
            S3ObjectState::None => ItemLocationState::NotExists,
        }
    }
}
