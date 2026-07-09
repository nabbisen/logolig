//! Locale enum (v1.6.0).
//!
//! English-only in v1.5.0; Japanese (`Ja`) added in v1.6.0.
//! Steps to add a new locale:
//!
//! 1. Add a variant to the `Locale` enum
//! 2. Update `Locale::all()` array length and elements
//! 3. Add a match arm to `as_bcp47` and `from_bcp47`
//! 4. Create `crates/logolig-i18n/locales/<tag>.toml`
//! 5. Add an `include_str!` branch in `dictionary::load`

use serde::{Deserialize, Serialize};

/// Supported locales.
///
/// Adding a variant forces compile errors in `Translator::for_locale`,
/// `Locale::all()`, `as_bcp47`, and `from_bcp47` match arms,
/// ensuring all sites are updated when a locale is added.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Locale {
    #[default]
    En,
    /// Japanese (v1.6.0).
    Ja,
}

impl Locale {
    /// All variants in order, for pick_list display.
    pub fn all() -> [Self; 2] {
        [Self::En, Self::Ja]
    }

    /// IETF BCP-47-style tag, matching the value stored in `PersistedSettings.locale`.
    pub fn as_bcp47(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Ja => "ja",
        }
    }

    /// Resolve a `Locale` from a BCP-47 tag string.
    /// Matches on the language subtag only (e.g. `"en-US"` → `"en"`).
    /// Returns `None` when unrecognised; the caller applies a fallback.
    ///
    /// Handles the major variations: `ja_JP.UTF-8` (macOS), `ja_JP` (Linux),
    /// `ja-JP` (browser), etc.
    pub fn from_bcp47(tag: &str) -> Option<Self> {
        // Match on the language subtag only to catch "en-US" / "en_US" / "en" etc.
        // Split on "." as well to handle POSIX locale strings like `ja_JP.UTF-8`.
        let primary = tag
            .split(['-', '_', '.'])
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        match primary.as_str() {
            "en" => Some(Self::En),
            "ja" => Some(Self::Ja),
            _ => None,
        }
    }
}

/// Detect the OS locale and resolve it to a `Locale`.
///
/// `sys-locale::get_locale()` abstracts LANG, macOS API, and Windows API.
/// Returns `Locale::default()` (English) when detection fails or the locale is unsupported.
///
/// In a v2 browser environment, `sys-locale` returns `navigator.language`,
/// so the same code works in the browser.
pub fn detect_system_locale() -> Locale {
    sys_locale::get_locale()
        .as_deref()
        .and_then(Locale::from_bcp47)
        .unwrap_or_default()
}
