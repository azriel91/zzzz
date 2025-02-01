use peace::progress_model::ProgressLimit;

#[test]
fn defaults_to_unknown() {
    assert_eq!(ProgressLimit::Unknown, ProgressLimit::default());
}

#[test]
fn clone() {
    let progress_limit_0 = ProgressLimit::Steps(3);
    let progress_limit_1 = Clone::clone(&progress_limit_0);

    assert_eq!(progress_limit_0, progress_limit_1);
}

#[test]
fn copy() {
    let progress_limit_0 = ProgressLimit::Steps(3);
    let progress_limit_1 = progress_limit_0;

    assert_eq!(progress_limit_0, progress_limit_1);
}

#[test]
fn deserialize() {
    assert_eq!(
        ProgressLimit::Steps(3),
        serde_yaml::from_str("!Steps 3\n").unwrap()
    )
}

#[test]
fn serialize() {
    assert_eq!(
        "!Steps 3\n",
        serde_yaml::to_string(&ProgressLimit::Steps(3)).unwrap()
    )
}

#[test]
fn eq() {
    assert_eq!(ProgressLimit::Steps(3), ProgressLimit::Steps(3));
    assert_eq!(ProgressLimit::Bytes(3), ProgressLimit::Bytes(3));

    // Should this be equal? It should match at least.
    assert_eq!(ProgressLimit::Unknown, ProgressLimit::Unknown);
}

#[test]
fn ne() {
    assert_ne!(ProgressLimit::Steps(3), ProgressLimit::Steps(4));
    assert_ne!(ProgressLimit::Steps(3), ProgressLimit::Bytes(3));
    assert_ne!(ProgressLimit::Steps(3), ProgressLimit::Unknown);
    assert_ne!(ProgressLimit::Bytes(3), ProgressLimit::Bytes(4));
    assert_ne!(ProgressLimit::Bytes(3), ProgressLimit::Unknown);
}

#[test]
fn debug() {
    assert_eq!(r#"Unknown"#, format!("{:?}", ProgressLimit::Unknown));
    assert_eq!(r#"Steps(3)"#, format!("{:?}", ProgressLimit::Steps(3)));
    assert_eq!(r#"Bytes(3)"#, format!("{:?}", ProgressLimit::Bytes(3)));
}
