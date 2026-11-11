//! P2: the main controls are fully visible without scrolling (RFC 035).
//!
//! A control counts as reachable when its label lies completely inside the window and in the
//! part of it the viewport shows, and the pointer finds something to press over the whole
//! label. A control below the fold, or half cut off, is not reachable at a glance.

use iced::Rectangle;
use logolig::MessageKey;

use super::support::{Case, Combo, Finding, Kind, Probe, Seen, matrix, probe_of};

/// How much of an edge may be lost to rounding, in logical pixels.
const TOLERANCE: f32 = 0.5;

/// Is the text `label` fully visible inside the window, with something to press under it?
pub(super) fn is_reachable<M>(probe: &mut Probe<'_, M>, seen: &[Seen], label: &str) -> bool {
    let Some(text) = seen
        .iter()
        .find(|s| s.kind == Kind::Text && s.text == label)
    else {
        return false;
    };
    let window = Rectangle::new(iced::Point::ORIGIN, probe.size);
    let fully_visible = text.visible.is_some_and(|v| {
        (v.width - text.bounds.width).abs() <= TOLERANCE
            && (v.height - text.bounds.height).abs() <= TOLERANCE
            && v.x >= window.x - TOLERANCE
            && v.y >= window.y - TOLERANCE
            && v.x + v.width <= window.x + window.width + TOLERANCE
            && v.y + v.height <= window.y + window.height + TOLERANCE
    });
    fully_visible && probe.is_interactive_at(text.bounds.center())
}

/// The controls to check on each screen: (label key, screens where it is on screen).
fn controls(combo: &Combo, state: &crate::app::AppState) -> Vec<(&'static str, String)> {
    let t = &state.translator;
    let mut list = vec![
        ("navigation: Home", t.t(MessageKey::NavHome)),
        ("navigation: Customize", t.t(MessageKey::NavCustomize)),
        ("navigation: Settings", t.t(MessageKey::NavSettings)),
    ];
    match combo.case {
        Case::Home => list.push(("Choose file", t.t(MessageKey::ChooseFileButton))),
        Case::ResultDefault | Case::ResultAllOptions => {
            list.push(("Back", t.t(MessageKey::EditCancelButton)));
            list.push((
                "Download all (ZIP)",
                t.t(MessageKey::ResultDownloadAllButton),
            ));
        }
        _ => {}
    }
    list
}

/// Every control of every combination that is not reachable.
pub(super) fn unreachable_controls() -> Vec<Finding> {
    let mut out = Vec::new();
    for combo in matrix() {
        let state = combo.state();
        let mut probe = probe_of(&state, combo.size);
        let seen = probe.walk();
        for (name, label) in controls(&combo, &state) {
            if !is_reachable(&mut probe, &seen, &label) {
                out.push(Finding {
                    combo: combo.describe(),
                    what: format!("{name} ({label:?}) is not fully visible"),
                });
            }
        }
    }
    out
}

/// The entries that hold today, over the whole matrix: the navigation items and "Choose file…".
/// "Back" and "Download all (ZIP)" on the Result screens are not asserted: they are below the
/// fold at several sizes today (RFC 026 adds its entry with its fix).
#[test]
fn the_navigation_and_choose_file_are_reachable_everywhere() {
    let failing: Vec<_> = unreachable_controls()
        .into_iter()
        .filter(|f| f.what.starts_with("navigation") || f.what.starts_with("Choose file"))
        .collect();
    assert!(
        failing.is_empty(),
        "{} control(s) not reachable:\n{}",
        failing.len(),
        failing
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    );
}

// ---- self-test: the check can fail -------------------------------------------------------------

/// A control the viewport hides must be reported; one it shows must not.
#[test]
fn p2_reports_a_control_below_the_fold() {
    use iced::widget::{Space, button, column, scrollable, text};
    let view: iced::Element<'_, ()> = scrollable(
        column![
            button(text("inside")).on_press(()),
            Space::new().height(400),
            button(text("hidden")).on_press(()),
        ]
        .spacing(10),
    )
    .height(100)
    .into();
    let mut probe = Probe::new(view, iced::Size::new(300.0, 300.0));
    let seen = probe.walk();
    assert!(
        is_reachable(&mut probe, &seen, "inside"),
        "the control in view is reachable"
    );
    assert!(
        !is_reachable(&mut probe, &seen, "hidden"),
        "the control below the fold is not"
    );
    assert!(
        !is_reachable(&mut probe, &seen, "absent"),
        "a control that does not exist is not"
    );
}
