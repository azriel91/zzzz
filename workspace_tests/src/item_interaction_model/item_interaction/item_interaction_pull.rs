use peace::item_interaction_model::{ItemInteractionPull, ItemLocation};

#[test]
fn location_client() {
    let item_interaction_pull = ItemInteractionPull::new(
        vec![ItemLocation::localhost()].into(),
        vec![ItemLocation::host("server".to_string())].into(),
    );

    assert_eq!(
        vec![ItemLocation::localhost()],
        item_interaction_pull.location_client()
    );
}

#[test]
fn location_server() {
    let item_interaction_pull = ItemInteractionPull::new(
        vec![ItemLocation::localhost()].into(),
        vec![ItemLocation::host("server".to_string())].into(),
    );

    assert_eq!(
        vec![ItemLocation::host("server".to_string())],
        item_interaction_pull.location_server()
    );
}
