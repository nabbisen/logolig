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

    /// The one colour rule of the application (RFC 012): `#RRGGBB` or the shorthand `#RGB`, any letter
    /// case, surrounding white space ignored. Returns the canonical form, `#RRGGBB` in upper case
    /// (`#1ab` becomes `#11AABB`), or `None` for anything else (no `#`, another length, a character
    /// that is not a hex digit, white space inside, a CSS function or a colour name). The stored form
    /// stays `#RRGGBB`; the shorthand is only ever accepted on input.
    pub fn normalize_color(s: &str) -> Option<String> {
        let digits = s.trim().strip_prefix('#')?;
        if !digits.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let expanded: String = match digits.len() {
            3 => digits.chars().flat_map(|c| [c, c]).collect(),
            6 => digits.to_string(),
            _ => return None,
        };
        Some(format!("#{}", expanded.to_ascii_uppercase()))
    }

    /// A name the manifest can carry: the text trimmed, which must not be empty. Returns the trimmed
    /// text, or `None` for an empty or white-space-only name.
    pub fn normalize_text(s: &str) -> Option<String> {
        let trimmed = s.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    }

    /// The settings as they may be used (RFC 012, the loader's defence): each valid value is kept in
    /// its canonical form (names trimmed, colours `#RRGGBB` upper case), each invalid one is replaced
    /// by its default. The flag says whether any value had to be **replaced** (canonicalising a valid
    /// value is not a replacement).
    pub fn sanitized(&self) -> (Self, bool) {
        let defaults = Self::default();
        let mut replaced = false;
        let mut pick = |value: Option<String>, default: String| {
            value.unwrap_or_else(|| {
                replaced = true;
                default
            })
        };
        let clean = Self {
            name: pick(Self::normalize_text(&self.name), defaults.name),
            short_name: pick(Self::normalize_text(&self.short_name), defaults.short_name),
            theme_color: pick(
                Self::normalize_color(&self.theme_color),
                defaults.theme_color,
            ),
            background_color: pick(
                Self::normalize_color(&self.background_color),
                defaults.background_color,
            ),
        };
        (clean, replaced)
    }

    /// Validate that `name` and `short_name` are both non-empty.
    /// Some browsers reject PWA install with empty names; checked before export.
    pub fn has_required_text(&self) -> bool {
        !self.name.trim().is_empty() && !self.short_name.trim().is_empty()
    }
}
