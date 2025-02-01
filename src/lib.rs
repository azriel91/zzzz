//! 🕊️ peace -- zero stress automation

// Re-exports so consumers don't need to depend on crates individually.
#[cfg(feature = "error_reporting")]
pub use miette;

pub use peace_cfg as cfg;
#[cfg(feature = "cli")]
pub use peace_cli as cli;
#[cfg(feature = "cli")]
pub use peace_cli_model as cli_model;
pub use peace_cmd as cmd;
pub use peace_cmd_model as cmd_model;
pub use peace_cmd_rt as cmd_rt;
pub use peace_data as data;
pub use peace_diff as diff;
pub use peace_flow_model as flow_model;
pub use peace_flow_rt as flow_rt;
pub use peace_fmt as fmt;
#[cfg(feature = "item_interactions")]
pub use peace_item_interaction_model as item_interaction_model;
pub use peace_item_model as item_model;
pub use peace_params as params;
pub use peace_profile_model as profile_model;
#[cfg(feature = "output_progress")]
pub use peace_progress_model as progress_model;
pub use peace_resource_rt as resource_rt;
pub use peace_rt as rt;
pub use peace_rt_model as rt_model;
pub use peace_state_rt as state_rt;
#[cfg(feature = "webi")]
pub use peace_webi as webi;
#[cfg(feature = "webi")]
pub use peace_webi_components as webi_components;
#[cfg(feature = "webi")]
pub use peace_webi_model as webi_model;

// We still can't build with `--all-features`, even with `indicatif 0.17.4`.
//
// The error we get is the same as in
// <https://github.com/rustwasm/wasm-bindgen/issues/2160>.
//
// This likely means at least one of the transitive dependencies of `indicatif`
// uses `std::env::*`.
//
// `console` is at lesat one of these dependencies.
#[cfg(all(target_arch = "wasm32", feature = "output_progress"))]
compile_error!(
    r#"The `"output_progress"` feature does not work on WASM, pending support from `indicatif`.
See <https://github.com/console-rs/indicatif/issues/513>."#
);
