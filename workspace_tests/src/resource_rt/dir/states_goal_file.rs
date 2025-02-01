use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use peace::{
    cfg::{app_name, profile},
    flow_model::flow_id,
    resource_rt::paths::{FlowDir, PeaceAppDir, PeaceDir, ProfileDir, StatesGoalFile},
};

#[test]
pub fn debug() {
    let states_goal_file = StatesGoalFile::from(Path::new("test_states_goal.yaml").to_path_buf());

    assert_eq!(
        r#"StatesGoalFile("test_states_goal.yaml")"#,
        format!("{states_goal_file:?}")
    );
}

#[test]
pub fn partial_eq() {
    let states_goal_file_0 = StatesGoalFile::from(Path::new("test_states_goal.yaml").to_path_buf());
    #[allow(clippy::redundant_clone)] // https://github.com/rust-lang/rust-clippy/issues/9011
    let states_goal_file_1 = states_goal_file_0.clone();

    assert_eq!(states_goal_file_0, states_goal_file_1);
}

#[test]
pub fn from_path_buf() {
    let states_goal_file = StatesGoalFile::from(Path::new("test_states_goal.yaml").to_path_buf());

    assert_eq!(Path::new("test_states_goal.yaml"), &*states_goal_file);
}

#[test]
pub fn from_flow_dir_relative() {
    let app_name = app_name!();
    let peace_dir = PeaceDir::from(Path::new(".").to_path_buf());
    let profile = profile!("test_profile");
    let peace_app_dir = PeaceAppDir::from((&peace_dir, &app_name));
    let profile_dir = ProfileDir::from((&peace_app_dir, &profile));
    let flow_dir = FlowDir::from((&profile_dir, &flow_id!("test_flow")));
    let states_goal_file = StatesGoalFile::from(&flow_dir);

    let path = PathBuf::from_iter([
        ".",
        &**app_name!(),
        "test_profile",
        "test_flow",
        "states_goal.yaml",
    ]);
    assert_eq!(path, &*states_goal_file);
}

#[test]
pub fn into_inner_returns_path_buf() {
    let states_goal_file = StatesGoalFile::new(Path::new("test_states_goal.yaml").to_path_buf());

    assert_eq!(
        Path::new("test_states_goal.yaml").to_path_buf(),
        states_goal_file.into_inner()
    );
}

#[test]
pub fn as_ref_os_str() {
    let states_goal_file = StatesGoalFile::new(Path::new("test_states_goal.yaml").to_path_buf());

    assert_eq!(
        OsStr::new("test_states_goal.yaml"),
        <StatesGoalFile as AsRef<OsStr>>::as_ref(&states_goal_file)
    );
}

#[test]
pub fn as_ref_path() {
    let states_goal_file = StatesGoalFile::new(Path::new("test_states_goal.yaml").to_path_buf());

    assert_eq!(
        Path::new("test_states_goal.yaml"),
        <StatesGoalFile as AsRef<Path>>::as_ref(&states_goal_file)
    );
}
