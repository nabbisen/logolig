//! The type scale (RFC 034, job J22): every text size of the interface is one of five named steps,
//! and every icon one of two. No literal size is written under `ui/` or in `shell.rs`; a source test
//! (`ui_layout_tests/type_scale.rs`) fails on one. The preview's mock-ups (`preview_panel/mockups.rs`)
//! depict other applications (a browser tab, a phone's home screen) and are exempt.
//!
//! Steps are in logical pixels and ascend. Text with no size set draws at iced's default of 16 px,
//! which is `BODY`.

/// `CAPTION`, 13: **only** text that no action depends on: the footer "Logolig — favicon generator".
/// The floor of the scale; nothing is smaller.
pub const CAPTION: f32 = 13.0;
/// `SECONDARY`, 14: supporting lines: the blurbs under a section, the details of a result card, the
/// navigation labels, the shortcut descriptions, a group's helper line.
pub const SECONDARY: f32 = 14.0;
/// `BODY`, 16: everything the user reads or acts on: labels, button text, what a control does,
/// the text of a field, messages.
pub const BODY: f32 = 16.0;
/// `TITLE`, 20: the titles of sections and of the preview.
pub const TITLE: f32 = 20.0;
/// `HEADING`, 28: the page headline ("Conversion complete!", the drop zone's invitation) and the
/// large glyph standing for a text asset.
pub const HEADING: f32 = 28.0;

/// The small icon: beside body text (a chevron, a download mark, an alert sign).
pub const ICON_SMALL: f32 = 16.0;
/// The large icon: the navigation items and the busy line.
pub const ICON_LARGE: f32 = 22.0;
