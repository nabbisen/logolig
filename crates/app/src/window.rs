//! Window settings (RFC 025).
//!
//! The only thing configured here is the window's application id, so that desktop
//! environments can match the window to its desktop entry (icon, name, grouping).
//! Every other window setting keeps its `iced` default.

use iced::window;

/// Application id of the Logolig window (Wayland `app_id`; X11 `WM_CLASS`).
///
/// It must equal the base name of the installed desktop file
/// (`io.github.nabbisen.Logolig.desktop`). Linux only: other platforms keep their
/// default window settings.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub(crate) const APP_ID: &str = "io.github.nabbisen.Logolig";

/// Window settings for the application.
///
/// On Linux the application id is set; on every other target the result is exactly
/// `window::Settings::default()`.
pub(crate) fn settings() -> window::Settings {
    #[cfg_attr(not(target_os = "linux"), allow(unused_mut))]
    let mut settings = window::Settings::default();
    #[cfg(target_os = "linux")]
    {
        settings.platform_specific.application_id = APP_ID.to_string();
    }
    settings
}

#[cfg(test)]
mod tests;
