use std::ops::{Deref, DerefMut};

use peace_item_model::ItemId;
use peace_params::AnySpecRtBoxed;
use peace_resource_rt::type_reg::untagged::TypeReg;

/// Type registry for each item's [`Params`]'s Spec.
///
/// This is used to deserialize [`ParamsSpecsFile`].
///
/// Note: [`StatesTypeReg`] uses [`BoxDtDisplay`], whereas this uses
/// [`AnySpecRtBoxed`].
///
/// [`AnySpecRtBoxed`]: peace_params::AnySpecRtBoxed
/// [`BoxDtDisplay`]: peace_resource_rt::type_reg::untagged::BoxDtDisplay
/// [`Params`]: peace_cfg::Item::Params
/// [`StatesTypeReg`]: crate::StatesTypeReg
#[derive(Debug, Default)]
pub struct ParamsSpecsTypeReg(TypeReg<ItemId, AnySpecRtBoxed>);

impl ParamsSpecsTypeReg {
    /// Returns a new `ParamsSpecsTypeReg`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deref for ParamsSpecsTypeReg {
    type Target = TypeReg<ItemId, AnySpecRtBoxed>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ParamsSpecsTypeReg {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
