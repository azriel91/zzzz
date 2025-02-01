use peace::{flow_model::ItemInfo, item_model::item_id};

#[test]
fn clone() {
    let item_info = ItemInfo::new(item_id!("item_id"));

    assert_eq!(item_info, Clone::clone(&item_info));
}

#[test]
fn debug() {
    let item_info = ItemInfo::new(item_id!("item_id"));

    assert_eq!(
        "ItemInfo { item_id: ItemId(\"item_id\") }",
        format!("{item_info:?}")
    );
}

#[test]
fn serialize() -> Result<(), serde_yaml::Error> {
    let item_info = ItemInfo::new(item_id!("item_id"));

    assert_eq!("item_id: item_id\n", serde_yaml::to_string(&item_info)?);
    Ok(())
}

#[test]
fn deserialize() -> Result<(), serde_yaml::Error> {
    let item_info = ItemInfo::new(item_id!("item_id"));

    assert_eq!(item_info, serde_yaml::from_str("item_id: item_id\n")?);
    Ok(())
}
