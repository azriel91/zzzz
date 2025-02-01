use peace::progress_model::{ProgressComplete, ProgressDelta, ProgressLimit, ProgressUpdate};

#[test]
fn clone() {
    let progress_update_0 = ProgressUpdate::Delta(ProgressDelta::Tick);
    #[allow(clippy::redundant_clone)] // https://github.com/rust-lang/rust-clippy/issues/9011
    let progress_update_1 = progress_update_0.clone();

    assert_eq!(progress_update_0, progress_update_1);
}

#[test]
fn deserialize() {
    assert_eq!(
        ProgressUpdate::Delta(ProgressDelta::Inc(3)),
        serde_yaml::from_str("!Delta\nInc: 3\n").unwrap()
    )
}

#[test]
fn serialize() {
    assert_eq!(
        "!Delta\nInc: 3\n",
        serde_yaml::to_string(&ProgressUpdate::Delta(ProgressDelta::Inc(3))).unwrap()
    )
}

#[test]
fn deserialize_json() {
    assert_eq!(
        ProgressUpdate::Delta(ProgressDelta::Inc(3)),
        serde_json::from_str(r#"{"Delta":{"Inc":3}}"#).unwrap()
    )
}

#[test]
fn serialize_json() {
    assert_eq!(
        r#"{"Delta":{"Inc":3}}"#,
        serde_json::to_string(&ProgressUpdate::Delta(ProgressDelta::Inc(3))).unwrap()
    )
}

#[test]
fn eq() {
    assert_eq!(
        ProgressUpdate::Limit(ProgressLimit::Steps(3)),
        ProgressUpdate::Limit(ProgressLimit::Steps(3))
    );
    assert_eq!(
        ProgressUpdate::Delta(ProgressDelta::Inc(3)),
        ProgressUpdate::Delta(ProgressDelta::Inc(3))
    );
    assert_eq!(
        ProgressUpdate::Complete(ProgressComplete::Success),
        ProgressUpdate::Complete(ProgressComplete::Success)
    );
}

#[test]
fn ne() {
    assert_ne!(
        ProgressUpdate::Limit(ProgressLimit::Steps(3)),
        ProgressUpdate::Limit(ProgressLimit::Steps(4))
    );
    assert_ne!(
        ProgressUpdate::Delta(ProgressDelta::Inc(3)),
        ProgressUpdate::Delta(ProgressDelta::Inc(4))
    );
    assert_ne!(
        ProgressUpdate::Complete(ProgressComplete::Success),
        ProgressUpdate::Complete(ProgressComplete::Fail)
    );
}

#[test]
fn debug() {
    assert_eq!(
        r#"Limit(Steps(3))"#,
        format!("{:?}", ProgressUpdate::Limit(ProgressLimit::Steps(3)))
    );
    assert_eq!(
        r#"Delta(Inc(3))"#,
        format!("{:?}", ProgressUpdate::Delta(ProgressDelta::Inc(3)))
    );
    assert_eq!(
        r#"Complete(Success)"#,
        format!("{:?}", ProgressUpdate::Complete(ProgressComplete::Success))
    );
}
