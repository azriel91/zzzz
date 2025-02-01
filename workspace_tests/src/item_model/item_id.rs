use std::{borrow::Cow, str::FromStr};

use peace::{
    fmt::Presentable,
    item_model::{ItemId, ItemIdInvalidFmt},
};

use crate::{FnInvocation, FnTrackerPresenter};

#[test]
fn from_str_returns_ok_owned_for_valid_id() -> Result<(), ItemIdInvalidFmt<'static>> {
    let item_id = ItemId::from_str("good_id")?;

    assert_eq!("good_id", *item_id);
    Ok(())
}

#[test]
fn underscore_is_valid_first_character() -> Result<(), ItemIdInvalidFmt<'static>> {
    let item_id = ItemId::new("_good_id")?;

    assert_eq!("_good_id", *item_id);
    Ok(())
}

#[test]
fn new_unchecked_does_not_validate_id() -> Result<(), ItemIdInvalidFmt<'static>> {
    let item_id = ItemId::new_unchecked("!valid");

    assert_eq!("!valid", *item_id);
    Ok(())
}

#[test]
fn try_from_str_returns_ok_borrowed_for_valid_id() -> Result<(), ItemIdInvalidFmt<'static>> {
    let item_id = ItemId::try_from("good_id")?;

    assert_eq!("good_id", *item_id);
    Ok(())
}

#[test]
fn try_from_string_returns_ok_owned_for_valid_id() -> Result<(), ItemIdInvalidFmt<'static>> {
    let item_id = ItemId::try_from(String::from("good_id"))?;

    assert_eq!("good_id", *item_id);
    Ok(())
}

#[test]
fn from_str_returns_err_owned_for_invalid_id() {
    let error = ItemId::from_str("has space").unwrap_err();

    assert!(matches!(error.value(), Cow::Owned(_)));
    assert_eq!("has space", error.value());
}

#[test]
fn try_from_str_returns_err_borrowed_for_invalid_id() {
    let error = ItemId::try_from("has space").unwrap_err();

    assert!(matches!(error.value(), Cow::Borrowed(_)));
    assert_eq!("has space", error.value());
}

#[test]
fn try_from_string_returns_err_owned_for_invalid_id() {
    let error = ItemId::try_from(String::from("has space")).unwrap_err();

    assert!(matches!(error.value(), Cow::Owned(_)));
    assert_eq!("has space", error.value());
}

#[test]
fn display_returns_inner_str() -> Result<(), ItemIdInvalidFmt<'static>> {
    let item_id = ItemId::try_from("good_id")?;

    assert_eq!("good_id", item_id.to_string());
    Ok(())
}

#[tokio::test]
async fn present_uses_code_inline() -> Result<(), Box<dyn std::error::Error>> {
    let mut presenter = FnTrackerPresenter::new();
    let item_id = ItemId::try_from("item_id")?;

    item_id.present(&mut presenter).await?;

    assert_eq!(
        vec![FnInvocation::new(
            "code_inline",
            vec![Some(r#""item_id""#.to_string())]
        )],
        presenter.fn_invocations()
    );
    Ok(())
}

#[test]
fn clone() -> Result<(), ItemIdInvalidFmt<'static>> {
    let item_id_0 = ItemId::new("id")?;
    #[allow(clippy::redundant_clone)] // https://github.com/rust-lang/rust-clippy/issues/9011
    let item_id_1 = item_id_0.clone();

    assert_eq!(item_id_0, item_id_1);
    Ok(())
}

#[test]
fn debug() -> Result<(), ItemIdInvalidFmt<'static>> {
    let item_id = ItemId::new("id")?;

    assert_eq!(r#"ItemId("id")"#, format!("{item_id:?}"));
    Ok(())
}

#[test]
fn partial_eq_ne() -> Result<(), ItemIdInvalidFmt<'static>> {
    let item_id_0 = ItemId::new("id0")?;
    let item_id_1 = ItemId::new("id1")?;

    assert!(item_id_0 != item_id_1);
    Ok(())
}
