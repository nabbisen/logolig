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

use iced::theme::palette::mix;
use iced::{Color, Theme};

use super::contrast::{NON_TEXT_TARGET, TEXT_TARGET, at_least};

// ---------------------------------------------------------------------------
// Text colours by semantic role
// ---------------------------------------------------------------------------

/// The accent of the active page's bar in the navigation (RFC 021 D1, D7; RFC 034 J21).
///
/// The accent colour is for the primary action and the active page; it no longer marks a selected
/// option (form controls are neutral). The bar is a mark of the control, so it reaches 3:1 (WCAG
/// 1.4.11) against **both** neighbours: the page it sits flush against and the sidebar's own
/// surface. In 1.28.0 it was 3.7:1 / 2.5:1 on Dark and 4.6:1 / 3.4:1 on Light. The item is also
/// filled and its label bold: the bar is never the only cue.
pub fn nav_accent(theme: &Theme) -> Color {
    let p = theme.extended_palette();
    // RFC 034 J21: the primary, moved only as far as 3:1 against **both** neighbours (the page and the
    // sidebar's own surface). On Light the primary already reaches 7.5:1 and 5.6:1; on Dark it is
    // lightened until it reaches 3:1 against the sidebar (it was 2.5:1 there).
    at_least(
        p.primary.base.color,
        &[surface_page(theme), surface_card(theme)],
        NON_TEXT_TARGET,
    )
}

/// Body text: the palette's own text colour on the page (reaches 7:1 by construction, see `theme`).
pub fn body_text(theme: &Theme) -> Color {
    theme.extended_palette().background.base.text
}

/// App name text colour. Unchanged in value from 1.28.0 (the palette's text, already at the rule):
/// de-emphasised by size, not by colour.
pub fn app_name(theme: &Theme) -> Color {
    text_on(theme, surface_card(theme))
}

/// File name colour shown in the header on the edit / Result screen.
///
/// The file is the star of the screen, so it needs more prominence than the app name.
/// Normal body text is strong enough.
pub fn file_name(theme: &Theme) -> Color {
    body_text(theme)
}

/// Page title colour (e.g. "Preview & Generate Favicon").
///
/// Top-level heading on the edit screen. Uses body-text strength;
/// prominence comes from size, not colour (colour emphasis would look heavy).
pub fn page_title(theme: &Theme) -> Color {
    body_text(theme)
}

/// Section title colour (e.g. "Preview" card label): unchanged in value from 1.28.0 (the palette's
/// text, lifted for the card it sits on).
pub fn section_label(theme: &Theme) -> Color {
    text_on(theme, surface_card(theme))
}

/// Large-group heading in the settings (accordion header).
///
/// Needs to look pressable, so use near-normal text strength.
pub fn group_heading(theme: &Theme) -> Color {
    body_text(theme)
}

/// Blurb / supporting description / "at defaults: …" annotation: **secondary text** (RFC 034 J21).
///
/// A **solid** colour (the old `0.6`-alpha body colour measured 5.0 to 5.7:1), between the body text
/// and the background, moved as far towards the body text as the rule needs: **7:1 against the page
/// and against a card** (the `weak` tier), which are the two surfaces secondary text is drawn on.
/// On Dark the card's tier leaves little room, so there this colour is close to the body text; the
/// hierarchy is carried by size and weight, not by a colour that fails.
pub fn muted_text(theme: &Theme) -> Color {
    muted_on(theme, &[surface_page(theme), surface_card(theme)])
}

/// The page surface (`background.base`).
pub fn surface_page(theme: &Theme) -> Color {
    theme.extended_palette().background.base.color
}

/// The card and sidebar surface (`background.weak`).
pub fn surface_card(theme: &Theme) -> Color {
    theme.extended_palette().background.weak.color
}

/// The hover / active fill of the navigation and of auxiliary buttons (`background.strong`).
pub fn surface_strong(theme: &Theme) -> Color {
    theme.extended_palette().background.strong.color
}

/// Secondary text that reaches 7:1 against **every** surface of `surfaces`.
pub fn muted_on(theme: &Theme, surfaces: &[Color]) -> Color {
    let p = theme.extended_palette();
    let start = mix(p.background.base.text, p.background.base.color, 0.35);
    at_least(start, surfaces, TEXT_TARGET)
}

/// Text on a fill `bg` that is not the page: the body colour moved until it reaches 7:1 on `bg`.
pub fn text_on(theme: &Theme, bg: Color) -> Color {
    at_least(
        theme.extended_palette().background.base.text,
        &[bg],
        TEXT_TARGET,
    )
}

/// The label of a navigation item (RFC 034 J21): the active one sits on the `strong` fill and is
/// body-strength text; an inactive one is secondary text that must read on the sidebar (`weak`)
/// **and** on the `strong` hover fill.
pub fn nav_label(theme: &Theme, active: bool) -> Color {
    if active {
        text_on(theme, surface_strong(theme))
    } else {
        muted_on(theme, &[surface_card(theme), surface_strong(theme)])
    }
}

/// The states of a filled primary control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimaryState {
    Rest,
    Hovered,
    Pressed,
}

/// The fill of a primary control (RFC 034 J21): the theme's primary at rest; **darker on Light,
/// lighter on Dark** when hovered or pressed, the direction in which the text keeps its ratio.
pub fn primary_fill(theme: &Theme, state: PrimaryState) -> Color {
    let p = theme.extended_palette();
    let base = p.primary.base.color;
    let (towards, amount) = match (p.is_dark, state) {
        (_, PrimaryState::Rest) => return base,
        (false, PrimaryState::Hovered) => (Color::BLACK, 0.12),
        (false, PrimaryState::Pressed) => (Color::BLACK, 0.24),
        (true, PrimaryState::Hovered) => (Color::WHITE, 0.12),
        (true, PrimaryState::Pressed) => (Color::WHITE, 0.24),
    };
    mix(base, towards, amount)
}

/// The text on a primary fill: white on Light, black on Dark, moved only if the ratio needs it.
pub fn on_primary(theme: &Theme, fill: Color) -> Color {
    let preferred = if theme.extended_palette().is_dark {
        Color::BLACK
    } else {
        Color::WHITE
    };
    at_least(preferred, &[fill], TEXT_TARGET)
}

/// The faint accent tint a card takes while a file hovers over it: the page with a little of the
/// primary in it. Pale on Light, dark on Dark, so that body and secondary text keep their ratio.
pub fn hover_tint(theme: &Theme) -> Color {
    let p = theme.extended_palette();
    mix(p.background.base.color, p.primary.base.color, 0.12)
}

/// The border of the "Reset to defaults" button: the danger colour moved until it reaches 3:1
/// against the page and a card (the colour says "careful", the words say what it does).
pub fn danger_border(theme: &Theme) -> Color {
    let p = theme.extended_palette();
    at_least(
        p.danger.base.color,
        &[surface_page(theme), surface_card(theme)],
        NON_TEXT_TARGET,
    )
}

/// The colour of a control's border or mark (RFC 034 J21: WCAG 1.4.11, 3:1): the `strong` tier moved
/// along the line to the text colour until it reaches 3:1 against the page **and** against a card.
pub fn control_border(theme: &Theme) -> Color {
    let p = theme.extended_palette();
    at_least(
        p.background.strong.color,
        &[surface_page(theme), surface_card(theme)],
        NON_TEXT_TARGET,
    )
}

/// The placeholder of a text field: secondary text on the field's own surface (the page).
pub fn placeholder(theme: &Theme) -> Color {
    muted_on(theme, &[surface_page(theme), surface_card(theme)])
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
