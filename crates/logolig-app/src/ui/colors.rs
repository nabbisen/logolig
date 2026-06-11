//! Theme-aware colour helpers (v1.14.0).
//!
//! ## Why functions instead of constants?
//!
//! Up to v1.13, colours were hardcoded constants like
//! `const FOO_COLOR: Color = Color::from_rgb(0.55, ...)` sized for the
//! light theme. On a dark background this caused:
//!
//! - App name (a muted dark grey) → invisible against a dark background
//! - Tagline (a light grey) → appears near-white on dark backgrounds
//! - Card borders (light grey) → disappear into the background
//!
//! Theme-reactive helper functions solve this by reading the active
//! palette at render time.

use iced::{Color, Theme};

// ---------------------------------------------------------------------------
// Text colours by semantic role
// ---------------------------------------------------------------------------

/// App name text colour.
///
/// Slightly de-emphasised (the image is the hero); still readable.
/// Role: secondary text —
/// uses `background.weak.text` from the palette
/// (good contrast with the background, not a focal point).
pub fn app_name(theme: &Theme) -> Color {
    let p = theme.extended_palette();
    p.background.weak.text
}

/// Tagline colour (the short description next to the app name).
///
/// Even more muted than the app name ("hint level"). The palette has
/// no "extra-muted" slot, so we use the same base as `app_name`
/// (`background.weak.text`) but dim it with **alpha=0.65**.
/// The relative weakening is preserved in both light and dark themes.
pub fn tagline(theme: &Theme) -> Color {
    with_alpha(app_name(theme), 0.65)
}

/// File name colour shown in the header on the edit / Result screen.
///
/// The file is the star of the screen, so it needs more prominence than the app name.
/// Normal body text (`background.base.text`) is strong enough.
pub fn file_name(theme: &Theme) -> Color {
    let p = theme.extended_palette();
    p.background.base.text
}

/// Page title colour (e.g. "Preview & Generate Favicon").
///
/// Top-level heading on the edit screen. Uses body-text strength;
/// prominence comes from size, not colour (colour emphasis would look heavy).
pub fn page_title(theme: &Theme) -> Color {
    let p = theme.extended_palette();
    p.background.base.text
}

/// Section title colour (e.g. "Preview" card label).
///
/// Secondary-level heading inside a card. Ideally between base and weak,
/// but the palette has no mid-slot — use weak (same tier as the app name).
pub fn section_label(theme: &Theme) -> Color {
    let p = theme.extended_palette();
    p.background.weak.text
}

/// Large-group heading in the settings (accordion header).
///
/// Needs to look pressable, so use near-normal text strength.
pub fn group_heading(theme: &Theme) -> Color {
    let p = theme.extended_palette();
    p.background.base.text
}

/// Blurb / supporting description / "at defaults: …" annotation.
///
/// Weaker than body text but still legible.

pub fn muted_text(theme: &Theme) -> Color {
    with_alpha(theme.extended_palette().background.base.text, 0.6)
}

/// Drop-zone headline colour.
///
/// The primary "Drop …" message — bold enough to be the focal element.
pub fn drop_zone_headline(theme: &Theme) -> Color {
    let p = theme.extended_palette();
    p.background.base.text
}

// ---------------------------------------------------------------------------
// Background / border colours
// ---------------------------------------------------------------------------
//
// Preview-card and drop-zone-card background/border are set inline
// in `container::style` closures that receive `&Theme` at render time.
// No double-wrapping needed — the closure is already theme-aware.
//
// v1.17.0: `badge_muted_bg` helper removed (was used for the "at defaults"
// badge, which was removed; trivial to restore if needed).


// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return a copy of the given `Color` with its alpha replaced. Used since
/// v1.14.0 to express "weaker" variants of theme colours.
fn with_alpha(c: Color, alpha: f32) -> Color {
    Color { a: alpha, ..c }
}
