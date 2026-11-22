//! `update` tests: a file that cannot be loaded leaves everything as it was (RFC 011 / RFC 014, job K1b).
//!
//! Nothing the user has is discarded until the new file has loaded. A rejected file (too large, not an
//! image, unreadable) changes nothing but the toast; the previous result, its "Last conversion" card
//! (shown on the start screen while there is a result), the source path and the screen come back. The
//! results of superseded jobs are still dropped by their job ids (RFC 011).

use super::test_support::{dummy_result_assets, dummy_source_asset, ingest_ok, plain};
use super::*;

fn bad_file() -> AppError {
    AppError::unsupported_file("notes.txt")
}

/// A session that has converted `first.png` and shows the Result screen.
fn converted() -> AppState {
    let mut state = AppState::default();
    let _ = update(&mut state, Message::FileDropped(PathBuf::from("first.png")));
    ingest_ok(&mut state, Some((1, 1)));
    let job = state.job_id;
    let _ = update(
        &mut state,
        Message::ConvertCompleted(job, Ok(dummy_result_assets())),
    );
    assert_eq!(state.screen, Screen::Result);
    state
}

/// Starts a file and has it rejected by the ingest of the current job.
fn reject(state: &mut AppState, name: &str) {
    let _ = update(state, Message::FileDropped(PathBuf::from(name)));
    let job = state.job_id;
    let _ = update(state, Message::IngestCompleted(job, Err(bad_file())));
}

#[test]
fn a_rejected_file_on_the_start_screen_keeps_the_last_result_and_its_card() {
    let mut state = converted();
    let _ = update(&mut state, Message::EditCancelled); // Back: the start screen with "View results →"
    assert_eq!(state.screen, Screen::Empty);
    assert!(state.result_assets.is_some(), "the history card is shown");
    let before = state.source_path.clone();

    reject(&mut state, "notes.txt");

    assert_eq!(state.screen, Screen::Empty, "back on the start screen");
    assert!(
        state.result_assets.is_some(),
        "the result and its card are still there"
    );
    assert_eq!(
        state.source_path, before,
        "the path of the loaded file, not the rejected one"
    );
    assert_eq!(state.toasts.len(), 1, "one error toast");
}

#[test]
fn a_rejected_file_chosen_on_the_result_screen_returns_to_the_result_screen() {
    let mut state = converted();
    state.result_preview_open = true;
    let before = state.source_path.clone();

    reject(&mut state, "notes.txt");

    assert_eq!(state.screen, Screen::Result);
    assert!(state.result_assets.is_some());
    assert!(state.result_preview_open, "an open preview stays open");
    assert_eq!(state.source_path, before);
    assert_eq!(state.toasts.len(), 1);
}

#[test]
fn the_next_accepted_file_replaces_everything_as_before() {
    let mut state = converted();
    let _ = update(&mut state, Message::EditCancelled);
    reject(&mut state, "notes.txt");
    state.toasts.clear();

    let _ = update(
        &mut state,
        Message::FileDropped(PathBuf::from("second.png")),
    );
    let job = state.job_id;
    let _ = update(
        &mut state,
        Message::IngestCompleted(
            job,
            Ok(plain(logolig::SourceAsset {
                path: PathBuf::from("second.png"),
                ..dummy_source_asset()
            })),
        ),
    );
    assert_eq!(state.screen, Screen::Converting);
    assert!(
        state.result_assets.is_none(),
        "the old result goes when the new file has loaded"
    );
    assert!(!state.result_preview_open);
    assert_eq!(state.source_path, Some(PathBuf::from("second.png")));

    // And the kept state does not come back by itself: a later failure is an ordinary conversion failure.
    let _ = update(
        &mut state,
        Message::ConvertCompleted(job, Err(AppError::export("conversion"))),
    );
    assert_eq!(state.screen, Screen::Empty, "no previous result any more");
}

#[test]
fn a_rejected_file_with_no_previous_result_returns_to_the_start_screen() {
    let mut state = AppState::default();
    reject(&mut state, "notes.txt");
    assert_eq!(state.screen, Screen::Empty);
    assert!(state.result_assets.is_none());
    assert_eq!(state.source_path, None, "no file is loaded");
    assert_eq!(state.toasts.len(), 1);
}

#[test]
fn the_race_a_second_file_fails_while_the_first_is_still_loading() {
    let mut state = converted();
    let before = state.source_path.clone();
    let _ = update(&mut state, Message::FileDropped(PathBuf::from("c.png"))); // job 2, still loading
    let _ = update(&mut state, Message::FileDropped(PathBuf::from("d.png"))); // job 3 supersedes it
    assert_eq!(state.screen, Screen::Converting);

    // The first file's answer is stale: dropped, nothing changes.
    let stale_job = state.job_id - 1;
    let _ = update(
        &mut state,
        Message::IngestCompleted(stale_job, Ok(plain(dummy_source_asset()))),
    );
    assert_eq!(
        state.screen,
        Screen::Converting,
        "a stale success changes nothing"
    );
    assert!(state.toasts.is_empty());

    // The newest file fails: what was on screen before the FIRST file comes back.
    let job = state.job_id;
    let _ = update(&mut state, Message::IngestCompleted(job, Err(bad_file())));
    assert_eq!(state.screen, Screen::Result);
    assert!(state.result_assets.is_some());
    assert_eq!(state.source_path, before);
    assert_eq!(state.toasts.len(), 1);
}

#[test]
fn a_stale_rejection_changes_nothing_and_raises_no_toast() {
    let mut state = converted();
    let _ = update(&mut state, Message::FileDropped(PathBuf::from("c.png"))); // job 2
    let _ = update(&mut state, Message::FileDropped(PathBuf::from("d.png"))); // job 3
    let _ = update(&mut state, Message::IngestCompleted(2, Err(bad_file())));
    assert_eq!(
        state.screen,
        Screen::Converting,
        "still waiting for the newest file"
    );
    assert!(state.toasts.is_empty());
}

#[test]
fn a_rejected_file_during_a_running_conversion_does_not_strand_the_screen() {
    // File A loaded and is converting (the screen shows Converting, there is no result yet).
    let mut state = AppState::default();
    let _ = update(&mut state, Message::FileDropped(PathBuf::from("a.png")));
    ingest_ok(&mut state, Some((1, 1)));
    assert_eq!(state.screen, Screen::Converting);
    // The user drops B, which is rejected. B is the newest intent and it failed; A's conversion is
    // superseded and its result will be dropped as stale, so the screen must not stay on Converting.
    reject(&mut state, "notes.txt");
    assert_eq!(
        state.screen,
        Screen::Empty,
        "the existing rule: Empty without a result"
    );
    assert_eq!(state.toasts.len(), 1);
    let a_job = state.job_id - 1;
    let _ = update(
        &mut state,
        Message::ConvertCompleted(a_job, Ok(dummy_result_assets())),
    );
    assert!(
        state.result_assets.is_none(),
        "A's result is stale and dropped"
    );
    assert_eq!(state.screen, Screen::Empty);
}
