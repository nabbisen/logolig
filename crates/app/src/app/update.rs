//! `update`: the dispatch from a `Message` to what it does (RFC 017).
//!
//! An arm of one to three body lines stays in the dispatch below. Every longer arm is a function
//! in one of the handler files, named after what happens; its body is the old arm body, unchanged.
//!
//! | File | Handles |
//! |---|---|
//! | `update/input.rs` | choosing a file, ingest, the SVG analysis, the preview build |
//! | `update/convert.rs` | preview context and background, the common error handler |
//! | `update/download.rs` | the save dialogs and the write results |
//! | `update/nav.rs` | going back, the last result, keyboard shortcuts |
//! | `update/settings.rs` | size sets, algorithm, transparency, reset, language, theme |
//! | `update/manifest.rs` | web manifest, monochrome and Microsoft app logo options |
//! | `update/toast.rs` | the reader's attention on a toast |

mod convert;
mod download;
mod input;
mod manifest;
mod nav;
mod settings;
mod toast;

use std::time::Instant;

use iced::Task;

use super::persist::persist_settings;
use super::toast::push_error_toast;
use super::{AppState, Message, Screen};
use convert::fail;
use input::start_ingest;
use nav::handle_shortcut;

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
        Message::EditCancelled => nav::edit_cancelled(state),
        Message::ShowLastResultRequested => nav::show_last_result_requested(state),
        Message::IngestCompleted(job, Ok(asset)) => input::ingest_completed(state, job, asset),
        Message::SvgAnalyzed(job, path, report) => input::svg_analyzed(state, job, path, report),
        Message::IngestCompleted(_job, Err(err)) => fail(state, err),

        Message::PreviewBuilt(job, Ok(cache)) => input::preview_built(state, job, cache),
        Message::PreviewBuilt(job, Err(err)) => input::preview_build_failed(state, job, err),
        Message::PreviewContextSelected(ctx) => convert::preview_context_selected(state, ctx),
        Message::PreviewBackgroundSelected(theme) => {
            convert::preview_background_selected(state, theme)
        }

        // v1.16.0: in-memory conversion complete
        Message::ConvertCompleted(_job, Ok(assets)) => {
            state.result_assets = Some(assets);
            state.screen = Screen::Result;
            Task::none()
        }
        Message::ConvertCompleted(_job, Err(err)) => fail(state, err),

        // v1.16.0: download buttons
        Message::DownloadOneRequested(idx) => download::download_one_requested(state, idx),
        Message::DownloadAllRequested => crate::task_queue::pick_save_all_task(),
        Message::DownloadOneTargetPicked(_, None) => Task::none(),
        Message::DownloadOneTargetPicked(idx, Some(path)) => {
            download::download_one_target_picked(state, idx, path)
        }
        Message::DownloadAllTargetPicked(None) => Task::none(),
        Message::DownloadAllTargetPicked(Some(path)) => {
            download::download_all_target_picked(state, path)
        }
        Message::DownloadOneCompleted(Ok(path)) => download::download_one_completed(state, path),
        Message::DownloadOneCompleted(Err(err)) => {
            push_error_toast(state, err);
            Task::none()
        }
        Message::DownloadAllCompleted(Ok(path)) => download::download_all_completed(state, path),
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
        Message::PngPresetSizeAdded(size) => settings::png_preset_size_added(state, size),
        // v1.22.0: side-nav page switch
        Message::NavPageSelected(page) => {
            state.nav_page = page;
            Task::none()
        }
        Message::ShortcutPressed(shortcut) => handle_shortcut(state, shortcut),
        Message::LocalePicked(opt) => settings::locale_picked(state, opt),
        Message::ThemePicked(theme) => {
            state.theme = theme;
            persist_settings(state);
            Task::none()
        }

        Message::AlgorithmChanged(alg) => settings::algorithm_changed(state, alg),
        Message::IncludeSvgToggled(on) => settings::include_svg_toggled(state, on),
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
        Message::PngSizeInputChanged(s) => settings::png_size_input_changed(state, s),
        Message::PngSizeAddRequested => settings::png_size_add_requested(state),
        Message::PngSizeRemoveRequested(size) => settings::png_size_remove_requested(state, size),
        // -----------------------------------------------------------------
        // v1.4.1: vtracer preset + ExportPlan reset
        // -----------------------------------------------------------------
        Message::VtracerPresetChanged(preset) => {
            state.export_plan.vtracer_preset = preset;
            persist_settings(state);
            Task::none()
        }
        Message::ExportPlanResetRequested => settings::export_plan_reset_requested(state),

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
        Message::IncludeWebManifestToggled(on) => manifest::include_web_manifest_toggled(state, on),
        Message::WebManifestNameChanged(s) => manifest::web_manifest_name_changed(state, s),
        Message::WebManifestShortNameChanged(s) => {
            manifest::web_manifest_short_name_changed(state, s)
        }
        Message::WebManifestThemeColorChanged(s) => {
            manifest::web_manifest_theme_color_changed(state, s)
        }
        Message::WebManifestBackgroundColorChanged(s) => {
            manifest::web_manifest_background_color_changed(state, s)
        }

        // -----------------------------------------------------------------
        // v1.9.0: monochrome output set
        // -----------------------------------------------------------------
        Message::IncludeMonochromeToggled(on) => manifest::include_monochrome_toggled(state, on),

        // -----------------------------------------------------------------
        // v1.26.0: Microsoft app logo output
        // -----------------------------------------------------------------
        Message::IncludeMicrosoftAppLogosToggled(on) => {
            manifest::include_microsoft_app_logos_toggled(state, on)
        }

        // -----------------------------------------------------------------
        // v1.21.0: keep-transparency toggle
        // -----------------------------------------------------------------
        Message::KeepTransparencyToggled(on) => settings::keep_transparency_toggled(state, on),

        // v1.19.0: ExportRequested / ExportDirPicked / ExportCompleted handlers removed.
        // v1.16.0 moved to the ConvertCompleted + per-asset download path.
        Message::ToastTick => {
            state.holds.sweep(&mut state.toasts, Instant::now());
            Task::none()
        }
        Message::ToastAttention(id, attention) => toast::toast_attention(state, id, attention),
        Message::DismissToast(id) => {
            state.toasts.retain(|t| t.id != id);
            Task::none()
        }
        // The application opens no modal or menu overlay, so there is nothing to close.
        Message::CloseModals => Task::none(),
    }
}
