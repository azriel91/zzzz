use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use fn_graph::{DataAccessDyn, TypeIds};
use peace_cfg::{FnSpec, FullSpec, State};
use peace_diff::Diff;
use serde::{de::DeserializeOwned, Serialize};

use crate::{Error, FullSpecWrapper};

pub use self::{
    clean_op_spec_rt::CleanOpSpecRt, ensure_op_spec_rt::EnsureOpSpecRt, full_spec_rt::FullSpecRt,
    status_fn_spec_rt::StatusFnSpecRt,
};

mod clean_op_spec_rt;
mod ensure_op_spec_rt;
mod full_spec_rt;
mod status_fn_spec_rt;

/// Defines all of the data and logic to manage a user defined item.
///
/// # Type Parameters
///
/// * `E`: Application specific error type.
#[derive(Debug)]
pub struct FullSpecBoxed<'op, E>(Box<dyn FullSpecRt<'op, Error<E>> + 'op>)
where
    E: std::error::Error;

impl<'op, E> Deref for FullSpecBoxed<'op, E>
where
    E: std::error::Error,
{
    type Target = dyn FullSpecRt<'op, Error<E>> + 'op;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<'op, E> DerefMut for FullSpecBoxed<'op, E>
where
    E: std::error::Error,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

impl<'op, FS, E, StateLogical, StatePhysical, StatusFnSpec, EnsureOpSpec, CleanOpSpec> From<FS>
    for FullSpecBoxed<'op, E>
where
    FS: Debug
        + FullSpec<
            'op,
            Error = E,
            StateLogical = StateLogical,
            StatePhysical = StatePhysical,
            StatusFnSpec = StatusFnSpec,
            EnsureOpSpec = EnsureOpSpec,
            CleanOpSpec = CleanOpSpec,
        > + Send
        + Sync
        + 'op,
    E: Debug + Send + Sync + std::error::Error + 'op,
    StateLogical: Debug + Diff + Serialize + DeserializeOwned + Send + Sync + 'op,
    StatePhysical: Debug + Serialize + DeserializeOwned + Send + Sync + 'op,
    StatusFnSpec: Debug
        + FnSpec<'op, Error = E, Output = State<StateLogical, StatePhysical>>
        + Send
        + Sync
        + 'op,
    EnsureOpSpec: Debug
        + peace_cfg::EnsureOpSpec<
            'op,
            Error = E,
            StateLogical = StateLogical,
            StatePhysical = StatePhysical,
        > + Send
        + Sync
        + 'op,
    CleanOpSpec: Debug
        + peace_cfg::CleanOpSpec<
            'op,
            Error = E,
            StateLogical = StateLogical,
            StatePhysical = StatePhysical,
        > + Send
        + Sync
        + 'op,
{
    fn from(full_spec: FS) -> Self {
        Self(Box::new(FullSpecWrapper::from(full_spec)))
    }
}

impl<'op, E> DataAccessDyn for FullSpecBoxed<'op, E>
where
    E: std::error::Error,
{
    fn borrows(&self) -> TypeIds {
        DataAccessDyn::borrows(self.0.as_ref())
    }

    fn borrow_muts(&self) -> TypeIds {
        DataAccessDyn::borrow_muts(self.0.as_ref())
    }
}
