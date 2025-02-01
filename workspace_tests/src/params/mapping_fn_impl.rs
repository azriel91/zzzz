use peace::params::MappingFnImpl;

#[test]
fn debug() {
    let mapping_fn_impl = MappingFnImpl::from((
        Some(String::from("field_name")),
        #[cfg_attr(coverage_nightly, coverage(off))]
        |_: &bool| None::<Option<u16>>,
    ));
    assert_eq!(
        "MappingFnImpl { \
            field_name: Some(\"field_name\"), \
            fn_map: \"Some(Fn(&bool,) -> Option<Option<u16>>)\", \
            marker: PhantomData<(core::option::Option<u16>, (bool,))> \
        }",
        format!("{mapping_fn_impl:?}")
    );

    let mapping_fn_impl = MappingFnImpl::from((
        Some(String::from("field_name")),
        #[cfg_attr(coverage_nightly, coverage(off))]
        |_: &u32, _: &u64| None::<Option<u16>>,
    ));
    assert_eq!(
        "MappingFnImpl { \
            field_name: Some(\"field_name\"), \
            fn_map: \"Some(Fn(&u32, &u64) -> Option<Option<u16>>)\", \
            marker: PhantomData<(core::option::Option<u16>, (u32, u64))> \
        }",
        format!("{mapping_fn_impl:?}")
    );
}

mapping_tests!(apply_dry, ApplyDry);
mapping_tests!(current, Current);
mapping_tests!(goal, Goal);
mapping_tests!(clean, Clean);

macro_rules! mapping_tests {
    ($module_name:ident, $value_resolution_mode:ident) => {
        mod $module_name {
            use peace::{
                data::marker::$value_resolution_mode,
                item_model::item_id,
                params::{
                    MappingFn, MappingFnImpl, ParamsResolveError,
                    ValueResolutionCtx, ValueResolutionMode,
                },
                resource_rt::{resources::ts::SetUp, Resources},
            };

            #[test]
            fn mapping_fn_map_returns_ok_when_referenced_values_are_present_directly()
            -> Result<(), ParamsResolveError> {
                let mapping_fn_impl =
                    MappingFnImpl::from((Some(String::from("field_name")), |a: &u32, b: &u64| {
                        let a = u16::try_from(*a).ok()?;
                        let b = u16::try_from(*b).ok()?;
                        a.checked_add(b)
                    }));
                let resources = {
                    let mut resources = Resources::new();
                    resources.insert(1u32);
                    resources.insert(2u64);
                    Resources::<SetUp>::from(resources)
                };
                let mut value_resolution_ctx = ValueResolutionCtx::new(
                    ValueResolutionMode::$value_resolution_mode,
                    item_id!("mapping_fn_map"),
                    String::from(crate::fn_name_short!()),
                );

                let sum = MappingFn::map(
                    &mapping_fn_impl,
                    &resources,
                    &mut value_resolution_ctx,
                )?;
                assert_eq!(3, sum);

                Ok(())
            }

            #[test]
            fn mapping_fn_map_returns_ok_when_referenced_values_are_present_through_data_marker()
            -> Result<(), ParamsResolveError> {
                let mapping_fn_impl =
                    MappingFnImpl::from((Some(String::from("field_name")), |a: &u32, b: &u64| {
                        let a = u16::try_from(*a).ok()?;
                        let b = u16::try_from(*b).ok()?;
                        a.checked_add(b)
                    }));
                let resources = {
                    let mut resources = Resources::new();
                    resources.insert($value_resolution_mode(Some(1u32)));
                    resources.insert($value_resolution_mode(Some(2u64)));
                    Resources::<SetUp>::from(resources)
                };
                let mut value_resolution_ctx = ValueResolutionCtx::new(
                    ValueResolutionMode::$value_resolution_mode,
                    item_id!("mapping_fn_map"),
                    String::from(crate::fn_name_short!()),
                );

                let sum = MappingFn::map(
                    &mapping_fn_impl,
                    &resources,
                    &mut value_resolution_ctx,
                )?;
                assert_eq!(3, sum);

                Ok(())
            }

            #[test]
            fn mapping_fn_map_returns_err_when_referenced_value_is_none()
            -> Result<(), ParamsResolveError> {
                let mapping_fn_impl =
                    MappingFnImpl::from((Some(String::from("field_name")), |a: &u32, b: &u64| {
                        let a = u16::try_from(*a).ok()?;
                        let b = u16::try_from(*b).ok()?;
                        a.checked_add(b)
                    }));
                let resources = {
                    let mut resources = Resources::new();
                    resources.insert($value_resolution_mode(Some(1u32)));
                    resources.insert($value_resolution_mode(None::<u64>));
                    Resources::<SetUp>::from(resources)
                };
                let mut value_resolution_ctx = ValueResolutionCtx::new(
                    ValueResolutionMode::$value_resolution_mode,
                    item_id!("mapping_fn_map"),
                    String::from(crate::fn_name_short!()),
                );

                let sum_result = MappingFn::map(
                    &mapping_fn_impl,
                    &resources,
                    &mut value_resolution_ctx,
                );
                ({
                    #[cfg_attr(coverage_nightly, coverage(off))]
                    || {
                        assert!(
                            matches!(
                                &sum_result,
                                Err(ParamsResolveError::FromMap {
                                    value_resolution_ctx,
                                    from_type_name
                                })
                                if matches!(
                                    value_resolution_ctx,
                                    value_resolution_ctx
                                    if value_resolution_ctx.value_resolution_mode()
                                        == ValueResolutionMode::$value_resolution_mode
                                    && value_resolution_ctx.item_id()
                                        == &item_id!("mapping_fn_map")
                                    && value_resolution_ctx.params_type_name() == crate::fn_name_short!()
                                    && value_resolution_ctx.resolution_chain() == []
                                )
                                && from_type_name == "u64" // u64 is missing from `resources`
                            ),
                            "expected `sum_result` to be \
                            `Err(ParamsResolveError::FromMap {{ .. }}`,\n\
                            but was {sum_result:?}"
                        );
                    }
                })();

                Ok(())
            }

            #[test]
            fn mapping_fn_map_returns_err_when_referenced_value_is_absent()
            -> Result<(), ParamsResolveError> {
                let mapping_fn_impl =
                    MappingFnImpl::from((Some(String::from("field_name")), |a: &u32, b: &u64| {
                        let a = u16::try_from(*a).ok()?;
                        let b = u16::try_from(*b).ok()?;
                        a.checked_add(b)
                    }));
                let resources = {
                    let mut resources = Resources::new();
                    resources.insert($value_resolution_mode(Some(1u32)));
                    // resources.insert($value_resolution_mode(Some(2u64)));
                    Resources::<SetUp>::from(resources)
                };
                let mut value_resolution_ctx = ValueResolutionCtx::new(
                    ValueResolutionMode::$value_resolution_mode,
                    item_id!("mapping_fn_map"),
                    String::from(crate::fn_name_short!()),
                );

                let sum_result = MappingFn::map(
                    &mapping_fn_impl,
                    &resources,
                    &mut value_resolution_ctx,
                );

                ({
                    #[cfg_attr(coverage_nightly, coverage(off))]
                    || {
                        assert!(
                            matches!(
                                &sum_result,
                                Err(ParamsResolveError::FromMap {
                                    value_resolution_ctx,
                                    from_type_name
                                })
                                if matches!(
                                    value_resolution_ctx,
                                    value_resolution_ctx
                                    if value_resolution_ctx.value_resolution_mode()
                                        == ValueResolutionMode::$value_resolution_mode
                                    && value_resolution_ctx.item_id()
                                        == &item_id!("mapping_fn_map")
                                    && value_resolution_ctx.params_type_name() == crate::fn_name_short!()
                                    && value_resolution_ctx.resolution_chain() == []
                                )
                                && from_type_name == "u64" // u64 is missing from `resources`
                            ),
                            "expected `sum_result` to be \
                            `Err(ParamsResolveError::FromMap {{ .. }}`,\n\
                            but was {sum_result:?}"
                        );
                    }
                })();

                Ok(())
            }

            #[test]
            fn mapping_fn_try_map_returns_ok_some_when_referenced_values_are_present()
            -> Result<(), ParamsResolveError> {
                let mapping_fn_impl =
                    MappingFnImpl::from((Some(String::from("field_name")), |a: &u32, b: &u64| {
                        let a = u16::try_from(*a).ok()?;
                        let b = u16::try_from(*b).ok()?;
                        a.checked_add(b)
                    }));
                let resources = {
                    let mut resources = Resources::new();
                    resources.insert($value_resolution_mode(Some(1u32)));
                    resources.insert($value_resolution_mode(Some(2u64)));
                    Resources::<SetUp>::from(resources)
                };
                let mut value_resolution_ctx = ValueResolutionCtx::new(
                    ValueResolutionMode::$value_resolution_mode,
                    item_id!("mapping_fn_map"),
                    String::from(crate::fn_name_short!()),
                );

                let sum = MappingFn::try_map(
                    &mapping_fn_impl,
                    &resources,
                    &mut value_resolution_ctx,
                )?;
                assert_eq!(Some(3), sum);

                Ok(())
            }

            #[test]
            fn mapping_fn_try_map_returns_ok_none_when_referenced_value_is_none()
            -> Result<(), ParamsResolveError> {
                let mapping_fn_impl =
                    MappingFnImpl::from((Some(String::from("field_name")), |a: &u32, b: &u64| {
                        let a = u16::try_from(*a).ok()?;
                        let b = u16::try_from(*b).ok()?;
                        a.checked_add(b)
                    }));
                let resources = {
                    let mut resources = Resources::new();
                    resources.insert($value_resolution_mode(Some(1u32)));
                    resources.insert($value_resolution_mode(None::<u64>));
                    Resources::<SetUp>::from(resources)
                };
                let mut value_resolution_ctx = ValueResolutionCtx::new(
                    ValueResolutionMode::$value_resolution_mode,
                    item_id!("mapping_fn_map"),
                    String::from(crate::fn_name_short!()),
                );

                let sum = MappingFn::try_map(
                    &mapping_fn_impl,
                    &resources,
                    &mut value_resolution_ctx,
                )?;
                assert_eq!(None, sum);

                Ok(())
            }

            #[test]
            fn mapping_fn_try_map_returns_ok_none_when_referenced_value_is_absent()
            -> Result<(), ParamsResolveError> {
                let mapping_fn_impl =
                    MappingFnImpl::from((Some(String::from("field_name")), |a: &u32, b: &u64| {
                        let a = u16::try_from(*a).ok()?;
                        let b = u16::try_from(*b).ok()?;
                        a.checked_add(b)
                    }));
                let resources = {
                    let mut resources = Resources::new();
                    resources.insert($value_resolution_mode(Some(1u32)));
                    // resources.insert($value_resolution_mode(Some(2u64)));
                    Resources::<SetUp>::from(resources)
                };
                let mut value_resolution_ctx = ValueResolutionCtx::new(
                    ValueResolutionMode::$value_resolution_mode,
                    item_id!("mapping_fn_map"),
                    String::from(crate::fn_name_short!()),
                );

                let sum = MappingFn::try_map(
                    &mapping_fn_impl,
                    &resources,
                    &mut value_resolution_ctx,
                )?;
                assert_eq!(None, sum);

                Ok(())
            }
        }
    };
}

use mapping_tests;
