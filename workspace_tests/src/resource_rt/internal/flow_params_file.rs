use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use peace::{
    cfg::{app_name, profile},
    flow_model::flow_id,
    resource_rt::{
        internal::FlowParamsFile,
        paths::{FlowDir, PeaceAppDir, PeaceDir, ProfileDir},
    },
};

#[test]
pub fn debug() {
    let flow_params_file = FlowParamsFile::from(Path::new("flow_params.yaml").to_path_buf());

    assert_eq!(
        r#"FlowParamsFile("flow_params.yaml")"#,
        format!("{flow_params_file:?}")
    );
}

#[test]
pub fn partial_eq() {
    let flow_params_file_0 = FlowParamsFile::from(Path::new("flow_params.yaml").to_path_buf());
    #[allow(clippy::redundant_clone)] // https://github.com/rust-lang/rust-clippy/issues/9011
    let flow_params_file_1 = flow_params_file_0.clone();

    assert_eq!(flow_params_file_0, flow_params_file_1);
}

#[test]
pub fn from_path_buf() {
    let flow_params_file = FlowParamsFile::from(Path::new("flow_params.yaml").to_path_buf());

    assert_eq!(Path::new("flow_params.yaml"), &*flow_params_file);
}

#[test]
pub fn from_flow_dir_relative() {
    let app_name = app_name!();
    let peace_dir = PeaceDir::from(Path::new(".").to_path_buf());
    let profile = profile!("test_profile");
    let peace_app_dir = PeaceAppDir::from((&peace_dir, &app_name));
    let profile_dir = ProfileDir::from((&peace_app_dir, &profile));
    let flow_dir = FlowDir::from((&profile_dir, &flow_id!("test_flow")));
    let flow_params_file = FlowParamsFile::from(&flow_dir);

    let path = PathBuf::from_iter([
        ".",
        &**app_name!(),
        "test_profile",
        "test_flow",
        "flow_params.yaml",
    ]);
    assert_eq!(path, &*flow_params_file);
}

#[test]
pub fn into_inner_returns_path_buf() {
    let flow_params_file = FlowParamsFile::new(Path::new("flow_params.yaml").to_path_buf());

    assert_eq!(
        Path::new("flow_params.yaml").to_path_buf(),
        flow_params_file.into_inner()
    );
}

#[test]
pub fn as_ref_os_str() {
    let flow_params_file = FlowParamsFile::new(Path::new("flow_params.yaml").to_path_buf());

    assert_eq!(
        OsStr::new("flow_params.yaml"),
        <FlowParamsFile as AsRef<OsStr>>::as_ref(&flow_params_file)
    );
}

#[test]
pub fn as_ref_path() {
    let flow_params_file = FlowParamsFile::new(Path::new("flow_params.yaml").to_path_buf());

    assert_eq!(
        Path::new("flow_params.yaml"),
        <FlowParamsFile as AsRef<Path>>::as_ref(&flow_params_file)
    );
}
