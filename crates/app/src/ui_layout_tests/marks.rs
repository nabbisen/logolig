//! The selection marks are the accent, in both themes (RFC 021 amendment of 2026-11-24, RFC 034 J21b).
//!
//! snora's engine path draws a checked box and a selected radio in the theme's `primary.strong`, held at
//! 3:1 against the page and the two tints a hovered and a pressed row put behind it (`held_against`,
//! floor 3.05), and falls back to black or white when it does not hold. The application decides the mark
//! by what `ui/theme.rs` makes `primary.strong`: here it is **the same colour as the primary fill** of
//! the theme. Measured on the drawn picture: the mark in the box and the dot in the ring.

use std::collections::HashMap;

use iced::{Color, Size};
use logolig_i18n::Locale;

use super::support::{Case, Combo, probe_of};
use crate::app::resolve_theme;
use crate::ui::colors::{self, PrimaryState};
use crate::ui::contrast::contrast_ratio;

const SIZE: Size = Size::new(1280.0, 3000.0);
const SCALE: f32 = 2.0;

fn rgb8(c: Color) -> [u8; 3] {
    let b = |v: f32| (v * 255.0).round() as u8;
    [b(c.r), b(c.g), b(c.b)]
}

/// Pixels in the box left of the text `label` (the checkbox or radio of its row), as a histogram.
fn mark_histogram(dark: bool, label: &str) -> HashMap<[u8; 3], usize> {
    let mut state = Combo {
        case: Case::CustomizeAdvanced,
        size: SIZE,
        locale: Locale::En,
        dark,
    }
    .state();
    state.theme = if dark {
        logolig::ThemeMode::Dark
    } else {
        logolig::ThemeMode::Light
    };
    let theme = resolve_theme(&state);
    let mut probe = probe_of(&state, SIZE);
    let walk = probe.walk();
    let text = walk
        .iter()
        .find(|s| s.text == label)
        .unwrap_or_else(|| panic!("no text {label:?}"));
    let rgba = probe.rgba(&theme, SCALE);
    let width_px = (SIZE.width * SCALE) as usize;
    let (x0, x1) = (
        ((text.bounds.x - 30.0) * SCALE) as usize,
        ((text.bounds.x - 2.0) * SCALE) as usize,
    );
    let (y0, y1) = (
        ((text.bounds.y - 4.0) * SCALE) as usize,
        ((text.bounds.y + text.bounds.height + 4.0) * SCALE) as usize,
    );
    let mut hist = HashMap::new();
    for y in y0..y1 {
        for x in x0..x1 {
            let i = (y * width_px + x) * 4;
            *hist.entry([rgba[i], rgba[i + 1], rgba[i + 2]]).or_default() += 1;
        }
    }
    hist
}

fn count_near(hist: &HashMap<[u8; 3], usize>, want: [u8; 3]) -> usize {
    hist.iter()
        .filter(|(p, _)| p.iter().zip(want).all(|(a, b)| a.abs_diff(b) <= 3))
        .map(|(_, n)| *n)
        .sum()
}

fn theme_of(dark: bool) -> iced::Theme {
    if dark {
        crate::ui::theme::dark()
    } else {
        crate::ui::theme::light()
    }
}

#[test]
fn the_mark_is_the_primary_fill_in_both_themes_drawn() {
    for dark in [false, true] {
        let theme = theme_of(dark);
        let fill = rgb8(colors::primary_fill(&theme, PrimaryState::Rest));
        for (what, label) in [
            ("checked box", "32 × 32"),
            ("selected radio", "Default (balanced)"),
        ] {
            let hist = mark_histogram(dark, label);
            let n = count_near(&hist, fill);
            assert!(
                n >= 20,
                "{} {what}: expected the primary fill {fill:?}, found {n} such pixels (not black/white, not another blue)",
                if dark { "Dark" } else { "Light" }
            );
        }
    }
}

#[test]
fn the_unchecked_outlines_are_unchanged() {
    // J21's measurements (control-marks-measured): black on white / #E6E6E6 on the dark page, both 10:1 or more.
    for (dark, outline, want) in [
        (false, "16 × 16", [0u8, 0, 0]),
        (true, "16 × 16", [0xE6, 0xE6, 0xE6]),
    ] {
        let hist = mark_histogram(dark, outline);
        assert!(
            count_near(&hist, want) >= 10,
            "dark={dark}: the outline of an unchecked box is {want:?}"
        );
    }
}

/// What snora holds the mark against: the page and the hovered and pressed tints (`background.weak`,
/// `background.strong`); its floor is 3.05. The tick or dot against the mark needs 7:1.
#[test]
fn the_mark_holds_snoras_contrast_and_its_tick_reads() {
    for dark in [false, true] {
        let theme = theme_of(dark);
        let p = theme.extended_palette();
        let mark = p.primary.strong.color;
        for (name, behind) in [
            ("page", p.background.base.color),
            ("hovered row (weak)", p.background.weak.color),
            ("pressed row (strong)", p.background.strong.color),
        ] {
            let r = contrast_ratio(mark, behind);
            assert!(
                r >= 3.05,
                "{} mark against {name}: {r:.2}",
                if dark { "Dark" } else { "Light" }
            );
        }
        assert!(
            contrast_ratio(p.primary.strong.text, mark) >= 7.0,
            "the tick on the mark"
        );
        assert_eq!(
            rgb8(mark),
            rgb8(p.primary.base.color),
            "the mark is the same colour as the primary fill"
        );
    }
}
