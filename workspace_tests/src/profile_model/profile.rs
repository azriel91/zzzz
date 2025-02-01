use std::{borrow::Cow, collections::HashMap, str::FromStr};

use peace::{
    fmt::Presentable,
    profile_model::{Profile, ProfileInvalidFmt},
};

use crate::{FnInvocation, FnTrackerPresenter};

#[test]
fn from_str_returns_ok_owned_for_valid_profile() -> Result<(), ProfileInvalidFmt<'static>> {
    let profile = Profile::from_str("good_profile")?;

    assert_eq!("good_profile", *profile);
    Ok(())
}

#[test]
fn underscore_is_valid_first_character() -> Result<(), ProfileInvalidFmt<'static>> {
    let profile = Profile::new("_good_profile")?;

    assert_eq!("_good_profile", *profile);
    Ok(())
}

#[test]
fn new_unchecked_does_not_validate_profile() -> Result<(), ProfileInvalidFmt<'static>> {
    let profile = Profile::new_unchecked("!valid");

    assert_eq!("!valid", *profile);
    Ok(())
}

#[test]
fn try_from_str_returns_ok_borrowed_for_valid_profile() -> Result<(), ProfileInvalidFmt<'static>> {
    let profile = Profile::try_from("good_profile")?;

    assert_eq!("good_profile", *profile);
    Ok(())
}

#[test]
fn try_from_string_returns_ok_owned_for_valid_profile() -> Result<(), ProfileInvalidFmt<'static>> {
    let profile = Profile::try_from(String::from("good_profile"))?;

    assert_eq!("good_profile", *profile);
    Ok(())
}

#[test]
fn from_str_returns_err_owned_for_invalid_profile() {
    let error = Profile::from_str("has space").unwrap_err();

    assert!(matches!(error.value(), Cow::Owned(_)));
    assert_eq!("has space", error.value());
}

#[test]
fn try_from_str_returns_err_borrowed_for_invalid_profile() {
    let error = Profile::try_from("has space").unwrap_err();

    assert!(matches!(error.value(), Cow::Borrowed(_)));
    assert_eq!("has space", error.value());
}

#[test]
fn try_from_string_returns_err_owned_for_invalid_profile() {
    let error = Profile::try_from(String::from("has space")).unwrap_err();

    assert!(matches!(error.value(), Cow::Owned(_)));
    assert_eq!("has space", error.value());
}

#[test]
fn display_returns_inner_str() -> Result<(), ProfileInvalidFmt<'static>> {
    let profile = Profile::try_from("good_profile")?;

    assert_eq!("good_profile", profile.to_string());
    Ok(())
}

#[tokio::test]
async fn present_uses_tag() -> Result<(), Box<dyn std::error::Error>> {
    let mut presenter = FnTrackerPresenter::new();
    let profile = Profile::try_from("profile")?;

    profile.present(&mut presenter).await?;

    assert_eq!(
        vec![FnInvocation::new(
            "tag",
            vec![Some(r#""profile""#.to_string())]
        )],
        presenter.fn_invocations()
    );
    Ok(())
}

#[test]
fn clone() -> Result<(), ProfileInvalidFmt<'static>> {
    let profile_0 = Profile::new("profile")?;
    #[allow(clippy::redundant_clone)] // https://github.com/rust-lang/rust-clippy/issues/9011
    let profile_1 = profile_0.clone();

    assert_eq!(profile_0, profile_1);
    Ok(())
}

#[test]
fn debug() -> Result<(), ProfileInvalidFmt<'static>> {
    let profile = Profile::new("profile")?;

    assert_eq!(r#"Profile("profile")"#, format!("{profile:?}"));
    Ok(())
}

#[test]
fn hash() -> Result<(), ProfileInvalidFmt<'static>> {
    let profile = Profile::new("profile")?;

    let mut hash_map = HashMap::new();
    hash_map.insert(profile, ());

    Ok(())
}

#[test]
fn partial_eq_ne() -> Result<(), ProfileInvalidFmt<'static>> {
    let profile_0 = Profile::new("profile0")?;
    let profile_1 = Profile::new("profile1")?;

    assert!(profile_0 != profile_1);
    Ok(())
}

#[test]
fn serialize() -> Result<(), Box<dyn std::error::Error>> {
    let profile = Profile::new("profile")?;

    assert_eq!("profile\n", serde_yaml::to_string(&profile)?);
    Ok(())
}

#[test]
fn deserialize() -> Result<(), Box<dyn std::error::Error>> {
    let profile = Profile::new("profile")?;

    assert_eq!(profile, serde_yaml::from_str("profile")?);
    Ok(())
}
