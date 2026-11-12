//! P3: every interactive area is at least 24 × 24 px (RFC 035).
//!
//! An interactive area is where the pointer finds something to press (a button, a checkbox, a
//! pick list, a text input, the slider). 24 px is the least a finger or a pointer hits reliably
//! (WCAG 2.2, target size minimum). The interface is laid out at the matrix's width with the
//! height stretched to 4000 px, so that nothing the viewport hides is skipped: the size of a
//! control does not depend on the window's height.

use iced::{Rectangle, Size};

use super::support::{Combo, Finding, Seen, label_in, layout_matrix, probe_of};

/// The least size, in logical pixels, of an interactive area.
pub(super) const MINIMUM: f32 = 24.0;
/// Measurement tolerance (the area is measured to a quarter pixel).
const TOLERANCE: f32 = 0.3;

/// The areas of `areas` smaller than [`MINIMUM`] in either direction.
pub(super) fn too_small(areas: &[Rectangle]) -> Vec<Rectangle> {
    areas
        .iter()
        .copied()
        .filter(|a| a.width < MINIMUM - TOLERANCE || a.height < MINIMUM - TOLERANCE)
        .collect()
}

/// The findings of one combination.
pub(super) fn findings_of(combo: &Combo) -> Vec<Finding> {
    let state = combo.state();
    let tall = Size::new(combo.size.width, 4000.0);
    let mut probe = probe_of(&state, tall);
    let seen: Vec<Seen> = probe.walk();
    let areas = probe.interactive_areas(&seen);
    too_small(&areas)
        .into_iter()
        .map(|a| Finding {
            combo: combo.describe(),
            what: format!(
                "{:?} at ({:.0}, {:.0}) is {:.1} x {:.1} px, smallest side {}",
                label_in(&seen, a),
                a.x,
                a.y,
                a.width,
                a.height,
                a.width.min(a.height).floor() as u32
            ),
        })
        .collect()
}

/// P3 over the whole matrix, minus the exceptions named in [`KNOWN`].
#[test]
fn every_interactive_area_is_at_least_24_by_24() {
    let all: Vec<Finding> = layout_matrix().iter().flat_map(findings_of).collect();
    let unexpected: Vec<_> = all.iter().filter(|f| !is_known(f)).collect();
    assert!(
        unexpected.is_empty(),
        "{} finding(s) of {} combinations:\n{}",
        unexpected.len(),
        layout_matrix().len(),
        unexpected
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// Interactive areas smaller than 24 x 24 today. Cause: every checkbox row is 16.5 px high, and
/// the custom-size field and its button are 23.5 px high, and the SVG slider is 16 px high.
/// Belongs to RFC 034, job J22: each entry is a control by its label (the field and the slider
/// have none) and the whole pixels of its smallest side, as the finding prints them; the job
/// deletes entries as it fixes them, and the list must end empty. No new `(no label)` entry may
/// be added. A loosened threshold would hide them; a named list does not.
const KNOWN: &[(&str, &str)] = &[
    ("(no label)", "15"),
    ("(no label)", "23"),
    ("+ Add custom size", "23"),
    ("+ カスタムサイズ追加", "23"),
    ("16 × 16", "16"),
    ("192 × 192", "16"),
    ("32 × 32", "16"),
    ("48 × 48", "16"),
    ("512 × 512", "16"),
    ("96 × 96", "16"),
    ("Keep transparency (alpha)", "16"),
    ("Microsoft app logos を出力", "16"),
    ("Output Microsoft app logos", "16"),
    ("Output favicon.svg", "16"),
    ("Output manifest.webmanifest", "16"),
    ("Output mono/ grayscale set", "16"),
    ("Vectorize raster sources to SVG (vtracer)", "16"),
    ("apple-touch-icon.png (iOS / iPadOS home screen)", "16"),
    ("apple-touch-icon.png (iOS / iPadOS のホーム画面用)", "16"),
    ("favicon-snippet.html (<head> に貼れるマークアップ)", "16"),
    ("favicon-snippet.html (paste-ready <head> markup)", "16"),
    ("favicon.svg を出力", "16"),
    ("manifest.webmanifest を出力", "16"),
    ("mono/ グレースケールセットを出力", "16"),
    ("ラスタ画像を SVG にベクトル化 (vtracer)", "16"),
    ("透過 (アルファ) を維持する", "16"),
];

fn entry_matches(entry: &(&str, &str), f: &Finding) -> bool {
    f.what.starts_with(&format!("{:?} at ", entry.0))
        && f.what.ends_with(&format!("smallest side {}", entry.1))
}

fn is_known(f: &Finding) -> bool {
    KNOWN.iter().any(|entry| entry_matches(entry, f))
}

// ---- self-test: the property can fail ---------------------------------------------------------

/// A 10 x 10 button must be reported; a 40 x 40 one must not.
#[test]
fn p3_reports_a_ten_by_ten_button() {
    use iced::widget::{button, column, text};
    let view = column![
        button(text("")).width(10).height(10).on_press(()),
        button(text("")).width(40).height(40).on_press(()),
    ]
    .spacing(20);
    let mut probe = super::support::Probe::new(view.into(), Size::new(300.0, 300.0));
    let seen = probe.walk();
    let areas = probe.interactive_areas(&seen);
    assert_eq!(areas.len(), 2, "both buttons are found: {areas:?}");
    let small = too_small(&areas);
    assert_eq!(
        small.len(),
        1,
        "only the 10 x 10 button is reported: {small:?}"
    );
    assert!(
        (small[0].width - 10.0).abs() < 1.0 && (small[0].height - 10.0).abs() < 1.0,
        "the reported area is about 10 x 10: {:?}",
        small[0]
    );
}

// ---- self-test: the measurement finds every interactive point ---------------------------------

/// Sweeps a grid over two pages and checks that every point where the pointer finds something
/// to press lies inside an area `interactive_areas` reported, so the property does not miss a
/// control (the slider and the pick lists give no text to start from).
#[test]
fn the_measured_areas_cover_every_interactive_point() {
    use super::support::Case;
    for (case, height) in [
        (Case::CustomizeAdvanced, 1600.0),
        (Case::ResultAllOptions, 2400.0),
    ] {
        let combo = Combo {
            case,
            size: Size::new(1280.0, 720.0),
            locale: logolig_i18n::Locale::En,
            dark: false,
        };
        let state = combo.state();
        let mut probe = probe_of(&state, Size::new(1280.0, height));
        let seen = probe.walk();
        let areas = probe.interactive_areas(&seen);
        let mut missed = Vec::new();
        let mut y = 2.0;
        while y < height {
            let mut x = 2.0;
            while x < 1280.0 {
                let p = iced::Point::new(x, y);
                if probe.is_interactive_at(p) && !areas.iter().any(|a| inflated(a).contains(p)) {
                    missed.push(p);
                }
                x += 9.0;
            }
            y += 9.0;
        }
        assert!(
            missed.is_empty(),
            "{case:?}: {} interactive point(s) outside every area, first {:?}",
            missed.len(),
            missed.first()
        );
    }
}

/// The area grown by half a pixel on each side: a grid point exactly on an edge belongs to it.
fn inflated(a: &Rectangle) -> Rectangle {
    Rectangle::new(
        iced::Point::new(a.x - 0.5, a.y - 0.5),
        Size::new(a.width + 1.0, a.height + 1.0),
    )
}
