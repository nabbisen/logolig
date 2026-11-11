//! Handlers for the settings: size sets, algorithm, preset, transparency, reset, language and theme
//! (RFC 017: extracted from `update`; every body is the old arm body unchanged).

use iced::Task;

use logolig::{ExportPlan, MessageKey, ResizeAlgorithm};
use logolig_i18n::{Locale, Translator, detect_system_locale};

use super::super::persist::persist_settings;
use super::super::toast::{push_success_toast, push_warning_toast};
use super::super::{AppState, Message};

/// Handler for `Message::PngPresetSizeAdded`.
pub(super) fn png_preset_size_added(state: &mut AppState, size: u32) -> Task<Message> {
    if state.export_plan.add_png_size(size) {
        persist_settings(state);
    }
    Task::none()
}

/// Handler for `Message::LocalePicked`.
pub(super) fn locale_picked(state: &mut AppState, opt: Option<Locale>) -> Task<Message> {
    state.locale_override = opt;
    let resolved = opt.unwrap_or_else(detect_system_locale);
    state.translator = Translator::for_locale(resolved);
    persist_settings(state);
    Task::none()
}

/// Handler for `Message::AlgorithmChanged`.
pub(super) fn algorithm_changed(state: &mut AppState, alg: ResizeAlgorithm) -> Task<Message> {
    state.export_plan.algorithm = alg;
    persist_settings(state);
    // Resize algorithm changed — rebuild the preview cache.
    // Discard stale cache (UI shows a loading state while cache is None).

    state.preview_cache = None;
    if let Some(asset) = state.source_asset.as_ref() {
        let arc = std::sync::Arc::new(asset.clone());
        crate::task_queue::build_preview_task(arc, alg, state.job_id)
    } else {
        Task::none()
    }
}

/// Handler for `Message::IncludeSvgToggled`.
pub(super) fn include_svg_toggled(state: &mut AppState, on: bool) -> Task<Message> {
    // Output plan change only; does not affect the preview.
    state.export_plan.include_svg = on;
    persist_settings(state);
    Task::none()
}

/// Handler for `Message::PngSizeInputChanged`.
pub(super) fn png_size_input_changed(state: &mut AppState, s: String) -> Task<Message> {
    // Lightweight digit filter — digits pass through; non-digits are ignored.
    // Whitespace from pastes is stripped at parse time.
    state.png_size_input = s;
    Task::none()
}

/// Handler for `Message::PngSizeAddRequested`.
pub(super) fn png_size_add_requested(state: &mut AppState) -> Task<Message> {
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

/// Handler for `Message::PngSizeRemoveRequested`.
pub(super) fn png_size_remove_requested(state: &mut AppState, size: u32) -> Task<Message> {
    if state.export_plan.remove_png_size(size) {
        persist_settings(state);
    }
    Task::none()
}

/// Handler for `Message::ExportPlanResetRequested`.
pub(super) fn export_plan_reset_requested(state: &mut AppState) -> Task<Message> {
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
        crate::task_queue::build_preview_task(arc, state.export_plan.algorithm, state.job_id)
    } else {
        Task::none()
    }
}

/// Handler for `Message::KeepTransparencyToggled`.
pub(super) fn keep_transparency_toggled(state: &mut AppState, on: bool) -> Task<Message> {
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

// ----------------------------------------------------------------------
// v1.3.0: size input parsing
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
