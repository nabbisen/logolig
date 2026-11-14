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

// ---- one file chooser at a time (decision 3) -----------------------------------------------

use crate::shortcuts::Shortcut;

/// The task a message starts, as a count of started units (0 = nothing).
fn units_of(state: &mut AppState, message: Message) -> usize {
    update(state, message).units()
}

/// 7: a second request opens nothing while the chooser is open; the answer, a cancel or a
/// choice, makes it work again; a drop neither clears the flag nor opens anything.
#[test]
fn a_second_chooser_is_not_opened_while_one_is_open() {
    let mut state = AppState::default();
    assert!(
        units_of(&mut state, Message::PickFileRequested) > 0,
        "the first opens a chooser"
    );
    assert!(state.dialog_open);
    assert_eq!(
        units_of(&mut state, Message::PickFileRequested),
        0,
        "the second opens nothing"
    );
    assert!(state.dialog_open, "and the first is still open");

    // Cancelled: the flag is cleared, a new request works.
    let _ = update(&mut state, Message::FilePicked(None));
    assert!(!state.dialog_open);
    assert!(
        units_of(&mut state, Message::PickFileRequested) > 0,
        "works after a cancel"
    );

    // Chosen: the flag is cleared (and the file starts a job), a new request works.
    let _ = update(
        &mut state,
        Message::FilePicked(Some(PathBuf::from("a.png"))),
    );
    assert!(!state.dialog_open);
    assert_eq!(state.job_id, 1, "the chosen file is a job");
    assert!(
        units_of(&mut state, Message::PickFileRequested) > 0,
        "works after a choice"
    );
}

/// 7 (drop): a drop while the chooser is open is accepted, but it does not touch the flag, and
/// the chooser's own answer later starts its own, newer job.
#[test]
fn a_drop_while_the_chooser_is_open_leaves_it_open_and_the_choice_still_wins() {
    let mut state = AppState::default();
    assert!(units_of(&mut state, Message::PickFileRequested) > 0);

    let drop = units_of(
        &mut state,
        Message::FileDropped(PathBuf::from("dropped.png")),
    );
    assert!(drop > 0, "the drop starts its job (latest intent wins)");
    assert_eq!(state.job_id, 1);
    assert!(
        state.dialog_open,
        "a drop must not clear a flag that belongs to the chooser"
    );
    assert_eq!(
        units_of(&mut state, Message::PickFileRequested),
        0,
        "and opens no second chooser"
    );

    let _ = update(
        &mut state,
        Message::FilePicked(Some(PathBuf::from("chosen.png"))),
    );
    assert_eq!(
        state.job_id, 2,
        "the choice, arriving later, is the newest intent"
    );
    assert_eq!(state.source_path, Some(PathBuf::from("chosen.png")));
    assert!(!state.dialog_open);
}

/// 8: Ctrl+O goes through the same message, so it does nothing while a chooser is open (a
/// double Ctrl+O, or a click and Ctrl+O), and works again after the answer.
#[test]
fn ctrl_o_does_nothing_while_a_chooser_is_open() {
    let mut state = AppState::default();
    let press = |state: &mut AppState| {
        update(state, Message::ShortcutPressed(Shortcut::ChooseFile)).units()
    };

    assert!(press(&mut state) > 0, "the first Ctrl+O opens the chooser");
    assert_eq!(press(&mut state), 0, "the second Ctrl+O opens nothing");
    assert_eq!(
        units_of(&mut state, Message::PickFileRequested),
        0,
        "nor does the button"
    );

    let _ = update(&mut state, Message::FilePicked(None));
    assert!(
        press(&mut state) > 0,
        "Ctrl+O works again after the chooser was cancelled"
    );
}

// ---- one dialog at a time, of any kind (RFC 031, D1) -----------------------------------------

/// A Result screen with one asset, as a Save request needs.
fn result_with_one_asset() -> AppState {
    AppState {
        screen: Screen::Result,
        source_asset: Some(dummy_source_asset()),
        result_assets: Some(assets_named("a.png")),
        ..AppState::default()
    }
}

/// The three ways to ask for a dialog: (name, message).
fn dialog_requests() -> Vec<(&'static str, Message)> {
    vec![
        ("choose file", Message::PickFileRequested),
        ("save one", Message::DownloadOneRequested(0)),
        ("save all", Message::DownloadAllRequested),
    ]
}

/// A second request for the same Save dialog does nothing, whatever sent it (the button message
/// or Ctrl+S).
#[test]
fn a_second_save_request_opens_nothing_while_one_is_open() {
    for (name, message) in [
        ("save all", Message::DownloadAllRequested),
        ("save one", Message::DownloadOneRequested(0)),
    ] {
        let mut state = result_with_one_asset();
        assert!(
            units_of(&mut state, message.clone()) > 0,
            "{name}: the first opens a dialog"
        );
        assert!(state.dialog_open, "{name}");
        assert_eq!(
            units_of(&mut state, message),
            0,
            "{name}: the second opens nothing"
        );
    }
    let mut state = result_with_one_asset();
    let press = |state: &mut AppState| {
        update(state, Message::ShortcutPressed(Shortcut::DownloadAll)).units()
    };
    assert!(press(&mut state) > 0, "the first Ctrl+S opens the dialog");
    assert_eq!(press(&mut state), 0, "the second Ctrl+S opens nothing");
}

/// While any dialog is open a request for any other kind does nothing, in every order.
#[test]
fn no_dialog_opens_while_another_kind_is_open() {
    for (first, first_msg) in dialog_requests() {
        for (second, second_msg) in dialog_requests() {
            let mut state = result_with_one_asset();
            assert!(
                units_of(&mut state, first_msg.clone()) > 0,
                "{first}: opens"
            );
            assert_eq!(
                units_of(&mut state, second_msg),
                0,
                "{second} must do nothing while {first} is open"
            );
        }
    }
}

/// Every answer of every dialog re-enables the next request: the chooser's two, and the two of
/// each Save dialog.
#[test]
fn every_dialog_answer_clears_the_flag() {
    let answers: Vec<(&str, Message, Message)> = vec![
        (
            "chooser cancel",
            Message::PickFileRequested,
            Message::FilePicked(None),
        ),
        (
            "chooser choice",
            Message::PickFileRequested,
            Message::FilePicked(Some(PathBuf::from("a.png"))),
        ),
        (
            "save one cancel",
            Message::DownloadOneRequested(0),
            Message::DownloadOneTargetPicked(0, None),
        ),
        (
            "save one choice",
            Message::DownloadOneRequested(0),
            Message::DownloadOneTargetPicked(0, Some(PathBuf::from("out.png"))),
        ),
        (
            "save all cancel",
            Message::DownloadAllRequested,
            Message::DownloadAllTargetPicked(None),
        ),
        (
            "save all choice",
            Message::DownloadAllRequested,
            Message::DownloadAllTargetPicked(Some(PathBuf::from("out.zip"))),
        ),
    ];
    for (name, request, answer) in answers {
        let mut state = result_with_one_asset();
        assert!(
            units_of(&mut state, request.clone()) > 0,
            "{name}: the request opens a dialog"
        );
        assert!(state.dialog_open, "{name}");
        let _ = update(&mut state, answer);
        assert!(!state.dialog_open, "{name}: the answer clears the flag");
        // A chosen file starts a new job and drops the old result, so only a request that does
        // not need the old result is asked for again in that case.
        if !name.starts_with("chooser") {
            assert!(
                units_of(&mut state, request.clone()) > 0,
                "{name}: the same dialog works again"
            );
            state.dialog_open = false;
        }
        assert!(
            units_of(&mut state, Message::PickFileRequested) > 0,
            "{name}: the chooser works again"
        );
    }
}

/// A drop while a Save dialog is open neither clears the flag nor opens anything.
#[test]
fn a_drop_leaves_a_save_dialog_open() {
    let mut state = result_with_one_asset();
    assert!(units_of(&mut state, Message::DownloadAllRequested) > 0);
    let _ = update(
        &mut state,
        Message::FileDropped(PathBuf::from("dropped.png")),
    );
    assert!(
        state.dialog_open,
        "a drop must not clear a flag that belongs to a Save dialog"
    );
    assert_eq!(units_of(&mut state, Message::DownloadAllRequested), 0);
}

/// A request that has nothing to do (no result, an unknown card) opens no dialog and so must
/// not set the flag.
#[test]
fn a_request_without_a_dialog_does_not_set_the_flag() {
    let mut state = AppState::default();
    assert_eq!(units_of(&mut state, Message::DownloadOneRequested(0)), 0);
    assert!(!state.dialog_open);
    let mut state = result_with_one_asset();
    assert_eq!(units_of(&mut state, Message::DownloadOneRequested(99)), 0);
    assert!(!state.dialog_open);
}

// ---- a dialog that cannot appear (RFC 031, D2) ------------------------------------------------

/// The handler of `DialogUnavailable` ends the dialog (flag cleared, so the next request works)
/// and tells the user exactly once, with a persistent error toast and the text of the kind.
#[test]
fn an_unavailable_dialog_clears_the_flag_and_pushes_exactly_one_error_toast() {
    use crate::app::DialogKind;
    for kind in [DialogKind::Chooser, DialogKind::Save] {
        let mut state = result_with_one_asset();
        state.dialog_open = true;
        let task = update(&mut state, Message::DialogUnavailable(kind));
        assert_eq!(task.units(), 0, "{kind:?}: nothing else is started");
        assert!(!state.dialog_open, "{kind:?}: the flag is cleared");
        assert_eq!(state.toasts.len(), 1, "{kind:?}: exactly one toast");
        let toast = &state.toasts[0];
        assert_eq!(toast.intent, snora::ToastIntent::Error, "{kind:?}");
        assert_eq!(toast.lifetime, snora::ToastLifetime::Persistent, "{kind:?}");
        let t = &state.translator;
        let body_key = match kind {
            DialogKind::Chooser => logolig::MessageKey::ToastDialogUnavailableChooserBody,
            DialogKind::Save => logolig::MessageKey::ToastDialogUnavailableSaveBody,
        };
        assert_eq!(
            toast.title,
            t.t(logolig::MessageKey::ToastDialogUnavailableTitle)
        );
        assert_eq!(toast.message, t.t(body_key));
        // The next request works again.
        assert!(
            units_of(&mut state, Message::PickFileRequested) > 0,
            "{kind:?}"
        );
    }
}
