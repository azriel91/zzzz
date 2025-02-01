use peace::{
    item_model::item_id,
    params::{FieldNameAndType, ValueResolutionCtx, ValueResolutionMode},
};

use crate::mock_item::MockSrc;

#[test]
fn debug() {
    let value_resolution_ctx = ValueResolutionCtx::new(
        ValueResolutionMode::Current,
        item_id!("item_id"),
        tynm::type_name::<MockSrc>(),
    );

    assert_eq!(
        "ValueResolutionCtx { \
            value_resolution_mode: Current, \
            item_id: ItemId(\"item_id\"), \
            params_type_name: \"MockSrc\", \
            resolution_chain: [] \
        }",
        format!("{value_resolution_ctx:?}")
    );
}

#[test]
fn partial_eq() {
    let value_resolution_ctx_0 = ValueResolutionCtx::new(
        ValueResolutionMode::Current,
        item_id!("item_id_0"),
        tynm::type_name::<MockSrc>(),
    );

    let value_resolution_ctx_1 = ValueResolutionCtx::new(
        ValueResolutionMode::Current,
        item_id!("item_id_1"),
        tynm::type_name::<MockSrc>(),
    );

    assert_eq!(value_resolution_ctx_0, value_resolution_ctx_0);
    assert_ne!(value_resolution_ctx_0, value_resolution_ctx_1);
}

#[test]
fn display_no_resolution_chain() {
    let value_resolution_ctx = ValueResolutionCtx::new(
        ValueResolutionMode::Current,
        item_id!("item_id"),
        tynm::type_name::<MockSrc>(),
    );

    assert_eq!("MockSrc {}", format!("{value_resolution_ctx}"));
}

#[test]
fn display_with_resolution_chain() {
    let mut value_resolution_ctx = ValueResolutionCtx::new(
        ValueResolutionMode::Current,
        item_id!("item_id"),
        tynm::type_name::<MockSrc>(),
    );
    value_resolution_ctx.push(FieldNameAndType::new(
        String::from("intermediate"),
        String::from("Something"),
    ));
    value_resolution_ctx.push(FieldNameAndType::new(
        String::from("inner"),
        tynm::type_name::<u8>(),
    ));

    assert_eq!(
        r#"MockSrc {
    intermediate: Something {
        inner: u8,
        ..
    },
    ..
}"#,
        format!("{value_resolution_ctx}")
    );
}
