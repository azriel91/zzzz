use std::time::Duration;

use peace::{
    progress_model::{ProgressLimit, ProgressStatus, ProgressTracker},
    rt_model::indicatif::ProgressBar,
};

#[test]
fn progress_status_begins_as_initialized() {
    let progress_tracker = ProgressTracker::new(ProgressBar::hidden());

    assert_eq!(
        &ProgressStatus::Initialized,
        progress_tracker.progress_status()
    );
}

#[test]
fn inc_increases_progress_bar_position_and_updates_last_update_dt() {
    let progress_bar = ProgressBar::hidden();
    progress_bar.set_length(10);
    progress_bar.set_position(1);
    let mut progress_tracker = ProgressTracker::new(progress_bar);

    let last_update_dt_before = progress_tracker.last_update_dt();
    progress_tracker.inc(3);

    assert_eq!(4, progress_tracker.units_current());
    assert!(progress_tracker.last_update_dt() > last_update_dt_before);
}

#[test]
fn tick_ticks_progress_bar_and_updates_last_update_dt() {
    let progress_bar = ProgressBar::hidden();
    progress_bar.set_length(10);
    progress_bar.set_position(1);
    let mut progress_tracker = ProgressTracker::new(progress_bar);

    let last_update_dt_before = progress_tracker.last_update_dt();
    progress_tracker.tick();

    // We can't actually check internal state, but we can check that the position
    // hasn't changed.
    assert_eq!(1, progress_tracker.units_current());
    assert!(progress_tracker.last_update_dt() > last_update_dt_before);
}

#[test]
fn set_progress_status_with_sets_progress_status_and_updates_last_update_dt() {
    let progress_bar = ProgressBar::hidden();
    let mut progress_tracker = ProgressTracker::new(progress_bar);

    let last_update_dt_before = progress_tracker.last_update_dt();
    progress_tracker.set_progress_status(ProgressStatus::Running);

    assert_eq!(&ProgressStatus::Running, progress_tracker.progress_status());
    assert!(progress_tracker.last_update_dt() > last_update_dt_before);
}

#[test]
fn set_progress_limit_with_unknown_does_not_set_progress_bar_length_and_updates_last_update_dt() {
    let progress_bar = ProgressBar::hidden();
    let mut progress_tracker = ProgressTracker::new(progress_bar);

    let last_update_dt_before = progress_tracker.last_update_dt();
    progress_tracker.set_progress_limit(ProgressLimit::Unknown);

    assert_eq!(
        Some(ProgressLimit::Unknown),
        progress_tracker.progress_limit()
    );
    assert_eq!(None, progress_tracker.progress_bar().length());
    assert_eq!(None, progress_tracker.units_total());
    assert!(progress_tracker.last_update_dt() > last_update_dt_before);
}

#[test]
fn set_progress_limit_with_steps_sets_progress_bar_length_and_updates_last_update_dt() {
    let progress_bar = ProgressBar::hidden();
    progress_bar.set_length(10);
    let mut progress_tracker = ProgressTracker::new(progress_bar);

    let last_update_dt_before = progress_tracker.last_update_dt();
    progress_tracker.set_progress_limit(ProgressLimit::Steps(123));

    assert_eq!(
        Some(ProgressLimit::Steps(123)),
        progress_tracker.progress_limit()
    );
    assert_eq!(Some(123), progress_tracker.progress_bar().length());
    assert_eq!(Some(123), progress_tracker.units_total());
    assert!(progress_tracker.last_update_dt() > last_update_dt_before);
}

#[test]
fn set_progress_limit_with_bytes_sets_progress_bar_length_and_updates_last_update_dt() {
    let progress_bar = ProgressBar::hidden();
    progress_bar.set_length(10);
    let mut progress_tracker = ProgressTracker::new(progress_bar);

    let last_update_dt_before = progress_tracker.last_update_dt();
    progress_tracker.set_progress_limit(ProgressLimit::Bytes(123));

    assert_eq!(
        Some(ProgressLimit::Bytes(123)),
        progress_tracker.progress_limit()
    );
    assert_eq!(Some(123), progress_tracker.progress_bar().length());
    assert_eq!(Some(123), progress_tracker.units_total());
    assert!(progress_tracker.last_update_dt() > last_update_dt_before);
}

#[test]
fn eta() {
    let progress_bar = ProgressBar::hidden();
    progress_bar.set_length(100);
    let mut progress_tracker = ProgressTracker::new(progress_bar);

    progress_tracker.inc(50);
    let eta = progress_tracker.eta();

    assert!(eta < Duration::from_millis(500));
}

#[test]
fn elapsed() {
    let progress_bar = ProgressBar::hidden();
    progress_bar.set_length(10);
    let mut progress_tracker = ProgressTracker::new(progress_bar);
    progress_tracker.tick();

    let elapsed = progress_tracker.elapsed();

    assert!(elapsed > Duration::from_millis(0));
    assert!(elapsed < Duration::from_millis(500));
}

#[test]
fn debug() {
    let progress_tracker = ProgressTracker::new(ProgressBar::hidden());

    assert!(format!("{progress_tracker:?}").starts_with("ProgressTracker"));
}
