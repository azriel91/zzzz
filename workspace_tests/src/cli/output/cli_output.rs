use peace::{
    cfg::State,
    cli::output::{CliColorizeOpt, CliOutput, CliOutputBuilder},
    cli_model::OutputFormat,
    item_model::item_id,
    resource_rt::{
        internal::{StateDiffsMut, StatesMut},
        states::{StateDiffs, StatesCurrentStored},
    },
    rt_model::output::OutputWrite,
};

cfg_if::cfg_if! {
    if #[cfg(feature = "output_progress")] {
        use peace::{
            cli::output::{CliOutputTarget, CliProgressFormatOpt},
            progress_model::{
                ProgressComplete,
                ProgressDelta,
                ProgressLimit,
                ProgressMsgUpdate,
                ProgressStatus,
                ProgressTracker,
                ProgressUpdate,
                ProgressUpdateAndId,
            },
            rt_model::{
                indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget},
                CmdProgressTracker,
                IndexMap,
            },
        };
    }
}

#[tokio::test]
async fn outputs_states_as_text() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let mut cli_output = cli_output(&mut buffer, OutputFormat::Text);
    let states_current_stored = {
        let mut states = StatesMut::new();
        states.insert(item_id!("item_0"), State::new("logical", 1.1));
        states.insert(item_id!("item_1"), State::new(1u8, true));
        StatesCurrentStored::from(states)
    };

    <CliOutput<_> as OutputWrite<Error>>::present(&mut cli_output, &states_current_stored).await?;

    assert_eq!(
        "\
        1. `item_0`: logical, 1.1\n\
        2. `item_1`: 1, true\n\
        ",
        String::from_utf8(buffer)?
    );
    Ok(())
}

#[tokio::test]
async fn outputs_state_diffs_as_text() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let mut cli_output = cli_output(&mut buffer, OutputFormat::Text);
    let state_diffs = {
        let mut state_diffs_mut = StateDiffsMut::new();
        state_diffs_mut.insert(item_id!("item_0"), "need one more server");
        state_diffs_mut.insert(item_id!("item_1"), 1);
        StateDiffs::from(state_diffs_mut)
    };

    <CliOutput<_> as OutputWrite<Error>>::present(&mut cli_output, &state_diffs).await?;

    assert_eq!(
        "\
        1. `item_0`: need one more server\n\
        2. `item_1`: 1\n\
        ",
        String::from_utf8(buffer)?
    );
    Ok(())
}

#[tokio::test]
async fn outputs_error_as_text() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let mut cli_output = cli_output(&mut buffer, OutputFormat::Text);
    let error = Error::CliOutputTest;

    <CliOutput<_> as OutputWrite<Error>>::write_err(&mut cli_output, &error).await?;

    assert_eq!(
        "CliOutputTest display message.\n",
        String::from_utf8(buffer)?
    );
    Ok(())
}

#[tokio::test]
async fn outputs_states_as_text_colorized() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let mut cli_output = cli_output_colorized(&mut buffer, OutputFormat::Text);
    let states_current_stored = {
        let mut states = StatesMut::new();
        states.insert(item_id!("item_0"), State::new("logical", 1.1));
        states.insert(item_id!("item_1"), State::new(1u8, true));
        StatesCurrentStored::from(states)
    };

    <CliOutput<_> as OutputWrite<Error>>::present(&mut cli_output, &states_current_stored).await?;

    let output = String::from_utf8(buffer)?;
    assert_eq!(
        "\
        \u{1b}[38;5;15m1.\u{1b}[0m \u{1b}[38;5;75m`item_0`\u{1b}[0m: logical, 1.1\n\
        \u{1b}[38;5;15m2.\u{1b}[0m \u{1b}[38;5;75m`item_1`\u{1b}[0m: 1, true\n",
        output
    );
    assert_eq!(
        "\
        1. `item_0`: logical, 1.1\n\
        2. `item_1`: 1, true\n\
        ",
        console::strip_ansi_codes(&output)
    );
    Ok(())
}

#[tokio::test]
async fn outputs_state_diffs_as_text_colorized() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let mut cli_output = cli_output_colorized(&mut buffer, OutputFormat::Text);
    let state_diffs = {
        let mut state_diffs_mut = StateDiffsMut::new();
        state_diffs_mut.insert(item_id!("item_0"), "need one more server");
        state_diffs_mut.insert(item_id!("item_1"), 1);
        StateDiffs::from(state_diffs_mut)
    };

    <CliOutput<_> as OutputWrite<Error>>::present(&mut cli_output, &state_diffs).await?;

    let output = String::from_utf8(buffer)?;
    assert_eq!(
        "\
        \u{1b}[38;5;15m1.\u{1b}[0m \u{1b}[38;5;75m`item_0`\u{1b}[0m: need one more server\n\
        \u{1b}[38;5;15m2.\u{1b}[0m \u{1b}[38;5;75m`item_1`\u{1b}[0m: 1\n",
        output
    );
    assert_eq!(
        "\
        1. `item_0`: need one more server\n\
        2. `item_1`: 1\n\
        ",
        console::strip_ansi_codes(&output)
    );
    Ok(())
}

#[tokio::test]
async fn outputs_error_as_text_colorized() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let mut cli_output = cli_output_colorized(&mut buffer, OutputFormat::Text);
    let error = Error::CliOutputTest;

    <CliOutput<_> as OutputWrite<Error>>::write_err(&mut cli_output, &error).await?;

    assert_eq!(
        "CliOutputTest display message.\n",
        String::from_utf8(buffer)?
    );
    Ok(())
}

#[tokio::test]
async fn outputs_states_as_yaml() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let mut cli_output = cli_output(&mut buffer, OutputFormat::Yaml);
    let states_current_stored = {
        let mut states = StatesMut::new();
        states.insert(item_id!("item_0"), State::new("logical", 1.1));
        states.insert(item_id!("item_1"), State::new(1u8, true));
        StatesCurrentStored::from(states)
    };

    <CliOutput<_> as OutputWrite<Error>>::present(&mut cli_output, &states_current_stored).await?;

    assert_eq!(
        r#"item_0:
  logical: logical
  physical: 1.1
item_1:
  logical: 1
  physical: true
"#,
        String::from_utf8(buffer)?
    );
    Ok(())
}

#[tokio::test]
async fn outputs_state_diffs_as_yaml() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let mut cli_output = cli_output(&mut buffer, OutputFormat::Yaml);
    let state_diffs = {
        let mut state_diffs_mut = StateDiffsMut::new();
        state_diffs_mut.insert(item_id!("item_0"), "need one more server");
        state_diffs_mut.insert(item_id!("item_1"), 1);
        StateDiffs::from(state_diffs_mut)
    };

    <CliOutput<_> as OutputWrite<Error>>::present(&mut cli_output, &state_diffs).await?;

    assert_eq!(
        r#"item_0: need one more server
item_1: 1
"#,
        String::from_utf8(buffer)?
    );
    Ok(())
}

#[tokio::test]
async fn outputs_error_as_yaml() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let mut cli_output = cli_output(&mut buffer, OutputFormat::Yaml);
    let error = Error::CliOutputTest;

    <CliOutput<_> as OutputWrite<Error>>::write_err(&mut cli_output, &error).await?;

    assert_eq!(
        r#"CliOutputTest display message.
"#,
        String::from_utf8(buffer)?
    );
    Ok(())
}

#[tokio::test]
async fn outputs_states_as_json() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let mut cli_output = cli_output(&mut buffer, OutputFormat::Json);
    let states_current_stored = {
        let mut states = StatesMut::new();
        states.insert(item_id!("item_0"), State::new("logical", 1.1));
        states.insert(item_id!("item_1"), State::new(1u8, true));
        StatesCurrentStored::from(states)
    };

    <CliOutput<_> as OutputWrite<Error>>::present(&mut cli_output, &states_current_stored).await?;

    assert_eq!(
        r#"{"item_0":{"logical":"logical","physical":1.1},"item_1":{"logical":1,"physical":true}}"#,
        String::from_utf8(buffer)?
    );
    Ok(())
}

#[tokio::test]
async fn outputs_state_diffs_as_json() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let mut cli_output = cli_output(&mut buffer, OutputFormat::Json);
    let state_diffs = {
        let mut state_diffs_mut = StateDiffsMut::new();
        state_diffs_mut.insert(item_id!("item_0"), "need one more server");
        state_diffs_mut.insert(item_id!("item_1"), 1);
        StateDiffs::from(state_diffs_mut)
    };

    <CliOutput<_> as OutputWrite<Error>>::present(&mut cli_output, &state_diffs).await?;

    assert_eq!(
        r#"{"item_0":"need one more server","item_1":1}"#,
        String::from_utf8(buffer)?
    );
    Ok(())
}

#[tokio::test]
async fn outputs_error_as_json() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let mut cli_output = cli_output(&mut buffer, OutputFormat::Json);
    let error = Error::CliOutputTest;

    <CliOutput<_> as OutputWrite<Error>>::write_err(&mut cli_output, &error).await?;

    assert_eq!(
        r#""CliOutputTest display message.""#,
        String::from_utf8(buffer)?
    );
    Ok(())
}

#[cfg(feature = "output_progress")]
mod color_always {
    use super::*;

    #[tokio::test]
    async fn progress_begin_sets_prefix_and_progress_bar_style() {
        let mut buffer = Vec::new();
        let mut cli_output = cli_output_progress(
            &mut buffer,
            OutputFormat::Text,
            CliColorizeOpt::Always,
            CliProgressFormatOpt::ProgressBar,
        );
        let (cmd_progress_tracker, progress_bar) = cmd_progress_tracker(&cli_output);

        <CliOutput<_> as OutputWrite<Error>>::progress_begin(
            &mut cli_output,
            &cmd_progress_tracker,
        )
        .await;
        // Hack: because we enable this in `progress_begin`
        // Remove when we properly tick progress updates in `ApplyCmd`.
        progress_bar.disable_steady_tick();

        // We can't inspect `ProgressStyle`'s fields, so we have to render the progress
        // and compare the output.
        assert_eq!(
            "\u{1b}[38;5;15m1.\u{1b}[0m \u{1b}[38;5;75mtest_item_id\u{1b}[0m",
            progress_bar.prefix()
        );
        let CliOutputTarget::InMemory(in_memory_term) = cli_output.progress_target() else {
            ({
                #[cfg_attr(coverage_nightly, coverage(off))]
                || -> ! { unreachable!("This is set in `cli_output_progress`.") }
            })();
        };
        assert_eq!(
            r#"⚫ 1. test_item_id ▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱ (el: 0s, eta: 0s)"#,
            //    ^             ^ ^                                      ^
            //    '-- 15 chars -' '-------------- 40 chars --------------'
            in_memory_term.contents()
        );
    }

    #[tokio::test]
    async fn progress_update_with_limit_sets_progress_bar_style() {
        let mut buffer = Vec::new();
        let mut cli_output = cli_output_progress(
            &mut buffer,
            OutputFormat::Text,
            CliColorizeOpt::Always,
            CliProgressFormatOpt::ProgressBar,
        );
        let (mut cmd_progress_tracker, progress_bar) = cmd_progress_tracker(&cli_output);

        <CliOutput<_> as OutputWrite<Error>>::progress_begin(
            &mut cli_output,
            &cmd_progress_tracker,
        )
        .await;
        // Hack: because we enable this in `progress_begin`
        // Remove when we properly tick progress updates in `ApplyCmd`.
        progress_bar.disable_steady_tick();

        let progress_trackers = cmd_progress_tracker.progress_trackers_mut();
        let progress_tracker = progress_trackers
            .get_mut(&item_id!("test_item_id"))
            .unwrap();

        // Adjust progress_bar length and units.
        progress_tracker.set_progress_status(ProgressStatus::Running);
        progress_tracker.set_progress_limit(ProgressLimit::Steps(100));
        progress_bar.set_length(100);

        let progress_update_and_id = ProgressUpdateAndId {
            item_id: item_id!("test_item_id"),
            progress_update: ProgressUpdate::Limit(ProgressLimit::Steps(100)),
            msg_update: ProgressMsgUpdate::NoChange,
        };
        <CliOutput<_> as OutputWrite<Error>>::progress_update(
            &mut cli_output,
            progress_tracker,
            &progress_update_and_id,
        )
        .await;

        assert_eq!(
            "\u{1b}[38;5;15m1.\u{1b}[0m \u{1b}[38;5;75mtest_item_id\u{1b}[0m",
            progress_bar.prefix()
        );
        let CliOutputTarget::InMemory(in_memory_term) = cli_output.progress_target() else {
            ({
                #[cfg_attr(coverage_nightly, coverage(off))]
                || -> ! { unreachable!("This is set in `cli_output_progress`.") }
            })();
        };
        progress_bar.set_position(20);
        assert_eq!(
            r#"🔵 1. test_item_id ▰▰▰▰▰▰▰▰▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱ 20/100 (el: 0s, eta: 0s)"#,
            in_memory_term.contents()
        );
        progress_bar.set_position(21);
        assert_eq!(
            r#"🔵 1. test_item_id ▰▰▰▰▰▰▰▰▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱ 21/100 (el: 0s, eta: 0s)"#,
            in_memory_term.contents()
        );
        progress_bar.set_position(22);
        assert_eq!(
            r#"🔵 1. test_item_id ▰▰▰▰▰▰▰▰▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱ 22/100 (el: 0s, eta: 0s)"#,
            in_memory_term.contents()
        );
    }

    #[tokio::test]
    async fn progress_update_with_complete_success_finishes_progress_bar() {
        let mut buffer = Vec::new();
        let mut cli_output = cli_output_progress(
            &mut buffer,
            OutputFormat::Text,
            CliColorizeOpt::Always,
            CliProgressFormatOpt::ProgressBar,
        );
        let (mut cmd_progress_tracker, progress_bar) = cmd_progress_tracker(&cli_output);

        <CliOutput<_> as OutputWrite<Error>>::progress_begin(
            &mut cli_output,
            &cmd_progress_tracker,
        )
        .await;
        // Hack: because we enable this in `progress_begin`
        // Remove when we properly tick progress updates in `ApplyCmd`.
        progress_bar.disable_steady_tick();

        let progress_trackers = cmd_progress_tracker.progress_trackers_mut();
        let progress_tracker = progress_trackers
            .get_mut(&item_id!("test_item_id"))
            .unwrap();

        // Adjust progress_bar length and units.
        progress_tracker.set_progress_status(ProgressStatus::Running);
        progress_tracker.set_progress_limit(ProgressLimit::Steps(100));
        progress_bar.set_length(100);

        let progress_update_and_id = ProgressUpdateAndId {
            item_id: item_id!("test_item_id"),
            progress_update: ProgressUpdate::Limit(ProgressLimit::Steps(100)),
            msg_update: ProgressMsgUpdate::NoChange,
        };
        <CliOutput<_> as OutputWrite<Error>>::progress_update(
            &mut cli_output,
            progress_tracker,
            &progress_update_and_id,
        )
        .await;

        // Check current position
        let CliOutputTarget::InMemory(in_memory_term) = cli_output.progress_target() else {
            ({
                #[cfg_attr(coverage_nightly, coverage(off))]
                || -> ! { unreachable!("This is set in `cli_output_progress`.") }
            })();
        };
        progress_bar.set_position(20);
        assert_eq!(
            r#"🔵 1. test_item_id ▰▰▰▰▰▰▰▰▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱ 20/100 (el: 0s, eta: 0s)"#,
            in_memory_term.contents()
        );

        let progress_complete = ProgressComplete::Success;
        progress_tracker.set_progress_status(ProgressStatus::Complete(progress_complete.clone()));
        progress_tracker.set_message(Some(String::from("done")));
        let progress_update_and_id = ProgressUpdateAndId {
            item_id: item_id!("test_item_id"),
            progress_update: ProgressUpdate::Complete(progress_complete),
            msg_update: ProgressMsgUpdate::Set(String::from("done")),
        };
        <CliOutput<_> as OutputWrite<Error>>::progress_update(
            &mut cli_output,
            progress_tracker,
            &progress_update_and_id,
        )
        .await;
        let CliOutputTarget::InMemory(in_memory_term) = cli_output.progress_target() else {
            ({
                #[cfg_attr(coverage_nightly, coverage(off))]
                || -> ! { unreachable!("This is set in `cli_output_progress`.") }
            })();
        };
        assert_eq!(
            r#"✅ 1. test_item_id ▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰ done"#,
            in_memory_term.contents(),
        );
    }

    #[tokio::test]
    async fn progress_update_with_complete_fail_abandons_progress_bar() {
        let mut buffer = Vec::new();
        let mut cli_output = cli_output_progress(
            &mut buffer,
            OutputFormat::Text,
            CliColorizeOpt::Always,
            CliProgressFormatOpt::ProgressBar,
        );
        let (mut cmd_progress_tracker, progress_bar) = cmd_progress_tracker(&cli_output);

        <CliOutput<_> as OutputWrite<Error>>::progress_begin(
            &mut cli_output,
            &cmd_progress_tracker,
        )
        .await;
        // Hack: because we enable this in `progress_begin`
        // Remove when we properly tick progress updates in `ApplyCmd`.
        progress_bar.disable_steady_tick();

        let progress_trackers = cmd_progress_tracker.progress_trackers_mut();
        let progress_tracker = progress_trackers
            .get_mut(&item_id!("test_item_id"))
            .unwrap();

        // Adjust progress_bar length and units.
        progress_tracker.set_progress_status(ProgressStatus::Running);
        progress_tracker.set_progress_limit(ProgressLimit::Steps(100));
        progress_bar.set_length(100);

        let progress_update_and_id = ProgressUpdateAndId {
            item_id: item_id!("test_item_id"),
            progress_update: ProgressUpdate::Limit(ProgressLimit::Steps(100)),
            msg_update: ProgressMsgUpdate::NoChange,
        };
        <CliOutput<_> as OutputWrite<Error>>::progress_update(
            &mut cli_output,
            progress_tracker,
            &progress_update_and_id,
        )
        .await;

        // Check current position
        let CliOutputTarget::InMemory(in_memory_term) = cli_output.progress_target() else {
            ({
                #[cfg_attr(coverage_nightly, coverage(off))]
                || -> ! { unreachable!("This is set in `cli_output_progress`.") }
            })();
        };
        progress_bar.set_position(20);
        assert_eq!(
            r#"🔵 1. test_item_id ▰▰▰▰▰▰▰▰▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱ 20/100 (el: 0s, eta: 0s)"#,
            in_memory_term.contents()
        );

        let progress_complete = ProgressComplete::Fail;
        progress_tracker.set_progress_status(ProgressStatus::Complete(progress_complete.clone()));
        progress_tracker.set_message(Some(String::from("done")));
        let progress_update_and_id = ProgressUpdateAndId {
            item_id: item_id!("test_item_id"),
            progress_update: ProgressUpdate::Complete(progress_complete),
            msg_update: ProgressMsgUpdate::Set(String::from("done")),
        };
        <CliOutput<_> as OutputWrite<Error>>::progress_update(
            &mut cli_output,
            progress_tracker,
            &progress_update_and_id,
        )
        .await;
        let CliOutputTarget::InMemory(in_memory_term) = cli_output.progress_target() else {
            ({
                #[cfg_attr(coverage_nightly, coverage(off))]
                || -> ! { unreachable!("This is set in `cli_output_progress`.") }
            })();
        };
        assert_eq!(
            r#"❌ 1. test_item_id ▰▰▰▰▰▰▰▰▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱ 20/100 done"#,
            in_memory_term.contents()
        );
    }

    #[tokio::test]
    async fn progress_update_delta_with_progress_format_outcome_writes_yaml() {
        let mut buffer = Vec::new();
        let mut cli_output = cli_output_progress(
            &mut buffer,
            OutputFormat::Text,
            CliColorizeOpt::Always,
            CliProgressFormatOpt::Outcome,
        );
        let (mut cmd_progress_tracker, progress_bar) = cmd_progress_tracker(&cli_output);

        <CliOutput<_> as OutputWrite<Error>>::progress_begin(
            &mut cli_output,
            &cmd_progress_tracker,
        )
        .await;
        // Hack: because we enable this in `progress_begin`
        // Remove when we properly tick progress updates in `ApplyCmd`.
        progress_bar.disable_steady_tick();

        let progress_trackers = cmd_progress_tracker.progress_trackers_mut();
        let progress_tracker = progress_trackers
            .get_mut(&item_id!("test_item_id"))
            .unwrap();

        let progress_update_and_id = ProgressUpdateAndId {
            item_id: item_id!("test_item_id"),
            progress_update: ProgressUpdate::Limit(ProgressLimit::Steps(100)),
            msg_update: ProgressMsgUpdate::NoChange,
        };
        <CliOutput<_> as OutputWrite<Error>>::progress_update(
            &mut cli_output,
            progress_tracker,
            &progress_update_and_id,
        )
        .await;

        // Adjust progress_bar length and units.
        progress_tracker.set_progress_status(ProgressStatus::Running);
        progress_tracker.set_progress_limit(ProgressLimit::Steps(100));
        progress_bar.set_length(100);

        let progress_update_and_id = ProgressUpdateAndId {
            item_id: item_id!("test_item_id"),
            progress_update: ProgressUpdate::Delta(ProgressDelta::Inc(21)),
            msg_update: ProgressMsgUpdate::NoChange,
        };
        <CliOutput<_> as OutputWrite<Error>>::progress_update(
            &mut cli_output,
            progress_tracker,
            &progress_update_and_id,
        )
        .await;
        let CliOutputTarget::InMemory(in_memory_term) = cli_output.progress_target() else {
            ({
                #[cfg_attr(coverage_nightly, coverage(off))]
                || -> ! { unreachable!("This is set in `cli_output_progress`.") }
            })();
        };
        assert_eq!(
            r#"---
item_id: test_item_id
progress_update: !Limit
  Steps: 100
msg_update: NoChange
---
item_id: test_item_id
progress_update: !Delta
  Inc: 21
msg_update: NoChange"#,
            in_memory_term.contents()
        );
    }

    #[cfg(feature = "output_progress")]
    #[tokio::test]
    async fn progress_update_delta_with_progress_format_outcome_writes_json() {
        let mut buffer = Vec::new();
        let mut cli_output = cli_output_progress(
            &mut buffer,
            OutputFormat::Json,
            CliColorizeOpt::Always,
            CliProgressFormatOpt::Outcome,
        );
        let (mut cmd_progress_tracker, progress_bar) = cmd_progress_tracker(&cli_output);

        <CliOutput<_> as OutputWrite<Error>>::progress_begin(
            &mut cli_output,
            &cmd_progress_tracker,
        )
        .await;
        // Hack: because we enable this in `progress_begin`
        // Remove when we properly tick progress updates in `ApplyCmd`.
        progress_bar.disable_steady_tick();

        let progress_trackers = cmd_progress_tracker.progress_trackers_mut();
        let progress_tracker = progress_trackers
            .get_mut(&item_id!("test_item_id"))
            .unwrap();

        let progress_update_and_id = ProgressUpdateAndId {
            item_id: item_id!("test_item_id"),
            progress_update: ProgressUpdate::Limit(ProgressLimit::Steps(100)),
            msg_update: ProgressMsgUpdate::NoChange,
        };
        <CliOutput<_> as OutputWrite<Error>>::progress_update(
            &mut cli_output,
            progress_tracker,
            &progress_update_and_id,
        )
        .await;

        // Adjust progress_bar length and units.
        progress_tracker.set_progress_status(ProgressStatus::Running);
        progress_tracker.set_progress_limit(ProgressLimit::Steps(100));
        progress_bar.set_length(100);

        let progress_update_and_id = ProgressUpdateAndId {
            item_id: item_id!("test_item_id"),
            progress_update: ProgressUpdate::Delta(ProgressDelta::Inc(21)),
            msg_update: ProgressMsgUpdate::NoChange,
        };
        <CliOutput<_> as OutputWrite<Error>>::progress_update(
            &mut cli_output,
            progress_tracker,
            &progress_update_and_id,
        )
        .await;
        let CliOutputTarget::InMemory(in_memory_term) = cli_output.progress_target() else {
            ({
                #[cfg_attr(coverage_nightly, coverage(off))]
                || -> ! { unreachable!("This is set in `cli_output_progress`.") }
            })();
        };
        assert_eq!(
            r#"{"item_id":"test_item_id","progress_update":{"Limit":{"Steps":100}},"msg_update":"NoChange"}
{"item_id":"test_item_id","progress_update":{"Delta":{"Inc":21}},"msg_update":"NoChange"}"#,
            in_memory_term.contents()
        );
    }
}

#[cfg(feature = "output_progress")]
mod color_never {
    use super::*;

    #[tokio::test]
    async fn progress_begin_sets_prefix_and_progress_bar_style() {
        let mut buffer = Vec::new();
        let mut cli_output = cli_output_progress(
            &mut buffer,
            OutputFormat::Text,
            CliColorizeOpt::Never,
            CliProgressFormatOpt::ProgressBar,
        );
        let (cmd_progress_tracker, progress_bar) = cmd_progress_tracker(&cli_output);

        <CliOutput<_> as OutputWrite<Error>>::progress_begin(
            &mut cli_output,
            &cmd_progress_tracker,
        )
        .await;
        // Hack: because we enable this in `progress_begin`
        // Remove when we properly tick progress updates in `ApplyCmd`.
        progress_bar.disable_steady_tick();

        // We can't inspect `ProgressStyle`'s fields, so we have to render the progress
        // and compare the output.
        assert_eq!("1. test_item_id", progress_bar.prefix());
        let CliOutputTarget::InMemory(in_memory_term) = cli_output.progress_target() else {
            ({
                #[cfg_attr(coverage_nightly, coverage(off))]
                || -> ! { unreachable!("This is set in `cli_output_progress`.") }
            })();
        };
        assert_eq!(
            r#"⚫ 1. test_item_id ▰▰▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱ (el: 0s, eta: 0s)"#,
            //    ^             ^ ^                                      ^
            //    '-- 15 chars -' '-------------- 40 chars --------------'
            in_memory_term.contents()
        );
    }

    #[tokio::test]
    async fn progress_update_with_limit_sets_progress_bar_style() {
        let mut buffer = Vec::new();
        let mut cli_output = cli_output_progress(
            &mut buffer,
            OutputFormat::Text,
            CliColorizeOpt::Never,
            CliProgressFormatOpt::ProgressBar,
        );
        let (mut cmd_progress_tracker, progress_bar) = cmd_progress_tracker(&cli_output);

        <CliOutput<_> as OutputWrite<Error>>::progress_begin(
            &mut cli_output,
            &cmd_progress_tracker,
        )
        .await;
        // Hack: because we enable this in `progress_begin`
        // Remove when we properly tick progress updates in `ApplyCmd`.
        progress_bar.disable_steady_tick();

        let progress_trackers = cmd_progress_tracker.progress_trackers_mut();
        let progress_tracker = progress_trackers
            .get_mut(&item_id!("test_item_id"))
            .unwrap();

        // Adjust progress_bar length and units.
        progress_tracker.set_progress_status(ProgressStatus::Running);
        progress_tracker.set_progress_limit(ProgressLimit::Steps(100));
        progress_bar.set_length(100);

        let progress_update_and_id = ProgressUpdateAndId {
            item_id: item_id!("test_item_id"),
            progress_update: ProgressUpdate::Limit(ProgressLimit::Steps(100)),
            msg_update: ProgressMsgUpdate::NoChange,
        };
        <CliOutput<_> as OutputWrite<Error>>::progress_update(
            &mut cli_output,
            progress_tracker,
            &progress_update_and_id,
        )
        .await;

        assert_eq!("1. test_item_id", progress_bar.prefix());
        let CliOutputTarget::InMemory(in_memory_term) = cli_output.progress_target() else {
            ({
                #[cfg_attr(coverage_nightly, coverage(off))]
                || -> ! { unreachable!("This is set in `cli_output_progress`.") }
            })();
        };
        progress_bar.set_position(20);
        assert_eq!(
            r#"🔵 1. test_item_id ▰▰▰▰▰▰▰▰▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱ 20/100 (el: 0s, eta: 0s)"#,
            in_memory_term.contents()
        );
        progress_bar.set_position(21);
        assert_eq!(
            r#"🔵 1. test_item_id ▰▰▰▰▰▰▰▰▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱ 21/100 (el: 0s, eta: 0s)"#,
            in_memory_term.contents()
        );
        progress_bar.set_position(22);
        assert_eq!(
            r#"🔵 1. test_item_id ▰▰▰▰▰▰▰▰▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱ 22/100 (el: 0s, eta: 0s)"#,
            in_memory_term.contents()
        );
    }

    #[tokio::test]
    async fn progress_update_with_complete_success_finishes_progress_bar() {
        let mut buffer = Vec::new();
        let mut cli_output = cli_output_progress(
            &mut buffer,
            OutputFormat::Text,
            CliColorizeOpt::Never,
            CliProgressFormatOpt::ProgressBar,
        );
        let (mut cmd_progress_tracker, progress_bar) = cmd_progress_tracker(&cli_output);

        <CliOutput<_> as OutputWrite<Error>>::progress_begin(
            &mut cli_output,
            &cmd_progress_tracker,
        )
        .await;
        // Hack: because we enable this in `progress_begin`
        // Remove when we properly tick progress updates in `ApplyCmd`.
        progress_bar.disable_steady_tick();

        let progress_trackers = cmd_progress_tracker.progress_trackers_mut();
        let progress_tracker = progress_trackers
            .get_mut(&item_id!("test_item_id"))
            .unwrap();

        // Adjust progress_bar length and units.
        progress_tracker.set_progress_status(ProgressStatus::Running);
        progress_tracker.set_progress_limit(ProgressLimit::Steps(100));
        progress_bar.set_length(100);

        let progress_update_and_id = ProgressUpdateAndId {
            item_id: item_id!("test_item_id"),
            progress_update: ProgressUpdate::Limit(ProgressLimit::Steps(100)),
            msg_update: ProgressMsgUpdate::NoChange,
        };
        <CliOutput<_> as OutputWrite<Error>>::progress_update(
            &mut cli_output,
            progress_tracker,
            &progress_update_and_id,
        )
        .await;

        // Check current position
        let CliOutputTarget::InMemory(in_memory_term) = cli_output.progress_target() else {
            ({
                #[cfg_attr(coverage_nightly, coverage(off))]
                || -> ! { unreachable!("This is set in `cli_output_progress`.") }
            })();
        };
        progress_bar.set_position(20);
        assert_eq!(
            r#"🔵 1. test_item_id ▰▰▰▰▰▰▰▰▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱ 20/100 (el: 0s, eta: 0s)"#,
            in_memory_term.contents()
        );

        let progress_complete = ProgressComplete::Success;
        progress_tracker.set_progress_status(ProgressStatus::Complete(progress_complete.clone()));
        progress_tracker.set_message(Some(String::from("done")));
        let progress_update_and_id = ProgressUpdateAndId {
            item_id: item_id!("test_item_id"),
            progress_update: ProgressUpdate::Complete(progress_complete),
            msg_update: ProgressMsgUpdate::Set(String::from("done")),
        };
        <CliOutput<_> as OutputWrite<Error>>::progress_update(
            &mut cli_output,
            progress_tracker,
            &progress_update_and_id,
        )
        .await;
        let CliOutputTarget::InMemory(in_memory_term) = cli_output.progress_target() else {
            ({
                #[cfg_attr(coverage_nightly, coverage(off))]
                || -> ! { unreachable!("This is set in `cli_output_progress`.") }
            })();
        };
        assert_eq!(
            r#"✅ 1. test_item_id ▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰ done"#,
            in_memory_term.contents(),
        );
    }

    #[tokio::test]
    async fn progress_update_with_complete_fail_abandons_progress_bar() {
        let mut buffer = Vec::new();
        let mut cli_output = cli_output_progress(
            &mut buffer,
            OutputFormat::Text,
            CliColorizeOpt::Never,
            CliProgressFormatOpt::ProgressBar,
        );
        let (mut cmd_progress_tracker, progress_bar) = cmd_progress_tracker(&cli_output);

        <CliOutput<_> as OutputWrite<Error>>::progress_begin(
            &mut cli_output,
            &cmd_progress_tracker,
        )
        .await;
        // Hack: because we enable this in `progress_begin`
        // Remove when we properly tick progress updates in `ApplyCmd`.
        progress_bar.disable_steady_tick();

        let progress_trackers = cmd_progress_tracker.progress_trackers_mut();
        let progress_tracker = progress_trackers
            .get_mut(&item_id!("test_item_id"))
            .unwrap();

        // Adjust progress_bar length and units.
        progress_tracker.set_progress_status(ProgressStatus::Running);
        progress_tracker.set_progress_limit(ProgressLimit::Steps(100));
        progress_bar.set_length(100);

        let progress_update_and_id = ProgressUpdateAndId {
            item_id: item_id!("test_item_id"),
            progress_update: ProgressUpdate::Limit(ProgressLimit::Steps(100)),
            msg_update: ProgressMsgUpdate::NoChange,
        };
        <CliOutput<_> as OutputWrite<Error>>::progress_update(
            &mut cli_output,
            progress_tracker,
            &progress_update_and_id,
        )
        .await;

        // Check current position
        let CliOutputTarget::InMemory(in_memory_term) = cli_output.progress_target() else {
            ({
                #[cfg_attr(coverage_nightly, coverage(off))]
                || -> ! { unreachable!("This is set in `cli_output_progress`.") }
            })();
        };
        progress_bar.set_position(20);
        assert_eq!(
            r#"🔵 1. test_item_id ▰▰▰▰▰▰▰▰▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱ 20/100 (el: 0s, eta: 0s)"#,
            in_memory_term.contents()
        );

        let progress_complete = ProgressComplete::Fail;
        progress_tracker.set_progress_status(ProgressStatus::Complete(progress_complete.clone()));
        progress_tracker.set_message(Some(String::from("done")));
        let progress_update_and_id = ProgressUpdateAndId {
            item_id: item_id!("test_item_id"),
            progress_update: ProgressUpdate::Complete(progress_complete),
            msg_update: ProgressMsgUpdate::Set(String::from("done")),
        };
        <CliOutput<_> as OutputWrite<Error>>::progress_update(
            &mut cli_output,
            progress_tracker,
            &progress_update_and_id,
        )
        .await;
        let CliOutputTarget::InMemory(in_memory_term) = cli_output.progress_target() else {
            ({
                #[cfg_attr(coverage_nightly, coverage(off))]
                || -> ! { unreachable!("This is set in `cli_output_progress`.") }
            })();
        };
        assert_eq!(
            r#"❌ 1. test_item_id ▰▰▰▰▰▰▰▰▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱▱ 20/100 done"#,
            in_memory_term.contents()
        );
    }

    #[tokio::test]
    async fn progress_update_delta_with_progress_format_outcome_writes_yaml() {
        let mut buffer = Vec::new();
        let mut cli_output = cli_output_progress(
            &mut buffer,
            OutputFormat::Text,
            CliColorizeOpt::Never,
            CliProgressFormatOpt::Outcome,
        );
        let (mut cmd_progress_tracker, progress_bar) = cmd_progress_tracker(&cli_output);

        <CliOutput<_> as OutputWrite<Error>>::progress_begin(
            &mut cli_output,
            &cmd_progress_tracker,
        )
        .await;
        // Hack: because we enable this in `progress_begin`
        // Remove when we properly tick progress updates in `ApplyCmd`.
        progress_bar.disable_steady_tick();

        let progress_trackers = cmd_progress_tracker.progress_trackers_mut();
        let progress_tracker = progress_trackers
            .get_mut(&item_id!("test_item_id"))
            .unwrap();

        let progress_update_and_id = ProgressUpdateAndId {
            item_id: item_id!("test_item_id"),
            progress_update: ProgressUpdate::Limit(ProgressLimit::Steps(100)),
            msg_update: ProgressMsgUpdate::NoChange,
        };
        <CliOutput<_> as OutputWrite<Error>>::progress_update(
            &mut cli_output,
            progress_tracker,
            &progress_update_and_id,
        )
        .await;

        // Adjust progress_bar length and units.
        progress_tracker.set_progress_status(ProgressStatus::Running);
        progress_tracker.set_progress_limit(ProgressLimit::Steps(100));
        progress_bar.set_length(100);

        let progress_update_and_id = ProgressUpdateAndId {
            item_id: item_id!("test_item_id"),
            progress_update: ProgressUpdate::Delta(ProgressDelta::Inc(21)),
            msg_update: ProgressMsgUpdate::NoChange,
        };
        <CliOutput<_> as OutputWrite<Error>>::progress_update(
            &mut cli_output,
            progress_tracker,
            &progress_update_and_id,
        )
        .await;
        let CliOutputTarget::InMemory(in_memory_term) = cli_output.progress_target() else {
            ({
                #[cfg_attr(coverage_nightly, coverage(off))]
                || -> ! { unreachable!("This is set in `cli_output_progress`.") }
            })();
        };
        assert_eq!(
            r#"---
item_id: test_item_id
progress_update: !Limit
  Steps: 100
msg_update: NoChange
---
item_id: test_item_id
progress_update: !Delta
  Inc: 21
msg_update: NoChange"#,
            in_memory_term.contents()
        );
    }

    #[cfg(feature = "output_progress")]
    #[tokio::test]
    async fn progress_update_delta_with_progress_format_outcome_writes_json() {
        let mut buffer = Vec::new();
        let mut cli_output = cli_output_progress(
            &mut buffer,
            OutputFormat::Json,
            CliColorizeOpt::Never,
            CliProgressFormatOpt::Outcome,
        );
        let (mut cmd_progress_tracker, progress_bar) = cmd_progress_tracker(&cli_output);

        <CliOutput<_> as OutputWrite<Error>>::progress_begin(
            &mut cli_output,
            &cmd_progress_tracker,
        )
        .await;
        // Hack: because we enable this in `progress_begin`
        // Remove when we properly tick progress updates in `ApplyCmd`.
        progress_bar.disable_steady_tick();

        let progress_trackers = cmd_progress_tracker.progress_trackers_mut();
        let progress_tracker = progress_trackers
            .get_mut(&item_id!("test_item_id"))
            .unwrap();

        let progress_update_and_id = ProgressUpdateAndId {
            item_id: item_id!("test_item_id"),
            progress_update: ProgressUpdate::Limit(ProgressLimit::Steps(100)),
            msg_update: ProgressMsgUpdate::NoChange,
        };
        <CliOutput<_> as OutputWrite<Error>>::progress_update(
            &mut cli_output,
            progress_tracker,
            &progress_update_and_id,
        )
        .await;

        // Adjust progress_bar length and units.
        progress_tracker.set_progress_status(ProgressStatus::Running);
        progress_tracker.set_progress_limit(ProgressLimit::Steps(100));
        progress_bar.set_length(100);

        let progress_update_and_id = ProgressUpdateAndId {
            item_id: item_id!("test_item_id"),
            progress_update: ProgressUpdate::Delta(ProgressDelta::Inc(21)),
            msg_update: ProgressMsgUpdate::NoChange,
        };
        <CliOutput<_> as OutputWrite<Error>>::progress_update(
            &mut cli_output,
            progress_tracker,
            &progress_update_and_id,
        )
        .await;
        let CliOutputTarget::InMemory(in_memory_term) = cli_output.progress_target() else {
            ({
                #[cfg_attr(coverage_nightly, coverage(off))]
                || -> ! { unreachable!("This is set in `cli_output_progress`.") }
            })();
        };
        assert_eq!(
            r#"{"item_id":"test_item_id","progress_update":{"Limit":{"Steps":100}},"msg_update":"NoChange"}
{"item_id":"test_item_id","progress_update":{"Delta":{"Inc":21}},"msg_update":"NoChange"}"#,
            in_memory_term.contents()
        );
    }
}

#[derive(Debug, thiserror::Error)]
enum Error {
    /// CliOutputTest display message.
    #[error("CliOutputTest display message.")]
    CliOutputTest,

    // Framework errors
    /// A `peace` runtime error occurred.
    #[error("A `peace` runtime error occurred.")]
    PeaceRtError(#[from] peace::rt_model::Error),
}

fn cli_output(buffer: &mut Vec<u8>, outcome_format: OutputFormat) -> CliOutput<&mut Vec<u8>> {
    CliOutputBuilder::new_with_writer(buffer)
        .with_outcome_format(outcome_format)
        .build()
}

fn cli_output_colorized(
    buffer: &mut Vec<u8>,
    outcome_format: OutputFormat,
) -> CliOutput<&mut Vec<u8>> {
    CliOutputBuilder::new_with_writer(buffer)
        .with_outcome_format(outcome_format)
        .with_colorize(CliColorizeOpt::Always)
        .build()
}

#[cfg(feature = "output_progress")]
fn cli_output_progress(
    buffer: &mut Vec<u8>,
    outcome_format: OutputFormat,
    colorize: CliColorizeOpt,
    progress_format: CliProgressFormatOpt,
) -> CliOutput<&mut Vec<u8>> {
    CliOutputBuilder::new_with_writer(buffer)
        .with_outcome_format(outcome_format)
        .with_colorize(colorize)
        .with_progress_target(CliOutputTarget::in_memory(50, 120))
        .with_progress_format(progress_format)
        .build()
}

#[cfg(feature = "output_progress")]
fn cmd_progress_tracker(cli_output: &CliOutput<&mut Vec<u8>>) -> (CmdProgressTracker, ProgressBar) {
    let CliOutputTarget::InMemory(in_memory_term) = cli_output.progress_target() else {
        unreachable!("This is set in `cli_output_progress`.");
    };
    let multi_progress = MultiProgress::with_draw_target(ProgressDrawTarget::term_like(Box::new(
        in_memory_term.clone(),
    )));
    let mut progress_trackers = IndexMap::new();
    let progress_bar = multi_progress.add(ProgressBar::hidden());
    let progress_tracker = ProgressTracker::new(progress_bar.clone());
    progress_trackers.insert(item_id!("test_item_id"), progress_tracker);
    let cmd_progress_tracker = CmdProgressTracker::new(multi_progress, progress_trackers);

    (cmd_progress_tracker, progress_bar)
}
