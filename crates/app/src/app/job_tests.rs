//! `update` tests: conversion job identity and the one-file-chooser rule (RFC 011).
//!
//! The results of the asynchronous work carry the id of the job that started them; only the
//! newest job may change state or raise a toast. Messages are built by hand, so no task runs.

use super::test_support::{dummy_source_asset, ingest_ok};
use super::*;

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
