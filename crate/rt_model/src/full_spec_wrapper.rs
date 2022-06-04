use std::{fmt::Debug, marker::PhantomData};

use diff::Diff;
use fn_graph::{DataAccess, DataAccessDyn, TypeIds};
use peace_cfg::{async_trait, FullSpec, OpSpec, OpSpecDry};
use peace_data::Resources;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    full_spec_boxed::{CleanOpSpecRt, EnsureOpSpecRt, FullSpecRt, StatusOpSpecRt},
    Error,
};

/// Wraps a type implementing [`FullSpec`].
#[derive(Debug)]
pub struct FullSpecWrapper<'op, FS, E, ResIds, State, StatusOpSpec, EnsureOpSpec, CleanOpSpec>(
    FS,
    PhantomData<&'op (E, ResIds, State, StatusOpSpec, EnsureOpSpec, CleanOpSpec)>,
);

impl<'op, FS, E, ResIds, State, StatusOpSpec, EnsureOpSpec, CleanOpSpec> From<FS>
    for FullSpecWrapper<'op, FS, E, ResIds, State, StatusOpSpec, EnsureOpSpec, CleanOpSpec>
where
    FS: Debug
        + FullSpec<
            'op,
            State = State,
            Error = E,
            ResIds = ResIds,
            StatusOpSpec = StatusOpSpec,
            EnsureOpSpec = EnsureOpSpec,
            CleanOpSpec = CleanOpSpec,
        > + Send
        + Sync,
    E: Debug + Send + Sync + std::error::Error,
    ResIds: Debug + Serialize + DeserializeOwned + Send + Sync,
    State: Debug + Diff + Serialize + DeserializeOwned + Send + Sync,
    StatusOpSpec: Debug + OpSpec<'op, State = (), Output = State> + Send + Sync,
    EnsureOpSpec: Debug + OpSpecDry<'op, State = State, Output = ResIds> + Send + Sync,
    CleanOpSpec: Debug + OpSpecDry<'op, State = State, Output = ResIds> + Send + Sync,
{
    fn from(full_spec: FS) -> Self {
        Self(full_spec, PhantomData)
    }
}

impl<'op, FS, E, ResIds, State, StatusOpSpec, EnsureOpSpec, CleanOpSpec> DataAccessDyn
    for FullSpecWrapper<'op, FS, E, ResIds, State, StatusOpSpec, EnsureOpSpec, CleanOpSpec>
where
    FS: Debug
        + FullSpec<
            'op,
            State = State,
            Error = E,
            ResIds = ResIds,
            StatusOpSpec = StatusOpSpec,
            EnsureOpSpec = EnsureOpSpec,
            CleanOpSpec = CleanOpSpec,
        > + Send
        + Sync,
    E: Debug + Send + Sync + std::error::Error,
    ResIds: Debug + Serialize + DeserializeOwned + Send + Sync,
    State: Debug + Diff + Serialize + DeserializeOwned + Send + Sync,
    StatusOpSpec: Debug + OpSpec<'op, State = (), Output = State> + Send + Sync,
    EnsureOpSpec: Debug + OpSpecDry<'op, State = State, Output = ResIds> + Send + Sync,
    CleanOpSpec: Debug + OpSpecDry<'op, State = State, Output = ResIds> + Send + Sync,
{
    fn borrows(&self) -> TypeIds {
        <EnsureOpSpec::Data as DataAccess>::borrows()
    }

    fn borrow_muts(&self) -> TypeIds {
        <EnsureOpSpec::Data as DataAccess>::borrow_muts()
    }
}

impl<'op, FS, E, ResIds, State, StatusOpSpec, EnsureOpSpec, CleanOpSpec> FullSpecRt<'op, Error<E>>
    for FullSpecWrapper<'op, FS, E, ResIds, State, StatusOpSpec, EnsureOpSpec, CleanOpSpec>
where
    FS: Debug
        + FullSpec<
            'op,
            State = State,
            Error = E,
            ResIds = ResIds,
            StatusOpSpec = StatusOpSpec,
            EnsureOpSpec = EnsureOpSpec,
            CleanOpSpec = CleanOpSpec,
        > + Send
        + Sync,
    E: Debug + Send + Sync + std::error::Error,
    ResIds: Debug + Serialize + DeserializeOwned + Send + Sync,
    State: Debug + Diff + Serialize + DeserializeOwned + Send + Sync,
    StatusOpSpec: Debug + OpSpec<'op, State = (), Error = E, Output = State> + Send + Sync,
    EnsureOpSpec: Debug + OpSpecDry<'op, State = State, Error = E, Output = ResIds> + Send + Sync,
    CleanOpSpec: Debug + OpSpecDry<'op, State = State, Error = E, Output = ResIds> + Send + Sync,
{
}

#[async_trait]
impl<'op, FS, E, ResIds, State, StatusOpSpec, EnsureOpSpec, CleanOpSpec> StatusOpSpecRt<'op>
    for FullSpecWrapper<'op, FS, E, ResIds, State, StatusOpSpec, EnsureOpSpec, CleanOpSpec>
where
    FS: Debug
        + FullSpec<
            'op,
            State = State,
            Error = E,
            ResIds = ResIds,
            StatusOpSpec = StatusOpSpec,
            EnsureOpSpec = EnsureOpSpec,
            CleanOpSpec = CleanOpSpec,
        > + Send
        + Sync,
    E: Debug + Send + Sync + std::error::Error,
    ResIds: Debug + Serialize + DeserializeOwned + Send + Sync,
    State: Debug + Diff + Serialize + DeserializeOwned + Send + Sync,
    StatusOpSpec: Debug + OpSpec<'op, State = (), Error = E, Output = State> + Send + Sync,
    EnsureOpSpec: Debug + OpSpecDry<'op, State = State, Error = E, Output = ResIds> + Send + Sync,
    CleanOpSpec: Debug + OpSpecDry<'op, State = State, Error = E, Output = ResIds> + Send + Sync,
{
    type Error = Error<E>;

    async fn setup(&self, resources: &mut Resources) -> Result<(), Self::Error> {
        let _progress_limit = <CleanOpSpec as OpSpec>::setup(resources)
            .await
            .map_err(Error::CleanSetup)?;
        Ok(())
    }

    async fn check(&self, _resources: &Resources) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn exec(&self, _resources: &Resources) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[async_trait]
impl<'op, FS, E, ResIds, State, StatusOpSpec, EnsureOpSpec, CleanOpSpec> EnsureOpSpecRt<'op>
    for FullSpecWrapper<'op, FS, E, ResIds, State, StatusOpSpec, EnsureOpSpec, CleanOpSpec>
where
    FS: Debug
        + FullSpec<
            'op,
            State = State,
            Error = E,
            ResIds = ResIds,
            StatusOpSpec = StatusOpSpec,
            EnsureOpSpec = EnsureOpSpec,
            CleanOpSpec = CleanOpSpec,
        > + Send
        + Sync,
    E: Debug + Send + Sync + std::error::Error,
    ResIds: Debug + Serialize + DeserializeOwned + Send + Sync,
    State: Debug + Diff + Serialize + DeserializeOwned + Send + Sync,
    StatusOpSpec: Debug + OpSpec<'op, State = (), Error = E, Output = State> + Send + Sync,
    EnsureOpSpec: Debug + OpSpecDry<'op, State = State, Error = E, Output = ResIds> + Send + Sync,
    CleanOpSpec: Debug + OpSpecDry<'op, State = State, Error = E, Output = ResIds> + Send + Sync,
{
    type Error = Error<E>;

    async fn setup(&self, resources: &mut Resources) -> Result<(), Self::Error> {
        let _progress_limit = <EnsureOpSpec as OpSpec>::setup(resources)
            .await
            .map_err(Error::EnsureSetup)?;
        Ok(())
    }

    async fn check(&self, _resources: &Resources) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn exec(&self, _resources: &Resources) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[async_trait]
impl<'op, FS, E, ResIds, State, StatusOpSpec, EnsureOpSpec, CleanOpSpec> CleanOpSpecRt<'op>
    for FullSpecWrapper<'op, FS, E, ResIds, State, StatusOpSpec, EnsureOpSpec, CleanOpSpec>
where
    FS: Debug
        + FullSpec<
            'op,
            State = State,
            Error = E,
            ResIds = ResIds,
            StatusOpSpec = StatusOpSpec,
            EnsureOpSpec = EnsureOpSpec,
            CleanOpSpec = CleanOpSpec,
        > + Send
        + Sync,
    E: Debug + Send + Sync + std::error::Error,
    ResIds: Debug + Serialize + DeserializeOwned + Send + Sync,
    State: Debug + Diff + Serialize + DeserializeOwned + Send + Sync,
    StatusOpSpec: Debug + OpSpec<'op, State = (), Error = E, Output = State> + Send + Sync,
    EnsureOpSpec: Debug + OpSpecDry<'op, State = State, Error = E, Output = ResIds> + Send + Sync,
    CleanOpSpec: Debug + OpSpecDry<'op, State = State, Error = E, Output = ResIds> + Send + Sync,
{
    type Error = Error<E>;

    async fn setup(&self, resources: &mut Resources) -> Result<(), Self::Error> {
        let _progress_limit = <StatusOpSpec as OpSpec>::setup(resources)
            .await
            .map_err(Error::StatusSetup)?;
        Ok(())
    }

    async fn check(&self, _resources: &Resources) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn exec(&self, _resources: &Resources) -> Result<(), Self::Error> {
        Ok(())
    }
}
