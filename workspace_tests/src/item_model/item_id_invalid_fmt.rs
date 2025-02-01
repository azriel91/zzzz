use std::borrow::Cow;

use peace::item_model::ItemIdInvalidFmt;

#[test]
fn debug() {
    let item_id_invalid_fmt = ItemIdInvalidFmt::new(Cow::Borrowed("invalid id"));

    assert_eq!(
        r#"ItemIdInvalidFmt { value: "invalid id" }"#,
        format!("{item_id_invalid_fmt:?}")
    );
}

#[test]
fn display() {
    let item_id_invalid_fmt = ItemIdInvalidFmt::new(Cow::Borrowed("invalid id"));

    assert_eq!(
        "`invalid id` is not a valid `ItemId`.\n\
        `ItemId`s must begin with a letter or underscore, and contain only letters, numbers, or underscores.",
        format!("{item_id_invalid_fmt}")
    );
}
