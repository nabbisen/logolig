//! Handlers for the web manifest, monochrome and Microsoft app logo options
//! (RFC 017: extracted from `update`; every body is the old arm body unchanged).

use iced::Task;

use super::super::persist::persist_settings;
use super::super::{AppState, ManifestField, ManifestInputs, Message};

/// Handler for `Message::IncludeWebManifestToggled`.
pub(super) fn include_web_manifest_toggled(state: &mut AppState, on: bool) -> Task<Message> {
    // On: insert WebManifestSettings::default(). Off: set to None.
    // Any user-entered values are discarded on toggle-off (simple state model); the edit buffers go
    // with them, so no error or half-typed text comes back (RFC 012).
    state.export_plan.web_manifest = if on {
        Some(logolig::WebManifestSettings::default())
    } else {
        None
    };
    state.manifest_inputs = match state.export_plan.web_manifest.as_ref() {
        Some(settings) => ManifestInputs::from_settings(settings),
        None => ManifestInputs::default(),
    };
    persist_settings(state);
    Task::none()
}

/// The plan's value of `field` (the canonical text in use).
fn plan_value(m: &mut logolig::WebManifestSettings, field: ManifestField) -> &mut String {
    match field {
        ManifestField::Name => &mut m.name,
        ManifestField::ShortName => &mut m.short_name,
        ManifestField::ThemeColor => &mut m.theme_color,
        ManifestField::BackgroundColor => &mut m.background_color,
    }
}

/// Puts the canonical value into the plan and saves, **only if it differs** from what is there: an
/// unchanged value writes nothing (RFC 012: no more saving on every keystroke).
fn commit(state: &mut AppState, field: ManifestField, canonical: String) {
    let Some(m) = state.export_plan.web_manifest.as_mut() else {
        return;
    };
    let slot = plan_value(m, field);
    if *slot != canonical {
        *slot = canonical;
        persist_settings(state);
    }
}

/// A keystroke in a field (RFC 012): the buffer takes the text as typed; the plan takes it only if it
/// is valid (canonicalised), otherwise the previous valid value stays and nothing is saved. iced's
/// `text_input` has no blur event, so a keystroke that makes the text valid commits at once and
/// Enter (`web_manifest_submitted`) additionally shows the canonical form.
fn edited(state: &mut AppState, field: ManifestField, text: String) -> Task<Message> {
    if state.export_plan.web_manifest.is_none() {
        return Task::none();
    }
    let canonical = field.normalize(&text);
    state.manifest_inputs.set(field, text);
    if let Some(canonical) = canonical {
        commit(state, field, canonical);
    }
    keep_error_in_view(state, field)
}

/// A line that has just appeared under the focused field must be in view (release 1.29.0, R2b): the
/// field and its line are one container for `keep_in_view`, and the operation runs after the page is
/// rebuilt with the line, in the same step (no timer). Nothing is asked when the field shows no error.
fn keep_error_in_view(state: &AppState, field: ManifestField) -> Task<Message> {
    if state.manifest_inputs.shows_error(field) {
        crate::focus::keep_in_view()
    } else {
        Task::none()
    }
}

/// Handler for `Message::WebManifestNameChanged`.
pub(super) fn web_manifest_name_changed(state: &mut AppState, s: String) -> Task<Message> {
    edited(state, ManifestField::Name, s)
}

/// Handler for `Message::WebManifestShortNameChanged`.
pub(super) fn web_manifest_short_name_changed(state: &mut AppState, s: String) -> Task<Message> {
    edited(state, ManifestField::ShortName, s)
}

/// Handler for `Message::WebManifestThemeColorChanged`.
pub(super) fn web_manifest_theme_color_changed(state: &mut AppState, s: String) -> Task<Message> {
    edited(state, ManifestField::ThemeColor, s)
}

/// Handler for `Message::WebManifestBackgroundColorChanged`.
pub(super) fn web_manifest_background_color_changed(
    state: &mut AppState,
    s: String,
) -> Task<Message> {
    edited(state, ManifestField::BackgroundColor, s)
}

/// Handler for `Message::WebManifestSubmitted` (Enter in a field): the field is touched; a valid text
/// is committed and the buffer shows the canonical form (`what you see is what is written`); an
/// invalid one changes nothing and shows its error.
pub(super) fn web_manifest_submitted(state: &mut AppState, field: ManifestField) -> Task<Message> {
    if state.export_plan.web_manifest.is_none() {
        return Task::none();
    }
    state.manifest_inputs.touch(field);
    if let Some(canonical) = field.normalize(state.manifest_inputs.get(field)) {
        state.manifest_inputs.set(field, canonical.clone());
        commit(state, field, canonical);
    }
    keep_error_in_view(state, field)
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
