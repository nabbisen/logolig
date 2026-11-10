//! `update`: the state transitions (RFC 017: moved out of `app.rs`, unchanged apart from visibility).

mod input;

mod convert;

use std::time::Instant;

use iced::Task;

use logolig::{ExportPlan, MessageKey};
use logolig_i18n::{Translator, detect_system_locale};

use super::persist::persist_settings;
use super::toast::{
    push_error_toast, push_long_success_toast, push_success_toast, push_warning_toast,
};
use super::{AppState, Message, NavPage, Screen};
use convert::fail;
use input::start_ingest;

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
        Message::PickFileRequested => input::pick_file_requested(),

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
        Message::IngestCompleted(Ok(asset)) => input::ingest_completed(state, asset),
        Message::SvgAnalyzed(path, report) => input::svg_analyzed(state, path, report),
        Message::IngestCompleted(Err(err)) => fail(state, err),

        Message::PreviewBuilt(Ok(cache)) => input::preview_built(state, cache),
        Message::PreviewBuilt(Err(err)) => input::preview_build_failed(state, err),
        Message::PreviewContextSelected(ctx) => convert::preview_context_selected(state, ctx),
        Message::PreviewBackgroundSelected(theme) => {
            convert::preview_background_selected(state, theme)
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
