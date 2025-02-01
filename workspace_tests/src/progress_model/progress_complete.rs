use peace::progress_model::ProgressComplete;

#[test]
fn debug() {
    assert_eq!("Success", format!("{:?}", ProgressComplete::Success))
}

#[test]
fn clone() {
    assert_eq!(ProgressComplete::Success, ProgressComplete::Success.clone())
}

#[test]
fn deserialize() {
    assert_eq!(
        ProgressComplete::Success,
        serde_yaml::from_str("Success\n").unwrap()
    )
}

#[test]
fn serialize() {
    assert_eq!(
        "Success\n",
        serde_yaml::to_string(&ProgressComplete::Success).unwrap()
    )
}

#[test]
fn is_successful() {
    assert!(ProgressComplete::Success.is_successful());
    assert!(!ProgressComplete::Fail.is_successful());
}

#[test]
fn is_failure() {
    assert!(!ProgressComplete::Success.is_failure());
    assert!(ProgressComplete::Fail.is_failure());
}
