//! `update` tests: conversion job identity and the one-file-chooser rule (RFC 011).
//!
//! The results of the asynchronous work carry the id of the job that started them; only the
//! newest job may change state or raise a toast. Messages are built by hand, so no task runs.

use super::test_support::{
    SVG_BOTH, dummy_source_asset, ingest_ok, opaque_preview_cache, svg_asset,
};
use super::*;

use crate::result::{ResultAssetItem, ResultAssetKind, ResultAssets};

/// `job_id` is incremented exactly once per `start_ingest` (a drop or a chosen file) and by
/// nothing else.
#[test]
fn job_id_increments_once_per_new_file_and_on_no_other_message() {
    let mut state = AppState::default();
    assert_eq!(state.job_id, 0, "no job yet");

    let _ = update(&mut state, Message::FileDropped(PathBuf::from("a.png")));
    assert_eq!(state.job_id, 1, "a drop starts job 1");

    // Everything that is not a new file leaves the id alone.
    ingest_ok(&mut state, Some((1, 1)));
    let others = [
        Message::FileDragHovered,
        Message::FileDragLeft,
        Message::FilePicked(None),
        Message::NavPageSelected(NavPage::Customize),
        Message::NavPageSelected(NavPage::Home),
        Message::KeepTransparencyToggled(false),
        Message::AlgorithmChanged(logolig::ResizeAlgorithm::Nearest),
        Message::ExportPlanResetRequested,
        Message::EditCancelled,
        Message::ShowLastResultRequested,
        Message::ResultPreviewToggled,
        Message::ToastTick,
    ];
    for message in others {
        let _ = update(&mut state, message);
        assert_eq!(state.job_id, 1, "only a new file changes the job id");
    }

    let _ = update(
        &mut state,
        Message::FilePicked(Some(PathBuf::from("b.png"))),
    );
    assert_eq!(state.job_id, 2, "a chosen file starts job 2");
    let _ = update(&mut state, Message::FileDropped(PathBuf::from("c.png")));
    assert_eq!(
        state.job_id, 3,
        "a second drop while job 2 runs starts job 3"
    );

    // A source that is already loaded: the rebuilds of a settings change belong to this job.
    state.source_asset = Some(dummy_source_asset());
    let _ = update(
        &mut state,
        Message::AlgorithmChanged(logolig::ResizeAlgorithm::Bilinear),
    );
    assert_eq!(
        state.job_id, 3,
        "a preview rebuild is part of the current job"
    );
}

// ---- stale results (decisions 1 and 2) -----------------------------------------------------

/// A result bundle with one item named `name`, to tell two jobs' results apart.
fn assets_named(name: &str) -> ResultAssets {
    ResultAssets {
        items: vec![ResultAssetItem {
            file_name: name.to_string(),
            relative_path: name.to_string(),
            bytes: Vec::new(),
            kind: ResultAssetKind::HtmlSnippet,
            dimensions: None,
            thumbnail: None,
        }],
    }
}

fn first_item(state: &AppState) -> Option<&str> {
    state
        .result_assets
        .as_ref()
        .and_then(|a| a.items.first())
        .map(|i| i.file_name.as_str())
}

/// Job 1 is running, then a second file is dropped: job 2 is the newest.
fn state_in_job_two() -> AppState {
    let mut state = AppState::default();
    let _ = update(&mut state, Message::FileDropped(PathBuf::from("a.png")));
    let _ = update(&mut state, Message::FileDropped(PathBuf::from("b.png")));
    assert_eq!(state.job_id, 2, "precondition: job 2 is the newest");
    assert_eq!(state.screen, Screen::Converting, "precondition");
    state
}

/// 1: a conversion of job 1 that arrives after a new drop changes nothing.
#[test]
fn a_stale_conversion_after_a_new_drop_changes_nothing() {
    let mut state = AppState::default();
    let _ = update(&mut state, Message::FileDropped(PathBuf::from("a.png")));
    let _ = update(
        &mut state,
        Message::IngestCompleted(1, Ok(dummy_source_asset())),
    );
    let _ = update(&mut state, Message::FileDropped(PathBuf::from("b.png")));
    assert_eq!(state.job_id, 2);

    let _ = update(
        &mut state,
        Message::ConvertCompleted(1, Ok(assets_named("job-1"))),
    );

    assert!(
        state.result_assets.is_none(),
        "the stale result must not be shown"
    );
    assert_eq!(
        state.screen,
        Screen::Converting,
        "the screen must not change"
    );
}

/// 2: two conversions arrive in reverse order; only the newest job's is applied.
#[test]
fn only_the_newest_jobs_conversion_is_applied_whatever_the_order() {
    let mut state = state_in_job_two();

    let _ = update(
        &mut state,
        Message::ConvertCompleted(2, Ok(assets_named("job-2"))),
    );
    assert_eq!(first_item(&state), Some("job-2"));
    assert_eq!(state.screen, Screen::Result);

    let _ = update(
        &mut state,
        Message::ConvertCompleted(1, Ok(assets_named("job-1"))),
    );
    assert_eq!(
        first_item(&state),
        Some("job-2"),
        "the older job must not replace it"
    );
    assert_eq!(state.screen, Screen::Result);
}

/// 3: a preview of job 1 for the very same path, after a new drop of that path, is ignored.
#[test]
fn a_stale_preview_for_the_same_path_is_ignored() {
    let mut state = AppState::default();
    let _ = update(&mut state, Message::FileDropped(PathBuf::from("logo.png")));
    ingest_ok(&mut state, Some((1, 1)));
    let _ = update(&mut state, Message::FileDropped(PathBuf::from("logo.png")));
    assert_eq!(state.job_id, 2);
    let toasts = state.toasts.len();

    let cache = opaque_preview_cache("logo.png", state.export_plan.algorithm);
    let _ = update(&mut state, Message::PreviewBuilt(1, Ok(cache)));

    assert!(
        state.preview_cache.is_none(),
        "the stale preview must not be stored"
    );
    assert!(state.transparency.is_none(), "and must not be audited");
    assert_eq!(
        state.toasts.len(),
        toasts,
        "and must not raise the advisory"
    );
}

/// 4: the stale error of any of the three results raises no toast and changes no screen.
#[test]
fn a_stale_error_raises_no_toast_and_changes_no_screen() {
    let mut state = state_in_job_two();
    let failures = [
        Message::IngestCompleted(1, Err(AppError::export("old ingest"))),
        Message::ConvertCompleted(1, Err(AppError::export("old conversion"))),
        Message::PreviewBuilt(1, Err(AppError::export("old preview"))),
    ];
    for message in failures {
        let _ = update(&mut state, message);
        assert!(state.toasts.is_empty(), "no toast for a stale error");
        assert_eq!(
            state.screen,
            Screen::Converting,
            "no screen change for a stale error"
        );
    }
}

/// 5: a stale SVG analysis raises no advisory; the current one does (control).
#[test]
fn a_stale_svg_analysis_raises_no_advisory() {
    let mut state = AppState::default();
    let _ = update(&mut state, Message::FileDropped(PathBuf::from("logo.svg")));
    ingest_ok_svg(&mut state);
    let _ = update(&mut state, Message::FileDropped(PathBuf::from("logo.svg")));
    assert_eq!(state.job_id, 2);
    let report = || logolig::services::svg::analyze(SVG_BOTH.as_bytes());

    let _ = update(
        &mut state,
        Message::SvgAnalyzed(1, PathBuf::from("logo.svg"), report()),
    );
    assert!(state.toasts.is_empty(), "the stale analysis must not warn");

    let _ = update(
        &mut state,
        Message::SvgAnalyzed(2, PathBuf::from("logo.svg"), report()),
    );
    assert_eq!(state.toasts.len(), 1, "control: the current analysis warns");
}

/// Loads `logo.svg` as the source of the current job without starting its tasks' results.
fn ingest_ok_svg(state: &mut AppState) {
    let job = state.job_id;
    let _ = update(
        state,
        Message::IngestCompleted(job, Ok(svg_asset("logo.svg"))),
    );
}

/// 6: the current job's errors are shown exactly as before.
#[test]
fn the_current_jobs_errors_are_shown() {
    // Ingest or conversion failure: back to the start screen (nothing to show) with an error toast.
    for message in [
        Message::IngestCompleted(1, Err(AppError::export("ingest"))),
        Message::ConvertCompleted(1, Err(AppError::export("conversion"))),
    ] {
        let mut state = AppState::default();
        let _ = update(&mut state, Message::FileDropped(PathBuf::from("a.png")));
        let _ = update(&mut state, message);
        assert_eq!(state.toasts.len(), 1, "an error toast for the current job");
        assert_eq!(state.screen, Screen::Empty, "back to the start screen");
    }
    // A preview failure is not fatal: a toast, the screen stays.
    let mut state = AppState::default();
    let _ = update(&mut state, Message::FileDropped(PathBuf::from("a.png")));
    let _ = update(
        &mut state,
        Message::PreviewBuilt(1, Err(AppError::export("preview"))),
    );
    assert_eq!(state.toasts.len(), 1);
    assert_eq!(state.screen, Screen::Converting);
}
