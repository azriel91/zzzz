use std::{borrow::Cow, collections::HashMap, str::FromStr};

use peace::{
    flow_model::{FlowId, FlowIdInvalidFmt},
    fmt::Presentable,
};

use crate::{FnInvocation, FnTrackerPresenter};

#[test]
fn from_str_returns_ok_owned_for_valid_id() -> Result<(), FlowIdInvalidFmt<'static>> {
    let flow_id = FlowId::from_str("good_id")?;

    assert_eq!("good_id", *flow_id);
    Ok(())
}

#[test]
fn underscore_is_valid_first_character() -> Result<(), FlowIdInvalidFmt<'static>> {
    let flow_id = FlowId::new("_good_id")?;

    assert_eq!("_good_id", *flow_id);
    Ok(())
}

#[test]
fn new_unchecked_does_not_validate_id() -> Result<(), FlowIdInvalidFmt<'static>> {
    let flow_id = FlowId::new_unchecked("!valid");

    assert_eq!("!valid", *flow_id);
    Ok(())
}

#[test]
fn try_from_str_returns_ok_borrowed_for_valid_id() -> Result<(), FlowIdInvalidFmt<'static>> {
    let flow_id = FlowId::try_from("good_id")?;

    assert_eq!("good_id", *flow_id);
    Ok(())
}

#[test]
fn try_from_string_returns_ok_owned_for_valid_id() -> Result<(), FlowIdInvalidFmt<'static>> {
    let flow_id = FlowId::try_from(String::from("good_id"))?;

    assert_eq!("good_id", *flow_id);
    Ok(())
}

#[test]
fn from_str_returns_err_owned_for_invalid_id() {
    let error = FlowId::from_str("has space").unwrap_err();

    assert!(matches!(error.value(), Cow::Owned(_)));
    assert_eq!("has space", error.value());
}

#[test]
fn try_from_str_returns_err_borrowed_for_invalid_id() {
    let error = FlowId::try_from("has space").unwrap_err();

    assert!(matches!(error.value(), Cow::Borrowed(_)));
    assert_eq!("has space", error.value());
}

#[test]
fn try_from_string_returns_err_owned_for_invalid_id() {
    let error = FlowId::try_from(String::from("has space")).unwrap_err();

    assert!(matches!(error.value(), Cow::Owned(_)));
    assert_eq!("has space", error.value());
}

#[test]
fn display_returns_inner_str() -> Result<(), FlowIdInvalidFmt<'static>> {
    let flow_id = FlowId::try_from("good_id")?;

    assert_eq!("good_id", flow_id.to_string());
    Ok(())
}

#[tokio::test]
async fn present_uses_code_inline() -> Result<(), Box<dyn std::error::Error>> {
    let mut presenter = FnTrackerPresenter::new();
    let flow_id = FlowId::try_from("flow_id")?;

    flow_id.present(&mut presenter).await?;

    assert_eq!(
        vec![FnInvocation::new(
            "code_inline",
            vec![Some(r#""flow_id""#.to_string())]
        )],
        presenter.fn_invocations()
    );
    Ok(())
}

#[test]
fn clone() -> Result<(), FlowIdInvalidFmt<'static>> {
    let flow_id_0 = FlowId::new("id")?;
    #[allow(clippy::redundant_clone)] // https://github.com/rust-lang/rust-clippy/issues/9011
    let flow_id_1 = flow_id_0.clone();

    assert_eq!(flow_id_0, flow_id_1);
    Ok(())
}

#[test]
fn debug() -> Result<(), FlowIdInvalidFmt<'static>> {
    let flow_id = FlowId::new("id")?;

    assert_eq!(r#"FlowId("id")"#, format!("{flow_id:?}"));
    Ok(())
}

#[test]
fn hash() -> Result<(), FlowIdInvalidFmt<'static>> {
    let flow_id = FlowId::new("flow_id")?;

    let mut hash_map = HashMap::new();
    hash_map.insert(flow_id, ());

    Ok(())
}

#[test]
fn partial_eq_ne() -> Result<(), FlowIdInvalidFmt<'static>> {
    let flow_id_0 = FlowId::new("id0")?;
    let flow_id_1 = FlowId::new("id1")?;

    assert!(flow_id_0 != flow_id_1);
    Ok(())
}

#[test]
fn serialize() -> Result<(), Box<dyn std::error::Error>> {
    let flow_id = FlowId::new("flow_id")?;

    assert_eq!("flow_id\n", serde_yaml::to_string(&flow_id)?);
    Ok(())
}

#[test]
fn deserialize() -> Result<(), Box<dyn std::error::Error>> {
    let flow_id = FlowId::new("flow_id")?;

    assert_eq!(flow_id, serde_yaml::from_str("flow_id")?);
    Ok(())
}
