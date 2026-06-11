//! Accessibility helpers (§12, ABDD).
//!
//! - Labels are centralised for screen-reader consistency
//! - Use plain short phrases, not abbreviations
//! - Do not convey state by colour alone; pair with a text marker
//!
//! Some labels are defined here for future use and are not yet wired up.

#[allow(dead_code)]
pub mod label {
    pub const APP_TITLE: &str = "Logolig";
    pub const DROP_ZONE: &str = "Drop a PNG, SVG, or WebP image here, or activate to choose a file";
    pub const CHOOSE_FILE_BTN: &str = "Choose source image file";
    pub const TOGGLE_THEME_BTN: &str = "Toggle theme (System / Light / Dark)";
    pub const TOGGLE_ADVANCED_BTN: &str = "Show or hide advanced settings";
    pub const EXPORT_BTN: &str = "Export favicons to disk";
}

#[allow(dead_code)]
pub mod marker {
    pub const BUSY: &str = "⏳";
    pub const ERROR: &str = "⚠";
    pub const READY: &str = "✓";
}
