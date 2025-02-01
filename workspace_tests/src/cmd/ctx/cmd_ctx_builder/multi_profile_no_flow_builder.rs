use std::collections::BTreeMap;

use peace::{
    cfg::app_name,
    cmd::ctx::CmdCtx,
    profile_model::{profile, Profile, ProfileInvalidFmt},
    resource_rt::paths::{ProfileDir, ProfileHistoryDir},
    rt_model::{params::ParamsTypeRegs, Error as PeaceRtError, NativeError},
};

use crate::{no_op_output::NoOpOutput, test_support::workspace_with, PeaceTestError};

#[tokio::test]
async fn build() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let profile = profile!("test_profile");
    let profile_other = profile!("test_profile_other");
    let workspace = workspace_with(
        &tempdir,
        app_name!("test_multi_profile_no_flow"),
        &[profile.clone(), profile_other.clone()],
        None,
    )
    .await?;

    let output = NoOpOutput;
    let cmd_ctx = CmdCtx::builder_multi_profile_no_flow::<PeaceTestError, _>(
        output.into(),
        (&workspace).into(),
    )
    .build()
    .await?;

    let peace_app_dir = workspace.dirs().peace_app_dir();
    let (profile_dirs, profile_history_dirs) = {
        let mut profile_dirs = BTreeMap::new();
        let mut profile_history_dirs = BTreeMap::new();

        let profile_dir = ProfileDir::from((peace_app_dir, &profile));
        let profile_history_dir = ProfileHistoryDir::from(&profile_dir);

        let profile_other_dir = ProfileDir::from((peace_app_dir, &profile_other));
        let profile_other_history_dir = ProfileHistoryDir::from(&profile_other_dir);

        profile_dirs.insert(profile.clone(), profile_dir);
        profile_dirs.insert(profile_other.clone(), profile_other_dir);

        profile_history_dirs.insert(profile.clone(), profile_history_dir);
        profile_history_dirs.insert(profile_other.clone(), profile_other_history_dir);

        (profile_dirs, profile_history_dirs)
    };

    let scope = cmd_ctx.scope();
    assert!(std::ptr::eq(&workspace, cmd_ctx.workspace()));
    assert_eq!(peace_app_dir, cmd_ctx.workspace().dirs().peace_app_dir());
    assert_eq!(&[profile.clone(), profile_other], scope.profiles());
    assert_eq!(&profile_dirs, scope.profile_dirs());
    assert_eq!(&profile_history_dirs, scope.profile_history_dirs());
    Ok(())
}

#[tokio::test]
async fn build_with_workspace_params() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let profile = profile!("test_profile");
    let profile_other = profile!("test_profile_other");
    let workspace = workspace_with(
        &tempdir,
        app_name!("test_multi_profile_no_flow"),
        &[profile.clone(), profile_other.clone()],
        None,
    )
    .await?;

    let output = NoOpOutput;
    let cmd_ctx = CmdCtx::builder_multi_profile_no_flow::<PeaceTestError, _>(
        output.into(),
        (&workspace).into(),
    )
    .with_workspace_param_value(String::from("profile"), Some(profile.clone()))
    .with_workspace_param_value(
        String::from("ws_param_1"),
        Some("ws_param_1_value".to_string()),
    )
    .build()
    .await?;

    let peace_app_dir = workspace.dirs().peace_app_dir();
    let (profile_dirs, profile_history_dirs) = {
        let mut profile_dirs = BTreeMap::new();
        let mut profile_history_dirs = BTreeMap::new();

        let profile_dir = ProfileDir::from((peace_app_dir, &profile));
        let profile_history_dir = ProfileHistoryDir::from(&profile_dir);

        let profile_other_dir = ProfileDir::from((peace_app_dir, &profile_other));
        let profile_other_history_dir = ProfileHistoryDir::from(&profile_other_dir);

        profile_dirs.insert(profile.clone(), profile_dir);
        profile_dirs.insert(profile_other.clone(), profile_other_dir);

        profile_history_dirs.insert(profile.clone(), profile_history_dir);
        profile_history_dirs.insert(profile_other.clone(), profile_other_history_dir);

        (profile_dirs, profile_history_dirs)
    };

    let scope = cmd_ctx.scope();
    let workspace_params = scope.workspace_params();
    assert!(std::ptr::eq(&workspace, cmd_ctx.workspace()));
    assert_eq!(peace_app_dir, cmd_ctx.workspace().dirs().peace_app_dir());
    assert_eq!(&[profile.clone(), profile_other], scope.profiles());
    assert_eq!(&profile_dirs, scope.profile_dirs());
    assert_eq!(&profile_history_dirs, scope.profile_history_dirs());
    assert_eq!(Some(&profile), workspace_params.get("profile"));
    assert_eq!(
        Some(&"ws_param_1_value".to_string()),
        workspace_params.get("ws_param_1")
    );
    Ok(())
}

#[tokio::test]
async fn build_with_profile_params() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let profile = profile!("test_profile");
    let profile_other = profile!("test_profile_other");
    let workspace = workspace_with(
        &tempdir,
        app_name!("test_multi_profile_no_flow"),
        &[profile.clone(), profile_other.clone()],
        None,
    )
    .await?;

    let output = NoOpOutput;
    let cmd_ctx = CmdCtx::builder_multi_profile_no_flow::<PeaceTestError, _>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile_params_k::<String>()
    .with_profile_param::<u32>(String::from("profile_param_0"))
    .with_profile_param::<u64>(String::from("profile_param_1"))
    .build()
    .await?;

    let peace_app_dir = workspace.dirs().peace_app_dir();
    let (profile_dirs, profile_history_dirs) = {
        let mut profile_dirs = BTreeMap::new();
        let mut profile_history_dirs = BTreeMap::new();

        let profile_dir = ProfileDir::from((peace_app_dir, &profile));
        let profile_history_dir = ProfileHistoryDir::from(&profile_dir);

        let profile_other_dir = ProfileDir::from((peace_app_dir, &profile_other));
        let profile_other_history_dir = ProfileHistoryDir::from(&profile_other_dir);

        profile_dirs.insert(profile.clone(), profile_dir);
        profile_dirs.insert(profile_other.clone(), profile_other_dir);

        profile_history_dirs.insert(profile.clone(), profile_history_dir);
        profile_history_dirs.insert(profile_other.clone(), profile_other_history_dir);

        (profile_dirs, profile_history_dirs)
    };

    let scope = cmd_ctx.scope();
    assert!(std::ptr::eq(&workspace, cmd_ctx.workspace()));
    assert_eq!(peace_app_dir, cmd_ctx.workspace().dirs().peace_app_dir());
    assert_eq!(&[profile.clone(), profile_other], scope.profiles());
    assert_eq!(&profile_dirs, scope.profile_dirs());
    assert_eq!(&profile_history_dirs, scope.profile_history_dirs());
    Ok(())
}

#[tokio::test]
async fn build_with_workspace_params_with_profile_params() -> Result<(), Box<dyn std::error::Error>>
{
    let tempdir = tempfile::tempdir()?;
    let profile = profile!("test_profile");
    let profile_other = profile!("test_profile_other");
    let workspace = workspace_with(
        &tempdir,
        app_name!("test_multi_profile_no_flow"),
        &[profile.clone(), profile_other.clone()],
        None,
    )
    .await?;

    let output = NoOpOutput;
    let cmd_ctx = CmdCtx::builder_multi_profile_no_flow::<PeaceTestError, _>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile_params_k::<String>()
    .with_profile_param::<u32>(String::from("profile_param_0"))
    .with_workspace_param_value(String::from("profile"), Some(profile.clone()))
    .with_profile_param::<u64>(String::from("profile_param_1"))
    .with_workspace_param_value(
        String::from("ws_param_1"),
        Some("ws_param_1_value".to_string()),
    )
    .build()
    .await?;

    let peace_app_dir = workspace.dirs().peace_app_dir();
    let (profile_dirs, profile_history_dirs) = {
        let mut profile_dirs = BTreeMap::new();
        let mut profile_history_dirs = BTreeMap::new();

        let profile_dir = ProfileDir::from((peace_app_dir, &profile));
        let profile_history_dir = ProfileHistoryDir::from(&profile_dir);

        let profile_other_dir = ProfileDir::from((peace_app_dir, &profile_other));
        let profile_other_history_dir = ProfileHistoryDir::from(&profile_other_dir);

        profile_dirs.insert(profile.clone(), profile_dir);
        profile_dirs.insert(profile_other.clone(), profile_other_dir);

        profile_history_dirs.insert(profile.clone(), profile_history_dir);
        profile_history_dirs.insert(profile_other.clone(), profile_other_history_dir);

        (profile_dirs, profile_history_dirs)
    };

    let scope = cmd_ctx.scope();
    let workspace_params = scope.workspace_params();
    let profile_to_profile_params = scope.profile_to_profile_params();
    let profile_params = profile_to_profile_params
        .get(&profile)
        .expect("Expected profile params to exist.");
    assert!(std::ptr::eq(&workspace, cmd_ctx.workspace()));
    assert_eq!(peace_app_dir, cmd_ctx.workspace().dirs().peace_app_dir());
    assert_eq!(&[profile.clone(), profile_other], scope.profiles());
    assert_eq!(&profile_dirs, scope.profile_dirs());
    assert_eq!(&profile_history_dirs, scope.profile_history_dirs());
    assert_eq!(Some(&profile), workspace_params.get("profile"));
    assert_eq!(
        Some(&"ws_param_1_value".to_string()),
        workspace_params.get("ws_param_1")
    );
    assert_eq!(Some(&1u32), profile_params.get("profile_param_0"));
    assert_eq!(Some(&2u64), profile_params.get("profile_param_1"));
    Ok(())
}

#[tokio::test]
async fn build_with_workspace_params_with_profile_filter() -> Result<(), Box<dyn std::error::Error>>
{
    let tempdir = tempfile::tempdir()?;
    let profile = profile!("test_profile");
    let profile_other = profile!("test_profile_other");
    let workspace = workspace_with(
        &tempdir,
        app_name!("test_multi_profile_no_flow"),
        &[profile.clone(), profile_other.clone()],
        None,
    )
    .await?;

    let output = NoOpOutput;
    let cmd_ctx = CmdCtx::builder_multi_profile_no_flow::<PeaceTestError, _>(
        output.into(),
        (&workspace).into(),
    )
    .with_workspace_param_value(String::from("profile"), Some(profile.clone()))
    .with_workspace_param_value(
        String::from("ws_param_1"),
        Some("ws_param_1_value".to_string()),
    )
    .with_profile_filter(|profile| **profile == "test_profile")
    .build()
    .await?;

    let peace_app_dir = workspace.dirs().peace_app_dir();
    let (profile_dirs, profile_history_dirs) = {
        let mut profile_dirs = BTreeMap::new();
        let mut profile_history_dirs = BTreeMap::new();

        let profile_dir = ProfileDir::from((peace_app_dir, &profile));
        let profile_history_dir = ProfileHistoryDir::from(&profile_dir);

        profile_dirs.insert(profile.clone(), profile_dir);

        profile_history_dirs.insert(profile.clone(), profile_history_dir);

        (profile_dirs, profile_history_dirs)
    };

    let scope = cmd_ctx.scope();
    let workspace_params = scope.workspace_params();
    assert!(std::ptr::eq(&workspace, cmd_ctx.workspace()));
    assert_eq!(peace_app_dir, cmd_ctx.workspace().dirs().peace_app_dir());
    assert_eq!(&[profile.clone()], scope.profiles());
    assert_eq!(&profile_dirs, scope.profile_dirs());
    assert_eq!(&profile_history_dirs, scope.profile_history_dirs());
    assert_eq!(Some(&profile), workspace_params.get("profile"));
    assert_eq!(
        Some(&"ws_param_1_value".to_string()),
        workspace_params.get("ws_param_1")
    );
    Ok(())
}

#[tokio::test]
async fn build_with_workspace_params_with_profile_params_with_profile_filter(
) -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let profile = profile!("test_profile");
    let profile_other = profile!("test_profile_other");
    let workspace = workspace_with(
        &tempdir,
        app_name!("test_multi_profile_no_flow"),
        &[profile.clone(), profile_other.clone()],
        None,
    )
    .await?;

    let output = NoOpOutput;
    let cmd_ctx = CmdCtx::builder_multi_profile_no_flow::<PeaceTestError, _>(
        output.into(),
        (&workspace).into(),
    )
    .with_profile_params_k::<String>()
    .with_profile_param::<u32>(String::from("profile_param_0"))
    .with_workspace_param_value(String::from("profile"), Some(profile.clone()))
    .with_profile_param::<u64>(String::from("profile_param_1"))
    .with_workspace_param_value(
        String::from("ws_param_1"),
        Some("ws_param_1_value".to_string()),
    )
    .with_profile_filter(|profile| **profile == "test_profile")
    .build()
    .await?;

    let peace_app_dir = workspace.dirs().peace_app_dir();
    let (profile_dirs, profile_history_dirs) = {
        let mut profile_dirs = BTreeMap::new();
        let mut profile_history_dirs = BTreeMap::new();

        let profile_dir = ProfileDir::from((peace_app_dir, &profile));
        let profile_history_dir = ProfileHistoryDir::from(&profile_dir);

        profile_dirs.insert(profile.clone(), profile_dir);

        profile_history_dirs.insert(profile.clone(), profile_history_dir);

        (profile_dirs, profile_history_dirs)
    };

    let scope = cmd_ctx.scope();
    let workspace_params = scope.workspace_params();
    let profile_to_profile_params = scope.profile_to_profile_params();
    let profile_params = profile_to_profile_params
        .get(&profile)
        .expect("Expected profile params to exist.");
    assert!(std::ptr::eq(&workspace, cmd_ctx.workspace()));
    assert_eq!(peace_app_dir, cmd_ctx.workspace().dirs().peace_app_dir());
    assert_eq!(&[profile.clone()], scope.profiles());
    assert_eq!(&profile_dirs, scope.profile_dirs());
    assert_eq!(&profile_history_dirs, scope.profile_history_dirs());
    assert_eq!(Some(&profile), workspace_params.get("profile"));
    assert_eq!(
        Some(&"ws_param_1_value".to_string()),
        workspace_params.get("ws_param_1")
    );
    assert_eq!(Some(&1u32), profile_params.get("profile_param_0"));
    assert_eq!(Some(&2u64), profile_params.get("profile_param_1"));
    Ok(())
}

#[tokio::test]
async fn list_profile_dirs_invalid_profile_name() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let profile = profile!("test_profile");
    // sneaky usage of `new_unchecked`.
    let profile_other = Profile::new_unchecked("test_profile_spécïál");
    let workspace = workspace_with(
        &tempdir,
        app_name!("test_multi_profile_no_flow"),
        &[profile.clone(), profile_other.clone()],
        None,
    )
    .await?;

    let output = NoOpOutput;
    let cmd_ctx_result = CmdCtx::builder_multi_profile_no_flow::<PeaceTestError, _>(
        output.into(),
        (&workspace).into(),
    )
    .build()
    .await;

    let profile_other_dir = ProfileDir::from((workspace.dirs().peace_app_dir(), &profile_other));
    ({
        #[cfg_attr(coverage_nightly, coverage(off))]
        || {
            assert!(
                matches!(
                    &cmd_ctx_result,
                    Err(PeaceTestError::PeaceRt(PeaceRtError::Native(
                        NativeError::ProfileDirInvalidName {
                            dir_name,
                            path,
                            error,
                        }
                    )))

                    if dir_name == "test_profile_spécïál"
                    && path == &*profile_other_dir
                    && error == &ProfileInvalidFmt::new("test_profile_spécïál".into())
                ),
                "expected `cmd_ctx_result` to be \n\
                `Err(PeaceTestError::PeaceRt(PeaceRtError::Native(NativeError::ProfileDirInvalidName {{ .. }})))`,\n\
                but was {cmd_ctx_result:?}",
            );
        }
    })();

    Ok(())
}

#[tokio::test]
async fn getters() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let profile = profile!("test_profile");
    let profile_other = profile!("test_profile_other");
    let workspace = workspace_with(
        &tempdir,
        app_name!("test_multi_profile_no_flow"),
        &[profile.clone(), profile_other.clone()],
        None,
    )
    .await?;

    let output = NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_multi_profile_no_flow::<PeaceTestError, _>(
        output.into(),
        (&workspace).into(),
    )
    .build()
    .await?;

    assert_eq!(workspace.dirs().workspace_dir(), cmd_ctx.workspace_dir());
    assert_eq!(workspace.dirs().peace_dir(), cmd_ctx.peace_dir());
    assert_eq!(workspace.dirs().peace_app_dir(), cmd_ctx.peace_app_dir());
    assert!(matches!(cmd_ctx.output(), NoOpOutput));
    assert!(matches!(cmd_ctx.output_mut(), NoOpOutput));
    assert!(matches!(cmd_ctx.params_type_regs(), ParamsTypeRegs { .. }));

    Ok(())
}

#[tokio::test]
async fn debug() -> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    let profile = profile!("test_profile");
    let profile_other = profile!("test_profile_other");
    let workspace = workspace_with(
        &tempdir,
        app_name!("test_multi_profile_no_flow"),
        &[profile.clone(), profile_other.clone()],
        None,
    )
    .await?;

    let output = NoOpOutput;
    let mut cmd_ctx = CmdCtx::builder_multi_profile_no_flow::<PeaceTestError, _>(
        output.into(),
        (&workspace).into(),
    )
    .build()
    .await?;

    let multi_profile_no_flow = cmd_ctx.scope();
    assert!(format!("{multi_profile_no_flow:?}").contains("MultiProfileNoFlow {"));

    let multi_profile_no_flow_view = cmd_ctx.view();
    assert!(format!("{multi_profile_no_flow_view:?}").contains("MultiProfileNoFlowView {"));

    Ok(())
}
