use peace::{
    item_model::item_id,
    params::{
        AnySpecRt, AnySpecRtBoxed, ParamsFieldless, ParamsResolveError, ParamsSpecFieldless,
        ValueResolutionCtx, ValueResolutionMode, ValueSpecRt,
    },
    resource_rt::{resources::ts::SetUp, Resources},
};

use crate::mock_item::MockSrc;

#[test]
fn clone() {
    let _params_spec_fieldless =
        ParamsSpecFieldless::<MockSrc>::Value { value: MockSrc(1) }.clone();
}

#[test]
fn debug() {
    assert_eq!(
        "Stored",
        format!("{:?}", ParamsSpecFieldless::<MockSrc>::Stored)
    );
    assert_eq!(
        "Value(MockSrc(1))",
        format!(
            "{:?}",
            ParamsSpecFieldless::<MockSrc>::Value { value: MockSrc(1) }
        )
    );
    assert_eq!(
        "InMemory",
        format!("{:?}", ParamsSpecFieldless::<MockSrc>::InMemory)
    );
    assert_eq!(
        "MappingFn(MappingFnImpl { \
            field_name: Some(\"field\"), \
            fn_map: \"Some(Fn(&u8,) -> Option<MockSrc>)\", \
            marker: PhantomData<(workspace_tests::mock_item::MockSrc, (u8,))> \
        })",
        format!(
            "{:?}",
            ParamsSpecFieldless::<MockSrc>::from_map(
                Some(String::from("field")),
                #[cfg_attr(coverage_nightly, coverage(off))]
                |_: &u8| None
            )
        )
    );
}

#[test]
fn serialize_stored() -> Result<(), serde_yaml::Error> {
    let u8_spec: <u8 as ParamsFieldless>::Spec = <u8 as ParamsFieldless>::Spec::Stored;
    assert_eq!(
        r#"Stored
"#,
        serde_yaml::to_string(&u8_spec)?,
    );

    Ok(())
}

#[test]
fn serialize_value() -> Result<(), serde_yaml::Error> {
    let u8_spec: <u8 as ParamsFieldless>::Spec = 1u8.into();
    assert_eq!(
        r#"!Value
value: 1
"#,
        serde_yaml::to_string(&u8_spec)?,
    );

    Ok(())
}

#[test]
fn serialize_in_memory() -> Result<(), serde_yaml::Error> {
    let u8_spec: <u8 as ParamsFieldless>::Spec = ParamsSpecFieldless::<u8>::InMemory;
    assert_eq!(
        r#"InMemory
"#,
        serde_yaml::to_string(&u8_spec)?,
    );

    Ok(())
}

#[test]
fn serialize_from_map() -> Result<(), serde_yaml::Error> {
    let u8_spec: <u8 as ParamsFieldless>::Spec =
        ParamsSpecFieldless::<u8>::from_map(None, |_: &bool, _: &u16| Some(1u8));
    assert_eq!(
        r#"!MappingFn
field_name: null
fn_map: Some(Fn(&bool, &u16) -> Option<u8>)
marker: null
"#,
        serde_yaml::to_string(&u8_spec)?,
    );

    Ok(())
}

#[test]
fn deserialize_stored() -> Result<(), serde_yaml::Error> {
    assert!(matches!(
        serde_yaml::from_str(
            r#"Stored
        "#
        )?,
        ParamsSpecFieldless::<u8>::Stored
    ));

    Ok(())
}

#[test]
fn deserialize_value() -> Result<(), serde_yaml::Error> {
    assert!(matches!(
        serde_yaml::from_str(
            r#"!Value
value: 1
"#
        )?,
        ParamsSpecFieldless::<u8>::Value { value }
        if value == 1u8
    ));

    Ok(())
}

#[test]
fn deserialize_in_memory() -> Result<(), serde_yaml::Error> {
    let deserialized = serde_yaml::from_str(
        r#"InMemory
"#,
    )?;

    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(&deserialized, ParamsSpecFieldless::<u8>::InMemory),
                "was {deserialized:?}"
            );
        }
    })();

    Ok(())
}

#[test]
fn deserialize_from_map() -> Result<(), serde_yaml::Error> {
    let deserialized = serde_yaml::from_str(
        r#"!MappingFn
field_name: null
fn_map: Some(Fn(&bool, &u16) -> Option<Vec<u8>>)
marker: null
"#,
    )?;

    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    &deserialized,
                    ParamsSpecFieldless::<u8>::MappingFn(mapping_fn)
                    if !mapping_fn.is_valued()
                ),
                "was {deserialized:?}"
            );
        }
    })();

    Ok(())
}

#[test]
fn is_usable_returns_false_for_stored() {
    assert!(!ParamsSpecFieldless::<u8>::Stored.is_usable());
}

#[test]
fn is_usable_returns_true_for_value_and_in_memory() {
    assert!(ParamsSpecFieldless::<u8>::Value { value: 1u8 }.is_usable());
    assert!(ParamsSpecFieldless::<u8>::InMemory.is_usable());
}

#[test]
fn is_usable_returns_true_when_mapping_fn_is_some() {
    assert!(ParamsSpecFieldless::<u8>::from_map(None, |_: &u8| None).is_usable());
}

#[test]
fn is_usable_returns_false_when_mapping_fn_is_none() -> Result<(), serde_yaml::Error> {
    let params_spec: ParamsSpecFieldless<u8> = serde_yaml::from_str(
        r#"!MappingFn
field_name: null
fn_map: Some(Fn(&bool, &u16) -> Option<u8>)
marker: null
"#,
    )?;

    assert!(!params_spec.is_usable());
    Ok(())
}

#[test]
fn resolve_stored_param() -> Result<(), ParamsResolveError> {
    let resources = {
        let mut resources = Resources::new();
        resources.insert(MockSrc(1));
        Resources::<SetUp>::from(resources)
    };
    let mut value_resolution_ctx = ValueResolutionCtx::new(
        ValueResolutionMode::Current,
        item_id!("resolve_stored_param"),
        tynm::type_name::<MockSrc>(),
    );
    let mock_src_spec = ParamsSpecFieldless::<MockSrc>::Stored;

    let mock_src = ValueSpecRt::resolve(&mock_src_spec, &resources, &mut value_resolution_ctx)?;

    assert_eq!(MockSrc(1), mock_src);
    Ok(())
}

#[test]
fn resolve_in_memory() -> Result<(), ParamsResolveError> {
    let resources = {
        let mut resources = Resources::new();
        resources.insert(MockSrc(1));
        Resources::<SetUp>::from(resources)
    };
    let mut value_resolution_ctx = ValueResolutionCtx::new(
        ValueResolutionMode::Current,
        item_id!("resolve_in_memory"),
        tynm::type_name::<MockSrc>(),
    );
    let mock_src_spec = ParamsSpecFieldless::<MockSrc>::InMemory;

    let mock_src = ValueSpecRt::resolve(&mock_src_spec, &resources, &mut value_resolution_ctx)?;

    assert_eq!(MockSrc(1), mock_src);
    Ok(())
}

#[test]
fn resolve_in_memory_returns_err_when_not_found() -> Result<(), ParamsResolveError> {
    let resources = Resources::<SetUp>::from(Resources::new());
    let mut value_resolution_ctx = ValueResolutionCtx::new(
        ValueResolutionMode::Current,
        item_id!("resolve_in_memory_returns_err_when_not_found"),
        tynm::type_name::<MockSrc>(),
    );
    let mock_src_spec = ParamsSpecFieldless::<MockSrc>::InMemory;

    let mock_src_result =
        ValueSpecRt::resolve(&mock_src_spec, &resources, &mut value_resolution_ctx);

    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    &mock_src_result,
                    Err(ParamsResolveError::InMemory { value_resolution_ctx })
                    if value_resolution_ctx.value_resolution_mode() == ValueResolutionMode::Current
                    && value_resolution_ctx.item_id()
                        == &item_id!("resolve_in_memory_returns_err_when_not_found")
                    && value_resolution_ctx.params_type_name() == "MockSrc"
                    && value_resolution_ctx.resolution_chain().is_empty()
                ),
                "expected `mock_src_result` to be `Err(ParamsResolveError::InMemory {{ .. }})`\n\
                but was `{mock_src_result:?}`"
            );
        }
    })();
    Ok(())
}

#[test]
fn resolve_in_memory_returns_err_when_mutably_borrowed() -> Result<(), ParamsResolveError> {
    let resources = {
        let mut resources = Resources::new();
        resources.insert(MockSrc(1));
        Resources::<SetUp>::from(resources)
    };
    let mut value_resolution_ctx = ValueResolutionCtx::new(
        ValueResolutionMode::Current,
        item_id!("resolve_in_memory_returns_err_when_mutably_borrowed"),
        tynm::type_name::<MockSrc>(),
    );
    let mock_src_spec = ParamsSpecFieldless::<MockSrc>::InMemory;

    let _mock_src_mut_borrowed = resources.borrow_mut::<MockSrc>();
    let mock_src_result =
        ValueSpecRt::resolve(&mock_src_spec, &resources, &mut value_resolution_ctx);

    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    &mock_src_result,
                    Err(ParamsResolveError::InMemoryBorrowConflict { value_resolution_ctx })
                    if value_resolution_ctx.value_resolution_mode() == ValueResolutionMode::Current
                    && value_resolution_ctx.item_id()
                        == &item_id!("resolve_in_memory_returns_err_when_mutably_borrowed")
                    && value_resolution_ctx.params_type_name() == "MockSrc"
                    && value_resolution_ctx.resolution_chain().is_empty()
                ),
                "expected `mock_src_result` to be \
                `Err(ParamsResolveError::InMemoryBorrowConflict {{ .. }})`\n\
                with `resolution_chain`: `[]`,\n\
                but was `{mock_src_result:?}`"
            );
        }
    })();
    Ok(())
}

#[test]
fn resolve_value() -> Result<(), ParamsResolveError> {
    let resources = Resources::<SetUp>::from(Resources::new());
    let mut value_resolution_ctx = ValueResolutionCtx::new(
        ValueResolutionMode::Current,
        item_id!("resolve_value"),
        tynm::type_name::<MockSrc>(),
    );
    let mock_src_spec = ParamsSpecFieldless::<MockSrc>::Value { value: MockSrc(1) };

    let mock_src = ValueSpecRt::resolve(&mock_src_spec, &resources, &mut value_resolution_ctx)?;

    assert_eq!(MockSrc(1), mock_src);
    Ok(())
}

#[test]
fn resolve_mapping_fn() -> Result<(), ParamsResolveError> {
    let resources = {
        let mut resources = Resources::new();
        resources.insert(1u8);
        Resources::<SetUp>::from(resources)
    };
    let mut value_resolution_ctx = ValueResolutionCtx::new(
        ValueResolutionMode::Current,
        item_id!("resolve_mapping_fn"),
        tynm::type_name::<MockSrc>(),
    );
    let mock_src_spec = ParamsSpecFieldless::<MockSrc>::from_map(None, |n: &u8| Some(MockSrc(*n)));

    let mock_src = ValueSpecRt::resolve(&mock_src_spec, &resources, &mut value_resolution_ctx)?;

    assert_eq!(MockSrc(1), mock_src);
    Ok(())
}

#[test]
fn resolve_mapping_fn_returns_err_when_mutably_borrowed() -> Result<(), ParamsResolveError> {
    let resources = {
        let mut resources = Resources::new();
        resources.insert(1u8);
        resources.insert(2u16);
        Resources::<SetUp>::from(resources)
    };
    let mut value_resolution_ctx = ValueResolutionCtx::new(
        ValueResolutionMode::Current,
        item_id!("resolve_mapping_fn_returns_err_when_mutably_borrowed"),
        tynm::type_name::<MockSrc>(),
    );
    let mock_src_spec =
        ParamsSpecFieldless::<MockSrc>::from_map(None, |n: &u8, _m: &u16| Some(MockSrc(*n)));

    let _u8_borrowed = resources.borrow::<u8>();
    let _u16_mut_borrowed = resources.borrow_mut::<u16>();
    let mock_src_result =
        ValueSpecRt::resolve(&mock_src_spec, &resources, &mut value_resolution_ctx);

    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    &mock_src_result,
                    Err(ParamsResolveError::FromMapBorrowConflict { value_resolution_ctx, from_type_name })
                    if value_resolution_ctx.value_resolution_mode() == ValueResolutionMode::Current
                    && value_resolution_ctx.item_id()
                        == &item_id!("resolve_mapping_fn_returns_err_when_mutably_borrowed")
                    && value_resolution_ctx.params_type_name() == "MockSrc"
                    && value_resolution_ctx.resolution_chain().is_empty()
                    && from_type_name == "u16"
                ),
                "expected `mock_src_result` to be \
                `Err(ParamsResolveError::FromMapBorrowConflict {{ .. }})`\n\
                with `resolution_chain`: `[]`,\n\
                but was `{mock_src_result:?}`"
            );
        }
    })();
    Ok(())
}

#[test]
fn try_resolve_stored_param() -> Result<(), ParamsResolveError> {
    let resources = {
        let mut resources = Resources::new();
        resources.insert(MockSrc(1));
        Resources::<SetUp>::from(resources)
    };
    let mut value_resolution_ctx = ValueResolutionCtx::new(
        ValueResolutionMode::Current,
        item_id!("try_resolve_stored_param"),
        tynm::type_name::<MockSrc>(),
    );
    let mock_src_spec = ParamsSpecFieldless::<MockSrc>::Stored;

    let mock_src = ValueSpecRt::try_resolve(&mock_src_spec, &resources, &mut value_resolution_ctx)?;

    assert_eq!(Some(MockSrc(1)), mock_src);
    Ok(())
}

#[test]
fn try_resolve_in_memory() -> Result<(), ParamsResolveError> {
    let resources = {
        let mut resources = Resources::new();
        resources.insert(MockSrc(1));
        Resources::<SetUp>::from(resources)
    };
    let mut value_resolution_ctx = ValueResolutionCtx::new(
        ValueResolutionMode::Current,
        item_id!("try_resolve_in_memory"),
        tynm::type_name::<MockSrc>(),
    );
    let mock_src_spec = ParamsSpecFieldless::<MockSrc>::InMemory;

    let mock_src = ValueSpecRt::try_resolve(&mock_src_spec, &resources, &mut value_resolution_ctx)?;

    assert_eq!(Some(MockSrc(1)), mock_src);
    Ok(())
}

#[test]
fn try_resolve_in_memory_returns_none_when_not_found() -> Result<(), ParamsResolveError> {
    let resources = Resources::<SetUp>::from(Resources::new());
    let mut value_resolution_ctx = ValueResolutionCtx::new(
        ValueResolutionMode::Current,
        item_id!("try_resolve_in_memory_returns_none_when_not_found"),
        tynm::type_name::<MockSrc>(),
    );
    let mock_src_spec = ParamsSpecFieldless::<MockSrc>::InMemory;

    let mock_src = ValueSpecRt::try_resolve(&mock_src_spec, &resources, &mut value_resolution_ctx)?;

    assert_eq!(None, mock_src);
    Ok(())
}

#[test]
fn try_resolve_in_memory_returns_err_when_mutably_borrowed() -> Result<(), ParamsResolveError> {
    let resources = {
        let mut resources = Resources::new();
        resources.insert(MockSrc(1));
        Resources::<SetUp>::from(resources)
    };
    let mut value_resolution_ctx = ValueResolutionCtx::new(
        ValueResolutionMode::Current,
        item_id!("try_resolve_in_memory_returns_err_when_mutably_borrowed"),
        tynm::type_name::<MockSrc>(),
    );
    let mock_src_spec = ParamsSpecFieldless::<MockSrc>::InMemory;

    let _mock_src_mut_borrowed = resources.borrow_mut::<MockSrc>();
    let mock_src_result =
        ValueSpecRt::try_resolve(&mock_src_spec, &resources, &mut value_resolution_ctx);

    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    &mock_src_result,
                    Err(ParamsResolveError::InMemoryBorrowConflict { value_resolution_ctx })
                    if value_resolution_ctx.value_resolution_mode() == ValueResolutionMode::Current
                    && value_resolution_ctx.item_id()
                        == &item_id!("try_resolve_in_memory_returns_err_when_mutably_borrowed")
                    && value_resolution_ctx.params_type_name() == "MockSrc"
                    && value_resolution_ctx.resolution_chain().is_empty()
                ),
                "expected `mock_src_result` to be \
                `Err(ParamsResolveError::InMemoryBorrowConflict {{ .. }})`\n\
                with `resolution_chain`: `[]`,\n\
                but was `{mock_src_result:?}`"
            );
        }
    })();
    Ok(())
}

#[test]
fn try_resolve_value() -> Result<(), ParamsResolveError> {
    let resources = Resources::<SetUp>::from(Resources::new());
    let mut value_resolution_ctx = ValueResolutionCtx::new(
        ValueResolutionMode::Current,
        item_id!("try_resolve_value"),
        tynm::type_name::<MockSrc>(),
    );
    let mock_src_spec = ParamsSpecFieldless::<MockSrc>::Value { value: MockSrc(1) };

    let mock_src = ValueSpecRt::try_resolve(&mock_src_spec, &resources, &mut value_resolution_ctx)?;

    assert_eq!(Some(MockSrc(1)), mock_src);
    Ok(())
}

#[test]
fn try_resolve_mapping_fn() -> Result<(), ParamsResolveError> {
    let resources = {
        let mut resources = Resources::new();
        resources.insert(1u8);
        Resources::<SetUp>::from(resources)
    };
    let mut value_resolution_ctx = ValueResolutionCtx::new(
        ValueResolutionMode::Current,
        item_id!("try_resolve_mapping_fn"),
        tynm::type_name::<MockSrc>(),
    );
    let mock_src_spec = ParamsSpecFieldless::<MockSrc>::from_map(None, |n: &u8| Some(MockSrc(*n)));

    let mock_src = ValueSpecRt::try_resolve(&mock_src_spec, &resources, &mut value_resolution_ctx)?;

    assert_eq!(Some(MockSrc(1)), mock_src);
    Ok(())
}

#[test]
fn try_resolve_mapping_fn_returns_err_when_mutably_borrowed() -> Result<(), ParamsResolveError> {
    let resources = {
        let mut resources = Resources::new();
        resources.insert(1u8);
        resources.insert(2u16);
        Resources::<SetUp>::from(resources)
    };
    let mut value_resolution_ctx = ValueResolutionCtx::new(
        ValueResolutionMode::Current,
        item_id!("try_resolve_mapping_fn_returns_err_when_mutably_borrowed"),
        tynm::type_name::<MockSrc>(),
    );
    let mock_src_spec =
        ParamsSpecFieldless::<MockSrc>::from_map(None, |n: &u8, _m: &u16| Some(MockSrc(*n)));

    let _u8_borrowed = resources.borrow::<u8>();
    let _u16_mut_borrowed = resources.borrow_mut::<u16>();
    let mock_src_result =
        ValueSpecRt::try_resolve(&mock_src_spec, &resources, &mut value_resolution_ctx);

    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    &mock_src_result,
                    Err(ParamsResolveError::FromMapBorrowConflict { value_resolution_ctx, from_type_name })
                    if value_resolution_ctx.value_resolution_mode() == ValueResolutionMode::Current
                    && value_resolution_ctx.item_id()
                        == &item_id!("try_resolve_mapping_fn_returns_err_when_mutably_borrowed")
                    && value_resolution_ctx.params_type_name() == "MockSrc"
                    && value_resolution_ctx.resolution_chain().is_empty()
                    && from_type_name == "u16"
                ),
                "expected `mock_src_result` to be \
                `Err(ParamsResolveError::FromMapBorrowConflict {{ .. }})`\n\
                with `resolution_chain`: `[]`,\n\
                but was `{mock_src_result:?}`"
            );
        }
    })();
    Ok(())
}

#[test]
fn merge_stored_with_other_uses_other() {
    let mut params_spec_fieldless_a = ParamsSpecFieldless::<MockSrc>::Stored;
    let params_spec_fieldless_b = AnySpecRtBoxed::new(ParamsSpecFieldless::<MockSrc>::InMemory);

    params_spec_fieldless_a.merge(&*params_spec_fieldless_b);

    assert!(matches!(
        &params_spec_fieldless_a,
        ParamsSpecFieldless::<MockSrc>::InMemory
    ));
}

#[test]
fn merge_value_with_other_no_change() {
    let mut params_spec_fieldless_a = ParamsSpecFieldless::<MockSrc>::Value { value: MockSrc(1) };
    let params_spec_fieldless_b = AnySpecRtBoxed::new(ParamsSpecFieldless::<MockSrc>::from_map(
        None,
        #[cfg_attr(coverage_nightly, coverage(off))]
        |_: &u8| None,
    ));

    params_spec_fieldless_a.merge(&*params_spec_fieldless_b);

    assert!(
        matches!(&params_spec_fieldless_a, ParamsSpecFieldless::<MockSrc>::Value { value } if value == &MockSrc(1))
    );
}

#[test]
fn merge_in_memory_with_other_no_change() {
    let mut params_spec_fieldless_a = ParamsSpecFieldless::<MockSrc>::InMemory;
    let params_spec_fieldless_b = AnySpecRtBoxed::new(ParamsSpecFieldless::<MockSrc>::from_map(
        None,
        #[cfg_attr(coverage_nightly, coverage(off))]
        |_: &u8| None,
    ));

    params_spec_fieldless_a.merge(&*params_spec_fieldless_b);

    assert!(matches!(
        &params_spec_fieldless_a,
        ParamsSpecFieldless::<MockSrc>::InMemory
    ));
}

#[test]
fn merge_mapping_fn_with_other_no_change() {
    let mut params_spec_fieldless_a = ParamsSpecFieldless::<MockSrc>::from_map(
        None,
        #[cfg_attr(coverage_nightly, coverage(off))]
        |_: &u8| None,
    );
    let params_spec_fieldless_b = AnySpecRtBoxed::new(ParamsSpecFieldless::<MockSrc>::InMemory);

    params_spec_fieldless_a.merge(&*params_spec_fieldless_b);

    assert!(matches!(
        &params_spec_fieldless_a,
        ParamsSpecFieldless::<MockSrc>::MappingFn(_)
    ));
}
