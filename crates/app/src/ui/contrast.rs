//! WCAG contrast (RFC 034, job J21): the ratio, and a way to derive a colour that is guaranteed to
//! reach a ratio against a background.
//!
//! The application's rule (RFC 034, requirement 3): **text reaches 7:1** (WCAG AAA) against the
//! surface it is drawn on, **controls' borders and marks reach 3:1** (WCAG 1.4.11). The relative
//! luminance and the ratio are the WCAG 2.1 definitions (`https://www.w3.org/TR/WCAG21/#dfn-contrast-ratio`).

use iced::Color;

/// The minimum ratio of text (WCAG AAA).
pub const TEXT_MIN: f32 = 7.0;
/// The minimum ratio of a control's border or mark (WCAG 1.4.11).
pub const NON_TEXT_MIN: f32 = 3.0;
/// What a derived text colour is aimed at: the minimum plus a margin, because the colour is stored in
/// 8 bits per channel and the picture is rounded again (a colour derived at exactly 7.00 can measure
/// 6.99 once drawn).
pub const TEXT_TARGET: f32 = TEXT_MIN + 0.1;
/// The same for borders and marks.
pub const NON_TEXT_TARGET: f32 = NON_TEXT_MIN + 0.1;

/// WCAG 2.1 relative luminance of an opaque colour (alpha is ignored: composite first with [`over`]).
pub fn relative_luminance(c: Color) -> f32 {
    let lin = |v: f32| {
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(c.r) + 0.7152 * lin(c.g) + 0.0722 * lin(c.b)
}

/// WCAG 2.1 contrast ratio, 1.0 to 21.0, of two opaque colours (order does not matter).
pub fn contrast_ratio(a: Color, b: Color) -> f32 {
    let (la, lb) = (relative_luminance(a), relative_luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// The colour **nearest to `fg`** on the straight line towards black or white (whichever is further
/// from `bg`) that reaches `min` against every colour of `backgrounds`. `fg` itself if it already
/// does. Deterministic and monotone: the result keeps the hue of `fg` as far as the ratio allows,
/// and the ratio is **guaranteed** (the extreme, black or white, is the last resort).
pub fn at_least(fg: Color, backgrounds: &[Color], min: f32) -> Color {
    let ok = |c: Color| backgrounds.iter().all(|bg| contrast_ratio(c, *bg) >= min);
    let fg = Color { a: 1.0, ..fg };
    if ok(fg) {
        return fg;
    }
    // Towards white when the backgrounds are dark on average, towards black otherwise.
    let mean = backgrounds
        .iter()
        .map(|b| relative_luminance(*b))
        .sum::<f32>()
        / backgrounds.len() as f32;
    let target = if mean < 0.18 {
        Color::WHITE
    } else {
        Color::BLACK
    };
    for step in 1..=100 {
        let t = step as f32 / 100.0;
        let c = Color::from_rgb(
            fg.r + (target.r - fg.r) * t,
            fg.g + (target.g - fg.g) * t,
            fg.b + (target.b - fg.b) * t,
        );
        if ok(c) {
            return c;
        }
    }
    target
}
