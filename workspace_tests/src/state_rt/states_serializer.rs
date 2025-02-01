use peace::{
    flow_model::flow_id,
    flow_rt::ItemGraphBuilder,
    item_model::item_id,
    resource_rt::{
        internal::StatesMut, paths::StatesCurrentFile, states::StatesCurrentStored,
        type_reg::untagged::TypeReg,
    },
    rt_model::{Error, Storage},
    state_rt::StatesSerializer,
};
use pretty_assertions::assert_eq;

use crate::{
    mock_item::{MockItem, MockState},
    vec_copy_item::VecCopyState,
    PeaceTestError, VecCopyItem,
};

#[tokio::test]
async fn serialize() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let storage = Storage;
    let states_current_file = StatesCurrentFile::new(tempdir.path().join("states_current.yaml"));

    let item_one = item_id!("one");
    let item_two = item_id!("two");
    let item_three = item_id!("three");
    let item_graph = {
        let mut item_graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        item_graph_builder.add_fns([
            VecCopyItem::new(item_one.clone()).into(),
            MockItem::<()>::new(item_two.clone()).into(),
            MockItem::<()>::new(item_three.clone()).into(),
        ]);
        item_graph_builder.build()
    };
    let states = {
        let mut states_mut = StatesMut::new();
        states_mut.insert(item_one.clone(), VecCopyState::from(vec![1u8]));
        states_mut.insert(item_two.clone(), MockState(2u8));
        StatesCurrentStored::from(states_mut)
    };
    StatesSerializer::<PeaceTestError>::serialize(
        &storage,
        &item_graph,
        &states,
        &states_current_file,
    )
    .await?;

    let serialized = tokio::fs::read_to_string(states_current_file).await?;
    assert_eq!(
        "\
        one:\n\
          - 1\n\
        two: 2\n\
        three: null\n\
        ",
        serialized
    );

    Ok(())
}

#[tokio::test]
async fn deserialize_stored() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let flow_id = flow_id!("test_flow");
    let storage = Storage;
    let states_current_file = StatesCurrentFile::new(tempdir.path().join("states_current.yaml"));

    let item_one = item_id!("one");
    let item_two = item_id!("two");
    let item_three = item_id!("three");
    let item_graph = {
        let mut item_graph_builder = ItemGraphBuilder::<PeaceTestError>::new();
        item_graph_builder.add_fns([
            VecCopyItem::new(item_one.clone()).into(),
            MockItem::<()>::new(item_two.clone()).into(),
            MockItem::<()>::new(item_three.clone()).into(),
        ]);
        item_graph_builder.build()
    };
    let states = {
        let mut states_mut = StatesMut::new();
        states_mut.insert(item_one.clone(), VecCopyState::from(vec![1u8]));
        states_mut.insert(item_two.clone(), MockState(2u8));
        StatesCurrentStored::from(states_mut)
    };
    let mut states_type_reg = TypeReg::new_typed();
    states_type_reg.register::<VecCopyState>(item_one.clone());
    states_type_reg.register::<MockState>(item_two.clone());
    states_type_reg.register::<MockState>(item_three.clone());
    StatesSerializer::<PeaceTestError>::serialize(
        &storage,
        &item_graph,
        &states,
        &states_current_file,
    )
    .await?;

    let states_deserialized = StatesSerializer::<PeaceTestError>::deserialize_stored(
        &flow_id,
        &storage,
        &states_type_reg,
        &states_current_file,
    )
    .await?;

    assert_eq!(
        Some(VecCopyState::from(vec![1u8])),
        states_deserialized
            .get::<VecCopyState, _>(&item_one)
            .cloned()
    );
    assert_eq!(
        Some(MockState(2u8)),
        states_deserialized.get::<MockState, _>(&item_two).cloned()
    );
    assert_eq!(
        None,
        states_deserialized
            .get::<MockState, _>(&item_three)
            .cloned()
    );

    Ok(())
}

#[tokio::test]
async fn deserialize_stored_error_maps_byte_indices() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let flow_id = flow_id!("test_flow");
    let storage = Storage;
    let item_id = item_id!("a");
    let mut states_type_reg = TypeReg::new_typed();
    states_type_reg.register::<u32>(item_id.clone());
    let states_current_file = StatesCurrentFile::new(tempdir.path().join("states_current.yaml"));

    let contents = "a: [123]\n";
    tokio::fs::write(&states_current_file, contents).await?;

    let error = StatesSerializer::<PeaceTestError>::deserialize_stored(
        &flow_id,
        &storage,
        &states_type_reg,
        &states_current_file,
    )
    .await
    .unwrap_err();

    #[cfg(feature = "error_reporting")]
    {
        use peace::miette::SourceOffset;
        let error_span_expected = {
            let line = 1;
            let column = 4;
            Some(SourceOffset::from_location(contents, line, column))
        };

        if let PeaceTestError::PeaceRt(Error::StatesDeserialize {
            flow_id: flow_id_actual,
            states_file_source: _,
            error_span,
            error_message,
            context_span,
            error: _,
        }) = error
        {
            assert_eq!(flow_id, flow_id_actual);
            assert_eq!(error_span_expected, error_span);
            assert_eq!("a: invalid type: sequence, expected u32", error_message);
            assert_eq!(None, context_span);
        } else {
            panic!("Expected error to be `Error::StatesDeserialize {{ .. }}`, but was {error:?}");
        }
    }
    #[cfg(not(feature = "error_reporting"))]
    {
        assert!(matches!(
            error,
            PeaceTestError::PeaceRt(Error::StatesDeserialize {
                flow_id: flow_id_actual,
                error: _
            })
            if flow_id == flow_id_actual
        ));
    }

    Ok(())
}
