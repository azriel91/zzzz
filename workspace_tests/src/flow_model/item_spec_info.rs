use peace::{flow_model::ItemSpecInfo, item_model::item_id};

#[test]
fn clone() {
    let item_spec_info = ItemSpecInfo::new(item_id!("item_id"));

    assert_eq!(item_spec_info, Clone::clone(&item_spec_info));
}

#[test]
fn debug() {
    let item_spec_info = ItemSpecInfo::new(item_id!("item_id"));

    assert_eq!(
        "ItemSpecInfo { item_id: ItemId(\"item_id\") }",
        format!("{item_spec_info:?}")
    );
}

#[test]
fn serialize() -> Result<(), serde_yaml::Error> {
    let item_spec_info = ItemSpecInfo::new(item_id!("item_id"));

    assert_eq!(
        "item_id: item_id\n",
        serde_yaml::to_string(&item_spec_info)?
    );
    Ok(())
}

#[test]
fn deserialize() -> Result<(), serde_yaml::Error> {
    let item_spec_info = ItemSpecInfo::new(item_id!("item_id"));

    assert_eq!(item_spec_info, serde_yaml::from_str("item_id: item_id\n")?);
    Ok(())
}
