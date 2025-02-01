use std::borrow::Cow;

use peace::flow_model::FlowIdInvalidFmt;

#[test]
fn debug() {
    let flow_id_invalid_fmt = FlowIdInvalidFmt::new(Cow::Borrowed("invalid id"));

    assert_eq!(
        r#"FlowIdInvalidFmt { value: "invalid id" }"#,
        format!("{flow_id_invalid_fmt:?}")
    );
}

#[test]
fn display() {
    let flow_id_invalid_fmt = FlowIdInvalidFmt::new(Cow::Borrowed("invalid id"));

    assert_eq!(
        "`invalid id` is not a valid `FlowId`.\n\
        `FlowId`s must begin with a letter or underscore, and contain only letters, numbers, or underscores.",
        format!("{flow_id_invalid_fmt}")
    );
}
