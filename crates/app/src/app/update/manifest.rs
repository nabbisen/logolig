//! Handlers for the web manifest, monochrome and Microsoft app logo options
//! (RFC 017: extracted from `update`; every body is the old arm body unchanged).

use iced::Task;

use super::super::persist::persist_settings;
use super::super::{AppState, Message};

/// Handler for `Message::IncludeWebManifestToggled`.
pub(super) fn include_web_manifest_toggled(state: &mut AppState, on: bool) -> Task<Message> {
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

/// Handler for `Message::WebManifestNameChanged`.
pub(super) fn web_manifest_name_changed(state: &mut AppState, s: String) -> Task<Message> {
    if let Some(m) = state.export_plan.web_manifest.as_mut() {
        m.name = s;
        persist_settings(state);
    }
    Task::none()
}

/// Handler for `Message::WebManifestShortNameChanged`.
pub(super) fn web_manifest_short_name_changed(state: &mut AppState, s: String) -> Task<Message> {
    if let Some(m) = state.export_plan.web_manifest.as_mut() {
        m.short_name = s;
        persist_settings(state);
    }
    Task::none()
}

/// Handler for `Message::WebManifestThemeColorChanged`.
pub(super) fn web_manifest_theme_color_changed(state: &mut AppState, s: String) -> Task<Message> {
    // No live validation — interrupting the user mid-type is poor UX.
    // `is_valid_color` is validated at export time, not here,
    // so partial values like #FF or #FFFFF are accepted while typing.
    if let Some(m) = state.export_plan.web_manifest.as_mut() {
        m.theme_color = s;
        persist_settings(state);
    }
    Task::none()
}

/// Handler for `Message::WebManifestBackgroundColorChanged`.
pub(super) fn web_manifest_background_color_changed(
    state: &mut AppState,
    s: String,
) -> Task<Message> {
    if let Some(m) = state.export_plan.web_manifest.as_mut() {
        m.background_color = s;
        persist_settings(state);
    }
    Task::none()
}

/// Handler for `Message::IncludeMonochromeToggled`.
pub(super) fn include_monochrome_toggled(state: &mut AppState, on: bool) -> Task<Message> {
    // Simple bool flag. Same persist_settings pattern as include_ico
    // and include_apple_touch.
    //
    // No effect on the preview cache — monochrome output is generated
    // only at export time and has no preview representation.

    state.export_plan.monochrome = on;
    persist_settings(state);
    Task::none()
}

/// Handler for `Message::IncludeMicrosoftAppLogosToggled`.
pub(super) fn include_microsoft_app_logos_toggled(state: &mut AppState, on: bool) -> Task<Message> {
    // Advanced, opt-in output group. Like monochrome, it only affects
    // generated result assets; preview cache is unchanged.
    state.export_plan.include_microsoft_app_logos = on;
    persist_settings(state);
    Task::none()
}
