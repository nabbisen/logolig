//! P3: target sizes (RFC 035; RFC 034, job J22).
//!
//! An interactive area is where the pointer finds something to press (a button, a checkbox, a
//! pick list, a text input). The rule of RFC 034 (requirement 4), in logical pixels, in **both**
//! axes: a **primary** control is at least [`PRIMARY_MIN`] (44) and every other control at least
//! [`MIN`] (32). The primary controls are the named list in [`is_primary`]: adding a primary control
//! without listing it is a review question.
//!
//! **The one exception, pinned and commented:** snora's own checkbox rows (the label at 16 px plus
//! snora's padding; `Checkbox` has no padding or size setter) measure about 30.5 px. They are held to
//! [`CHECKBOX_MIN`] (30), the shortfall against 32 is reported in the review request, and the request
//! to snora is the architect's. The interface is laid out at the matrix's width with the height
//! stretched to 4000 px, so that nothing the viewport hides is skipped.

use iced::{Rectangle, Size};

use super::support::{Combo, Finding, Seen, label_in, layout_matrix, probe_of};
use crate::app::AppState;
use logolig::MessageKey;

/// The least size of a primary control.
pub(super) const PRIMARY_MIN: f32 = 44.0;
/// The least size of any other control.
pub(super) const MIN: f32 = 32.0;
/// snora's checkbox row (RFC 034 J22): measured about 30.5 px; held to 30.
pub(super) const CHECKBOX_MIN: f32 = 30.0;
/// Measurement tolerance (the area is measured to a quarter pixel).
const TOLERANCE: f32 = 0.3;

/// The primary controls, by the label the area carries, in the language of `state`: "Download all
/// (ZIP)", "← Back" and the three navigation items. ("Choose file…" is the chip inside the drop-zone
/// card, whose whole area is the control and is far larger than 44 px.)
fn is_primary(state: &AppState, label: &str) -> bool {
    let t = &state.translator;
    [
        MessageKey::ResultDownloadAllButton,
        MessageKey::EditCancelButton,
        MessageKey::NavHome,
        MessageKey::NavCustomize,
        MessageKey::NavSettings,
    ]
    .iter()
    .any(|key| t.t(*key) == label)
}

/// Is this the row of one of snora's checkboxes (by its label)? The preset sizes ("32 × 32") and the
/// labelled options of the Customize page.
fn is_checkbox_row(state: &AppState, label: &str) -> bool {
    let t = &state.translator;
    let size_row =
        label.split(" × ").count() == 2 && label.split(" × ").all(|n| n.parse::<u32>().is_ok());
    size_row
        || [
            MessageKey::KeepTransparency,
            MessageKey::IncludeAppleTouchLabel,
            MessageKey::IncludeHtmlSnippetLabel,
            MessageKey::IncludeSvgLabel,
            MessageKey::VectorizeOnRasterLabel,
            MessageKey::IncludeWebManifestLabel,
            MessageKey::IncludeMicrosoftAppLogosLabel,
            MessageKey::IncludeMonochromeLabel,
        ]
        .iter()
        .any(|key| t.t(*key) == label)
}

/// Every text inside the area (a control with an icon has the icon's glyph first).
fn labels_in(seen: &[Seen], area: Rectangle) -> Vec<String> {
    seen.iter()
        .filter(|s| {
            s.kind == super::support::Kind::Text
                && !s.text.trim().is_empty()
                && area.contains(s.bounds.center())
        })
        .map(|s| s.text.clone())
        .collect()
}

/// The tick of a checked snora checkbox (a glyph of the bundled icon font).
const TICK: &str = "\u{e06c}";

/// What an area with these texts has to measure.
fn needed(state: &AppState, labels: &[String]) -> f32 {
    if labels.iter().any(|l| is_primary(state, l)) {
        PRIMARY_MIN
    } else if labels
        .iter()
        .any(|l| l == TICK || is_checkbox_row(state, l))
    {
        CHECKBOX_MIN
    } else {
        MIN
    }
}

/// The areas of `areas` smaller than `need` in either direction.
pub(super) fn smaller_than(areas: &[Rectangle], need: f32) -> Vec<Rectangle> {
    areas
        .iter()
        .copied()
        .filter(|a| a.width < need - TOLERANCE || a.height < need - TOLERANCE)
        .collect()
}

/// The findings of one combination.
pub(super) fn findings_of(combo: &Combo) -> Vec<Finding> {
    let state = combo.state();
    let tall = Size::new(combo.size.width, 4000.0);
    let mut probe = probe_of(&state, tall);
    let seen: Vec<Seen> = probe.walk();
    let areas = probe.interactive_areas(&seen);
    areas
        .into_iter()
        .filter_map(|a| {
            let labels = labels_in(&seen, a);
            let label = label_in(&seen, a);
            let need = needed(&state, &labels);
            (a.width < need - TOLERANCE || a.height < need - TOLERANCE).then(|| Finding {
                combo: combo.describe(),
                what: format!(
                    "{:?} at ({:.0}, {:.0}) is {:.1} x {:.1} px, needs {need:.0}",
                    label, a.x, a.y, a.width, a.height,
                ),
            })
        })
        .collect()
}

/// P3 over the whole matrix: no exception list.
#[test]
fn primary_controls_are_44_and_every_other_control_32_with_snoras_checkbox_pinned_at_30() {
    let all: Vec<Finding> = layout_matrix().iter().flat_map(findings_of).collect();
    assert!(
        all.is_empty(),
        "{} finding(s) of {} combinations:\n{}",
        all.len(),
        layout_matrix().len(),
        all.iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    );
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
    let small = smaller_than(&areas, 24.0);
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
        (Case::ResultPreview, 2400.0),
        (Case::ResultPreviewChecker, 2400.0),
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

// ---- the toast's close control, and the measured exception -------------------------------------

/// What the toast's close control measures: 23.75 px, snora's (`control`'s rule: `max(24, 1.5 x text)`, drawn
/// by `snora::toast`; the application sets neither). **Not the rule of RFC 034 (32) and not decided by me:**
/// it is held to the 1.28.0 rule of 23.5 (the old 24 minus the measurement tolerance) until the architect says
/// whether it joins the snora request with the checkbox rows; the shortfall is reported in the J22 request.
const TOAST_CLOSE_MEASURED: f32 = 23.5;

/// Toasts of every intent over the Home page: one close control each, none smaller than it is today.
#[test]
fn the_toasts_close_control_is_snoras_and_is_not_smaller_than_it_was() {
    use snora::{Toast, ToastIntent};
    for dark in [false, true] {
        let size = Size::new(1280.0, 800.0);
        let mut state = Combo {
            case: super::support::Case::Home,
            size,
            locale: logolig_i18n::Locale::En,
            dark,
        }
        .state();
        state.toasts = [
            ToastIntent::Error,
            ToastIntent::Warning,
            ToastIntent::Success,
        ]
        .into_iter()
        .enumerate()
        .map(|(i, intent)| {
            Toast::new(
                i as u64,
                intent,
                "Title".to_string(),
                "A message of a few words.".to_string(),
                crate::app::Message::DismissToast(i as u64),
            )
            .persistent()
        })
        .collect();
        let before = {
            let plain = Combo {
                case: super::support::Case::Home,
                size,
                locale: logolig_i18n::Locale::En,
                dark,
            }
            .state();
            let mut p = probe_of(&plain, size);
            let seen = p.walk();
            p.interactive_areas(&seen).len()
        };
        let mut probe = probe_of(&state, size);
        let seen = probe.walk();
        let areas = probe.interactive_areas(&seen);
        assert_eq!(areas.len(), before + 3, "one close control per toast");
        let small = smaller_than(&areas, TOAST_CLOSE_MEASURED + 0.3);
        assert!(small.is_empty(), "dark={dark}: {small:?}");
    }
}
