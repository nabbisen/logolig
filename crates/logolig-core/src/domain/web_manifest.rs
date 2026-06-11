//! Web manifest settings (v1.8.0).
//!
//! Input data for generating `manifest.webmanifest`
//! (W3C Web App Manifest spec: https://www.w3.org/TR/appmanifest/).
//!
//! ## v1.8 scope decision
//!
//! The W3C spec defines 30+ fields. logolig covers only what is needed
//! to make a PWA installable to the home screen with a correct appearance:
//!
//! - `name`             — full app name (home-screen label)
//! - `short_name`       — used in narrow contexts (≤12 chars recommended)
//! - `theme_color`      — browser UI accent colour (`#RRGGBB`)
//! - `background_color` — splash-screen background (`#RRGGBB`)
//! - `icons`            — auto-generated from configured PNG sizes
//!
//! Fields like `description`, `categories`, `screenshots`, and `widgets`
//! are out of scope for a favicon generator.

use serde::{Deserialize, Serialize};

/// Web manifest settings.
///
/// `serde(default)` on all fields lets settings.json written by earlier versions
/// load cleanly even when new fields are added.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebManifestSettings {
    /// Full app name; shown as the home-screen label on PWA install.
    /// Some browsers reject install when empty; avoid leaving this blank.
    #[serde(default = "default_name")]
    pub name: String,

    /// Short name. Fallback when space is tight on the home screen.
    /// Shorter than `name`; 12 characters or fewer recommended (W3C).
    #[serde(default = "default_short_name")]
    pub short_name: String,

    /// Theme colour (`#RRGGBB`). Used as the browser UI accent colour.
    /// E.g. address-bar / notification-bar background.
    #[serde(default = "default_theme_color")]
    pub theme_color: String,

    /// Background colour (`#RRGGBB`). Splash-screen background on PWA launch.
    /// Choose a colour that contrasts with the icon.
    #[serde(default = "default_background_color")]
    pub background_color: String,
}

fn default_name() -> String {
    "My App".to_string()
}

fn default_short_name() -> String {
    "App".to_string()
}

fn default_theme_color() -> String {
    "#FFFFFF".to_string()
}

fn default_background_color() -> String {
    "#FFFFFF".to_string()
}

impl Default for WebManifestSettings {
    fn default() -> Self {
        Self {
            name: default_name(),
            short_name: default_short_name(),
            theme_color: default_theme_color(),
            background_color: default_background_color(),
        }
    }
}

impl WebManifestSettings {
    /// Validate that the string is a 7-character `#RRGGBB` hex colour.
    /// The W3C spec also accepts `#RGB` and named colours (`red`), but logolig
    /// restricts to `#RRGGBB` per §5 "reduce decision fatigue".
    ///
    /// Validation is called by the UI layer on user input. This function
    /// is a pure predicate with no side effects.
    pub fn is_valid_color(s: &str) -> bool {
        if s.len() != 7 {
            return false;
        }
        let bytes = s.as_bytes();
        if bytes[0] != b'#' {
            return false;
        }
        bytes[1..].iter().all(|c| c.is_ascii_hexdigit())
    }

    /// Validate that `name` and `short_name` are both non-empty.
    /// Some browsers reject PWA install with empty names; checked before export.
    pub fn has_required_text(&self) -> bool {
        !self.name.trim().is_empty() && !self.short_name.trim().is_empty()
    }
}
