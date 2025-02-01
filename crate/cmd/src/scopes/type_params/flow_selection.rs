use own::OwnedOrRef;
use peace_flow_rt::Flow;

/// A `Flow` is not yet selected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlowNotSelected;

/// A `Flow` is selected.
#[derive(Debug)]
pub struct FlowSelected<'ctx, E>(pub(crate) OwnedOrRef<'ctx, Flow<E>>);
