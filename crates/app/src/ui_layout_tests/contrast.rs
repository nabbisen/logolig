//! Contrast, measured on what is **drawn** (RFC 034, job J21).
//!
//! The rule (RFC 034, requirement 3): every text reaches **7:1** against the surface it is drawn on
//! (icons and glyphs of controls, which are not text, **3:1**), in Light and in Dark. This file
//! checks the pictures: every text piece of the interface tree is looked up in the CPU-rendered
//! image of its screen; the ground is the most common colour of its box, the ink is the pixel
//! furthest from it, and the WCAG ratio of the two must reach the rule. Ink is the furthest pixel,
//! so anti-aliasing can only make the measure lower, never higher. The palette inventory (the
//! colours the helper functions and the theme give, and their ratios) is `contrast_inventory.rs`.

use std::collections::HashMap;

use iced::{Color, Rectangle, Size};
use snora::{Toast, ToastIntent};

use super::support::{Case, Combo, Kind, probe_of};
use crate::app::{AppState, Message, resolve_theme};
use crate::ui::contrast::{NON_TEXT_MIN, TEXT_MIN, contrast_ratio};
use logolig_i18n::Locale;

/// Tall, so that nothing is below the fold, and wide enough for the desktop layout.
const SIZE: Size = Size::new(1280.0, 3000.0);
const PHONE: Size = Size::new(360.0, 3000.0);
const SCALE: f32 = 2.0;

/// One measured text piece.
#[derive(Debug, Clone)]
pub(super) struct Measure {
    pub(super) screen: String,
    pub(super) text: String,
    pub(super) icon: bool,
    pub(super) ground: Color,
    pub(super) ink: Color,
    pub(super) ratio: f32,
}

impl Measure {
    pub(super) fn needed(&self) -> f32 {
        if self.icon { NON_TEXT_MIN } else { TEXT_MIN }
    }
    pub(super) fn describe(&self) -> String {
        format!(
            "{} {:?}{}: {:.2}:1 (needs {:.0}), ink {} on {}",
            self.screen,
            self.text,
            if self.icon { " [icon]" } else { "" },
            self.ratio,
            self.needed(),
            hex(self.ink),
            hex(self.ground)
        )
    }
}

pub(super) fn hex(c: Color) -> String {
    let b = |v: f32| (v * 255.0).round() as u8;
    format!("#{:02X}{:02X}{:02X}", b(c.r), b(c.g), b(c.b))
}

/// Glyphs of the bundled icon fonts are in the private use area: not text, 3:1.
fn is_icon(text: &str) -> bool {
    text.chars().all(|c| ('\u{e000}'..='\u{f8ff}').contains(&c))
}

/// The ground and the ink of the box `b` in an RGBA image `scale` pixels per logical pixel wide.
fn ground_and_ink(
    rgba: &[u8],
    width_px: usize,
    height_px: usize,
    b: Rectangle,
    inset: f32,
) -> Option<(Color, Color)> {
    let x0 = (((b.x + inset) * SCALE).floor().max(0.0)) as usize;
    let y0 = (((b.y + inset) * SCALE).floor().max(0.0)) as usize;
    let x1 = (((b.x + b.width - inset) * SCALE).ceil() as usize).min(width_px);
    let y1 = (((b.y + b.height - inset) * SCALE).ceil() as usize).min(height_px);
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    let px = |x: usize, y: usize| {
        let i = (y * width_px + x) * 4;
        [rgba[i], rgba[i + 1], rgba[i + 2]]
    };
    let mut hist: HashMap<[u8; 3], usize> = HashMap::new();
    for y in y0..y1 {
        for x in x0..x1 {
            *hist.entry(px(x, y)).or_default() += 1;
        }
    }
    let to_color = |p: [u8; 3]| Color::from_rgb8(p[0], p[1], p[2]);
    let ground = to_color(hist.iter().max_by_key(|(_, n)| **n).map(|(p, _)| *p)?);
    let ink = hist.keys().map(|p| to_color(*p)).max_by(|a, b| {
        contrast_ratio(*a, ground)
            .partial_cmp(&contrast_ratio(*b, ground))
            .expect("a ratio")
    })?;
    (contrast_ratio(ink, ground) > 1.05).then_some((ground, ink))
}

/// Every text piece of `state` at `size`, drawn with the theme the application resolves.
pub(super) fn measure_screen(name: &str, state: &AppState, size: Size) -> Vec<Measure> {
    let theme = resolve_theme(state);
    let mut probe = probe_of(state, size);
    let pieces = probe.walk();
    let rgba = probe.rgba(&theme, SCALE);
    let (w, h) = (
        (size.width * SCALE) as usize,
        (size.height * SCALE) as usize,
    );
    let mut out = Vec::new();
    for piece in pieces {
        let inset = if piece.kind == Kind::TextInput {
            4.0
        } else {
            0.0
        };
        if !matches!(piece.kind, Kind::Text | Kind::TextInput) || piece.text.trim().is_empty() {
            continue;
        }
        let Some((ground, ink)) = ground_and_ink(&rgba, w, h, piece.bounds, inset) else {
            continue;
        };
        out.push(Measure {
            screen: name.to_string(),
            icon: is_icon(&piece.text),
            text: piece.text.clone(),
            ratio: contrast_ratio(ink, ground),
            ground,
            ink,
        });
    }
    out
}

pub(super) fn combo(case: Case, size: Size, dark: bool) -> AppState {
    let mut state = Combo {
        case,
        size,
        locale: Locale::En,
        dark,
    }
    .state();
    // System resolves to what the case says; the explicit theme is what is drawn.
    state.theme = if dark {
        logolig::ThemeMode::Dark
    } else {
        logolig::ThemeMode::Light
    };
    state
}

/// Toasts of every intent the application raises, over the Converting screen (little else on it, so
/// that no text lies under a toast).
pub(super) fn with_toasts(dark: bool) -> AppState {
    let mut state = combo(Case::Converting, SIZE, dark);
    let toast = |id: u64, intent: ToastIntent, title: &str, body: &str| {
        let t = Toast::new(
            id,
            intent,
            title.to_string(),
            body.to_string(),
            Message::DismissToast(id),
        );
        if intent == ToastIntent::Error {
            t.persistent()
        } else {
            t
        }
    };
    state.toasts = vec![
        toast(
            1,
            ToastIntent::Error,
            "Operation failed",
            "notes.txt is not a supported image. Choose a PNG, SVG, WebP or JPEG file.",
        ),
        toast(
            2,
            ToastIntent::Warning,
            "Your logo is not square",
            "It is shown in full, with empty space around it, so the icon may look small.",
        ),
        toast(3, ToastIntent::Success, "Saved", "favicon.ico was saved."),
    ];
    state
}

/// All the screens the rule is measured on, both themes. The Checker case is left out: its Surface
/// group is disabled, and a disabled control is exempt from the ratio.
pub(super) fn all_measures() -> Vec<Measure> {
    let mut all = Vec::new();
    for dark in [false, true] {
        let theme_name = if dark { "Dark" } else { "Light" };
        for case in super::support::CASES {
            if case == Case::ResultPreviewChecker {
                continue;
            }
            all.extend(measure_screen(
                &format!("{case:?}/{theme_name}"),
                &combo(case, SIZE, dark),
                SIZE,
            ));
        }
        all.extend(measure_screen(
            &format!("HomePhone/{theme_name}"),
            &combo(Case::Home, PHONE, dark),
            PHONE,
        ));
        all.extend(measure_screen(
            &format!("Toasts/{theme_name}"),
            &with_toasts(dark),
            SIZE,
        ));
    }
    all
}

#[test]
fn the_ratio_function_is_the_wcag_one() {
    let (b, w) = (Color::BLACK, Color::WHITE);
    assert!((contrast_ratio(b, w) - 21.0).abs() < 0.01);
    assert!(
        (contrast_ratio(w, b) - 21.0).abs() < 0.01,
        "order does not matter"
    );
    assert!(
        (contrast_ratio(Color::from_rgb8(0x76, 0x76, 0x76), w) - 4.54).abs() < 0.01,
        "#767676 on white"
    );
    assert!((contrast_ratio(w, w) - 1.0).abs() < 1e-6);
}

#[test]
fn every_drawn_text_reaches_seven_to_one_and_every_icon_three_to_one() {
    let all = all_measures();
    assert!(
        all.len() > 400,
        "the measure ran over {} pieces only",
        all.len()
    );
    let mut failing: Vec<String> = all
        .iter()
        .filter(|m| m.ratio < m.needed())
        .map(Measure::describe)
        .collect();
    failing.sort();
    failing.dedup();
    assert!(
        failing.is_empty(),
        "{} of {} measured pieces are under the rule:\n{}",
        failing.len(),
        all.len(),
        failing.join("\n")
    );
}
