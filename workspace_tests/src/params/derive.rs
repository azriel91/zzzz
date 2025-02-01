mod unit {
    use std::any::TypeId;

    use serde::{Deserialize, Serialize};

    use peace::params::{Params, ParamsSpec};

    #[derive(Clone, Debug, Params, Serialize, Deserialize)]
    pub struct UnitParams;

    super::params_tests!(UnitParams, UnitParamsFieldWise, UnitParamsPartial, []);

    #[test]
    fn spec_from_params() {
        let params = UnitParams;

        assert!(matches!(
            ParamsSpec::from(params),
            ParamsSpec::Value { value: UnitParams }
        ));
    }

    #[test]
    fn field_wise_from_params() {
        let params = UnitParams;

        assert!(matches!(
            UnitParamsFieldWise::from(params),
            UnitParamsFieldWise
        ));
    }

    #[test]
    fn spec_debug() {
        assert_eq!(
            r#"UnitParamsFieldWise"#,
            format!("{:?}", UnitParamsFieldWise)
        );
    }

    #[test]
    fn params_partial_debug() {
        assert_eq!(r#"UnitParamsPartial"#, format!("{:?}", UnitParamsPartial));
    }

    #[test]
    fn params_try_from_partial_returns_ok() {
        let params_partial = UnitParamsPartial;

        assert!(matches!(
            UnitParams::try_from(params_partial),
            Ok(UnitParams)
        ));
    }

    #[test]
    fn params_try_from_partial_ref_returns_ok() {
        let params_partial = UnitParamsPartial;

        assert!(matches!(
            UnitParams::try_from(&params_partial),
            Ok(UnitParams)
        ));
    }
}

mod struct_params {
    use std::any::TypeId;

    use serde::{Deserialize, Serialize};

    use peace::{
        item_model::item_id,
        params::{Params, ParamsSpec, ValueResolutionCtx, ValueResolutionMode, ValueSpec},
        resource_rt::{resources::ts::SetUp, Resources},
    };

    #[derive(Clone, Debug, Params, Serialize, Deserialize)]
    pub struct StructParams {
        /// Source / goal value for the state.
        src: String,
        /// Destination storage for the state.
        dest: String,
    }

    super::params_tests!(StructParams, StructParamsFieldWise, StructParamsPartial, []);

    #[test]
    fn spec_from_params() {
        let params = StructParams {
            src: String::from("a"),
            dest: String::from("b"),
        };

        assert!(matches!(
            ParamsSpec::from(params),
            ParamsSpec::Value {
                value: StructParams {
                    src,
                    dest,
                }
            }
            if src == "a"
            && dest == "b"
        ));
    }

    #[test]
    fn field_wise_from_params() {
        let params = StructParams {
            src: String::from("a"),
            dest: String::from("b"),
        };

        assert!(matches!(
            StructParamsFieldWise::from(params),
            StructParamsFieldWise {
                src: ValueSpec::Value { value: src_value },
                dest: ValueSpec::Value { value: dest_value },
            }
            if src_value == "a"
            && dest_value == "b"
        ));
    }

    #[test]
    fn field_wise_from_field_wise_builder() {
        let field_wise = StructParams::field_wise_spec()
            .with_src(String::from("a"))
            .with_dest_from_map(|_: &u32| Some(String::from("b")))
            .build();
        let resources: Resources<SetUp> = {
            let mut resources = Resources::new();
            resources.insert(1u32);
            Resources::from(resources)
        };
        let mut value_resolution_ctx = ValueResolutionCtx::new(
            ValueResolutionMode::ApplyDry,
            item_id!("field_wise_from_field_wise_builder"),
            String::from("StructParams"),
        );

        assert!(matches!(
            field_wise,
            ParamsSpec::FieldWise {
                field_wise_spec: StructParamsFieldWise {
                    src: ValueSpec::Value { value: src_value },
                    dest: ValueSpec::MappingFn(mapping_fn),
                }
            }
            if src_value == "a"
            && matches!(
                mapping_fn.map(&resources, &mut value_resolution_ctx),
                Ok(dest_mapped)
                if dest_mapped == "b"
            )
        ));
    }

    #[test]
    fn spec_debug() {
        assert_eq!(
            r#"StructParamsFieldWise { src: Value("a"), dest: Value("b") }"#,
            format!(
                "{:?}",
                StructParamsFieldWise {
                    src: ValueSpec::Value {
                        value: String::from("a")
                    },
                    dest: ValueSpec::Value {
                        value: String::from("b")
                    },
                }
            )
        );
    }

    #[test]
    fn params_partial_debug() {
        assert_eq!(
            r#"StructParamsPartial { src: Some("a"), dest: Some("b") }"#,
            format!(
                "{:?}",
                StructParamsPartial {
                    src: Some(String::from("a")),
                    dest: Some(String::from("b")),
                }
            )
        );
    }

    #[test]
    fn params_try_from_partial_returns_ok_when_all_fields_are_some() {
        let params_partial = StructParamsPartial {
            src: Some(String::from("a")),
            dest: Some(String::from("b")),
        };

        assert!(matches!(
            StructParams::try_from(params_partial),
            Ok(StructParams {
                src,
                dest,
            })
            if src == "a"
            && dest == "b"
        ));
    }

    #[test]
    fn params_try_from_partial_returns_err_when_some_fields_are_none() {
        let params_partial = StructParamsPartial {
            src: Some(String::from("a")),
            dest: None,
        };

        assert!(matches!(
            StructParams::try_from(params_partial),
            Err(StructParamsPartial {
                src,
                dest,
            })
            if src == Some(String::from("a"))
            && dest.is_none()
        ));
    }

    #[test]
    fn params_try_from_partial_ref_returns_ok_when_all_fields_are_some() {
        let params_partial = StructParamsPartial {
            src: Some(String::from("a")),
            dest: Some(String::from("b")),
        };

        assert!(matches!(
            StructParams::try_from(&params_partial),
            Ok(StructParams {
                src,
                dest,
            })
            if src == "a"
            && dest == "b"
        ));
    }

    #[test]
    fn params_try_from_partial_ref_returns_err_when_some_fields_are_none() {
        let params_partial = StructParamsPartial {
            src: Some(String::from("a")),
            dest: None,
        };

        assert!(matches!(
            StructParams::try_from(&params_partial),
            Err(StructParamsPartial {
                src,
                dest,
            })
            if src == &Some(String::from("a"))
            && dest.is_none()
        ));
    }
}

mod struct_with_type_params {
    use std::{any::TypeId, marker::PhantomData};

    use derivative::Derivative;
    use serde::{Deserialize, Serialize};

    use peace::{
        item_model::item_id,
        params::{Params, ParamsSpec, ValueResolutionCtx, ValueResolutionMode, ValueSpec},
        resource_rt::{resources::ts::SetUp, Resources},
    };

    #[derive(Derivative, Params, Serialize, Deserialize)]
    #[derivative(Clone, Debug)]
    pub struct StructWithTypeParams<Id> {
        /// Source / goal value for the state.
        src: String,
        /// Destination storage for the state.
        dest: String,
        /// Marker for unique parameters type.
        marker: PhantomData<Id>,
    }

    super::params_tests!(
        StructWithTypeParams,
        StructWithTypeParamsFieldWise,
        StructWithTypeParamsPartial,
        [<()>]
    );

    #[test]
    fn spec_from_params() {
        let params = StructWithTypeParams::<()> {
            src: String::from("a"),
            dest: String::from("b"),
            marker: PhantomData,
        };

        assert!(matches!(
            ParamsSpec::from(params),
            ParamsSpec::Value {
                value: StructWithTypeParams {
                    src,
                    dest,
                    marker: PhantomData,
                }
            }
            if src == "a"
            && dest == "b"
        ));
    }

    #[test]
    fn field_wise_from_params() {
        let params = StructWithTypeParams::<()> {
            src: String::from("a"),
            dest: String::from("b"),
            marker: PhantomData,
        };

        assert!(matches!(
            StructWithTypeParamsFieldWise::from(params),
            StructWithTypeParamsFieldWise {
                src: ValueSpec::Value { value: src_value },
                dest: ValueSpec::Value { value: dest_value },
                marker: PhantomData,
            }
            if src_value == "a"
            && dest_value == "b"
        ));
    }

    #[test]
    fn field_wise_from_field_wise_builder() {
        let field_wise = StructWithTypeParams::<()>::field_wise_spec()
            .with_src_in_memory()
            .with_dest_from_map(|_: &u32| Some(String::from("b")))
            .build();
        let resources: Resources<SetUp> = {
            let mut resources = Resources::new();
            resources.insert(1u32);
            Resources::from(resources)
        };
        let mut value_resolution_ctx = ValueResolutionCtx::new(
            ValueResolutionMode::ApplyDry,
            item_id!("field_wise_from_field_wise_builder"),
            String::from("StructWithTypeParams<()>"),
        );

        assert!(matches!(
            field_wise,
            ParamsSpec::FieldWise {
                field_wise_spec: StructWithTypeParamsFieldWise {
                    src: ValueSpec::InMemory,
                    dest: ValueSpec::MappingFn(mapping_fn),
                    marker: PhantomData,
                }
            }
            if matches!(
                mapping_fn.map(&resources, &mut value_resolution_ctx),
                Ok(dest_mapped)
                if dest_mapped == "b"
            )
        ));
    }

    #[test]
    fn spec_debug() {
        assert_eq!(
            r#"StructWithTypeParamsFieldWise { src: Value("a"), dest: Value("b"), marker: PhantomData<()> }"#,
            format!(
                "{:?}",
                StructWithTypeParamsFieldWise::<()> {
                    src: ValueSpec::Value {
                        value: String::from("a")
                    },
                    dest: ValueSpec::Value {
                        value: String::from("b")
                    },
                    marker: PhantomData,
                }
            )
        );
    }

    #[test]
    fn params_partial_debug() {
        assert_eq!(
            r#"StructWithTypeParamsPartial { src: Some("a"), dest: Some("b"), marker: PhantomData<()> }"#,
            format!(
                "{:?}",
                StructWithTypeParamsPartial::<()> {
                    src: Some(String::from("a")),
                    dest: Some(String::from("b")),
                    marker: PhantomData,
                }
            )
        );
    }

    #[test]
    fn params_try_from_partial_returns_ok_when_all_fields_are_some() {
        let params_partial = StructWithTypeParamsPartial::<()> {
            src: Some(String::from("a")),
            dest: Some(String::from("b")),
            marker: PhantomData,
        };

        assert!(matches!(
            StructWithTypeParams::try_from(params_partial),
            Ok(StructWithTypeParams {
                src,
                dest,
                marker: PhantomData,
            })
            if src == "a"
            && dest == "b"
        ));
    }

    #[test]
    fn params_try_from_partial_returns_err_when_some_fields_are_none() {
        let params_partial = StructWithTypeParamsPartial::<()> {
            src: Some(String::from("a")),
            dest: None,
            marker: PhantomData,
        };

        assert!(matches!(
            StructWithTypeParams::try_from(params_partial),
            Err(StructWithTypeParamsPartial {
                src,
                dest,
                marker: PhantomData,
            })
            if src == Some(String::from("a"))
            && dest.is_none()
        ));
    }

    #[test]
    fn params_try_from_partial_ref_returns_ok_when_all_fields_are_some() {
        let params_partial = StructWithTypeParamsPartial::<()> {
            src: Some(String::from("a")),
            dest: Some(String::from("b")),
            marker: PhantomData,
        };

        assert!(matches!(
            StructWithTypeParams::try_from(&params_partial),
            Ok(StructWithTypeParams {
                src,
                dest,
                marker: PhantomData,
            })
            if src == "a"
            && dest == "b"
        ));
    }

    #[test]
    fn params_try_from_partial_ref_returns_err_when_some_fields_are_none() {
        let params_partial = StructWithTypeParamsPartial::<()> {
            src: Some(String::from("a")),
            dest: None,
            marker: PhantomData,
        };

        assert!(matches!(
            StructWithTypeParams::try_from(&params_partial),
            Err(StructWithTypeParamsPartial {
                src,
                dest,
                marker: PhantomData,
            })
            if src == &Some(String::from("a"))
            && dest.is_none()
        ));
    }
}

mod tuple_params {
    use std::any::TypeId;

    use serde::{Deserialize, Serialize};

    use peace::{
        item_model::item_id,
        params::{Params, ParamsSpec, ValueResolutionCtx, ValueResolutionMode, ValueSpec},
        resource_rt::{resources::ts::SetUp, Resources},
    };

    #[derive(Clone, Debug, Params, Serialize, Deserialize)]
    pub struct TupleParams(
        /// Source / goal value for the state.
        String,
        /// Destination storage for the state.
        String,
    );

    super::params_tests!(TupleParams, TupleParamsFieldWise, TupleParamsPartial, []);

    #[test]
    fn spec_from_params() {
        let params = TupleParams(String::from("a"), String::from("b"));

        assert!(matches!(
            ParamsSpec::from(params),
            ParamsSpec::Value {
                value: TupleParams (
                    src,
                    dest,
                )
            }
            if src == "a"
            && dest == "b"
        ));
    }

    #[test]
    fn field_wise_from_params() {
        let params = TupleParams(String::from("a"), String::from("b"));

        assert!(matches!(
            TupleParamsFieldWise::from(params),
            TupleParamsFieldWise (
                ValueSpec::Value { value: src_value },
                ValueSpec::Value { value: dest_value },
            )
            if src_value == "a"
            && dest_value == "b"
        ));
    }

    #[test]
    fn field_wise_from_field_wise_builder() {
        let field_wise = TupleParams::field_wise_spec()
            .with_0_in_memory()
            .with_1_from_map(|_: &u32| Some(String::from("b")))
            .build();
        let resources: Resources<SetUp> = {
            let mut resources = Resources::new();
            resources.insert(1u32);
            Resources::from(resources)
        };
        let mut value_resolution_ctx = ValueResolutionCtx::new(
            ValueResolutionMode::ApplyDry,
            item_id!("field_wise_from_field_wise_builder"),
            String::from("TupleParams"),
        );

        assert!(matches!(
            field_wise,
            ParamsSpec::FieldWise {
                field_wise_spec: TupleParamsFieldWise(
                    ValueSpec::InMemory,
                    ValueSpec::MappingFn(mapping_fn),
                )
            }
            if matches!(
                mapping_fn.map(&resources, &mut value_resolution_ctx),
                Ok(dest_mapped)
                if dest_mapped == "b"
            )
        ));
    }

    #[test]
    fn spec_debug() {
        assert_eq!(
            r#"TupleParamsFieldWise(Value("a"), Value("b"))"#,
            format!(
                "{:?}",
                TupleParamsFieldWise(
                    ValueSpec::Value {
                        value: String::from("a")
                    },
                    ValueSpec::Value {
                        value: String::from("b")
                    },
                )
            )
        );
    }

    #[test]
    fn params_partial_debug() {
        assert_eq!(
            r#"TupleParamsPartial(Some("a"), Some("b"))"#,
            format!(
                "{:?}",
                TupleParamsPartial(Some(String::from("a")), Some(String::from("b")),)
            )
        );
    }

    #[test]
    fn params_try_from_partial_returns_ok_when_all_fields_are_some() {
        let params_partial = TupleParamsPartial(Some(String::from("a")), Some(String::from("b")));

        assert!(matches!(
            TupleParams::try_from(params_partial),
            Ok(TupleParams (
                src,
                dest,
            ))
            if src == "a"
            && dest == "b"
        ));
    }

    #[test]
    fn params_try_from_partial_returns_err_when_some_fields_are_none() {
        let params_partial = TupleParamsPartial(Some(String::from("a")), None);

        assert!(matches!(
            TupleParams::try_from(params_partial),
            Err(TupleParamsPartial (
                src,
                dest,
            ))
            if src == Some(String::from("a"))
            && dest.is_none()
        ));
    }

    #[test]
    fn params_try_from_partial_ref_returns_ok_when_all_fields_are_some() {
        let params_partial = TupleParamsPartial(Some(String::from("a")), Some(String::from("b")));

        assert!(matches!(
            TupleParams::try_from(&params_partial),
            Ok(TupleParams (
                src,
                dest,
            ))
            if src == "a"
            && dest == "b"
        ));
    }

    #[test]
    fn params_try_from_partial_ref_returns_err_when_some_fields_are_none() {
        let params_partial = TupleParamsPartial(Some(String::from("a")), None);

        assert!(matches!(
            TupleParams::try_from(&params_partial),
            Err(TupleParamsPartial (
                src,
                dest,
            ))
            if src == &Some(String::from("a"))
            && dest.is_none()
        ));
    }
}

mod tuple_with_type_params {
    use std::{any::TypeId, fmt::Debug, marker::PhantomData};

    use serde::{Deserialize, Serialize};

    use peace::{
        item_model::item_id,
        params::{Params, ParamsSpec, ValueResolutionCtx, ValueResolutionMode, ValueSpec},
        resource_rt::{resources::ts::SetUp, Resources},
    };

    #[derive(Clone, Debug, Params, Serialize, Deserialize)]
    pub struct TupleWithTypeParams<Id>(String, String, PhantomData<Id>)
    where
        Id: Clone + Debug;

    super::params_tests!(
        TupleWithTypeParams,
        TupleWithTypeParamsFieldWise,
        TupleWithTypeParamsPartial,
        [<()>]
    );

    #[test]
    fn spec_from_params() {
        let params = TupleWithTypeParams::<()>(String::from("a"), String::from("b"), PhantomData);

        assert!(matches!(
            ParamsSpec::from(params),
            ParamsSpec::Value {
                value: TupleWithTypeParams (
                    src,
                    dest,
                    PhantomData,
                )
            }
            if src == "a"
            && dest == "b"
        ));
    }

    #[test]
    fn field_wise_from_params() {
        let params = TupleWithTypeParams::<()>(String::from("a"), String::from("b"), PhantomData);

        assert!(matches!(
            TupleWithTypeParamsFieldWise::from(params),
            TupleWithTypeParamsFieldWise::<()>(
                ValueSpec::Value { value: src_value },
                ValueSpec::Value { value: dest_value },
                PhantomData,
            )
            if src_value == "a"
            && dest_value == "b"
        ));
    }

    #[test]
    fn field_wise_from_field_wise_builder() {
        let field_wise = TupleWithTypeParams::<()>::field_wise_spec()
            .with_0_in_memory()
            .with_1_from_map(|_: &u32| Some(String::from("b")))
            .build();
        let resources: Resources<SetUp> = {
            let mut resources = Resources::new();
            resources.insert(1u32);
            Resources::from(resources)
        };
        let mut value_resolution_ctx = ValueResolutionCtx::new(
            ValueResolutionMode::ApplyDry,
            item_id!("field_wise_from_field_wise_builder"),
            String::from("TupleWithTypeParams<()>"),
        );

        assert!(matches!(
            field_wise,
            ParamsSpec::FieldWise {
                field_wise_spec: TupleWithTypeParamsFieldWise(
                    ValueSpec::InMemory,
                    ValueSpec::MappingFn(mapping_fn),
                    PhantomData,
                )
            }
            if matches!(
                mapping_fn.map(&resources, &mut value_resolution_ctx),
                Ok(dest_mapped)
                if dest_mapped == "b"
            )
        ));
    }

    #[test]
    fn spec_debug() {
        assert_eq!(
            r#"TupleWithTypeParamsFieldWise(Value("a"), Value("b"), PhantomData<()>)"#,
            format!(
                "{:?}",
                TupleWithTypeParamsFieldWise::<()>(
                    ValueSpec::Value {
                        value: String::from("a")
                    },
                    ValueSpec::Value {
                        value: String::from("b")
                    },
                    PhantomData,
                )
            )
        );
    }

    #[test]
    fn params_partial_debug() {
        assert_eq!(
            r#"TupleWithTypeParamsPartial(Some("a"), Some("b"), PhantomData<()>)"#,
            format!(
                "{:?}",
                TupleWithTypeParamsPartial::<()>(
                    Some(String::from("a")),
                    Some(String::from("b")),
                    PhantomData,
                )
            )
        );
    }

    #[test]
    fn params_try_from_partial_returns_ok_when_all_fields_are_some() {
        let params_partial = TupleWithTypeParamsPartial::<()>(
            Some(String::from("a")),
            Some(String::from("b")),
            PhantomData,
        );

        assert!(matches!(
            TupleWithTypeParams::try_from(params_partial),
            Ok(TupleWithTypeParams::<()> (
                src,
                dest,
                PhantomData,
            ))
            if src == "a"
            && dest == "b"
        ));
    }

    #[test]
    fn params_try_from_partial_returns_err_when_some_fields_are_none() {
        let params_partial =
            TupleWithTypeParamsPartial::<()>(Some(String::from("a")), None, PhantomData);

        assert!(matches!(
            TupleWithTypeParams::try_from(params_partial),
            Err(TupleWithTypeParamsPartial::<()> (
                src,
                dest,
                PhantomData,
            ))
            if src == Some(String::from("a"))
            && dest.is_none()
        ));
    }

    #[test]
    fn params_try_from_partial_ref_returns_ok_when_all_fields_are_some() {
        let params_partial = TupleWithTypeParamsPartial::<()>(
            Some(String::from("a")),
            Some(String::from("b")),
            PhantomData,
        );

        assert!(matches!(
            TupleWithTypeParams::try_from(&params_partial),
            Ok(TupleWithTypeParams::<()> (
                src,
                dest,
                PhantomData,
            ))
            if src == "a"
            && dest == "b"
        ));
    }

    #[test]
    fn params_try_from_partial_ref_returns_err_when_some_fields_are_none() {
        let params_partial =
            TupleWithTypeParamsPartial::<()>(Some(String::from("a")), None, PhantomData);

        assert!(matches!(
            TupleWithTypeParams::try_from(&params_partial),
            Err(TupleWithTypeParamsPartial::<()> (
                src,
                dest,
                PhantomData,
            ))
            if src == &Some(String::from("a"))
            && dest.is_none()
        ));
    }
}

mod enum_params {
    use std::{any::TypeId, marker::PhantomData};

    use derivative::Derivative;
    use serde::{Deserialize, Serialize};

    use peace::{
        item_model::item_id,
        params::{Params, ParamsSpec, ValueResolutionCtx, ValueResolutionMode, ValueSpec},
        resource_rt::{resources::ts::SetUp, Resources},
    };

    #[derive(Derivative, Params, Serialize, Deserialize)]
    #[derivative(Clone, Debug)]
    pub enum EnumParams<Id> {
        Named {
            /// Source / goal value for the state.
            src: String,
            /// Marker for unique parameters type.
            marker: PhantomData<Id>,
        },
        Tuple(String),
        TupleMarker(String, PhantomData<Id>),
        Unit,
    }

    super::params_tests!(
        EnumParams,
        EnumParamsFieldWise,
        EnumParamsPartial,
        [<()>]
    );

    #[test]
    fn spec_named_from_params() {
        let params = EnumParams::<()>::Named {
            src: String::from("a"),
            marker: PhantomData,
        };

        assert!(matches!(
            ParamsSpec::from(params),
            ParamsSpec::Value {
                value: EnumParams::Named {
                    src,
                    marker: PhantomData,
                }
            }
            if src == "a"
        ));
    }

    #[test]
    fn spec_tuple_from_params() {
        let params = EnumParams::<()>::Tuple(String::from("a"));

        assert!(matches!(
            ParamsSpec::from(params),
            ParamsSpec::Value {
                    value: EnumParams::Tuple(src)}

            if src == "a"
        ));
    }

    #[test]
    fn spec_tuple_marker_from_params() {
        let params = EnumParams::<()>::TupleMarker(String::from("a"), PhantomData);

        assert!(matches!(
            ParamsSpec::from(params),
            ParamsSpec::Value {
                    value: EnumParams::TupleMarker(src, PhantomData)}

            if src == "a"
        ));
    }

    #[test]
    fn spec_unit_from_params() {
        let params = EnumParams::<()>::Unit;

        assert!(matches!(
            ParamsSpec::from(params),
            ParamsSpec::Value {
                value: EnumParams::Unit
            }
        ));
    }

    #[test]
    fn field_wise_named_from_params() {
        let params = EnumParams::<()>::Named {
            src: String::from("a"),
            marker: PhantomData,
        };

        assert!(matches!(
            EnumParamsFieldWise::from(params),
            EnumParamsFieldWise::<()>::Named {
                src: ValueSpec::Value { value },
                marker: PhantomData,
            }
            if value == "a"
        ));
    }

    #[test]
    fn field_wise_tuple_from_params() {
        let params = EnumParams::<()>::Tuple(String::from("a"));

        assert!(matches!(
            EnumParamsFieldWise::from(params),
            EnumParamsFieldWise::<()>::Tuple(ValueSpec::Value { value })
            if value == "a"
        ));
    }

    #[test]
    fn field_wise_tuple_marker_from_params() {
        let params = EnumParams::<()>::TupleMarker(String::from("a"), PhantomData);

        assert!(matches!(
            EnumParamsFieldWise::from(params),
            EnumParamsFieldWise::<()>::TupleMarker(ValueSpec::Value { value }, PhantomData)
            if value == "a"
        ));
    }

    #[test]
    fn field_wise_unit_from_params() {
        let params = EnumParams::<()>::Unit;

        assert!(matches!(
            EnumParamsFieldWise::from(params),
            EnumParamsFieldWise::<()>::Unit
        ));
    }

    #[test]
    fn field_wise_named_from_field_wise_builder() {
        let field_wise = EnumParams::<()>::field_wise_spec()
            .named()
            .with_src(String::from("a"))
            .with_src_in_memory()
            .with_src_from_map(|_: &u32| Some(String::from("b")))
            .build();
        let resources: Resources<SetUp> = {
            let mut resources = Resources::new();
            resources.insert(1u32);
            Resources::from(resources)
        };
        let mut value_resolution_ctx = ValueResolutionCtx::new(
            ValueResolutionMode::ApplyDry,
            item_id!("field_wise_named_from_field_wise_builder"),
            String::from("EnumParams<()>"),
        );

        assert!(matches!(
            field_wise,
            ParamsSpec::FieldWise {
                field_wise_spec: EnumParamsFieldWise::Named {
                    src: ValueSpec::MappingFn(mapping_fn),
                    marker: PhantomData,
                }
            }
            if matches!(
                mapping_fn.map(&resources, &mut value_resolution_ctx),
                Ok(src_mapped)
                if src_mapped == "b"
            )
        ));
    }

    #[test]
    fn field_wise_tuple_from_field_wise_builder() {
        let field_wise = EnumParams::<()>::field_wise_spec()
            .tuple()
            .with_0(String::from("a"))
            .with_0_in_memory()
            .with_0_from_map(|_: &u32| Some(String::from("b")))
            .build();
        let resources: Resources<SetUp> = {
            let mut resources = Resources::new();
            resources.insert(1u32);
            Resources::from(resources)
        };
        let mut value_resolution_ctx = ValueResolutionCtx::new(
            ValueResolutionMode::ApplyDry,
            item_id!("field_wise_tuple_from_field_wise_builder"),
            String::from("EnumParams<()>"),
        );

        assert!(matches!(
            field_wise,
            ParamsSpec::FieldWise {
                field_wise_spec: EnumParamsFieldWise::Tuple(
                    ValueSpec::MappingFn(mapping_fn),
                )
            }
            if matches!(
                mapping_fn.map(&resources, &mut value_resolution_ctx),
                Ok(src_mapped)
                if src_mapped == "b"
            )
        ));
    }

    #[test]
    fn field_wise_tuple_marker_from_field_wise_builder() {
        let field_wise = EnumParams::<()>::field_wise_spec()
            .tuple_marker()
            .with_0(String::from("a"))
            .with_0_in_memory()
            .with_0_from_map(|_: &u32| Some(String::from("b")))
            .build();
        let resources: Resources<SetUp> = {
            let mut resources = Resources::new();
            resources.insert(1u32);
            Resources::from(resources)
        };
        let mut value_resolution_ctx = ValueResolutionCtx::new(
            ValueResolutionMode::ApplyDry,
            item_id!("field_wise_tuple_marker_from_field_wise_builder"),
            String::from("EnumParams<()>"),
        );

        assert!(matches!(
            field_wise,
            ParamsSpec::FieldWise {
                field_wise_spec: EnumParamsFieldWise::TupleMarker(
                    ValueSpec::MappingFn(mapping_fn),
                    PhantomData,
                )
            }
            if matches!(
                mapping_fn.map(&resources, &mut value_resolution_ctx),
                Ok(src_mapped)
                if src_mapped == "b"
            )
        ));
    }

    #[test]
    fn field_wise_unit_from_field_wise_builder() {
        let field_wise = EnumParams::<()>::field_wise_spec().unit().build();

        assert!(matches!(
            field_wise,
            ParamsSpec::FieldWise {
                field_wise_spec: EnumParamsFieldWise::Unit
            }
        ));
    }

    #[test]
    fn spec_clone_named() {
        let spec = EnumParamsFieldWise::<()>::Named {
            src: ValueSpec::Value {
                value: String::from("a"),
            },
            marker: PhantomData,
        };
        let spec_clone = spec.clone();
        drop(spec);

        assert!(matches!(
            spec_clone,
            EnumParamsFieldWise::<()>::Named {
                src: ValueSpec::Value { value },
                marker: PhantomData
            }
            if value == "a"
        ));
    }

    #[test]
    fn spec_clone_tuple() {
        let spec = EnumParamsFieldWise::<()>::Tuple(ValueSpec::Value {
            value: String::from("a"),
        });
        let spec_clone = spec.clone();
        drop(spec);

        assert!(matches!(
            spec_clone,
            EnumParamsFieldWise::<()>::Tuple(ValueSpec::Value { value })
            if value == "a"
        ));
    }

    #[test]
    fn spec_clone_tuple_marker() {
        let spec = EnumParamsFieldWise::<()>::TupleMarker(
            ValueSpec::Value {
                value: String::from("a"),
            },
            PhantomData,
        );
        let spec_clone = spec.clone();
        drop(spec);

        assert!(matches!(
            spec_clone,
            EnumParamsFieldWise::<()>::TupleMarker(ValueSpec::Value { value }, PhantomData)
            if value == "a"
        ));
    }

    #[test]
    fn spec_clone_unit() {
        let spec = EnumParamsFieldWise::<()>::Unit;
        let spec_clone = spec.clone();
        drop(spec);

        assert!(matches!(spec_clone, EnumParamsFieldWise::<()>::Unit));
    }

    #[test]
    fn spec_debug_named() {
        assert_eq!(
            r#"Named { src: Value("a"), marker: PhantomData<()> }"#,
            format!(
                "{:?}",
                EnumParamsFieldWise::<()>::Named {
                    src: ValueSpec::Value {
                        value: String::from("a")
                    },
                    marker: PhantomData,
                }
            )
        );
    }

    #[test]
    fn spec_debug_tuple() {
        assert_eq!(
            r#"Tuple(Value("a"))"#,
            format!(
                "{:?}",
                EnumParamsFieldWise::<()>::Tuple(ValueSpec::Value {
                    value: String::from("a")
                })
            )
        );
    }

    #[test]
    fn spec_debug_tuple_marker() {
        assert_eq!(
            r#"TupleMarker(Value("a"), PhantomData<()>)"#,
            format!(
                "{:?}",
                EnumParamsFieldWise::<()>::TupleMarker(
                    ValueSpec::Value {
                        value: String::from("a")
                    },
                    PhantomData
                )
            )
        );
    }

    #[test]
    fn spec_debug_unit() {
        assert_eq!(r#"Unit"#, format!("{:?}", EnumParamsFieldWise::<()>::Unit));
    }

    #[test]
    fn params_partial_clone_named() {
        let params_partial = EnumParamsPartial::<()>::Named {
            src: Some(String::from("a")),
            marker: PhantomData,
        };
        let params_partial_clone = params_partial.clone();
        drop(params_partial);

        assert!(matches!(
            params_partial_clone,
            EnumParamsPartial::<()>::Named {
                src: Some(value),
                marker: PhantomData
            }
            if value == "a"
        ));
    }

    #[test]
    fn params_partial_clone_tuple() {
        let params_partial = EnumParamsPartial::<()>::Tuple(Some(String::from("a")));
        let params_partial_clone = params_partial.clone();
        drop(params_partial);

        assert!(matches!(
            params_partial_clone,
            EnumParamsPartial::<()>::Tuple(Some(value))
            if value == "a"
        ));
    }

    #[test]
    fn params_partial_clone_tuple_marker() {
        let params_partial =
            EnumParamsPartial::<()>::TupleMarker(Some(String::from("a")), PhantomData);
        let params_partial_clone = params_partial.clone();
        drop(params_partial);

        assert!(matches!(
            params_partial_clone,
            EnumParamsPartial::<()>::TupleMarker(Some(value), PhantomData)
            if value == "a"
        ));
    }

    #[test]
    fn params_partial_clone_unit() {
        let params_partial = EnumParamsPartial::<()>::Unit;
        let params_partial_clone = params_partial.clone();
        drop(params_partial);

        assert!(matches!(
            params_partial_clone,
            EnumParamsPartial::<()>::Unit
        ));
    }

    #[test]
    fn params_partial_debug_named() {
        assert_eq!(
            r#"Named { src: Some("a"), marker: PhantomData<()> }"#,
            format!(
                "{:?}",
                EnumParamsPartial::<()>::Named {
                    src: Some(String::from("a")),
                    marker: PhantomData,
                }
            )
        );
    }

    #[test]
    fn params_partial_debug_tuple() {
        assert_eq!(
            r#"Tuple(Some("a"))"#,
            format!(
                "{:?}",
                EnumParamsPartial::<()>::Tuple(Some(String::from("a")))
            )
        );
    }

    #[test]
    fn params_partial_debug_tuple_marker() {
        assert_eq!(
            r#"TupleMarker(Some("a"), PhantomData<()>)"#,
            format!(
                "{:?}",
                EnumParamsPartial::<()>::TupleMarker(Some(String::from("a")), PhantomData)
            )
        );
    }

    #[test]
    fn params_partial_debug_unit() {
        assert_eq!(r#"Unit"#, format!("{:?}", EnumParamsPartial::<()>::Unit));
    }

    #[test]
    fn params_try_from_partial_returns_ok_when_all_fields_are_some_named() {
        let params_partial = EnumParamsPartial::<()>::Named {
            src: Some(String::from("a")),
            marker: PhantomData,
        };

        assert!(matches!(
            EnumParams::<()>::try_from(params_partial),
            Ok(EnumParams::<()>::Named { src: value, marker: PhantomData})
            if value == "a"
        ));
    }

    #[test]
    fn params_try_from_partial_returns_err_when_some_fields_are_none_named() {
        let params_partial = EnumParamsPartial::<()>::Named {
            src: None,
            marker: PhantomData,
        };

        assert!(matches!(
            EnumParams::<()>::try_from(params_partial),
            Err(EnumParamsPartial::<()>::Named {
                src: None,
                marker: PhantomData
            })
        ));
    }

    #[test]
    fn params_try_from_partial_returns_ok_when_all_fields_are_some_tuple() {
        let params_partial = EnumParamsPartial::<()>::Tuple(Some(String::from("a")));

        assert!(matches!(
            EnumParams::<()>::try_from(params_partial),
            Ok(EnumParams::<()>::Tuple(value))
            if value == "a"
        ));
    }

    #[test]
    fn params_try_from_partial_returns_err_when_some_fields_are_none_tuple() {
        let params_partial = EnumParamsPartial::<()>::Tuple(None);

        assert!(matches!(
            EnumParams::<()>::try_from(params_partial),
            Err(EnumParamsPartial::<()>::Tuple(None))
        ));
    }

    #[test]
    fn params_try_from_partial_returns_ok_when_all_fields_are_some_tuple_marker() {
        let params_partial =
            EnumParamsPartial::<()>::TupleMarker(Some(String::from("a")), PhantomData);

        assert!(matches!(
            EnumParams::<()>::try_from(params_partial),
            Ok(EnumParams::<()>::TupleMarker(value, PhantomData))
            if value == "a"
        ));
    }

    #[test]
    fn params_try_from_partial_returns_err_when_some_fields_are_none_tuple_marker() {
        let params_partial = EnumParamsPartial::<()>::TupleMarker(None, PhantomData);

        assert!(matches!(
            EnumParams::<()>::try_from(params_partial),
            Err(EnumParamsPartial::<()>::TupleMarker(None, PhantomData))
        ));
    }

    #[test]
    fn params_try_from_partial_returns_ok_unit() {
        let params_partial = EnumParamsPartial::<()>::Unit;

        assert!(matches!(
            EnumParams::<()>::try_from(params_partial),
            Ok(EnumParams::<()>::Unit)
        ));
    }

    #[test]
    fn params_try_from_partial_ref_returns_ok_when_all_fields_are_some_named() {
        let params_partial = EnumParamsPartial::<()>::Named {
            src: Some(String::from("a")),
            marker: PhantomData,
        };

        assert!(matches!(
            EnumParams::<()>::try_from(&params_partial),
            Ok(EnumParams::<()>::Named { src: value, marker: PhantomData})
            if value == "a"
        ));
    }

    #[test]
    fn params_try_from_partial_ref_returns_err_when_some_fields_are_none_named() {
        let params_partial = EnumParamsPartial::<()>::Named {
            src: None,
            marker: PhantomData,
        };

        assert!(matches!(
            EnumParams::<()>::try_from(&params_partial),
            Err(EnumParamsPartial::<()>::Named {
                src: None,
                marker: PhantomData
            })
        ));
    }

    #[test]
    fn params_try_from_partial_ref_returns_ok_when_all_fields_are_some_tuple() {
        let params_partial = EnumParamsPartial::<()>::Tuple(Some(String::from("a")));

        assert!(matches!(
            EnumParams::<()>::try_from(&params_partial),
            Ok(EnumParams::<()>::Tuple(value))
            if value == "a"
        ));
    }

    #[test]
    fn params_try_from_partial_ref_returns_err_when_some_fields_are_none_tuple() {
        let params_partial = EnumParamsPartial::<()>::Tuple(None);

        assert!(matches!(
            EnumParams::<()>::try_from(&params_partial),
            Err(EnumParamsPartial::<()>::Tuple(None))
        ));
    }

    #[test]
    fn params_try_from_partial_ref_returns_ok_when_all_fields_are_some_tuple_marker() {
        let params_partial =
            EnumParamsPartial::<()>::TupleMarker(Some(String::from("a")), PhantomData);

        assert!(matches!(
            EnumParams::<()>::try_from(&params_partial),
            Ok(EnumParams::<()>::TupleMarker(value, PhantomData))
            if value == "a"
        ));
    }

    #[test]
    fn params_try_from_partial_ref_returns_err_when_some_fields_are_none_tuple_marker() {
        let params_partial = EnumParamsPartial::<()>::TupleMarker(None, PhantomData);

        assert!(matches!(
            EnumParams::<()>::try_from(&params_partial),
            Err(EnumParamsPartial::<()>::TupleMarker(None, PhantomData))
        ));
    }

    #[test]
    fn params_try_from_partial_ref_returns_ok_unit() {
        let params_partial = EnumParamsPartial::<()>::Unit;

        assert!(matches!(
            EnumParams::<()>::try_from(&params_partial),
            Ok(EnumParams::<()>::Unit)
        ));
    }
}

mod struct_recursive_value {
    use std::{any::TypeId, fmt::Debug};

    use serde::{Deserialize, Serialize};

    use peace::{
        item_model::item_id,
        params::{Params, ParamsSpec, ValueResolutionCtx, ValueResolutionMode, ValueSpec},
        resource_rt::{resources::ts::SetUp, Resources},
    };

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct InnerValue<T>(T)
    where
        T: Clone + Debug;

    impl<T> InnerValue<T>
    where
        T: Clone + Debug,
    {
        fn new(inner: T) -> Self {
            Self(inner)
        }
    }

    #[derive(Clone, Debug, Params, PartialEq, Eq, Serialize, Deserialize)]
    pub struct StructRecursiveValue<T>
    where
        T: Clone + Debug + Send + Sync + 'static,
    {
        /// Source / goal value for the state.
        src: InnerValue<T>,
        /// Destination storage for the state.
        dest: u32,
    }

    super::params_tests!(
        StructRecursiveValue,
        StructRecursiveValueFieldWise,
        StructRecursiveValuePartial,
        [<u8>]
    );

    #[test]
    fn spec_from_params() {
        let params = StructRecursiveValue::<u16> {
            src: InnerValue::<u16>::new(123),
            dest: 456,
        };

        assert!(matches!(
            ParamsSpec::from(params),
            ParamsSpec::Value {
                value: StructRecursiveValue {
                    src,
                    dest,
                }
            }
            if src == InnerValue::<u16>(123)
            && dest == 456
        ));
    }

    #[test]
    fn field_wise_from_params() {
        let params = StructRecursiveValue::<u16> {
            src: InnerValue::<u16>::new(123),
            dest: 456,
        };

        assert!(matches!(
            StructRecursiveValueFieldWise::from(params),
            StructRecursiveValueFieldWise {
                src: ValueSpec::Value { value: src_value },
                dest: ValueSpec::Value { value: dest_value },
            }
            if src_value == InnerValue::<u16>(123)
            && dest_value == 456
        ));
    }

    #[test]
    fn field_wise_from_field_wise_builder() {
        let field_wise = StructRecursiveValue::<()>::field_wise_spec()
            .with_src_in_memory()
            .with_dest_from_map(|_: &u32| Some(456))
            .build();
        let resources: Resources<SetUp> = {
            let mut resources = Resources::new();
            resources.insert(1u32);
            Resources::from(resources)
        };
        let mut value_resolution_ctx = ValueResolutionCtx::new(
            ValueResolutionMode::ApplyDry,
            item_id!("field_wise_from_field_wise_builder"),
            String::from("StructRecursiveValue<()>"),
        );

        assert!(matches!(
            field_wise,
            ParamsSpec::FieldWise {
                field_wise_spec: StructRecursiveValueFieldWise {
                    src: ValueSpec::InMemory,
                    dest: ValueSpec::MappingFn(mapping_fn),
                }
            }
            if matches!(
                mapping_fn.map(&resources, &mut value_resolution_ctx),
                Ok(dest_mapped)
                if dest_mapped == 456
            )
        ));
    }

    #[test]
    fn spec_debug() {
        assert_eq!(
            r#"StructRecursiveValueFieldWise { src: Value(InnerValue(123)), dest: Value(456) }"#,
            format!(
                "{:?}",
                StructRecursiveValueFieldWise::<u16> {
                    src: ValueSpec::Value {
                        value: InnerValue::<u16>::new(123)
                    },
                    dest: ValueSpec::Value { value: 456 },
                }
            )
        );
    }

    #[test]
    fn params_partial_debug() {
        assert_eq!(
            r#"StructRecursiveValuePartial { src: Some(InnerValue(123)), dest: Some(456) }"#,
            format!(
                "{:?}",
                StructRecursiveValuePartial::<u16> {
                    src: Some(InnerValue::<u16>::new(123)),
                    dest: Some(456),
                }
            )
        );
    }

    #[test]
    fn params_try_from_partial_returns_ok_when_all_fields_are_some() {
        let params_partial = StructRecursiveValuePartial::<u16> {
            src: Some(InnerValue::<u16>::new(123)),
            dest: Some(456),
        };

        assert!(matches!(
            StructRecursiveValue::try_from(params_partial),
            Ok(StructRecursiveValue {
                src,
                dest,
            })
            if src == InnerValue::<u16>(123)
            && dest == 456
        ));
    }

    #[test]
    fn params_try_from_partial_returns_err_when_some_fields_are_none() {
        let params_partial = StructRecursiveValuePartial::<u16> {
            src: Some(InnerValue::<u16>::new(123)),
            dest: None,
        };

        assert!(matches!(
            StructRecursiveValue::try_from(params_partial),
            Err(StructRecursiveValuePartial {
                src,
                dest,
            })
            if src == Some(InnerValue::<u16>::new(123))
            && dest.is_none()
        ));
    }

    #[test]
    fn params_try_from_partial_ref_returns_ok_when_all_fields_are_some() {
        let params_partial = StructRecursiveValuePartial::<u16> {
            src: Some(InnerValue::<u16>::new(123)),
            dest: Some(456),
        };

        assert!(matches!(
            StructRecursiveValue::try_from(&params_partial),
            Ok(StructRecursiveValue {
                src,
                dest,
            })
            if src == InnerValue::<u16>(123)
            && dest == 456
        ));
    }

    #[test]
    fn params_try_from_partial_ref_returns_err_when_some_fields_are_none() {
        let params_partial = StructRecursiveValuePartial::<u16> {
            src: Some(InnerValue::<u16>::new(123)),
            dest: None,
        };

        assert!(matches!(
            StructRecursiveValue::try_from(&params_partial),
            Err(StructRecursiveValuePartial {
                src,
                dest,
            })
            if src == &Some(InnerValue::<u16>::new(123))
            && dest.is_none()
        ));
    }
}

mod struct_recursive_value_no_bounds {
    use std::{any::TypeId, marker::PhantomData};

    use derivative::Derivative;
    use serde::{Deserialize, Serialize};

    use peace::params::{Params, ParamsSpec, ValueSpec};

    #[derive(Derivative, PartialEq, Eq, Serialize, Deserialize, Params)]
    #[derivative(Clone, Debug)]
    #[serde(bound = "")]
    pub struct InnerValue<Id> {
        /// Inner u32
        inner: u32,
        /// Marker for unique parameters type.
        marker: PhantomData<Id>,
    }

    impl<Id> InnerValue<Id> {
        fn new(inner: u32) -> Self {
            Self {
                inner,
                marker: PhantomData,
            }
        }
    }

    #[derive(Derivative, Params, PartialEq, Eq, Serialize, Deserialize)]
    #[derivative(Clone, Debug)]
    #[serde(bound = "")]
    pub struct StructRecursiveValueNoBounds<Id> {
        /// Source / goal value for the state.
        #[derivative(Clone(bound = ""), Debug(bound = ""))]
        src: InnerValue<Id>,
        /// Destination storage for the state.
        dest: u32,
    }

    super::params_tests!(
        StructRecursiveValueNoBounds,
        StructRecursiveValueNoBoundsFieldWise,
        StructRecursiveValueNoBoundsPartial,
        [<()>]
    );

    #[test]
    fn spec_from_params() {
        let params = StructRecursiveValueNoBounds::<()> {
            src: InnerValue::new(123),
            dest: 456,
        };

        assert!(matches!(
            ParamsSpec::from(params),
            ParamsSpec::Value {
                value: StructRecursiveValueNoBounds {
                    src,
                    dest,
                }
            }
            if src.inner == 123
            && dest == 456
        ));
    }

    #[test]
    fn field_wise_from_params() {
        let params = StructRecursiveValueNoBounds::<()> {
            src: InnerValue::new(123),
            dest: 456,
        };

        assert!(matches!(
            StructRecursiveValueNoBoundsFieldWise::from(params),
            StructRecursiveValueNoBoundsFieldWise {
                src: ValueSpec::Value { value: src_value },
                dest: ValueSpec::Value { value: dest_value },
            }
            if src_value.inner == 123
            && dest_value == 456
        ));
    }

    #[test]
    fn spec_debug() {
        assert_eq!(
            r#"StructRecursiveValueNoBoundsFieldWise { src: Value(InnerValue { inner: 123, marker: PhantomData<()> }), dest: Value(456) }"#,
            format!(
                "{:?}",
                StructRecursiveValueNoBoundsFieldWise::<()> {
                    src: ValueSpec::Value {
                        value: InnerValue::new(123)
                    },
                    dest: ValueSpec::Value { value: 456 },
                }
            )
        );
    }

    #[test]
    fn params_partial_debug() {
        assert_eq!(
            r#"StructRecursiveValueNoBoundsPartial { src: Some(InnerValue { inner: 123, marker: PhantomData<()> }), dest: Some(456) }"#,
            format!(
                "{:?}",
                StructRecursiveValueNoBoundsPartial::<()> {
                    src: Some(InnerValue::new(123)),
                    dest: Some(456),
                }
            )
        );
    }

    #[test]
    fn params_try_from_partial_returns_ok_when_all_fields_are_some() {
        let params_partial = StructRecursiveValueNoBoundsPartial::<()> {
            src: Some(InnerValue::new(123)),
            dest: Some(456),
        };

        assert!(matches!(
            StructRecursiveValueNoBounds::try_from(params_partial),
            Ok(StructRecursiveValueNoBounds {
                src,
                dest,
            })
            if src.inner == 123
            && dest == 456
        ));
    }

    #[test]
    fn params_try_from_partial_returns_err_when_some_fields_are_none() {
        let params_partial = StructRecursiveValueNoBoundsPartial::<()> {
            src: Some(InnerValue::new(123)),
            dest: None,
        };

        assert!(matches!(
            StructRecursiveValueNoBounds::try_from(params_partial),
            Err(StructRecursiveValueNoBoundsPartial {
                src,
                dest,
            })
            if src == Some(InnerValue::new(123))
            && dest.is_none()
        ));
    }

    #[test]
    fn params_try_from_partial_ref_returns_ok_when_all_fields_are_some() {
        let params_partial = StructRecursiveValueNoBoundsPartial::<()> {
            src: Some(InnerValue::new(123)),
            dest: Some(456),
        };

        assert!(matches!(
            StructRecursiveValueNoBounds::try_from(&params_partial),
            Ok(StructRecursiveValueNoBounds {
                src,
                dest,
            })
            if src.inner == 123
            && dest == 456
        ));
    }

    #[test]
    fn params_try_from_partial_ref_returns_err_when_some_fields_are_none() {
        let params_partial = StructRecursiveValueNoBoundsPartial::<()> {
            src: Some(InnerValue::new(123)),
            dest: None,
        };

        assert!(matches!(
            StructRecursiveValueNoBounds::try_from(&params_partial),
            Err(StructRecursiveValueNoBoundsPartial {
                src,
                dest,
            })
            if src == &Some(InnerValue::new(123))
            && dest.is_none()
        ));
    }
}

mod enum_recursive_value {
    use std::{any::TypeId, fmt::Debug};

    use serde::{Deserialize, Serialize};

    use peace::params::{Params, ParamsSpec, ValueSpec};

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub enum InnerValue<T>
    where
        T: Clone + Debug,
    {
        Tuple(T),
        Named { value: T },
    }

    #[derive(Clone, Debug, Params, PartialEq, Eq, Serialize, Deserialize)]
    pub enum EnumRecursiveValue<T>
    where
        T: Clone + Debug + Send + Sync + 'static,
    {
        Tuple(InnerValue<T>, u32),
        Named { src: InnerValue<T>, dest: u32 },
    }

    super::params_tests!(
        EnumRecursiveValue,
        EnumRecursiveValueFieldWise,
        EnumRecursiveValuePartial,
        [<u8>]
    );

    #[test]
    fn spec_from_params() {
        let params = EnumRecursiveValue::Named::<u16> {
            src: InnerValue::<u16>::Tuple(123),
            dest: 456,
        };

        assert!(matches!(
            ParamsSpec::from(params),
            ParamsSpec::Value {
                value: EnumRecursiveValue::Named {
                    src,
                    dest,
                }
            }
            if src == InnerValue::<u16>::Tuple(123)
            && dest == 456
        ));

        let params = EnumRecursiveValue::Tuple::<u16>(InnerValue::<u16>::Named { value: 123 }, 456);

        assert!(matches!(
            ParamsSpec::from(params),
            ParamsSpec::Value {
                value: EnumRecursiveValue::Tuple(
                    src,
                    dest,
                )
            }
            if src == InnerValue::<u16>::Named { value: 123 }
            && dest == 456
        ));
    }

    #[test]
    fn field_wise_from_params() {
        let params = EnumRecursiveValue::Named::<u16> {
            src: InnerValue::<u16>::Tuple(123),
            dest: 456,
        };

        assert!(matches!(
            EnumRecursiveValueFieldWise::from(params),
            EnumRecursiveValueFieldWise::Named {
                src: ValueSpec::Value { value: src_value },
                dest: ValueSpec::Value { value: dest_value },
            }
            if src_value == InnerValue::<u16>::Tuple(123)
            && dest_value == 456
        ));

        let params = EnumRecursiveValue::Tuple::<u16>(InnerValue::<u16>::Named { value: 123 }, 456);

        assert!(matches!(
            EnumRecursiveValueFieldWise::from(params),
            EnumRecursiveValueFieldWise::Tuple(
                ValueSpec::Value { value: src_value },
                ValueSpec::Value { value: dest_value },
            )
            if src_value == InnerValue::<u16>::Named { value: 123}
            && dest_value == 456
        ));
    }

    #[test]
    fn spec_debug() {
        assert_eq!(
            r#"Named { src: Value(Tuple(123)), dest: Value(456) }"#,
            format!(
                "{:?}",
                EnumRecursiveValueFieldWise::Named::<u16> {
                    src: ValueSpec::Value {
                        value: InnerValue::<u16>::Tuple(123)
                    },
                    dest: ValueSpec::Value { value: 456 },
                }
            )
        );
    }

    #[test]
    fn params_partial_debug() {
        assert_eq!(
            r#"Named { src: Some(Tuple(123)), dest: Some(456) }"#,
            format!(
                "{:?}",
                EnumRecursiveValuePartial::Named::<u16> {
                    src: Some(InnerValue::<u16>::Tuple(123)),
                    dest: Some(456),
                }
            )
        );
    }

    #[test]
    fn params_try_from_partial_returns_ok_when_all_fields_are_some() {
        let params_partial = EnumRecursiveValuePartial::Named::<u16> {
            src: Some(InnerValue::<u16>::Tuple(123)),
            dest: Some(456),
        };

        assert!(matches!(
            EnumRecursiveValue::try_from(params_partial),
            Ok(EnumRecursiveValue::Named {
                src,
                dest,
            })
            if src == InnerValue::<u16>::Tuple(123)
            && dest == 456
        ));

        let params_partial = EnumRecursiveValuePartial::Tuple::<u16>(
            Some(InnerValue::<u16>::Named { value: 123 }),
            Some(456),
        );

        assert!(matches!(
            EnumRecursiveValue::try_from(params_partial),
            Ok(EnumRecursiveValue::Tuple(
                src,
                dest,
            ))
            if src == InnerValue::<u16>::Named { value: 123 }
            && dest == 456
        ));
    }

    #[test]
    fn params_try_from_partial_returns_err_when_some_fields_are_none() {
        let params_partial = EnumRecursiveValuePartial::Named::<u16> {
            src: Some(InnerValue::<u16>::Tuple(123)),
            dest: None,
        };

        assert!(matches!(
            EnumRecursiveValue::try_from(params_partial),
            Err(EnumRecursiveValuePartial::Named {
                src,
                dest,
            })
            if src == Some(InnerValue::<u16>::Tuple(123))
            && dest.is_none()
        ));

        let params_partial = EnumRecursiveValuePartial::Tuple::<u16>(
            Some(InnerValue::<u16>::Named { value: 123 }),
            None,
        );

        assert!(matches!(
            EnumRecursiveValue::try_from(params_partial),
            Err(EnumRecursiveValuePartial::Tuple(
                src,
                dest,
            ))
            if src == Some(InnerValue::<u16>::Named { value: 123 })
            && dest.is_none()
        ));
    }

    #[test]
    fn params_try_from_partial_ref_returns_ok_when_all_fields_are_some() {
        let params_partial = EnumRecursiveValuePartial::Named::<u16> {
            src: Some(InnerValue::<u16>::Tuple(123)),
            dest: Some(456),
        };

        assert!(matches!(
            EnumRecursiveValue::try_from(&params_partial),
            Ok(EnumRecursiveValue::Named {
                src,
                dest,
            })
            if src == InnerValue::<u16>::Tuple(123)
            && dest == 456
        ));

        let params_partial = EnumRecursiveValuePartial::Tuple::<u16>(
            Some(InnerValue::<u16>::Named { value: 123 }),
            Some(456),
        );

        assert!(matches!(
            EnumRecursiveValue::try_from(&params_partial),
            Ok(EnumRecursiveValue::Tuple(
                src,
                dest,
            ))
            if src == InnerValue::<u16>::Named { value: 123 }
            && dest == 456
        ));
    }

    #[test]
    fn params_try_from_partial_ref_returns_err_when_some_fields_are_none() {
        let params_partial = EnumRecursiveValuePartial::Named::<u16> {
            src: Some(InnerValue::<u16>::Tuple(123)),
            dest: None,
        };

        assert!(matches!(
            EnumRecursiveValue::try_from(&params_partial),
            Err(EnumRecursiveValuePartial::Named {
                src,
                dest,
            })
            if src == &Some(InnerValue::<u16>::Tuple(123))
            && dest.is_none()
        ));

        let params_partial = EnumRecursiveValuePartial::Tuple::<u16>(
            Some(InnerValue::<u16>::Named { value: 123 }),
            None,
        );

        assert!(matches!(
            EnumRecursiveValue::try_from(&params_partial),
            Err(EnumRecursiveValuePartial::Tuple(
                src,
                dest,
            ))
            if src == &Some(InnerValue::<u16>::Named { value: 123 })
            && dest.is_none()
        ));
    }
}

mod enum_recursive_marker_no_bounds {
    use std::{any::TypeId, marker::PhantomData};

    use derivative::Derivative;
    use serde::{Deserialize, Serialize};

    use peace::params::{Params, ParamsSpec, ValueSpec};

    #[derive(Derivative, PartialEq, Eq, Serialize, Deserialize, Params)]
    #[derivative(Clone(bound = ""), Debug(bound = ""))]
    #[serde(bound = "")]
    pub enum InnerValue<Id> {
        Tuple(PhantomData<Id>),
        Named { marker: PhantomData<Id> },
    }

    #[derive(Derivative, PartialEq, Eq, Serialize, Deserialize, Params)]
    #[derivative(Clone(bound = ""), Debug(bound = ""))]
    #[serde(bound = "")]
    pub enum EnumRecursiveNoBounds<Id> {
        Tuple(InnerValue<Id>, u32),
        Named { src: InnerValue<Id>, dest: u32 },
    }

    super::params_tests!(
        EnumRecursiveNoBounds,
        EnumRecursiveNoBoundsFieldWise,
        EnumRecursiveNoBoundsPartial,
        [<u8>]
    );

    #[test]
    fn spec_from_params() {
        let params = EnumRecursiveNoBounds::Named::<u16> {
            src: InnerValue::<u16>::Tuple(PhantomData),
            dest: 456,
        };

        assert!(matches!(
            ParamsSpec::from(params),
            ParamsSpec::Value {
                value: EnumRecursiveNoBounds::Named {
                    src: InnerValue::<u16>::Tuple(PhantomData),
                    dest,
                }
            }
            if dest == 456
        ));

        let params = EnumRecursiveNoBounds::Tuple::<u16>(
            InnerValue::<u16>::Named {
                marker: PhantomData,
            },
            456,
        );

        assert!(matches!(
            ParamsSpec::from(params),
            ParamsSpec::Value {
                value: EnumRecursiveNoBounds::Tuple(
                    InnerValue::<u16>::Named { marker: PhantomData },
                    dest,
                )
            }
            if dest == 456
        ));
    }

    #[test]
    fn field_wise_from_params() {
        let params = EnumRecursiveNoBounds::Named::<u16> {
            src: InnerValue::<u16>::Tuple(PhantomData),
            dest: 456,
        };

        assert!(matches!(
            EnumRecursiveNoBoundsFieldWise::from(params),
            EnumRecursiveNoBoundsFieldWise::Named {
                src: ValueSpec::Value { value: InnerValue::<u16>::Tuple(PhantomData) },
                dest: ValueSpec::Value { value: dest_marker },
            }
            if dest_marker == 456
        ));

        let params = EnumRecursiveNoBounds::Tuple::<u16>(
            InnerValue::<u16>::Named {
                marker: PhantomData,
            },
            456,
        );

        assert!(matches!(
            EnumRecursiveNoBoundsFieldWise::from(params),
            EnumRecursiveNoBoundsFieldWise::Tuple(
                ValueSpec::Value { value: InnerValue::<u16>::Named { marker: PhantomData } },
                ValueSpec::Value { value: dest_marker },
            )
            if dest_marker == 456
        ));
    }

    #[test]
    fn spec_debug() {
        assert_eq!(
            r#"Named { src: Value(Tuple(PhantomData<u16>)), dest: Value(456) }"#,
            format!(
                "{:?}",
                EnumRecursiveNoBoundsFieldWise::Named::<u16> {
                    src: ValueSpec::Value {
                        value: InnerValue::<u16>::Tuple(PhantomData)
                    },
                    dest: ValueSpec::Value { value: 456 },
                }
            )
        );
    }

    #[test]
    fn params_partial_debug() {
        assert_eq!(
            r#"Named { src: Some(Tuple(PhantomData<u16>)), dest: Some(456) }"#,
            format!(
                "{:?}",
                EnumRecursiveNoBoundsPartial::Named::<u16> {
                    src: Some(InnerValue::<u16>::Tuple(PhantomData)),
                    dest: Some(456),
                }
            )
        );
    }

    #[test]
    fn params_try_from_partial_returns_ok_when_all_fields_are_some() {
        let params_partial = EnumRecursiveNoBoundsPartial::Named::<u16> {
            src: Some(InnerValue::<u16>::Tuple(PhantomData)),
            dest: Some(456),
        };

        assert!(matches!(
            EnumRecursiveNoBounds::try_from(params_partial),
            Ok(EnumRecursiveNoBounds::Named {
                src: InnerValue::<u16>::Tuple(PhantomData),
                dest,
            })
            if dest == 456
        ));

        let params_partial = EnumRecursiveNoBoundsPartial::Tuple::<u16>(
            Some(InnerValue::<u16>::Named {
                marker: PhantomData,
            }),
            Some(456),
        );

        assert!(matches!(
            EnumRecursiveNoBounds::try_from(params_partial),
            Ok(EnumRecursiveNoBounds::Tuple(
                InnerValue::<u16>::Named { marker: PhantomData },
                dest,
            ))
            if dest == 456
        ));
    }

    #[test]
    fn params_try_from_partial_returns_err_when_some_fields_are_none() {
        let params_partial = EnumRecursiveNoBoundsPartial::Named::<u16> {
            src: Some(InnerValue::<u16>::Tuple(PhantomData)),
            dest: None,
        };

        assert!(matches!(
            EnumRecursiveNoBounds::try_from(params_partial),
            Err(EnumRecursiveNoBoundsPartial::Named {
                src: Some(InnerValue::<u16>::Tuple(PhantomData)),
                dest,
            })
            if dest.is_none()
        ));

        let params_partial = EnumRecursiveNoBoundsPartial::Tuple::<u16>(
            Some(InnerValue::<u16>::Named {
                marker: PhantomData,
            }),
            None,
        );

        assert!(matches!(
            EnumRecursiveNoBounds::try_from(params_partial),
            Err(EnumRecursiveNoBoundsPartial::Tuple(
                Some(InnerValue::<u16>::Named { marker: PhantomData }),
                dest,
            ))
            if dest.is_none()
        ));
    }

    #[test]
    fn params_try_from_partial_ref_returns_ok_when_all_fields_are_some() {
        let params_partial = EnumRecursiveNoBoundsPartial::Named::<u16> {
            src: Some(InnerValue::<u16>::Tuple(PhantomData)),
            dest: Some(456),
        };

        assert!(matches!(
            EnumRecursiveNoBounds::try_from(&params_partial),
            Ok(EnumRecursiveNoBounds::Named {
                src: InnerValue::<u16>::Tuple(PhantomData),
                dest,
            })
            if dest == 456
        ));

        let params_partial = EnumRecursiveNoBoundsPartial::Tuple::<u16>(
            Some(InnerValue::<u16>::Named {
                marker: PhantomData,
            }),
            Some(456),
        );

        assert!(matches!(
            EnumRecursiveNoBounds::try_from(&params_partial),
            Ok(EnumRecursiveNoBounds::Tuple(
                InnerValue::<u16>::Named { marker: PhantomData },
                dest,
            ))
            if dest == 456
        ));
    }

    #[test]
    fn params_try_from_partial_ref_returns_err_when_some_fields_are_none() {
        let params_partial = EnumRecursiveNoBoundsPartial::Named::<u16> {
            src: Some(InnerValue::<u16>::Tuple(PhantomData)),
            dest: None,
        };

        assert!(matches!(
            EnumRecursiveNoBounds::try_from(&params_partial),
            Err(EnumRecursiveNoBoundsPartial::Named {
                src: Some(InnerValue::<u16>::Tuple(PhantomData)),
                dest,
            })
            if dest.is_none()
        ));

        let params_partial = EnumRecursiveNoBoundsPartial::Tuple::<u16>(
            Some(InnerValue::<u16>::Named {
                marker: PhantomData,
            }),
            None,
        );

        assert!(matches!(
            EnumRecursiveNoBounds::try_from(&params_partial),
            Err(EnumRecursiveNoBoundsPartial::Tuple(
                Some(InnerValue::<u16>::Named { marker: PhantomData }),
                dest,
            ))
            if dest.is_none()
        ));
    }
}

/// Tests `Value` derivation of external types in `Params`.
mod external_fields {
    use std::{any::TypeId, fmt::Debug};

    use serde::{Deserialize, Serialize};

    use peace::params::{Params, ParamsSpec, ValueSpec};

    // Note: no `Value` derive.
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct InnerValue(u32);

    #[derive(Clone, Debug, Params, PartialEq, Eq, Serialize, Deserialize)]
    pub struct StructExternalValue {
        /// Source / goal value for the state.
        #[value_spec(fieldless)]
        src: InnerValue,
        /// Destination storage for the state.
        dest: u32,
    }

    super::params_tests!(
        StructExternalValue,
        StructExternalValueFieldWise,
        StructExternalValuePartial,
        []
    );

    #[test]
    fn spec_from_params() {
        let params = StructExternalValue {
            src: InnerValue(123),
            dest: 456,
        };

        assert!(matches!(
            ParamsSpec::from(params),
            ParamsSpec::Value {
                value: StructExternalValue {
                    src,
                    dest,
                }
            }
            if src == InnerValue(123)
            && dest == 456
        ));
    }

    #[test]
    fn field_wise_from_params() {
        let params = StructExternalValue {
            src: InnerValue(123),
            dest: 456,
        };

        assert!(matches!(
            StructExternalValueFieldWise::from(params),
            StructExternalValueFieldWise {
                src: ValueSpec::Value { value: InnerValue(src_value) },
                dest: ValueSpec::Value { value: dest_value },
            }
            if src_value == 123
            && dest_value == 456
        ));
    }

    #[test]
    fn spec_debug() {
        assert_eq!(
            r#"StructExternalValueFieldWise { src: Value(InnerValue(123)), dest: Value(456) }"#,
            format!(
                "{:?}",
                StructExternalValueFieldWise {
                    src: ValueSpec::Value {
                        value: InnerValue(123)
                    },
                    dest: ValueSpec::Value { value: 456 },
                }
            )
        );
    }

    #[test]
    fn params_partial_debug() {
        assert_eq!(
            r#"StructExternalValuePartial { src: Some(InnerValue(123)), dest: Some(456) }"#,
            format!(
                "{:?}",
                StructExternalValuePartial {
                    src: Some(InnerValue(123)),
                    dest: Some(456),
                }
            )
        );
    }

    #[test]
    fn params_try_from_partial_returns_ok_when_all_fields_are_some() {
        let params_partial = StructExternalValuePartial {
            src: Some(InnerValue(123)),
            dest: Some(456),
        };

        assert!(matches!(
            StructExternalValue::try_from(params_partial),
            Ok(StructExternalValue {
                src: InnerValue(src_value),
                dest,
            })
            if src_value == 123
            && dest == 456
        ));
    }

    #[test]
    fn params_try_from_partial_returns_err_when_some_fields_are_none() {
        let params_partial = StructExternalValuePartial {
            src: Some(InnerValue(123)),
            dest: None,
        };

        assert!(matches!(
            StructExternalValue::try_from(params_partial),
            Err(StructExternalValuePartial {
                src: Some(InnerValue(src_value)),
                dest,
            })
            if src_value == 123
            && dest.is_none()
        ));
    }

    #[test]
    fn params_try_from_partial_ref_returns_ok_when_all_fields_are_some() {
        let params_partial = StructExternalValuePartial {
            src: Some(InnerValue(123)),
            dest: Some(456),
        };

        assert!(matches!(
            StructExternalValue::try_from(&params_partial),
            Ok(StructExternalValue {
                src: InnerValue(src_value),
                dest,
            })
            if src_value == 123
            && dest == 456
        ));
    }

    #[test]
    fn params_try_from_partial_ref_returns_err_when_some_fields_are_none() {
        let params_partial = StructExternalValuePartial {
            src: Some(InnerValue(123)),
            dest: None,
        };

        assert!(matches!(
            StructExternalValue::try_from(&params_partial),
            Err(StructExternalValuePartial {
                src: Some(InnerValue(src_value)),
                dest,
            })
            if src_value == &123
            && dest.is_none()
        ));
    }
}

macro_rules! params_tests {
    (
        $params_ty:ident,
        $params_field_wise_ty:ident,
        $params_partial_ty:ident,
        [$($generics:tt)*]
    ) => {
        #[test]
        fn params_field_wise_spec_associated_type_is_params_field_wise() {
            assert_eq!(
                TypeId::of::<<$params_ty $($generics)* as Params>::FieldWiseSpec>(),
                TypeId::of::<$params_field_wise_ty $($generics)*>()
            );
        }

        #[test]
        fn params_partial_associated_type_is_params_partial() {
            assert_eq!(
                TypeId::of::<<$params_ty $($generics)* as Params>::Partial>(),
                TypeId::of::<$params_partial_ty $($generics)*>()
            );
        }
    };
}

pub(crate) use params_tests;
