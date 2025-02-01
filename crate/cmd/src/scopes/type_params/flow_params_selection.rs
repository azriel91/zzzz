use std::{collections::BTreeMap, fmt::Debug, hash::Hash};

use peace_profile_model::Profile;
use peace_rt_model::params::FlowParams;
use serde::{de::DeserializeOwned, Serialize};

/// The application does not use any flow parameters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlowParamsNone;

/// The application has flow parameters.
#[derive(Debug)]
pub struct FlowParamsSome<FlowParamsK>(pub(crate) FlowParams<FlowParamsK>)
where
    FlowParamsK: Clone + Debug + Eq + Hash + DeserializeOwned + Serialize + Send + Sync + 'static;

impl<FlowParamsK> Default for FlowParamsSome<FlowParamsK>
where
    FlowParamsK: Clone + Debug + Eq + Hash + DeserializeOwned + Serialize + Send + Sync + 'static,
{
    fn default() -> Self {
        FlowParamsSome(FlowParams::default())
    }
}

/// The application has flow parameters from multiple profiles.
#[derive(Debug)]
pub struct FlowParamsSomeMulti<FlowParamsK>(pub(crate) BTreeMap<Profile, FlowParams<FlowParamsK>>)
where
    FlowParamsK: Clone + Debug + Eq + Hash + DeserializeOwned + Serialize + Send + Sync + 'static;

impl<FlowParamsK> Default for FlowParamsSomeMulti<FlowParamsK>
where
    FlowParamsK: Clone + Debug + Eq + Hash + DeserializeOwned + Serialize + Send + Sync + 'static,
{
    fn default() -> Self {
        FlowParamsSomeMulti(BTreeMap::default())
    }
}
