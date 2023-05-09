mod unit {
    use std::any::TypeId;

    use serde::{Deserialize, Serialize};

    use peace::params::Params;

    #[derive(Clone, Debug, Params, Serialize, Deserialize)]
    pub struct UnitParams;

    super::params_tests!(UnitParams, UnitParamsFieldWise, UnitParamsPartial, []);

    #[test]
    fn spec_from_params() {
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

    use peace::params::{Params, ValueSpec};

    #[derive(Clone, Debug, Params, Serialize, Deserialize)]
    pub struct StructParams {
        /// Source / desired value for the state.
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
            StructParamsFieldWise::from(params),
            StructParamsFieldWise {
                src: ValueSpec::Value(src_value),
                dest: ValueSpec::Value(dest_value),
            }
            if src_value == "a"
            && dest_value == "b"
        ));
    }

    #[test]
    fn spec_debug() {
        assert_eq!(
            r#"StructParamsFieldWise { src: Value("a"), dest: Value("b") }"#,
            format!(
                "{:?}",
                StructParamsFieldWise {
                    src: ValueSpec::Value(String::from("a")),
                    dest: ValueSpec::Value(String::from("b")),
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
    use std::{any::TypeId, fmt::Debug, marker::PhantomData};

    use serde::{Deserialize, Serialize};

    use peace::params::{Params, ValueSpec};

    #[derive(Clone, Debug, Params, Serialize, Deserialize)]
    pub struct StructWithTypeParams<Id>
    where
        Id: Clone + Debug,
    {
        /// Source / desired value for the state.
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
            StructWithTypeParamsFieldWise::from(params),
            StructWithTypeParamsFieldWise {
                src: ValueSpec::Value(src_value),
                dest: ValueSpec::Value(dest_value),
                marker: PhantomData,
            }
            if src_value == "a"
            && dest_value == "b"
        ));
    }

    #[test]
    fn spec_debug() {
        assert_eq!(
            r#"StructWithTypeParamsFieldWise { src: Value("a"), dest: Value("b"), marker: PhantomData<()> }"#,
            format!(
                "{:?}",
                StructWithTypeParamsFieldWise::<()> {
                    src: ValueSpec::Value(String::from("a")),
                    dest: ValueSpec::Value(String::from("b")),
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

    use peace::params::{Params, ValueSpec};

    #[derive(Clone, Debug, Params, Serialize, Deserialize)]
    pub struct TupleParams(
        /// Source / desired value for the state.
        String,
        /// Destination storage for the state.
        String,
    );

    super::params_tests!(TupleParams, TupleParamsFieldWise, TupleParamsPartial, []);

    #[test]
    fn spec_from_params() {
        let params = TupleParams(String::from("a"), String::from("b"));

        assert!(matches!(
            TupleParamsFieldWise::from(params),
            TupleParamsFieldWise (
                ValueSpec::Value(src_value),
                ValueSpec::Value(dest_value),
            )
            if src_value == "a"
            && dest_value == "b"
        ));
    }

    #[test]
    fn spec_debug() {
        assert_eq!(
            r#"TupleParamsFieldWise(Value("a"), Value("b"))"#,
            format!(
                "{:?}",
                TupleParamsFieldWise(
                    ValueSpec::Value(String::from("a")),
                    ValueSpec::Value(String::from("b")),
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

    use peace::params::{Params, ValueSpec};

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
            TupleWithTypeParamsFieldWise::from(params),
            TupleWithTypeParamsFieldWise::<()>(
                ValueSpec::Value(src_value),
                ValueSpec::Value(dest_value),
                PhantomData,
            )
            if src_value == "a"
            && dest_value == "b"
        ));
    }

    #[test]
    fn spec_debug() {
        assert_eq!(
            r#"TupleWithTypeParamsFieldWise(Value("a"), Value("b"), PhantomData<()>)"#,
            format!(
                "{:?}",
                TupleWithTypeParamsFieldWise::<()>(
                    ValueSpec::Value(String::from("a")),
                    ValueSpec::Value(String::from("b")),
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
    use std::{any::TypeId, fmt::Debug, marker::PhantomData};

    use serde::{Deserialize, Serialize};

    use peace::params::{Params, ValueSpec};

    #[derive(Clone, Debug, Params, Serialize, Deserialize)]
    pub enum EnumParams<Id>
    where
        Id: Clone + Debug,
    {
        Named {
            /// Source / desired value for the state.
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
            EnumParamsFieldWise::from(params),
            EnumParamsFieldWise::<()>::Named {
                src: ValueSpec::Value(value),
                marker: PhantomData,
            }
            if value == "a"
        ));
    }

    #[test]
    fn spec_tuple_from_params() {
        let params = EnumParams::<()>::Tuple(String::from("a"));

        assert!(matches!(
            EnumParamsFieldWise::from(params),
            EnumParamsFieldWise::<()>::Tuple(ValueSpec::Value(value))
            if value == "a"
        ));
    }

    #[test]
    fn spec_tuple_marker_from_params() {
        let params = EnumParams::<()>::TupleMarker(String::from("a"), PhantomData);

        assert!(matches!(
            EnumParamsFieldWise::from(params),
            EnumParamsFieldWise::<()>::TupleMarker(ValueSpec::Value(value), PhantomData)
            if value == "a"
        ));
    }

    #[test]
    fn spec_unit_from_params() {
        let params = EnumParams::<()>::Unit;

        assert!(matches!(
            EnumParamsFieldWise::from(params),
            EnumParamsFieldWise::<()>::Unit
        ));
    }

    #[test]
    fn spec_clone_named() {
        let spec = EnumParamsFieldWise::<()>::Named {
            src: ValueSpec::Value(String::from("a")),
            marker: PhantomData,
        };
        let spec_clone = spec.clone();
        drop(spec);

        assert!(matches!(
            spec_clone,
            EnumParamsFieldWise::<()>::Named {
                src: ValueSpec::Value(value),
                marker: PhantomData
            }
            if value == "a"
        ));
    }

    #[test]
    fn spec_clone_tuple() {
        let spec = EnumParamsFieldWise::<()>::Tuple(ValueSpec::Value(String::from("a")));
        let spec_clone = spec.clone();
        drop(spec);

        assert!(matches!(
            spec_clone,
            EnumParamsFieldWise::<()>::Tuple(ValueSpec::Value(value))
            if value == "a"
        ));
    }

    #[test]
    fn spec_clone_tuple_marker() {
        let spec = EnumParamsFieldWise::<()>::TupleMarker(
            ValueSpec::Value(String::from("a")),
            PhantomData,
        );
        let spec_clone = spec.clone();
        drop(spec);

        assert!(matches!(
            spec_clone,
            EnumParamsFieldWise::<()>::TupleMarker(ValueSpec::Value(value), PhantomData)
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
                    src: ValueSpec::Value(String::from("a")),
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
                EnumParamsFieldWise::<()>::Tuple(ValueSpec::Value(String::from("a")))
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
                    ValueSpec::Value(String::from("a")),
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
            Ok(EnumParams::<()>::Named{ src: value, marker: PhantomData})
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
            Ok(EnumParams::<()>::Named{ src: value, marker: PhantomData})
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
