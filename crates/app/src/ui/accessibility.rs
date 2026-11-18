//! Accessibility helpers (§12, ABDD).
//!
//! - Do not convey state by colour alone; pair with a text marker
//!
//! The text markers below are the only helpers; an earlier module of screen-reader label
//! constants was never used and was removed (iced 0.14 exposes no accessibility tree).

pub mod marker {
    pub const BUSY: &str = "⏳";
    // Unused today: errors are shown as toasts. Kept because the marker set (BUSY, ERROR)
    // is the documented ABDD vocabulary (docs/src/ui-a11y.md); the cues are reviewed in a later
    // job (RFC 021) and no constant may change meaning here. The selected-option cue is no
    // text marker any more: single choices are `snora::radio_group`s (`ui/choice.rs`).
    #[allow(dead_code)]
    pub const ERROR: &str = "⚠";
}
