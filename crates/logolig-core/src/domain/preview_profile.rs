//! Context preview settings (§5.2).
//!
//! Preview is not about displaying an image — it is about confirming how
//! the favicon looks in the context where it will actually be used.
//!
//! ## v1.10.0: TransparencyChecker promoted
//!
//! The `state.preview_checker: bool` introduced in v1.7.0 was merged into
//! `PreviewContext::TransparencyChecker`. This lets three mutually exclusive
//! views (Browser tab / Phone home / Checker) be represented by a single
//! enum, preventing impossible combinations like "tab view + checker" at
//! the type level.

use crate::domain::theme_mode::ThemeMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewContext {
    /// Simulate a 16×16 browser-tab favicon.
    BrowserTab16,
    /// Simulate a phone home-screen icon.
    SmartphoneIcon,
    /// Transparency checker view (v1.10.0). No framing — icon at actual size
    /// over a checkerboard. Promoted from the `state.preview_checker: bool` in v1.7.
    TransparencyChecker,
}

impl PreviewContext {
    pub fn label(self) -> &'static str {
        match self {
            Self::BrowserTab16 => "Browser tab (16×16)",
            Self::SmartphoneIcon => "Smartphone home",
            Self::TransparencyChecker => "Transparency checker",
        }
    }

    /// All variants in display order, for building the "View as" button group in the UI.
    /// 
    pub fn all() -> [Self; 3] {
        [
            Self::BrowserTab16,
            Self::SmartphoneIcon,
            Self::TransparencyChecker,
        ]
    }

    /// Whether the `Surface` background setting has any effect in this context.
    /// Checker uses a checkerboard background; Surface is ignored.
    /// Used by the UI to disable the Surface buttons in Checker mode.
    pub fn respects_surface(self) -> bool {
        match self {
            Self::BrowserTab16 | Self::SmartphoneIcon => true,
            Self::TransparencyChecker => false,
        }
    }
}

/// Preview display settings.
///
/// `background` is for display only; the image itself is never modified (§5.2).
#[derive(Debug, Clone, Copy)]
pub struct PreviewProfile {
    pub context: PreviewContext,
    pub background: ThemeMode,
}

impl Default for PreviewProfile {
    fn default() -> Self {
        Self {
            context: PreviewContext::BrowserTab16,
            background: ThemeMode::System,
        }
    }
}
