//! The application's two themes (RFC 034, job J21): **one place** defines Light and Dark.
//!
//! Both are iced `Theme::custom` themes. Their palettes keep the background and text colours of
//! iced's built-in Light and Dark and the look of the old accent, with the **primary colour moved
//! far enough for white (Light) or black (Dark) text to read on it at 7:1**. The extended palette
//! is iced's own, then **post-processed**: every tier's text colour is lifted until it reaches 7:1
//! against the tier's own fill (`lift_text`), so every default widget that draws tier text, and
//! snora's controls on the engine path, are at the rule by construction. A tier whose fill is in
//! the middle of the luminance range (about 0.10 to 0.30) cannot reach 7:1 with either black or
//! white; the palette below has none that carries text (the test in `contrast_inventory.rs` fails
//! if one appears).

use iced::theme::palette::{Extended, Pair, Palette};
use iced::{Color, Theme};

use super::contrast::{TEXT_TARGET, at_least};

/// The accent, Light: darkened from `#5865F2` (4.61:1 with white) to 7.5:1, the same blue family.
pub const PRIMARY_LIGHT: Color = Color::from_rgb(
    0x3C as f32 / 255.0,
    0x47 as f32 / 255.0,
    0xB8 as f32 / 255.0,
);
/// The accent, Dark: lightened so that **black** text reads at 8.6:1 and the fill stands out from the page.
/// J21b: `#8A93F7` (J21) became `#979FF8`, the nearest colour of the same hue that **also** holds snora's 3:1
/// against the pressed-row tint (`background.strong`: 2.75:1 before, 3.12:1 now), because the selection marks
/// are drawn in this same colour (see `generate`): one accent, one blue, in both themes.
pub const PRIMARY_DARK: Color = Color::from_rgb(
    0x97 as f32 / 255.0,
    0x9F as f32 / 255.0,
    0xF8 as f32 / 255.0,
);

/// Success, `#12664F` darkened (white on it was 6.9:1) until white text reads at 7:1+.
const SUCCESS: Color = Color::from_rgb(
    0x11 as f32 / 255.0,
    0x62 as f32 / 255.0,
    0x4C as f32 / 255.0,
);
/// Danger, `#C3423F` darkened (white on it was 5.0:1) until white text reads at 7:1+.
const DANGER: Color = Color::from_rgb(
    0x9C as f32 / 255.0,
    0x33 as f32 / 255.0,
    0x31 as f32 / 255.0,
);
/// Warning on Light, `#B77E33` lightened (black on it was 6.1:1) until black text reads at 7:1+.
/// (Dark keeps iced's `#FFC14E`: black on it is 13:1.)
const WARNING_LIGHT: Color = Color::from_rgb(
    0xC8 as f32 / 255.0,
    0x8A as f32 / 255.0,
    0x39 as f32 / 255.0,
);

/// iced's `Palette::LIGHT` with the primary, success, warning and danger colours moved as above.
pub const LIGHT: Palette = Palette {
    primary: PRIMARY_LIGHT,
    success: SUCCESS,
    warning: WARNING_LIGHT,
    danger: DANGER,
    ..Palette::LIGHT
};

/// iced's `Palette::DARK` with the primary, success and danger colours moved as above.
pub const DARK: Palette = Palette {
    primary: PRIMARY_DARK,
    success: SUCCESS,
    danger: DANGER,
    ..Palette::DARK
};

pub fn light() -> Theme {
    Theme::custom_with_fn("Logolig Light", LIGHT, generate)
}

pub fn dark() -> Theme {
    Theme::custom_with_fn("Logolig Dark", DARK, generate)
}

/// iced's extended palette with every tier's text lifted to 7:1 against its own fill.
fn generate(palette: Palette) -> Extended {
    let mut e = Extended::generate(palette);
    // J21b (RFC 021 amendment of 2026-11-24): the selection marks are the accent, the same colour as the
    // primary fill. snora draws a checked box and a selected radio in `primary.strong`, held at 3:1 (floor
    // 3.05) against the page and the hovered and pressed row tints, and falls back to black or white when
    // it does not hold. So `primary.strong` is the primary itself, and the two primaries above are chosen
    // to hold (Light 7.5 / 5.6 / 4.8:1, Dark 5.7 / 3.8 / 3.1:1; the test in `marks.rs` pins it).
    e.primary.strong.color = e.primary.base.color;
    let b = &mut e.background;
    for pair in [
        &mut b.base,
        &mut b.weakest,
        &mut b.weaker,
        &mut b.weak,
        &mut b.neutral,
        &mut b.strong,
        &mut b.stronger,
        &mut b.strongest,
    ] {
        lift(pair);
    }
    for pair in [
        &mut e.primary.base,
        &mut e.primary.weak,
        &mut e.primary.strong,
    ] {
        lift(pair);
    }
    for pair in [
        &mut e.secondary.base,
        &mut e.secondary.weak,
        &mut e.secondary.strong,
    ] {
        lift(pair);
    }
    for pair in [
        &mut e.success.base,
        &mut e.success.weak,
        &mut e.success.strong,
    ] {
        lift(pair);
    }
    for pair in [
        &mut e.warning.base,
        &mut e.warning.weak,
        &mut e.warning.strong,
    ] {
        lift(pair);
    }
    for pair in [&mut e.danger.base, &mut e.danger.weak, &mut e.danger.strong] {
        lift(pair);
    }
    e
}

fn lift(pair: &mut Pair) {
    pair.text = at_least(pair.text, &[pair.color], TEXT_TARGET);
}
