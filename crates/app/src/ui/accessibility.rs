//! Accessibility helpers (§12, ABDD).
//!
//! - Do not convey state by colour alone; pair with a text marker
//!
//! The text markers below are the only helpers; an earlier module of screen-reader label
//! constants was never used and was removed (iced 0.14 exposes no accessibility tree).

pub mod marker {
    pub const BUSY: &str = "⏳";
    // Unused today: errors are shown as toasts. Kept because the marker set (BUSY, ERROR, READY)
    // is the documented ABDD vocabulary (docs/src/ui-a11y.md); the cues are reviewed in a later
    // job and no constant may change meaning here.
    #[allow(dead_code)]
    pub const ERROR: &str = "⚠";
    pub const READY: &str = "✓";
}
