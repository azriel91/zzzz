use peace::{
    cmd_model::{CmdBlockOutcome, StreamOutcomeAndErrors, ValueAndStreamOutcome},
    item_model::item_id,
    rt_model::{fn_graph::StreamOutcome, IndexMap},
};

#[test]
fn is_ok() {
    assert!(cmd_block_outcome_single(123).is_ok());
    assert!(cmd_block_outcome_item_wise(123, None).is_ok());
    assert!(!cmd_block_outcome_item_wise(123, Some("err".to_string())).is_ok());
}

#[test]
fn is_err() {
    assert!(!cmd_block_outcome_single(123).is_err());
    assert!(!cmd_block_outcome_item_wise(123, None).is_err());
    assert!(cmd_block_outcome_item_wise(123, Some("err".to_string())).is_err());
}

#[test]
fn try_into_value() {
    assert_eq!(
        Ok(ValueAndStreamOutcome {
            value: 123,
            stream_outcome: None,
        }),
        cmd_block_outcome_single(123).try_into_value()
    );
    assert_eq!(
        Ok(ValueAndStreamOutcome {
            value: 123,
            stream_outcome: Some(StreamOutcome::finished_with((), Vec::new())),
        }),
        cmd_block_outcome_item_wise(123, None).try_into_value()
    );

    let mut errors = IndexMap::new();
    errors.insert(item_id!("mock"), "err".to_string());
    assert_eq!(
        Err(StreamOutcomeAndErrors {
            stream_outcome: StreamOutcome::finished_with(123, Vec::new()),
            errors,
        }),
        cmd_block_outcome_item_wise(123, Some("err".to_string())).try_into_value()
    );
}

#[test]
fn map() {
    assert_eq!(
        cmd_block_outcome_single(124),
        cmd_block_outcome_single(123).map(|value| value + 1)
    );
    assert_eq!(
        cmd_block_outcome_item_wise(124, None),
        cmd_block_outcome_item_wise(123, None).map(|value| value + 1)
    );

    assert_eq!(
        cmd_block_outcome_item_wise(124, Some("err".to_string())),
        cmd_block_outcome_item_wise(123, Some("err".to_string())).map(|value| value + 1)
    );
}

#[test]
fn clone() {
    let cmd_block_outcome = cmd_block_outcome_single(123);

    assert_eq!(cmd_block_outcome, Clone::clone(&cmd_block_outcome));
}

#[test]
fn debug() {
    let cmd_block_outcome = cmd_block_outcome_single(123);

    assert_eq!("Single(123)", format!("{cmd_block_outcome:?}"));
}

fn cmd_block_outcome_single<T>(value: T) -> CmdBlockOutcome<T, String> {
    CmdBlockOutcome::<T, String>::Single(value)
}

fn cmd_block_outcome_item_wise<T>(value: T, error: Option<String>) -> CmdBlockOutcome<T, String> {
    let mut errors = IndexMap::new();
    if let Some(error) = error {
        errors.insert(item_id!("mock"), error);
    }

    CmdBlockOutcome::<T, String>::ItemWise {
        stream_outcome: StreamOutcome::finished_with(value, Vec::new()),
        errors,
    }
}
