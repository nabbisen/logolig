//! Handlers for the input path: choosing a file, ingest, the SVG analysis and the preview build
//! (RFC 017: extracted from `update`; every body is the old arm body unchanged).

use std::path::PathBuf;

use iced::Task;

use logolig::{
    AppError, PreviewCache, PreviewProfile, SourceAsset,
    services::transparency_audit::audit as audit_transparency,
};

use super::super::toast::{
    is_non_square, push_error_toast, push_jpeg_input_warning, push_non_square_warning,
    push_svg_parts_warning, push_transparency_warning,
};
use super::super::{AppState, Message, Screen};
use super::convert::fail;

/// Handler for `Message::PickFileRequested`.
pub(super) fn pick_file_requested() -> Task<Message> {
    // Launch the native file picker as a Task (§12 accessibility alternative).
    // Result arrives as Message::FilePicked(Option<PathBuf>).
    // v1.12.0: re-select also reuses this message (keeps current screen,
    // cancelling returns to whatever screen was active).
    crate::task_queue::pick_file_task()
}

/// Handler for `Message::IngestCompleted`.
pub(super) fn ingest_completed(
    state: &mut AppState,
    job: u64,
    asset: SourceAsset,
) -> Task<Message> {
    // RFC 011: only the newest job may change anything; a stale result is dropped first.
    if job != state.job_id {
        return Task::none();
    }
    // RFC 010: decide before `asset` is moved into the state.
    let non_square = is_non_square(asset.intrinsic_size);
    // RFC 015 phase 2a: only SVG sources are analysed for parts that are not shown.
    let is_svg = asset.kind == logolig::SourceKind::Svg;
    // Arc the asset so it can be moved into the async preview task.
    // SourceAsset is Clone (Arc<[u8]>-backed) so this is cheap.
    let arc = std::sync::Arc::new(asset.clone());
    state.source_asset = Some(asset);
    state.preview = Some(PreviewProfile::default());
    state.preview_cache = None;
    // v1.24.0: clear the previous history now that a new file is being
    // processed. The history card (if shown) disappears at this point.
    state.result_assets = None;
    // v1.16.0: ingest complete → auto-convert. Prior to v1.15 the app
    // waited on the Preview screen; v1.16 converts immediately.
    // Two parallel tasks are launched:
    //   (a) preview build (for the collapsible preview panel)
    //   (b) conversion (in-memory, assembles ResultAssets)
    // (b) completion transitions to Screen::Result.
    state.screen = Screen::Converting;
    // v1.7.0: reset transparency status for the new image.
    // Audit runs in PreviewBuilt; this prevents:
    // - stale warnings carrying over on re-load
    // - duplicate warning toasts for the same image
    state.transparency = None;
    // RFC 010: one advisory per loaded image when the logo will be padded.
    if non_square {
        push_non_square_warning(state);
    }
    // Launch (a) and (b) in parallel; for an SVG also (c) the analysis for the advisory.
    let mut tasks = vec![
        crate::task_queue::build_preview_task(
            arc.clone(),
            state.export_plan.algorithm,
            state.job_id,
        ),
        crate::task_queue::convert_task(arc.clone(), state.export_plan.clone(), state.job_id),
    ];
    if is_svg {
        tasks.push(crate::task_queue::analyze_svg_task(arc, state.job_id));
    }
    Task::batch(tasks)
}

/// Handler for `Message::IngestCompleted` with an error.
pub(super) fn ingest_failed(state: &mut AppState, job: u64, err: AppError) -> Task<Message> {
    // RFC 011: only the newest job may change anything; a stale result is dropped first.
    if job != state.job_id {
        return Task::none();
    }
    fail(state, err)
}

/// Handler for `Message::SvgAnalyzed`.
pub(super) fn svg_analyzed(
    state: &mut AppState,
    job: u64,
    path: PathBuf,
    report: logolig::SvgReport,
) -> Task<Message> {
    // RFC 011: only the newest job may change anything; a stale result is dropped first.
    if job != state.job_id {
        return Task::none();
    }
    // Stale result (another file was loaded since): ignore.
    let is_current = state
        .source_asset
        .as_ref()
        .is_some_and(|asset| asset.path == path);
    if is_current && (report.has_text || report.ignored_external_images > 0) {
        push_svg_parts_warning(state, report);
    }
    Task::none()
}

/// Handler for `Message::PreviewBuilt`.
pub(super) fn preview_built(state: &mut AppState, job: u64, cache: PreviewCache) -> Task<Message> {
    // RFC 011: only the newest job may change anything; a stale result is dropped first.
    if job != state.job_id {
        return Task::none();
    }
    // Verify the cache matches the current source and algorithm.
    // Discard stale cache (wrong source or algorithm).
    let asset_path = state.source_asset.as_ref().map(|a| a.path.clone());
    let still_valid = asset_path.as_ref() == Some(&cache.source_path)
        && cache.algorithm == state.export_plan.algorithm;
    if still_valid {
        // v1.7.0: transparency audit
        // Run the transparency audit on the first cache built for this image.
        // Skip if already audited (state.transparency is Some) to avoid
        // duplicate warning toasts when the preview is rebuilt (e.g. after
        // changing the resize algorithm). The audit itself is idempotent,
        // but showing it once per image is the right UX.
        let needs_audit = state.transparency.is_none();
        if needs_audit {
            let report = audit_transparency(&cache.icon_120);
            state.transparency = Some(report);
            if report.needs_warning() {
                // v1.11.0: JPEG cannot carry alpha so the audit always returns
                // FullyOpaque. This is a format constraint, not a user mistake;
                // show the JPEG-specific educational warning instead of the
                // generic fully-opaque warning.

                let is_jpeg = state
                    .source_asset
                    .as_ref()
                    .map(|a| a.kind == logolig::SourceKind::Jpeg)
                    .unwrap_or(false);
                if is_jpeg {
                    push_jpeg_input_warning(state);
                } else {
                    push_transparency_warning(state, report);
                }
            }
        }
        state.preview_cache = Some(cache);
    }
    Task::none()
}

/// Handler for `Message::PreviewBuilt`.
pub(super) fn preview_build_failed(state: &mut AppState, job: u64, err: AppError) -> Task<Message> {
    // RFC 011: only the newest job may change anything; a stale result is dropped first.
    if job != state.job_id {
        return Task::none();
    }
    // Preview build failure is non-fatal (source data is intact).
    // Stay on the current screen; notify via toast only.
    push_error_toast(state, err);
    Task::none()
}

/// Shared helper: starts an ingest task for the given file path.
///
/// v1.16.0: the former Screen::Importing was merged into Screen::Converting.
/// All phases (ingest, preview build, asset bundle) now share the Converting state.
pub(super) fn start_ingest(state: &mut AppState, path: PathBuf) -> Task<Message> {
    // RFC 011: a new file is a new job; the results of every earlier job are stale from here on.
    state.job_id += 1;
    state.file_drag_hovering = false;
    state.source_path = Some(path.clone());
    state.screen = Screen::Converting;
    // Discard stale preview cache and asset bundle; they will be rebuilt
    // when the new ingest completes.
    state.preview_cache = None;
    state.result_assets = None;
    state.result_preview_open = false;
    crate::task_queue::ingest_task(path, state.job_id)
}
