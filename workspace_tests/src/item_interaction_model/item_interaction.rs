use peace::item_interaction_model::{
    ItemInteraction, ItemInteractionPull, ItemInteractionPush, ItemInteractionWithin, ItemLocation,
};

mod item_interaction_pull;
mod item_interaction_push;
mod item_interaction_within;

#[test]
fn from_item_interaction_push() {
    let item_interaction_push = ItemInteractionPush::new(
        vec![ItemLocation::localhost()].into(),
        vec![ItemLocation::host("server".to_string())].into(),
    );
    let item_interaction = ItemInteraction::from(item_interaction_push.clone());

    assert_eq!(
        ItemInteraction::Push(item_interaction_push),
        item_interaction
    );
}

#[test]
fn from_item_interaction_pull() {
    let item_interaction_pull = ItemInteractionPull::new(
        vec![ItemLocation::localhost()].into(),
        vec![ItemLocation::host("server".to_string())].into(),
    );
    let item_interaction = ItemInteraction::from(item_interaction_pull.clone());

    assert_eq!(
        ItemInteraction::Pull(item_interaction_pull),
        item_interaction
    );
}

#[test]
fn from_item_interaction_within() {
    let item_interaction_within =
        ItemInteractionWithin::new(vec![ItemLocation::localhost()].into());
    let item_interaction = ItemInteraction::from(item_interaction_within.clone());

    assert_eq!(
        ItemInteraction::Within(item_interaction_within),
        item_interaction
    );
}
