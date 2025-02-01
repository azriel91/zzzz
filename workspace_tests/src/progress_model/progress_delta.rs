use peace::progress_model::ProgressDelta;

#[test]
fn debug() {
    assert_eq!("Tick", format!("{:?}", ProgressDelta::Tick))
}

#[test]
fn clone() {
    assert_eq!(ProgressDelta::Tick, ProgressDelta::Tick.clone())
}

#[test]
fn deserialize() {
    assert_eq!(
        ProgressDelta::Inc(3),
        serde_yaml::from_str("!Inc 3\n").unwrap()
    )
}

#[test]
fn serialize() {
    assert_eq!(
        "!Inc 3\n",
        serde_yaml::to_string(&ProgressDelta::Inc(3)).unwrap()
    )
}
