use peace::{
    cfg::{app_name, profile, AppName, FlowId, ItemSpec, Profile},
    resources::states::{StatesEnsured, StatesEnsuredDry, StatesSaved},
    rt::cmds::{sub::StatesSavedReadCmd, EnsureCmd, StatesDiscoverCmd},
    rt_model::{cmd::CmdContext, ItemSpecGraphBuilder, Workspace, WorkspaceSpec},
};

use crate::{NoOpOutput, VecCopyError, VecCopyItemSpec, VecCopyState};

#[tokio::test]
async fn resources_ensured_dry_does_not_alter_state() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemSpecGraphBuilder::<VecCopyError>::new();
        graph_builder.add_fn(VecCopyItemSpec.into());
        graph_builder.build()
    };
    let mut output = NoOpOutput;

    // Write current and desired states to disk.
    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_profile(profile!("test_profile"))
        .with_flow_id(FlowId::new(crate::fn_name_short!())?)
        .await?;
    StatesDiscoverCmd::exec(cmd_context).await?;

    // Re-read states from disk.
    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_profile(profile!("test_profile"))
        .with_flow_id(FlowId::new(crate::fn_name_short!())?)
        .await?;
    let CmdContext { resources, .. } = EnsureCmd::exec_dry(cmd_context).await?;

    let states_ensured_dry = resources.borrow::<StatesEnsuredDry>();

    // TODO: When EnsureCmd returns the execution report, assert on the state that
    // was discovered.
    //
    // ```rust,ignore
    // let states = resources.borrow::<StatesCurrent>();
    // let states_desired = resources.borrow::<StatesDesired>();
    // assert_eq!(
    //     Some(VecCopyState::new()).as_ref(),
    //     states.get::<VecCopyState, _>(&VecCopyItemSpec.id())
    // );
    // assert_eq!(
    //     Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
    //     states_desired
    //         .get::<VecCopyState, _>(&VecCopyItemSpec.id())
    //
    // );
    // ```

    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        states_ensured_dry.get::<VecCopyState, _>(VecCopyItemSpec.id())
    ); // states_ensured_dry should be the same as the beginning.

    Ok(())
}

#[tokio::test]
async fn resources_ensured_contains_state_ensured_for_each_item_spec_when_state_not_yet_ensured()
-> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemSpecGraphBuilder::<VecCopyError>::new();
        graph_builder.add_fn(VecCopyItemSpec.into());
        graph_builder.build()
    };
    let mut output = NoOpOutput;

    // Write current and desired states to disk.
    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_profile(profile!("test_profile"))
        .with_flow_id(FlowId::new(crate::fn_name_short!())?)
        .await?;
    StatesDiscoverCmd::exec(cmd_context).await?;

    // Alter states.
    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_profile(profile!("test_profile"))
        .with_flow_id(FlowId::new(crate::fn_name_short!())?)
        .await?;
    let CmdContext {
        resources: resources_ensured,
        ..
    } = EnsureCmd::exec(cmd_context).await?;

    // Re-read states from disk.
    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_profile(profile!("test_profile"))
        .with_flow_id(FlowId::new(crate::fn_name_short!())?)
        .await?;
    let CmdContext {
        resources: resources_reread,
        ..
    } = StatesSavedReadCmd::exec(cmd_context).await?;

    let ensured_states_ensured = resources_ensured.borrow::<StatesEnsured>();
    let states_saved = resources_reread.borrow::<StatesSaved>();

    // TODO: When EnsureCmd returns the execution report, assert on the state that
    // was discovered.
    //
    // ```rust,ignore
    // let ensured_states_before = resources_ensured.borrow::<StatesCurrent>();
    // let ensured_states_desired = resources_ensured.borrow::<StatesDesired>();
    // assert_eq!(
    //     Some(VecCopyState::new()).as_ref(),
    //     ensured_states_before.get::<VecCopyState, _>(&VecCopyItemSpec.id())
    // );
    // assert_eq!(
    //     Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
    //     ensured_states_desired
    //         .get::<VecCopyState, _>(&VecCopyItemSpec.id())
    //
    // );
    // ```

    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        ensured_states_ensured.get::<VecCopyState, _>(VecCopyItemSpec.id())
    );
    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        states_saved.get::<VecCopyState, _>(VecCopyItemSpec.id())
    );

    Ok(())
}

#[tokio::test]
async fn resources_ensured_contains_state_ensured_for_each_item_spec_when_state_already_ensured()
-> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let workspace = Workspace::new(
        app_name!(),
        WorkspaceSpec::Path(tempdir.path().to_path_buf()),
    )?;
    let graph = {
        let mut graph_builder = ItemSpecGraphBuilder::<VecCopyError>::new();
        graph_builder.add_fn(VecCopyItemSpec.into());
        graph_builder.build()
    };
    let mut output = NoOpOutput;

    // Write current and desired states to disk.
    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_profile(profile!("test_profile"))
        .with_flow_id(FlowId::new(crate::fn_name_short!())?)
        .await?;
    StatesDiscoverCmd::exec(cmd_context).await?;

    // Alter states.
    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_profile(profile!("test_profile"))
        .with_flow_id(FlowId::new(crate::fn_name_short!())?)
        .await?;
    let CmdContext {
        resources: resources_ensured,
        ..
    } = EnsureCmd::exec(cmd_context).await?;

    // Dry ensure states.
    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_profile(profile!("test_profile"))
        .with_flow_id(FlowId::new(crate::fn_name_short!())?)
        .await?;
    let CmdContext {
        resources: resources_ensured_dry,
        ..
    } = EnsureCmd::exec_dry(cmd_context).await?;

    // Re-read states from disk.
    let cmd_context = CmdContext::builder(&workspace, &graph, &mut output)
        .with_profile(profile!("test_profile"))
        .with_flow_id(FlowId::new(crate::fn_name_short!())?)
        .await?;
    let CmdContext {
        resources: resources_reread,
        ..
    } = StatesSavedReadCmd::exec(cmd_context).await?;

    let ensured_states_ensured = resources_ensured.borrow::<StatesEnsured>();
    let ensured_states_ensured_dry = resources_ensured_dry.borrow::<StatesEnsuredDry>();
    let states_saved = resources_reread.borrow::<StatesSaved>();

    // TODO: When EnsureCmd returns the execution report, assert on the state that
    // was discovered.
    //
    // ```rust,ignore
    // let ensured_states_before = resources_ensured.borrow::<StatesCurrent>();
    // let ensured_states_desired = resources_ensured.borrow::<StatesDesired>();
    // assert_eq!(
    //     Some(VecCopyState::new()).as_ref(),
    //     ensured_states_before.get::<VecCopyState,
    // _>(&VecCopyItemSpec.id()) );
    // assert_eq!(
    //     Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
    //     ensured_states_desired
    //         .get::<VecCopyState, _>(&VecCopyItemSpec.id())
    //
    // );
    // ```
    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        ensured_states_ensured.get::<VecCopyState, _>(VecCopyItemSpec.id())
    ); // states_ensured.logical should be the same as states desired, if all went well.
    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        ensured_states_ensured_dry.get::<VecCopyState, _>(VecCopyItemSpec.id())
    ); // TODO: EnsureDry state should simulate the actual states, not return the actual current state
    assert_eq!(
        Some(VecCopyState::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])).as_ref(),
        states_saved.get::<VecCopyState, _>(VecCopyItemSpec.id())
    );

    Ok(())
}
