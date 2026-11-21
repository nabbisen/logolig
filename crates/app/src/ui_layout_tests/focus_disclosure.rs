//! The focus on a disclosure control (RFC 027, job J14d): the "Advanced settings" header and the
//! Result screen's preview toggle open and close a section below them, which **adds** (or removes)
//! Tab stops. A keyboard user who presses one must still be on it: the ring stays, Space or Enter
//! again closes it, and Tab goes on from there in layout order.

use iced::Size;
use iced::keyboard::{Modifiers, key::Named};

use super::focus_after::Session;
use super::support::{Case, Kind};

const SIZE: Size = Size::new(1280.0, 4000.0);

/// Every text inside the focused stop, joined (an icon glyph comes first in a row with an icon).
fn stop_text(s: &mut Session) -> String {
    s.with(|p| {
        let Some(stop) = p.focus_targets().into_iter().find(|t| t.focused) else {
            return String::new();
        };
        p.walk()
            .iter()
            .filter(|w| w.kind == Kind::Text && stop.bounds.contains(w.bounds.center()))
            .map(|w| w.text.clone())
            .collect::<Vec<_>>()
            .join("")
    })
}

/// Tabs until the focused stop holds `label` (a bounded walk: no page has more than 60 stops).
fn tab_to(s: &mut Session, label: &str) -> usize {
    for _ in 0..60 {
        s.tab();
        if stop_text(s).ends_with(label) {
            return s.focus().0.expect("a stop is focused");
        }
    }
    panic!("no Tab stop reads {label:?}");
}

/// The Customize page with the Advanced section closed.
fn customize_closed() -> Session {
    let mut s = Session::new(Case::CustomizeAdvanced, SIZE);
    s.state.advanced_extras_open = false;
    s
}

fn the_control_keeps_the_focus(
    s: &mut Session,
    label: &str,
    at: usize,
    opened: impl Fn(&Session) -> bool,
) {
    for (key, what) in [(Named::Space, "Space"), (Named::Enter, "Enter")] {
        let was = opened(s);
        s.press(key, Modifiers::empty());
        assert_eq!(opened(s), !was, "{what} toggled the section");
        assert_eq!(
            s.focus().0,
            Some(at),
            "{what}: the focus is still on {label:?} (open: {})",
            opened(s)
        );
        assert!(
            stop_text(s).ends_with(label),
            "{what}: and it reads {label:?}"
        );
        let view = s.with(|p| p.focused_view()).expect("a focused control");
        assert!(
            view.fully_visible(),
            "{what}: the control is in view: {view:?}"
        );
    }
}

#[test]
fn space_and_enter_on_the_advanced_header_keep_the_focus_on_it() {
    let mut s = customize_closed();
    let at = tab_to(&mut s, "Advanced settings");
    the_control_keeps_the_focus(&mut s, "Advanced settings", at, |s| {
        s.state.advanced_extras_open
    });
}

#[test]
fn space_and_enter_on_the_preview_toggle_keep_the_focus_on_it() {
    let mut s = Session::new(Case::ResultDefault, SIZE);
    let at = tab_to(&mut s, "Preview");
    the_control_keeps_the_focus(&mut s, "Preview", at, |s| s.state.result_preview_open);
}

/// From the control, Tab goes to the next stop in layout order (the first control of the section
/// that just opened, or the next one after it when the section closed), Shift+Tab to the one
/// before it.
#[test]
fn tab_goes_on_from_the_disclosure_control_in_layout_order() {
    for (case, label) in [
        (Case::CustomizeAdvanced, "Advanced settings"),
        (Case::ResultDefault, "Preview"),
    ] {
        let mut s = Session::new(case, SIZE);
        if case == Case::CustomizeAdvanced {
            s.state.advanced_extras_open = false;
        }
        let at = tab_to(&mut s, label);
        let closed_stops = s.focus().1;
        s.press(Named::Space, Modifiers::empty()); // opens
        let open_stops = s.focus().1;
        assert!(
            open_stops > closed_stops,
            "{label}: opening adds stops ({closed_stops} -> {open_stops})"
        );
        s.tab();
        assert_eq!(
            s.focus().0,
            Some(at + 1),
            "{label}: Tab goes to the next stop"
        );
        s.press_shift_tab();
        assert_eq!(s.focus().0, Some(at), "{label}: Shift+Tab comes back to it");
        s.press(Named::Space, Modifiers::empty()); // closes
        assert_eq!(
            s.focus().1,
            closed_stops,
            "{label}: closing removes them again"
        );
        assert_eq!(s.focus().0, Some(at), "{label}: still on it after closing");
        s.tab();
        assert_eq!(
            s.focus().0,
            Some(at + 1),
            "{label}: and Tab goes on after closing"
        );
    }
}

/// What the pointer does, stated: a click on the control publishes the same message as Space, so
/// it also leaves the control focused (a ring). This is a known consequence, not a goal: the
/// message cannot tell the two apart, and snora's public `Control` does not report focus
/// changes. If it ever can, the click should leave nothing focused and this test changes.
#[test]
fn a_click_on_the_control_leaves_it_focused_as_well() {
    let mut s = customize_closed();
    let at = s.with(|p| {
        p.walk()
            .iter()
            .find(|w| w.text == "Advanced settings")
            .expect("the header text")
            .bounds
            .center()
    });
    let events = super::support::click_events(at);
    let (_statuses, messages) = s.with(|p| p.send(&events, iced::mouse::Cursor::Available(at)));
    assert_eq!(messages.len(), 1, "one message: {messages:?}");
    for m in messages {
        s.apply(m);
    }
    assert!(s.state.advanced_extras_open, "the click opened it");
    assert!(s.focus().0.is_some(), "and the control holds the focus");
}
