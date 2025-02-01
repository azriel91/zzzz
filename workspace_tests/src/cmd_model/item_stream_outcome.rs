use peace::{
    cmd_model::ItemStreamOutcome, item_model::item_id, rt_model::fn_graph::StreamOutcomeState,
};

#[test]
fn map() {
    let item_stream_outcome = ItemStreamOutcome::finished_with(1u16, vec![item_id!("mock")]);

    let item_stream_outcome = item_stream_outcome.map(|n| n + 1);

    assert_eq!(
        ItemStreamOutcome::finished_with(2u16, vec![item_id!("mock")]),
        item_stream_outcome
    );
}

#[test]
fn replace() {
    let item_stream_outcome = ItemStreamOutcome::finished_with(1u16, vec![item_id!("mock")]);

    let (item_stream_outcome, n) = item_stream_outcome.replace(2u32);

    assert_eq!(1u16, n);
    assert_eq!(
        ItemStreamOutcome::finished_with(2u32, vec![item_id!("mock")]),
        item_stream_outcome
    );
}

#[test]
fn replace_with() {
    let item_stream_outcome =
        ItemStreamOutcome::finished_with((1u16, "value_to_extract"), vec![item_id!("mock")]);

    let (item_stream_outcome, value) = item_stream_outcome.replace_with(|(n, value)| (n, value));

    assert_eq!("value_to_extract", value);
    assert_eq!(
        ItemStreamOutcome::finished_with(1u16, vec![item_id!("mock")]),
        item_stream_outcome
    );
}

#[test]
fn into_value() {
    let item_stream_outcome = ItemStreamOutcome::finished_with(1u16, vec![item_id!("mock")]);

    let n = item_stream_outcome.into_value();

    assert_eq!(1u16, n);
}

#[test]
fn value() {
    let item_stream_outcome = ItemStreamOutcome::finished_with(1u16, vec![item_id!("mock")]);

    let n = item_stream_outcome.value();

    assert_eq!(1u16, *n);
}

#[test]
fn value_mut() {
    let mut item_stream_outcome = ItemStreamOutcome::finished_with(1u16, vec![item_id!("mock")]);

    *item_stream_outcome.value_mut() += 1;

    assert_eq!(2u16, *item_stream_outcome.value());
}

#[test]
fn state() {
    let item_stream_outcome = ItemStreamOutcome::finished_with(1u16, vec![item_id!("mock")]);

    assert_eq!(StreamOutcomeState::Finished, item_stream_outcome.state());
}

#[test]
fn item_ids_processed() {
    let item_stream_outcome = ItemStreamOutcome::finished_with(1u16, vec![item_id!("mock")]);

    assert_eq!(
        &[item_id!("mock")],
        item_stream_outcome.item_ids_processed()
    );
}

#[test]
fn item_ids_not_processed() {
    let mut item_stream_outcome = ItemStreamOutcome::finished_with(1u16, vec![item_id!("mock")]);
    item_stream_outcome.item_ids_not_processed = vec![item_id!("mock_1")];

    assert_eq!(
        &[item_id!("mock_1")],
        item_stream_outcome.item_ids_not_processed()
    );
}

#[test]
fn default() {
    let item_stream_outcome = ItemStreamOutcome::<u16>::default();

    assert_eq!(0u16, *item_stream_outcome.value());
    assert_eq!(StreamOutcomeState::NotStarted, item_stream_outcome.state());
    assert!(item_stream_outcome.item_ids_processed().is_empty());
    assert!(item_stream_outcome.item_ids_not_processed().is_empty());
}
