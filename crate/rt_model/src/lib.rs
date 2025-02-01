//! Runtime data types for the peace automation framework.
//!
//! This crate re-exports types from `peace_rt_model_native` or
//! `peace_rt_model_web` depending on the compilation target architecture.

// Re-exports
pub use peace_data::fn_graph::{self, FnRef};
pub use peace_rt_model_core::*;

#[cfg(not(target_arch = "wasm32"))]
pub use peace_rt_model_native::*;

#[cfg(target_arch = "wasm32")]
pub use peace_rt_model_web::*;

pub use crate::{
    in_memory_text_output::InMemoryTextOutput, item_boxed::ItemBoxed, item_rt::ItemRt,
    item_wrapper::ItemWrapper, params_specs_serializer::ParamsSpecsSerializer,
    params_specs_type_reg::ParamsSpecsTypeReg, states_type_reg::StatesTypeReg,
};

pub mod outcomes;

mod in_memory_text_output;
mod item_boxed;
mod item_rt;
mod item_wrapper;
mod params_specs_serializer;
mod params_specs_type_reg;
mod states_type_reg;
