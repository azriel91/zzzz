use peace::item_interaction_model::{ItemInteractionWithin, ItemLocation};

#[test]
fn location() {
    let item_interaction_within =
        ItemInteractionWithin::new(vec![ItemLocation::localhost()].into());

    assert_eq!(
        vec![ItemLocation::localhost()],
        item_interaction_within.location()
    );
}
