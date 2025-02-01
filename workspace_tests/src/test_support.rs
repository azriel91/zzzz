use futures::stream::{self, StreamExt, TryStreamExt};
use peace::{
    cfg::AppName,
    flow_model::FlowId,
    profile_model::Profile,
    resource_rt::{
        internal::{FlowParamsFile, ProfileParamsFile, WorkspaceParamsFile},
        paths::{FlowDir, ProfileDir},
        resources::ts::SetUp,
        Resources,
    },
    rt_model::{
        params::{FlowParams, ProfileParams, WorkspaceParams},
        Error, Storage, Workspace, WorkspaceSpec,
    },
};

pub(crate) fn workspace(
    tempdir: &tempfile::TempDir,
    app_name: AppName,
) -> Result<Workspace, Box<dyn std::error::Error>> {
    let workspace = {
        let workspace_spec = WorkspaceSpec::Path(tempdir.path().to_path_buf());
        Workspace::new(app_name, workspace_spec)?
    };
    Ok(workspace)
}

/// Returns a workspace with profile directories already created within it.
pub(crate) async fn workspace_with(
    tempdir: &tempfile::TempDir,
    app_name: AppName,
    profiles_existing: &[Profile],
    flow_id: Option<&FlowId>,
) -> Result<Workspace, Box<dyn std::error::Error>> {
    let workspace = {
        let workspace_spec = WorkspaceSpec::Path(tempdir.path().to_path_buf());
        Workspace::new(app_name, workspace_spec)?
    };

    let peace_app_dir = workspace.dirs().peace_app_dir();
    tokio::fs::create_dir_all(peace_app_dir).await?;

    let workspace_params_file = WorkspaceParamsFile::from(peace_app_dir);
    let mut workspace_params = WorkspaceParams::new();
    workspace_params.insert(String::from("profile"), profiles_existing[0].clone());
    workspace_params.insert(String::from("ws_param_1"), String::from("ws_param_1_value"));

    Storage
        .serialized_write(
            crate::fn_name_short!().to_string(),
            &workspace_params_file,
            &workspace_params,
            Error::WorkspaceParamsSerialize,
        )
        .await?;

    let profile_dirs = profiles_existing
        .iter()
        .map(|profile| ProfileDir::from((peace_app_dir, profile)));
    stream::iter(profile_dirs)
        .map(Result::<_, Box<dyn std::error::Error>>::Ok)
        .try_for_each(|profile_dir| async move {
            tokio::fs::create_dir_all(&profile_dir).await?;

            let profile_params_file = ProfileParamsFile::from(&profile_dir);
            let mut profile_params = ProfileParams::new();
            profile_params.insert(String::from("profile_param_0"), 1u32);
            profile_params.insert(String::from("profile_param_1"), 2u64);

            Storage
                .serialized_write(
                    crate::fn_name_short!().to_string(),
                    &profile_params_file,
                    &profile_params,
                    Error::ProfileParamsSerialize,
                )
                .await?;

            if let Some(flow_id) = flow_id {
                let flow_dir = FlowDir::from((&profile_dir, flow_id));
                tokio::fs::create_dir_all(&flow_dir).await?;

                let flow_params_file = FlowParamsFile::from(&flow_dir);
                let mut flow_params = FlowParams::new();
                flow_params.insert(String::from("flow_param_0"), true);
                flow_params.insert(String::from("flow_param_1"), 456u16);

                Storage
                    .serialized_write(
                        crate::fn_name_short!().to_string(),
                        &flow_params_file,
                        &flow_params,
                        Error::FlowParamsSerialize,
                    )
                    .await?;
            }

            Ok(())
        })
        .await?;

    Ok(workspace)
}

pub(crate) async fn assert_workspace_params(
    resources: &Resources<SetUp>,
) -> Result<(), std::io::Error> {
    let workspace_params_file = resources.borrow::<WorkspaceParamsFile>();
    let res_ws_param_1 = resources.borrow::<String>();
    assert!(workspace_params_file.exists());
    assert_eq!(
        "profile: test_profile\n\
        ws_param_1: ws_param_1_value\n",
        tokio::fs::read_to_string(&*workspace_params_file).await?
    );
    assert_eq!("ws_param_1_value", res_ws_param_1.as_str());

    Ok(())
}

pub(crate) async fn assert_profile_params(
    resources: &Resources<SetUp>,
) -> Result<(), std::io::Error> {
    let profile_params_file = resources.borrow::<ProfileParamsFile>();
    let res_profile_param_0 = resources.borrow::<u32>();
    let res_profile_param_1 = resources.borrow::<u64>();
    assert!(profile_params_file.exists());
    assert_eq!(
        "profile_param_0: 1\n\
        profile_param_1: 2\n",
        tokio::fs::read_to_string(&*profile_params_file).await?
    );
    assert_eq!(1u32, *res_profile_param_0);
    assert_eq!(2u64, *res_profile_param_1);

    Ok(())
}

pub(crate) async fn assert_flow_params(resources: &Resources<SetUp>) -> Result<(), std::io::Error> {
    let flow_params_file = resources.borrow::<FlowParamsFile>();
    let res_flow_param_0 = resources.borrow::<bool>();
    let res_flow_param_1 = resources.borrow::<u16>();
    assert!(flow_params_file.exists());
    assert_eq!(
        "flow_param_0: true\n\
        flow_param_1: 456\n",
        tokio::fs::read_to_string(&*flow_params_file).await?
    );
    assert!(*res_flow_param_0);
    assert_eq!(456u16, *res_flow_param_1);

    Ok(())
}
