use std::{ffi::OsStr, path::Path};

use peace::item_interaction_model::{url::ParseError, ItemLocation, ItemLocationType, Url};

#[test]
fn group() {
    let item_location = ItemLocation::group("Cloud".to_string());

    assert_eq!(
        ItemLocation::new(
            peace::item_interaction_model::ItemLocationType::Group,
            "Cloud".to_string(),
        ),
        item_location
    );
}

#[test]
fn host() {
    let item_location = ItemLocation::host("Server".to_string());

    assert_eq!(
        ItemLocation::new(
            peace::item_interaction_model::ItemLocationType::Host,
            "Server".to_string(),
        ),
        item_location
    );
}

#[test]
fn host_unknown() {
    let item_location = ItemLocation::host_unknown();

    assert_eq!(
        ItemLocation::new(
            peace::item_interaction_model::ItemLocationType::Host,
            ItemLocation::HOST_UNKNOWN.to_string(),
        ),
        item_location
    );
}

#[test]
fn host_from_url_https() -> Result<(), ParseError> {
    let item_location = ItemLocation::host_from_url(&Url::parse("https://example.com/resource")?);

    assert_eq!(
        ItemLocation::new(
            peace::item_interaction_model::ItemLocationType::Host,
            "🌐 example.com".to_string(),
        ),
        item_location
    );

    Ok(())
}

#[test]
fn host_from_url_file() -> Result<(), ParseError> {
    let item_location = ItemLocation::host_from_url(&Url::parse("file:///path/to/resource")?);

    assert_eq!(
        ItemLocation::new(
            peace::item_interaction_model::ItemLocationType::Host,
            ItemLocation::LOCALHOST.to_string(),
        ),
        item_location
    );

    Ok(())
}

#[test]
fn localhost() {
    let item_location = ItemLocation::localhost();

    assert_eq!(
        ItemLocation::new(
            peace::item_interaction_model::ItemLocationType::Host,
            ItemLocation::LOCALHOST.to_string(),
        ),
        item_location
    );
}

#[test]
fn path() {
    let item_location = ItemLocation::path("/path/to/resource".to_string());

    assert_eq!(
        ItemLocation::new(
            peace::item_interaction_model::ItemLocationType::Path,
            "/path/to/resource".to_string(),
        ),
        item_location
    );
}

#[test]
fn path_lossy() {
    let path = unsafe {
        Path::new(OsStr::from_encoded_bytes_unchecked(
            b"/path/to/lossy_fo\xF0\x90\x80.txt",
        ))
    };
    let item_location = ItemLocation::path_lossy(path);

    assert_eq!(
        ItemLocation::new(
            peace::item_interaction_model::ItemLocationType::Path,
            "/path/to/lossy_fo�.txt".to_string(),
        ),
        item_location
    );
}

#[test]
fn name() {
    let item_location = ItemLocation::path("/path/to/resource".to_string());

    assert_eq!("/path/to/resource", item_location.name());
}

#[test]
fn r#type() {
    let item_location = ItemLocation::path("/path/to/resource".to_string());

    assert_eq!(ItemLocationType::Path, item_location.r#type());
}
