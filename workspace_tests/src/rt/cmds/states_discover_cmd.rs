use peace::{
    cfg::{app_name, profile, AppName, FlowId, ItemSpecId, Profile},
    cmd::ctx::CmdCtx,
    resources::{
        paths::{StatesDesiredFile, StatesSavedFile},
        states::{StatesCurrent, StatesDesired, StatesSaved},
        type_reg::untagged::{BoxDtDisplay, TypeReg},
    },
    rt::cmds::StatesDiscoverCmd,
    rt_model::{Flow, ItemSpecGraphBuilder, Workspace, WorkspaceSpec},
};

use crate::{NoOpOutput, PeaceTestError, VecA, VecCopyError, VecCopyItemSpec, VecCopyState};

#[tokio::test]
async fn current_and_desired_discovers_both_states_current_and_desired()
-> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemSpecGraphBuilder::<PeaceTestError>::new();
        graph_builder.add_fn(VecCopyItemSpec::default().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let mut output = NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .with_item_spec_params::<VecCopyItemSpec>(
            VecCopyItemSpec::ID_DEFAULT.clone(),
            VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]),
        )
        .await?;

    let (states_current, states_desired) =
        StatesDiscoverCmd::current_and_desired(&mut cmd_ctx).await?;
    let resources = cmd_ctx.resources();

    let vec_copy_state = states_current.get::<VecCopyState, _>(VecCopyItemSpec::ID_DEFAULT);
    let states_on_disk = {
        let states_saved_file = resources.borrow::<StatesSavedFile>();
        let states_slice = std::fs::read(&*states_saved_file)?;

        let mut type_reg = TypeReg::<ItemSpecId, BoxDtDisplay>::new_typed();
        type_reg.register::<VecCopyState>(VecCopyItemSpec::ID_DEFAULT.clone());

        let deserializer = serde_yaml::Deserializer::from_slice(&states_slice);
        StatesCurrent::from(type_reg.deserialize_map(deserializer)?)
    };
    let vec_copy_desired_state = states_desired.get::<VecCopyState, _>(VecCopyItemSpec::ID_DEFAULT);
    let states_desired_on_disk = {
        let states_desired_file = resources.borrow::<StatesDesiredFile>();
        let states_slice = std::fs::read(&*states_desired_file)?;

        let mut type_reg = TypeReg::<ItemSpecId, BoxDtDisplay>::new_typed();
        type_reg.register::<VecCopyState>(VecCopyItemSpec::ID_DEFAULT.clone());

        let deserializer = serde_yaml::Deserializer::from_slice(&states_slice);
        StatesDesired::from(type_reg.deserialize_map(deserializer)?)
    };
    assert_eq!(Some(VecCopyState::new()).as_ref(), vec_copy_state);
    assert_eq!(
        states_current.get::<VecCopyState, _>(VecCopyItemSpec::ID_DEFAULT),
        states_on_disk.get::<VecCopyState, _>(VecCopyItemSpec::ID_DEFAULT)
    );
    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        vec_copy_desired_state
    );
    assert_eq!(
        states_desired.get::<VecCopyState, _>(VecCopyItemSpec::ID_DEFAULT),
        states_desired_on_disk.get::<VecCopyState, _>(VecCopyItemSpec::ID_DEFAULT)
    );

    Ok(())
}

#[tokio::test]
async fn current_runs_state_current_for_each_item_spec() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemSpecGraphBuilder::<PeaceTestError>::new();
        graph_builder.add_fn(VecCopyItemSpec::default().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let mut output = NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .with_item_spec_params::<VecCopyItemSpec>(
            VecCopyItemSpec::ID_DEFAULT.clone(),
            VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]),
        )
        .await?;

    let states_current = StatesDiscoverCmd::current(&mut cmd_ctx).await?;
    let resources = cmd_ctx.resources();

    let vec_copy_state = states_current.get::<VecCopyState, _>(VecCopyItemSpec::ID_DEFAULT);
    let states_on_disk = {
        let states_saved_file = resources.borrow::<StatesSavedFile>();
        let states_slice = std::fs::read(&*states_saved_file)?;

        let mut type_reg = TypeReg::<ItemSpecId, BoxDtDisplay>::new_typed();
        type_reg.register::<VecCopyState>(VecCopyItemSpec::ID_DEFAULT.clone());

        let deserializer = serde_yaml::Deserializer::from_slice(&states_slice);
        StatesCurrent::from(type_reg.deserialize_map(deserializer)?)
    };
    assert_eq!(Some(VecCopyState::new()).as_ref(), vec_copy_state);
    assert_eq!(
        states_current.get::<VecCopyState, _>(VecCopyItemSpec::ID_DEFAULT),
        states_on_disk.get::<VecCopyState, _>(VecCopyItemSpec::ID_DEFAULT)
    );

    Ok(())
}

#[tokio::test]
async fn current_inserts_states_saved_from_states_saved_file()
-> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemSpecGraphBuilder::<PeaceTestError>::new();
        graph_builder.add_fn(VecCopyItemSpec::default().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let mut output = NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .with_item_spec_params::<VecCopyItemSpec>(
            VecCopyItemSpec::ID_DEFAULT.clone(),
            VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]),
        )
        .await?;

    // Writes to states_saved_file.yaml
    StatesDiscoverCmd::current(&mut cmd_ctx).await?;

    // Execute again to ensure StatesSaved is included
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .with_item_spec_params::<VecCopyItemSpec>(
            VecCopyItemSpec::ID_DEFAULT.clone(),
            VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]),
        )
        .await?;
    StatesDiscoverCmd::current(&mut cmd_ctx).await?;
    let resources = cmd_ctx.resources();

    let states_saved = resources.borrow::<StatesSaved>();
    let vec_copy_state = states_saved.get::<VecCopyState, _>(VecCopyItemSpec::ID_DEFAULT);
    let states_on_disk = {
        let states_saved_file = resources.borrow::<StatesSavedFile>();
        let states_slice = std::fs::read(&*states_saved_file)?;

        let mut type_reg = TypeReg::<ItemSpecId, BoxDtDisplay>::new_typed();
        type_reg.register::<VecCopyState>(VecCopyItemSpec::ID_DEFAULT.clone());

        let deserializer = serde_yaml::Deserializer::from_slice(&states_slice);
        StatesCurrent::from(type_reg.deserialize_map(deserializer)?)
    };
    assert_eq!(Some(VecCopyState::new()).as_ref(), vec_copy_state);
    assert_eq!(
        states_saved.get::<VecCopyState, _>(VecCopyItemSpec::ID_DEFAULT),
        states_on_disk.get::<VecCopyState, _>(VecCopyItemSpec::ID_DEFAULT)
    );

    Ok(())
}

#[tokio::test]
async fn desired_runs_state_desired_for_each_item_spec() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemSpecGraphBuilder::<PeaceTestError>::new();
        graph_builder.add_fn(VecCopyItemSpec::default().into());
        graph_builder.build()
    };
    let flow = Flow::new(FlowId::new(crate::fn_name_short!())?, graph);
    let mut output = NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_single_profile_single_flow(&mut output, &workspace)
        .with_profile(profile!("test_profile"))
        .with_flow(&flow)
        .with_item_spec_params::<VecCopyItemSpec>(
            VecCopyItemSpec::ID_DEFAULT.clone(),
            VecA(vec![0, 1, 2, 3, 4, 5, 6, 7]),
        )
        .await?;

    let states_desired = StatesDiscoverCmd::desired(&mut cmd_ctx).await?;
    let resources = cmd_ctx.resources();

    let vec_copy_desired_state = states_desired.get::<VecCopyState, _>(VecCopyItemSpec::ID_DEFAULT);
    let states_desired_on_disk = {
        let states_desired_file = resources.borrow::<StatesDesiredFile>();
        let states_slice = std::fs::read(&*states_desired_file)?;

        let mut type_reg = TypeReg::<ItemSpecId, BoxDtDisplay>::new_typed();
        type_reg.register::<VecCopyState>(VecCopyItemSpec::ID_DEFAULT.clone());

        let deserializer = serde_yaml::Deserializer::from_slice(&states_slice);
        StatesDesired::from(type_reg.deserialize_map(deserializer)?)
    };
    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        vec_copy_desired_state
    );
    assert_eq!(
        states_desired.get::<VecCopyState, _>(VecCopyItemSpec::ID_DEFAULT),
        states_desired_on_disk.get::<VecCopyState, _>(VecCopyItemSpec::ID_DEFAULT)
    );

    Ok(())
}

#[test]
fn debug() {
    let debug_str = format!(
        "{:?}",
        StatesDiscoverCmd::<VecCopyError, NoOpOutput, ()>::default()
    );
    assert!(
        debug_str
            == r#"StatesDiscoverCmd(PhantomData<(workspace_tests::vec_copy_item_spec::VecCopyError, workspace_tests::no_op_output::NoOpOutput, ())>)"#
            || debug_str == r#"StatesDiscoverCmd(PhantomData)"#
    );
}
