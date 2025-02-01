use peace::progress_model::{ProgressComplete, ProgressStatus};

#[test]
fn debug() {
    assert_eq!("Initialized", format!("{:?}", ProgressStatus::Initialized))
}

#[test]
fn clone() {
    assert_eq!(
        ProgressStatus::Initialized,
        ProgressStatus::Initialized.clone()
    )
}

#[test]
fn deserialize() {
    assert_eq!(
        ProgressStatus::Complete(ProgressComplete::Success),
        serde_yaml::from_str("!Complete Success\n").unwrap()
    )
}

#[test]
fn serialize() {
    assert_eq!(
        "!Complete Success\n",
        serde_yaml::to_string(&ProgressStatus::Complete(ProgressComplete::Success)).unwrap()
    )
}
