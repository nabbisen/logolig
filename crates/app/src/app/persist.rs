//! Saving the settings.

use logolig::{MessageKey, SettingsStore};

use super::AppState;
use super::toast::push_warning_toast;

// ----------------------------------------------------------------------
// v1.4.0: settings persistence
// ----------------------------------------------------------------------

/// Assembles a `PersistedSettings` snapshot from the current `AppState`.
fn snapshot_persisted(state: &AppState) -> logolig::PersistedSettings {
    logolig::PersistedSettings {
        export_plan: state.export_plan.clone(),
        theme: state.theme,
        // v1.5.0: save the user locale override as a BCP-47 tag.
        // None means fall back to OS detection on next launch.
        locale: state.locale_override.map(|loc| loc.as_bcp47().to_string()),
    }
}

/// Saves settings immediately (eager-save strategy, §1.4.0).
///
/// No-op if `state.store` is `None` (storage init failed at startup).
/// Shows a warning toast on save failure; the app continues running.
///
/// Note: this is called on every settings change. The current
/// `PersistedSettings` payload is a few KB so I/O cost is negligible,
/// but a debounce / lazy-save strategy may be needed if it grows.
pub(super) fn persist_settings(state: &mut AppState) {
    let Some(store) = state.store.as_ref() else {
        return;
    };
    let snapshot = snapshot_persisted(state);
    if let Err(err) = store.save(&snapshot) {
        // Use a transient warning (not persistent) to avoid flooding the UI.
        // v1.5.0: uses the active Translator.
        let title = state.translator.t(MessageKey::ToastSettingsSaveFailedTitle);
        let body = state.translator.t_args(
            MessageKey::ToastSettingsSaveFailedBody,
            &[("error", &err.to_string())],
        );
        push_warning_toast(state, &title, &body);
    }
}
