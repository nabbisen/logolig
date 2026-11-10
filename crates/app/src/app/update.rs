//! `update`: the state transitions (RFC 017: moved out of `app.rs`, unchanged apart from visibility).

use std::path::PathBuf;
use std::time::Instant;

use iced::Task;

use logolig::{
    AppError, ExportPlan, MessageKey, PreviewProfile,
    services::transparency_audit::audit as audit_transparency,
};
use logolig_i18n::{Translator, detect_system_locale};

use super::persist::persist_settings;
use super::toast::{
    is_non_square, push_error_toast, push_jpeg_input_warning, push_long_success_toast,
    push_non_square_warning, push_success_toast, push_svg_parts_warning, push_transparency_warning,
    push_warning_toast,
};
use super::{AppState, Message, NavPage, Screen};

// ----------------------------------------------------------------------
// Update (state transitions)
// ----------------------------------------------------------------------

pub(super) fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::FileDropped(path) | Message::FilePicked(Some(path)) => {
            state.file_drag_hovering = false;
            start_ingest(state, path)
        }
        Message::FileDragHovered => {
            state.file_drag_hovering = true;
            Task::none()
        }
        Message::FileDragLeft => {
            state.file_drag_hovering = false;
            Task::none()
        }
        Message::FilePicked(None) => {
            // File picker cancelled — no state change.
            Task::none()
        }
        Message::PickFileRequested => {
            // Launch the native file picker as a Task (§12 accessibility alternative).
            // Result arrives as Message::FilePicked(Option<PathBuf>).
            // v1.12.0: re-select also reuses this message (keeps current screen,
            // cancelling returns to whatever screen was active).
            crate::task_queue::pick_file_task()
        }

        // v1.12.0: back navigation.
        //
        // v1.16.0: returns to Empty from any screen (Result, Converting, etc.).

        //
        // Persisted settings (export_plan, theme, locale) are preserved.
        // Resets transient UI state (nav page, extras open, etc.).
        Message::EditCancelled => {
            state.file_drag_hovering = false;
            // v1.24.0: result_assets and source_asset are intentionally kept so
            // the history card can offer "View results →" on the Empty screen.
            // Clearing them is deferred until a new file is ingested.
            state.preview = None;
            state.preview_cache = None;
            state.transparency = None;
            state.result_preview_open = false;
            state.screen = Screen::Empty;
            Task::none()
        }
        Message::ShowLastResultRequested => {
            // Return to the Result screen without re-converting.
            // result_assets is always Some here because the history card
            // only appears when it is Some.
            state.screen = Screen::Result;
            Task::none()
        }
        Message::IngestCompleted(Ok(asset)) => {
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
                crate::task_queue::build_preview_task(arc.clone(), state.export_plan.algorithm),
                crate::task_queue::convert_task(arc.clone(), state.export_plan.clone()),
            ];
            if is_svg {
                tasks.push(crate::task_queue::analyze_svg_task(arc));
            }
            Task::batch(tasks)
        }
        Message::SvgAnalyzed(path, report) => {
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
        Message::IngestCompleted(Err(err)) => fail(state, err),

        Message::PreviewBuilt(Ok(cache)) => {
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
        Message::PreviewBuilt(Err(err)) => {
            // Preview build failure is non-fatal (source data is intact).
            // Stay on the current screen; notify via toast only.
            push_error_toast(state, err);
            Task::none()
        }
        Message::PreviewContextSelected(ctx) => {
            if let Some(p) = state.preview.as_mut() {
                p.context = ctx;
            }
            Task::none()
        }
        Message::PreviewBackgroundSelected(theme) => {
            if let Some(p) = state.preview.as_mut() {
                p.background = theme;
            }
            Task::none()
        }

        // v1.16.0: in-memory conversion complete
        Message::ConvertCompleted(Ok(assets)) => {
            state.result_assets = Some(assets);
            state.screen = Screen::Result;
            Task::none()
        }
        Message::ConvertCompleted(Err(err)) => fail(state, err),

        // v1.16.0: download buttons
        Message::DownloadOneRequested(idx) => {
            // Extract the filename and open a save dialog with the right extension.
            let Some(assets) = state.result_assets.as_ref() else {
                return Task::none();
            };
            let Some(item) = assets.items.get(idx) else {
                return Task::none();
            };
            crate::task_queue::pick_save_one_task(idx, item.file_name.clone())
        }
        Message::DownloadAllRequested => crate::task_queue::pick_save_all_task(),
        Message::DownloadOneTargetPicked(_, None) => Task::none(),
        Message::DownloadOneTargetPicked(idx, Some(path)) => {
            let Some(assets) = state.result_assets.as_ref() else {
                return Task::none();
            };
            let Some(item) = assets.items.get(idx) else {
                return Task::none();
            };
            let bytes = item.bytes.clone();
            crate::task_queue::write_one_task(path, bytes)
        }
        Message::DownloadAllTargetPicked(None) => Task::none(),
        Message::DownloadAllTargetPicked(Some(path)) => {
            let Some(assets) = state.result_assets.as_ref() else {
                return Task::none();
            };
            // ZIP assembly runs as a task to avoid blocking the UI thread.
            let items_clone = assets.items.clone();
            crate::task_queue::write_zip_task(path, items_clone)
        }
        Message::DownloadOneCompleted(Ok(path)) => {
            let title = state.translator.t(MessageKey::ToastExportTitle);
            let body = format!("{}", path.display());
            push_long_success_toast(state, &title, &body);
            Task::none()
        }
        Message::DownloadOneCompleted(Err(err)) => {
            push_error_toast(state, err);
            Task::none()
        }
        Message::DownloadAllCompleted(Ok(path)) => {
            let title = state.translator.t(MessageKey::ToastExportTitle);
            let body = format!("{}", path.display());
            push_long_success_toast(state, &title, &body);
            Task::none()
        }
        Message::DownloadAllCompleted(Err(err)) => {
            push_error_toast(state, err);
            Task::none()
        }

        // v1.16.0: toggle the collapsible preview panel on the Result screen
        Message::ResultPreviewToggled => {
            state.result_preview_open = !state.result_preview_open;
            Task::none()
        }

        // v1.17.0: window resize → update window_size
        Message::WindowResized(size) => {
            state.window_size = size;
            Task::none()
        }
        // v1.17.0: toggle the Advanced extras collapsible
        Message::AdvancedExtrasToggled => {
            state.advanced_extras_open = !state.advanced_extras_open;
            Task::none()
        }
        // v1.17.0: PNG preset size checkbox toggled
        Message::PngPresetSizeAdded(size) => {
            if state.export_plan.add_png_size(size) {
                persist_settings(state);
            }
            Task::none()
        }
        // v1.22.0: side-nav page switch
        Message::NavPageSelected(page) => {
            state.nav_page = page;
            Task::none()
        }
        Message::ShortcutPressed(shortcut) => handle_shortcut(state, shortcut),
        Message::LocalePicked(opt) => {
            state.locale_override = opt;
            let resolved = opt.unwrap_or_else(detect_system_locale);
            state.translator = Translator::for_locale(resolved);
            persist_settings(state);
            Task::none()
        }
        Message::ThemePicked(theme) => {
            state.theme = theme;
            persist_settings(state);
            Task::none()
        }

        Message::AlgorithmChanged(alg) => {
            state.export_plan.algorithm = alg;
            persist_settings(state);
            // Resize algorithm changed — rebuild the preview cache.
            // Discard stale cache (UI shows a loading state while cache is None).

            state.preview_cache = None;
            if let Some(asset) = state.source_asset.as_ref() {
                let arc = std::sync::Arc::new(asset.clone());
                crate::task_queue::build_preview_task(arc, alg)
            } else {
                Task::none()
            }
        }
        Message::IncludeSvgToggled(on) => {
            // Output plan change only; does not affect the preview.
            state.export_plan.include_svg = on;
            persist_settings(state);
            Task::none()
        }
        Message::VectorizeOnRasterToggled(on) => {
            state.export_plan.vectorize_on_raster = on;
            persist_settings(state);
            Task::none()
        }

        // -----------------------------------------------------------------
        // v1.3.0: Customize page editing UI handlers
        // -----------------------------------------------------------------
        Message::IncludeAppleTouchToggled(on) => {
            state.export_plan.include_apple_touch = on;
            persist_settings(state);
            Task::none()
        }
        Message::IncludeHtmlSnippetToggled(on) => {
            state.export_plan.include_html_snippet = on;
            persist_settings(state);
            Task::none()
        }
        Message::PngSizeInputChanged(s) => {
            // Lightweight digit filter — digits pass through; non-digits are ignored.
            // Whitespace from pastes is stripped at parse time.
            state.png_size_input = s;
            Task::none()
        }
        Message::PngSizeAddRequested => {
            let raw = state.png_size_input.trim().to_string();
            match parse_size(&raw, logolig::PNG_SIZE_MIN, logolig::PNG_SIZE_MAX) {
                Ok(size) => {
                    if state.export_plan.add_png_size(size) {
                        state.png_size_input.clear();
                        persist_settings(state);
                    } else {
                        // parse_size already caught out-of-range; duplicates reach here.
                        let title = state.translator.t(MessageKey::ToastSizeAlreadyInSetTitle);
                        let body = state.translator.t_args(
                            MessageKey::ToastPngSizeAlreadyInSetBody,
                            &[("size", &size.to_string())],
                        );
                        push_warning_toast(state, &title, &body);
                    }
                }
                Err(SizeParseError::Empty) => {
                    // Ignore Add on empty input (prevents accidental Enter-key repeats).
                }
                Err(SizeParseError::NotANumber) => {
                    let title = state.translator.t(MessageKey::ToastInvalidSizeTitle);
                    let body = state
                        .translator
                        .t_args(MessageKey::ToastInvalidSizeBody, &[("input", &raw)]);
                    push_warning_toast(state, &title, &body);
                }
                Err(SizeParseError::OutOfRange { min, max }) => {
                    let title = state.translator.t(MessageKey::ToastSizeOutOfRangeTitle);
                    let body = state.translator.t_args(
                        MessageKey::ToastPngSizeOutOfRangeBody,
                        &[("min", &min.to_string()), ("max", &max.to_string())],
                    );
                    push_warning_toast(state, &title, &body);
                }
            }
            Task::none()
        }
        Message::PngSizeRemoveRequested(size) => {
            if state.export_plan.remove_png_size(size) {
                persist_settings(state);
            }
            Task::none()
        }
        // -----------------------------------------------------------------
        // v1.4.1: vtracer preset + ExportPlan reset
        // -----------------------------------------------------------------
        Message::VtracerPresetChanged(preset) => {
            state.export_plan.vtracer_preset = preset;
            persist_settings(state);
            Task::none()
        }
        Message::ExportPlanResetRequested => {
            // Reset ExportPlan only. theme, locale, and nav state are unaffected
            // (§v1.4.1 reset scope decision).
            state.export_plan = ExportPlan::default();
            // Also clear text input buffers so stale values don't reappear.
            state.png_size_input.clear();
            // Algorithm may have changed (reset restores Lanczos3); rebuild preview.
            // Discard stale cache; rebuild with the restored algorithm.
            state.preview_cache = None;
            persist_settings(state);
            // Transient success toast confirms the reset happened.
            let title = state.translator.t(MessageKey::ToastResetTitle);
            let body = state.translator.t(MessageKey::ToastResetBody);
            push_success_toast(state, &title, &body);
            // Rebuild preview cache if a source is loaded.
            if let Some(asset) = state.source_asset.as_ref() {
                let arc = std::sync::Arc::new(asset.clone());
                crate::task_queue::build_preview_task(arc, state.export_plan.algorithm)
            } else {
                Task::none()
            }
        }

        // v1.19.0: old LocaleCycled / AppCloseRequested handlers removed.

        // -----------------------------------------------------------------
        // v1.10.3 → v1.17.0: accordion handlers removed; settings are flat now.
        // -----------------------------------------------------------------

        // -----------------------------------------------------------------
        // v1.7.0 → v1.10.0: transparency checker merged into PreviewContextSelected.
        // Dedicated message removed.
        // -----------------------------------------------------------------

        // -----------------------------------------------------------------
        // v1.8.0: web manifest settings
        // -----------------------------------------------------------------
        Message::IncludeWebManifestToggled(on) => {
            // On: insert WebManifestSettings::default(). Off: set to None.
            // Any user-entered values are discarded on toggle-off (simple state model).
            // Preserving values across toggle-off can be added in a future version.
            state.export_plan.web_manifest = if on {
                Some(logolig::WebManifestSettings::default())
            } else {
                None
            };
            persist_settings(state);
            Task::none()
        }
        Message::WebManifestNameChanged(s) => {
            if let Some(m) = state.export_plan.web_manifest.as_mut() {
                m.name = s;
                persist_settings(state);
            }
            Task::none()
        }
        Message::WebManifestShortNameChanged(s) => {
            if let Some(m) = state.export_plan.web_manifest.as_mut() {
                m.short_name = s;
                persist_settings(state);
            }
            Task::none()
        }
        Message::WebManifestThemeColorChanged(s) => {
            // No live validation — interrupting the user mid-type is poor UX.
            // `is_valid_color` is validated at export time, not here,
            // so partial values like #FF or #FFFFF are accepted while typing.
            if let Some(m) = state.export_plan.web_manifest.as_mut() {
                m.theme_color = s;
                persist_settings(state);
            }
            Task::none()
        }
        Message::WebManifestBackgroundColorChanged(s) => {
            if let Some(m) = state.export_plan.web_manifest.as_mut() {
                m.background_color = s;
                persist_settings(state);
            }
            Task::none()
        }

        // -----------------------------------------------------------------
        // v1.9.0: monochrome output set
        // -----------------------------------------------------------------
        Message::IncludeMonochromeToggled(on) => {
            // Simple bool flag. Same persist_settings pattern as include_ico
            // and include_apple_touch.
            //
            // No effect on the preview cache — monochrome output is generated
            // only at export time and has no preview representation.

            state.export_plan.monochrome = on;
            persist_settings(state);
            Task::none()
        }

        // -----------------------------------------------------------------
        // v1.26.0: Microsoft app logo output
        // -----------------------------------------------------------------
        Message::IncludeMicrosoftAppLogosToggled(on) => {
            // Advanced, opt-in output group. Like monochrome, it only affects
            // generated result assets; preview cache is unchanged.
            state.export_plan.include_microsoft_app_logos = on;
            persist_settings(state);
            Task::none()
        }

        // -----------------------------------------------------------------
        // v1.21.0: keep-transparency toggle
        // -----------------------------------------------------------------
        Message::KeepTransparencyToggled(on) => {
            // Persisted (Q4-a). Backward compat with older settings JSON is handled
            // by the struct-level `#[serde(default)]` on ExportPlan; missing fields
            // default to true.
            //
            // The preview cache does not reflect keep_transparency in v1.21.0.
            // The setting's effect is visible in Result card thumbnails and
            // downloaded files only.

            state.export_plan.keep_transparency = on;
            persist_settings(state);
            Task::none()
        }

        // v1.19.0: ExportRequested / ExportDirPicked / ExportCompleted handlers removed.
        // v1.16.0 moved to the ConvertCompleted + per-asset download path.
        Message::ToastTick => {
            state.holds.sweep(&mut state.toasts, Instant::now());
            Task::none()
        }
        Message::ToastAttention(id, attention) => {
            state
                .holds
                .attend(id, attention, &mut state.toasts, Instant::now());
            Task::none()
        }
        Message::DismissToast(id) => {
            state.toasts.retain(|t| t.id != id);
            Task::none()
        }
        // The application opens no modal or menu overlay, so there is nothing to close.
        Message::CloseModals => Task::none(),
    }
}

/// Shared helper: starts an ingest task for the given file path.
///
/// v1.16.0: the former Screen::Importing was merged into Screen::Converting.
/// All phases (ingest, preview build, asset bundle) now share the Converting state.
fn start_ingest(state: &mut AppState, path: PathBuf) -> Task<Message> {
    state.file_drag_hovering = false;
    state.source_path = Some(path.clone());
    state.screen = Screen::Converting;
    // Discard stale preview cache and asset bundle; they will be rebuilt
    // when the new ingest completes.
    state.preview_cache = None;
    state.result_assets = None;
    state.result_preview_open = false;
    crate::task_queue::ingest_task(path)
}

/// Common error handler.
///
/// - returns to Result if a source is loaded, otherwise Empty
///   (v1.16: the former Preview state was removed)
/// - pushes a persistent error toast (stays until dismissed)
fn fail(state: &mut AppState, err: AppError) -> Task<Message> {
    state.screen = if state.result_assets.is_some() {
        Screen::Result
    } else {
        Screen::Empty
    };
    push_error_toast(state, err);
    Task::none()
}

/// Apply a keyboard shortcut (RFC 019 phase 2).
///
/// A shortcut acts exactly when the control it stands for is available to the mouse, and it
/// then goes through `update` as the message that control sends, so there is one code path for
/// the action and one set of state checks (this also keeps a second ingest from starting while
/// one is running).
fn handle_shortcut(state: &mut AppState, shortcut: crate::shortcuts::Shortcut) -> Task<Message> {
    use crate::shortcuts::Shortcut;

    let on_home = state.nav_page == NavPage::Home;
    let message = match shortcut {
        // The drop zone, or Back and then the drop zone, is reachable.
        Shortcut::ChooseFile
            if on_home && matches!(state.screen, Screen::Empty | Screen::Result) =>
        {
            Message::PickFileRequested
        }
        // The "Download all (ZIP)" button is on the Result screen when there are assets.
        Shortcut::DownloadAll
            if on_home && state.screen == Screen::Result && state.result_assets.is_some() =>
        {
            Message::DownloadAllRequested
        }
        Shortcut::EscapeOrBack => match state.toasts.last() {
            // Toasts are pushed in order, so the last one is the newest.
            Some(toast) => Message::DismissToast(toast.id),
            None if on_home && state.screen == Screen::Result => Message::EditCancelled,
            None => return Task::none(),
        },
        Shortcut::Page(page) => Message::NavPageSelected(page),
        Shortcut::Help => Message::NavPageSelected(NavPage::Settings),
        Shortcut::ChooseFile | Shortcut::DownloadAll => return Task::none(),
    };
    update(state, message)
}

// ----------------------------------------------------------------------
// v1.3.0: size input parsing + warning toast
// ----------------------------------------------------------------------

#[derive(Debug)]
enum SizeParseError {
    Empty,
    NotANumber,
    OutOfRange { min: u32, max: u32 },
}

/// Parses a digit string to `u32` and validates the range.
/// `add_*_size` also validates, but doing it here lets us show a specific
/// error (out-of-range / not a number / duplicate) to the user.
fn parse_size(raw: &str, min: u32, max: u32) -> Result<u32, SizeParseError> {
    if raw.is_empty() {
        return Err(SizeParseError::Empty);
    }
    let n: u32 = raw.parse().map_err(|_| SizeParseError::NotANumber)?;
    if n < min || n > max {
        return Err(SizeParseError::OutOfRange { min, max });
    }
    Ok(n)
}
