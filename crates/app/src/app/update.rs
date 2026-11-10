//! `update`: the state transitions (RFC 017: moved out of `app.rs`, unchanged apart from visibility).

mod input;

mod convert;

mod download;

mod nav;

mod settings;

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
        Message::KeepTransparencyToggled(on) => settings::keep_transparency_toggled(state, on),

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
