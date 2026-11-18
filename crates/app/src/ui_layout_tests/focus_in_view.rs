//! Focus stays in view (RFC 027, requirement of 2026-11-19; job J14a).
//!
//! A keyboard user who tabs into a control below the fold must see it: after every Tab and
//! Shift+Tab the focused control lies **fully inside the window and inside every scroll area that
//! holds it** (the control's own bounds; the ring sits 2 px further out and is kept visible by the
//! margin the scrolling leaves). Measured on the real key path (`focus_keys::press_tab`: the event,
//! the subscription's filter, `focus::on_tab`, the widget operations), over every page state and
//! size of the matrix, walking forward through all stops and back again.

use iced::widget::{column, scrollable, text};
use iced::{Element, Size};

use super::focus_keys::{shift_tab, tab};
use super::support::{Probe, layout_matrix, probe_of};

#[test]
fn after_every_tab_and_shift_tab_the_focused_control_is_fully_visible() {
    let mut problems: Vec<String> = Vec::new();
    let mut moves = 0;
    for combo in layout_matrix() {
        let state = combo.state();
        let mut p = probe_of(&state, combo.size);
        let stops = p.focus_targets().len();
        let mut check = |p: &mut Probe<'_, crate::app::Message>, what: String| {
            moves += 1;
            let view = p.focused_view().expect("a stop is focused after the move");
            if !view.fully_visible() {
                problems.push(format!(
                    "[{}] {what}: the focused control is at {:?} (viewports {:?})",
                    combo.describe(),
                    view.on_screen,
                    view.viewports
                ));
            }
        };
        for i in 0..stops {
            tab(&mut p);
            check(&mut p, format!("Tab to stop {i} of {stops}"));
        }
        for i in (0..stops.saturating_sub(1)).rev() {
            shift_tab(&mut p);
            check(&mut p, format!("Shift+Tab back to stop {i} of {stops}"));
        }
    }
    assert!(
        moves > 400,
        "the property ran over {moves} focus moves only"
    );
    problems.dedup();
    assert!(
        problems.is_empty(),
        "{} focus move(s) left the control out of view of {moves}; first ones:\n{}",
        problems.len(),
        problems
            .iter()
            .take(12)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// The check can fail, and the operation fixes what it finds: a list longer than its scroll area,
/// a control far below the fold.
#[test]
fn a_control_below_the_fold_fails_the_check_until_it_is_scrolled_into_view() {
    let long: Element<'_, ()> = scrollable(
        column((0..30).map(|i| snora::control(text(format!("row {i}"))).on_press(()).into()))
            .spacing(8),
    )
    .height(200)
    .into();
    let mut p = Probe::new(long, Size::new(300.0, 300.0));
    p.focus_target(25);
    let before = p.focused_view().expect("focused");
    assert!(
        !before.fully_visible(),
        "row 25 starts below the fold: {before:?}"
    );
    p.run_task(crate::focus::keep_in_view::<()>());
    let after = p.focused_view().expect("still focused");
    assert!(after.fully_visible(), "scrolled into view: {after:?}");
    // By the least amount, with room for the ring: the control ends 4 px above the area's edge.
    let (shown, control) = (after.viewports[0], after.on_screen);
    assert!(
        ((shown.y + shown.height) - (control.y + control.height) - 4.0).abs() < 0.6,
        "the control ends {} px above the bottom edge, expected the ring's 4",
        (shown.y + shown.height) - (control.y + control.height)
    );
    // And back up: twenty-five steps of Shift+Tab, the view still where it was.
    for _ in 0..25 {
        p.run_chained(iced_test::core::widget::operation::focusable::focus_previous::<()>());
    }
    let up = p.focused_view().expect("focused");
    assert!(
        !up.fully_visible(),
        "row 0 is above the shown part now: {up:?}"
    );
    p.run_task(crate::focus::keep_in_view::<()>());
    let top = p.focused_view().expect("focused");
    assert!(top.fully_visible(), "scrolled back up: {top:?}");
}
