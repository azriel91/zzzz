# Changelog

## unreleased

* Split `flow_rt` and `state_rt` crates from `rt_model`. ([#187], [#202])
* Split `peace_core::progress` into `peace_progress_model` crate. ([#187], [#202])
* Make `ValueResolutionCtx` and `FieldNameAndType` serializable. ([#187], [#202])


[#202]: https://github.com/azriel91/peace/pull/202


## 0.0.14 (2025-01-18)

* Move `Cli*` types to `peace_cli` crate under `cli::output` module. ([#182], [#189])
* Move `OutputFormat` and `OutputFormatParseError` to `peace_cli_model` crate. ([#182], [#189])
* Render progress and outcome diagram using `dot_ix`. ([#182], [#189], [#191])
* Experimented renaming `Item` trait to `Step` (reverted). ([#187], [#192])
* Rename `peace_resources` crate to `peace_resource_rt`. ([#182], [#187], [#193], [#194])
* Add experimental web frontend using `leptos 0.7`. ([#182], [#197], [#200])


[#182]: https://github.com/azriel91/peace/issues/182
[#187]: https://github.com/azriel91/peace/issues/187
[#189]: https://github.com/azriel91/peace/pull/189
[#191]: https://github.com/azriel91/peace/pull/191
[#192]: https://github.com/azriel91/peace/pull/192
[#193]: https://github.com/azriel91/peace/pull/193
[#194]: https://github.com/azriel91/peace/pull/194
[#197]: https://github.com/azriel91/peace/pull/197
[#200]: https://github.com/azriel91/peace/pull/200


## 0.0.13 (2024-02-03)

* Provide more accurate feedback about interruption on CLI. ([#172], [#173])
* Remove requirement to import `peace::cfg::AppName` when using `app_name!("..")` macro. ([#157], [#176])
* Remove requirement to import `peace::cfg::FlowId` when using `flow_id!("..")` macro. ([#157], [#176])
* Remove requirement to import `peace::cfg::ItemId` when using `item_id!("..")` macro. ([#157], [#176])
* Remove requirement to import `peace::cfg::Profile` when using `profile!("..")` macro. ([#157], [#176])
* Add `CmdCtxTypes` to group error, output, and params keys into one type parameter. ([#166], [#177])
* Add `Presenter::list_numbered_aligned` and `list_bulleted_aligned`. ([#151], [#178])
* Add `ListNumberedAligned`, `ListBulleted`, `ListBulletedAligned`. ([#151], [#178])
* Add `Either`, `PresentableExt`. ([#151], [#178])
* Remove `TS` type parameter from `SingleProfileSingleFlow` and `MultiProfileSingleFlow` scopes. ([#179], [#180])


[#172]: https://github.com/azriel91/peace/issues/172
[#173]: https://github.com/azriel91/peace/pull/173
[#157]: https://github.com/azriel91/peace/issues/157
[#176]: https://github.com/azriel91/peace/pull/176
[#166]: https://github.com/azriel91/peace/issues/166
[#177]: https://github.com/azriel91/peace/pull/177
[#151]: https://github.com/azriel91/peace/issues/151
[#178]: https://github.com/azriel91/peace/pull/178
[#179]: https://github.com/azriel91/peace/issues/179
[#180]: https://github.com/azriel91/peace/pull/180


## 0.0.12 (2023-12-30)

* Change `CmdOutcome` to be an `enum` indicating whether it is completed, interrupted, or erroneous. ([#141], [#163])
* Add `CmdBlock` trait to encompass one function for all items. ([#141], [#163])
* Add interruptibility support using [`interruptible`] through `CmdCtxBuilder::with_interruptibility`. ([#141], [#163])
* Add `ItemStreamOutcome` to track which `Item`s are processed or not processed. ([#164], [#165])
* Suppress progress rendering for `StatesCurrentReadCmd`, `StatesGoalReadCmd`, and `DiffCmd`. ([#167], [#168])
* Suppress control character echo on `stdin`, which removes progress bar rendering artifact. ([#169], [#170])


[`interruptible`]: https://github.com/azriel91/interruptible
[#141]: https://github.com/azriel91/peace/issues/141
[#163]: https://github.com/azriel91/peace/pull/163
[#164]: https://github.com/azriel91/peace/issues/164
[#165]: https://github.com/azriel91/peace/pull/165
[#167]: https://github.com/azriel91/peace/issues/167
[#168]: https://github.com/azriel91/peace/pull/168
[#169]: https://github.com/azriel91/peace/issues/169
[#170]: https://github.com/azriel91/peace/pull/170


## 0.0.11 (2023-06-27)

* Add `CmdBase` and `CmdIndependence` for easier command composition. ([#120], [#130])
* Rename `StatesDesired*` to `StatesGoal*`. ([#131], [#132])
* Add `StatesGoalStored` to distinguish between stored and discovered `StatesGoal`. ([#131], [#132])
* `DiffCmd::diff{,_with}` supports discovery of state during diffing. ([#133], [#134])
* Add `PartialEq` bound to `Item::State`. ([#59], [#135])
* Guard `EnsureCmd::{exec,exec_dry}` if stored current state or goal state is not in sync with actual. ([#59], [#135])
* Guard `CleanCmd::{exec,exec_dry}` if stored current state is not in sync with actual. ([#59], [#135])
* Add `*Cmd::*_with` for command logic to be executed as sub commands. ([#59], [#135])
* Removed "output_colorized" feature, and always include colorized output. ([#136], [#137])
* Removed "output_json" feature, and always include json output format. ([#136], [#137])
* Add tests where useful cases are missed. ([#136], [#137])
* Ignore unnecessary missed lines (e.g. panics in tests). ([#136], [#137])
* Remove `peace_item_sh_sync_cmd`. ([#136], [#137])
* Always store an entry for each item's state in `states_*.yaml`, in order of item insertion. ([#138], [#139])

[#120]: https://github.com/azriel91/peace/issues/120
[#130]: https://github.com/azriel91/peace/pull/130
[#131]: https://github.com/azriel91/peace/issues/131
[#132]: https://github.com/azriel91/peace/pull/132
[#133]: https://github.com/azriel91/peace/issues/133
[#134]: https://github.com/azriel91/peace/pull/134
[#59]: https://github.com/azriel91/peace/issues/59
[#135]: https://github.com/azriel91/peace/pull/135
[#136]: https://github.com/azriel91/peace/issues/136
[#137]: https://github.com/azriel91/peace/pull/137
[#138]: https://github.com/azriel91/peace/issues/138
[#139]: https://github.com/azriel91/peace/pull/139


## 0.0.10 (2023-06-03)

* Add `Item::Params` associated type. ([#116], [#117])
* Rename `OpCtx` to `FnCtx`. ([#116], [#117])
* Update `Item` functions to take in `Self::Params`. ([#116], [#117])
* Implement referential item param values. ([#94], [#118])
* Add `Params` derive. ([#94], [#118])
* Add `ParamsSpecs` to `*SingleFlow` command context scopes. ([#94], [#118])
* Take in `Params::Spec`s in `CmdCtxBuilder::with_item_params`. ([#94], [#118])
* Use `Params::Partial` in `Item::try_state_*` functions. ([#94], [#118])
* Implement one level recursion referential item params. ([#119], [#121])
* Implement deep merging of params specs. ([#122], [#123])
* Calculate padding for progress bar item IDs. ([#46], [#124])
* Implement `Clone`, `PartialEq` for `Flow`. ([#125], [#126])

[#116]: https://github.com/azriel91/peace/issues/116
[#117]: https://github.com/azriel91/peace/pull/117
[#94]: https://github.com/azriel91/peace/issues/94
[#118]: https://github.com/azriel91/peace/pull/118
[#119]: https://github.com/azriel91/peace/issues/119
[#121]: https://github.com/azriel91/peace/pull/121
[#122]: https://github.com/azriel91/peace/issues/122
[#123]: https://github.com/azriel91/peace/pull/123
[#46]: https://github.com/azriel91/peace/issues/46
[#124]: https://github.com/azriel91/peace/pull/124
[#125]: https://github.com/azriel91/peace/issues/125
[#126]: https://github.com/azriel91/peace/pull/126


## 0.0.9 (2023-04-13)

* Rename `app_cycle` example to `envman`. ([#35], [#107])
* Develop `envman` example to have sensible errors and state display messages. ([#35], [#107])
* Return both command outcome and errors in `ApplyCmd`. ([#107])
* `StatesDiscoverCmd` discovers states concurrently. ([#107])
* Serialize `StatesDesired` as part of `ApplyCmd`. ([#107])
* Consolidate `Item` functions into single trait. ([#96], [#109])
* Remove `StatesCurrentDiscoverCmd` and `StatesDesiredDiscoverCmd`. ([#110], [#111])
* Update `DiffCmd` to take in states to diff. ([#101], [#112])
* Add `DiffCmd::current_and_desired` to diff current and desired states of a profile. ([#113], [#114])
* Add `DiffCmd::diff_profiles_current` to diff current states of two profiles. ([#113], [#114])

[#107]: https://github.com/azriel91/peace/pull/107
[#96]: https://github.com/azriel91/peace/issues/96
[#109]: https://github.com/azriel91/peace/pull/109
[#110]: https://github.com/azriel91/peace/issues/110
[#111]: https://github.com/azriel91/peace/pull/111
[#101]: https://github.com/azriel91/peace/issues/101
[#112]: https://github.com/azriel91/peace/pull/112
[#113]: https://github.com/azriel91/peace/issues/113
[#114]: https://github.com/azriel91/peace/pull/114


## 0.0.8 (2023-03-25)

* Move `R, W, ROpt, WOpt, RMaybe, WMaybe` to `peace_data::accessors`. ([#94], [#95])
* Automatically insert `Current<Item::State>` after state current and ensure exec executions. ([#94], [#95])
* Automatically insert `Desired<Item::State>` after state desired discover execution. ([#94], [#95])
* Consolidate `EnsureOpSpec` and `CleanOpSpec` into `ApplyOpSpec`. ([#67], [#99])
* Add icons to CLI progress bars. ([#102], [#103])
* Add elapsed / ETA time to CLI progress bars. ([#102], [#103])
* Display messages in CLI progress bars. ([#102], [#103])
* Display progress bars during state discovery. ([#100], [#104])
* Clear progress bars on command end. ([#100], [#104])
* Include entry for current and discovered states, and diff in `envman` example. ([#91], [#105])
* Sort progress bars based on insertion order. ([#91], [#105])
* Use `▰` and `▱` parallelogram characters for progress bars. ([#91], [#105])
* Spinner progress is now rendered. ([#91], [#105])

[#94]: https://github.com/azriel91/peace/issues/94
[#95]: https://github.com/azriel91/peace/pull/95
[#67]: https://github.com/azriel91/peace/issues/67
[#99]: https://github.com/azriel91/peace/pull/99
[#102]: https://github.com/azriel91/peace/issues/102
[#103]: https://github.com/azriel91/peace/pull/103
[#100]: https://github.com/azriel91/peace/issues/100
[#104]: https://github.com/azriel91/peace/pull/104
[#91]: https://github.com/azriel91/peace/issues/91
[#105]: https://github.com/azriel91/peace/pull/105


## 0.0.7 (2023-03-06)

* Add [`cargo-deny`] and [`cargo-about`] CI checks. ([#76])
* Add [`peace::fmt::{Presentable, Presenter}`] traits. ([#77], [#79])
* Add [`peace::rt_model::CliMdPresenter`] which is the default for presenting text in `CliOutput`. ([#77], [#79])
* Hold `WorkspaceParams`, `ProfileParams`, and `FlowParams` type registries in `CmdContext`. ([#35], [#80])
* Add new `CmdCtx` that contains workspace / profile / flow scoped information. ([#81], [#82])
* Update examples to use `CmdCtx`. ([#83], [#85])
* Remove the old `CmdContext`. ([#83], [#85])

[`cargo-deny`]: https://github.com/EmbarkStudios/cargo-deny
[`cargo-about`]: https://github.com/EmbarkStudios/cargo-about
[#76]: https://github.com/azriel91/peace/pull/76
[#77]: https://github.com/azriel91/peace/issues/77
[#79]: https://github.com/azriel91/peace/pull/79
[#80]: https://github.com/azriel91/peace/pull/80
[#81]: https://github.com/azriel91/peace/issues/81
[#82]: https://github.com/azriel91/peace/pull/82
[#83]: https://github.com/azriel91/peace/issues/83
[#85]: https://github.com/azriel91/peace/pull/85


## 0.0.6 (2023-01-21)

* `FileDownload` item now supports base64 storage for WASM target. ([#62])
* Implement `TarXItem` for native target. ([#62])
* Support multiple workspace, profile, and flow parameters. ([#45], [#63])
* Support progress bars in `CliOutput`. ([#42], [#66])
* Consolidate `StateLogical` and `StatePhysical` into `Item::State`. ([#69], [#70])
* Use [ETag] to determine if a file needs to be re-downloaded. ([#68], [#71])
* Add `PeaceAppDir` layer so different Peace tools don't conflict with each other. ([#35], [#72])
* Move `profile` and `flow_id` parameters to `CmdContextBuilder`. ([#35], [#73])
* Support reading `Profile` from workspace params. ([#35], [#73])

[ETag]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/ETag
[#62]: https://github.com/azriel91/peace/pull/62
[#45]: https://github.com/azriel91/peace/issues/45
[#63]: https://github.com/azriel91/peace/pull/63
[#42]: https://github.com/azriel91/peace/issues/42
[#66]: https://github.com/azriel91/peace/pull/66
[#69]: https://github.com/azriel91/peace/issues/69
[#70]: https://github.com/azriel91/peace/pull/70
[#68]: https://github.com/azriel91/peace/issues/68
[#71]: https://github.com/azriel91/peace/pull/71
[#35]: https://github.com/azriel91/peace/issues/35
[#72]: https://github.com/azriel91/peace/pull/72
[#73]: https://github.com/azriel91/peace/pull/73


## 0.0.5 (2022-12-18)

* `ShCmdStateDiffFn` correctly runs `state_diff_sh_cmd` for state diff. ([#57])
* `ShCmdParams` now uses the `Id` type parameter so that different `ShCmdItem`s can be used correctly. ([#57])
* `ShCmdItem` takes in optional `ShCmdParams<Id>` and inserts it into `resources`. ([#57])
* `CmdContextBuilder` sets the current directory to the workspace directory. ([#57])
* `StatesDesired` is now stored as `State<Logical, Placeholder>`. ([#52], [#58])
* Re-read discovered `States` are now named `StatesSaved`. ([#52], [#60])
* `StatesCurrent` is only present when the discovered in the current execution. ([#52], [#60])
* `States*Deserialize` errors are consolidated into a single variant. ([#52], [#60])
* `States*Serialize` errors are consolidated into a single variant. ([#52], [#60])

[#57]: https://github.com/azriel91/peace/pull/57
[#52]: https://github.com/azriel91/peace/issues/52
[#58]: https://github.com/azriel91/peace/pull/58
[#60]: https://github.com/azriel91/peace/pull/60


## 0.0.4 (2022-11-29)

* Require `Display` to be implemented for `StateLogical` and `StatePhysical`. ([#28], [#37])
* Output states and diff as text on single line. ([#28], [#37])
* Support CLI output with colour with the `"output_colorized"` feature. ([#28], [#38])
* Support CLI output as YAML. ([#28], [#39])
* Support CLI output as JSON with the `"output_json"` feature. ([#28], [#39])
* Error compatibility with `miette` with the `"error_reporting"` feature. ([#28], [#40])
* Items are published as part of the `peace_items` crate. ([#44])
* `file_download` item is type parameterized. ([#44])
* Add `ShCmdItem`, which allows item logic to be defined by shell commands. ([#53], [#54])

[#28]: https://github.com/azriel91/peace/issues/28
[#37]: https://github.com/azriel91/peace/pull/37
[#38]: https://github.com/azriel91/peace/pull/38
[#39]: https://github.com/azriel91/peace/pull/39
[#40]: https://github.com/azriel91/peace/pull/40
[#44]: https://github.com/azriel91/peace/pull/44
[#53]: https://github.com/azriel91/peace/issues/53
[#54]: https://github.com/azriel91/peace/pull/54


## 0.0.3 (2022-09-30)

* Peace book &ndash; https://peace.mk/. ([#22], [#23])
* WASM Support. ([#20], [#21])
* Workspace, profile, and flow directories. ([#15], [#24], [#26])
* Workspace, profile, and flow initialization parameters. ([#26], [#29], [#30])
* `StatesDiscoverCmd` to discover both current and desired states. ([#24], [#26], [#27])
* `CleanCmd` to clean up an item. ([#33], [#34])

[#15]: https://github.com/azriel91/peace/issues/15
[#20]: https://github.com/azriel91/peace/issues/20
[#21]: https://github.com/azriel91/peace/pull/21
[#22]: https://github.com/azriel91/peace/issues/22
[#23]: https://github.com/azriel91/peace/pull/23
[#24]: https://github.com/azriel91/peace/issues/24
[#26]: https://github.com/azriel91/peace/pull/26
[#27]: https://github.com/azriel91/peace/pull/27
[#29]: https://github.com/azriel91/peace/issues/29
[#30]: https://github.com/azriel91/peace/pull/30
[#33]: https://github.com/azriel91/peace/issues/33
[#34]: https://github.com/azriel91/peace/pull/34


## 0.0.2 (2022-08-03)

* `StatesCurrentDiscoverCmd` to discover the current state.
* `StatesDesiredDiscoverCmd` to discover the desired state.
* `DiffCmd` to compute the difference between the current and desired states.
* `EnsureCmd` to transform the current state into the desired state. ([#17], [#18])

[#17]: https://github.com/azriel91/peace/issues/17
[#18]: https://github.com/azriel91/peace/pull/18
