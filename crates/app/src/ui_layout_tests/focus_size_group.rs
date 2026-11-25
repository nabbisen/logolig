//! The focus on the Interface-size group when the layout switches (RFC 034, job J24b).
//!
//! Choosing a step with an arrow key changes the scale, and at Extra large a desktop window
//! becomes a phone-layout window (the same window, 1042 px at Normal, 694.7 scaled px at Extra
//! large). The navigation then moves from first to last in the Tab order, and snora's rule for a
//! tree that changes shape is "the focus is wherever the application put it, or nowhere". The
//! keyboard user who is choosing the size must still be on the group: the ring stays and the next
//! arrow key chooses the next step.

use iced::Size;
use iced::keyboard::{Modifiers, key::Named};
use logolig::InterfaceSize;

use super::focus_after::Session;
use super::support::{Case, click_events};
use crate::app::is_mobile;

/// The real window of the J24 runs: 1042 x 1144 at Normal.
const WINDOW: Size = Size::new(1042.0, 1144.0);

/// Is the focused stop the Interface-size group? (The stop that holds the "Extra large" option.)
fn on_the_group(s: &mut Session) -> bool {
    s.with(|p| {
        let Some(stop) = p.focus_targets().into_iter().find(|t| t.focused) else {
            return false;
        };
        p.walk()
            .iter()
            .any(|w| w.text == "Extra large" && stop.bounds.contains(w.bounds.center()))
    })
}

/// Tabs to the group (a bounded walk: the Settings page has few stops).
fn tab_to_the_group(s: &mut Session) {
    for _ in 0..30 {
        s.tab();
        if on_the_group(s) {
            return;
        }
    }
    panic!("no Tab stop holds the Interface-size group");
}

fn settings() -> Session {
    let mut s = Session::new(Case::Settings, WINDOW);
    s.state.interface_size = InterfaceSize::Normal;
    s
}

/// After a press: the group holds the focus, the ring is drawn (the focused stop is the one the
/// ring tests draw), the control is on the screen, and the stored step is `step`.
fn on_the_group_at(s: &mut Session, step: InterfaceSize, what: &str) {
    assert_eq!(s.state.interface_size, step, "{what}: the step");
    assert!(
        on_the_group(s),
        "{what}: the focus is on the group, not {:?} (stop {:?})",
        s.focused_text(),
        s.focus()
    );
    let view = s.with(|p| p.focused_view()).expect("a focused control");
    assert!(view.fully_visible(), "{what}: in view: {view:?}");
}

/// The whole gesture across both layout switches: ↓ ↓ then ↑ ↑, the focus on the group after every
/// press, and the next press works after a switch.
#[test]
fn the_group_keeps_the_focus_across_the_layout_switch_in_both_directions() {
    let mut s = settings();
    assert!(!is_mobile(&s.state), "the desktop layout at the start");
    tab_to_the_group(&mut s);
    on_the_group_at(&mut s, InterfaceSize::Normal, "tabbed in");

    s.press(Named::ArrowDown, Modifiers::empty());
    on_the_group_at(&mut s, InterfaceSize::Large, "down: Large");
    assert!(!is_mobile(&s.state), "Large stays on the desktop layout");

    s.press(Named::ArrowDown, Modifiers::empty());
    assert!(is_mobile(&s.state), "Extra large is the phone layout");
    on_the_group_at(&mut s, InterfaceSize::ExtraLarge, "down: Extra large");

    // Back across the switch, and the next press after the first switch worked.
    s.press(Named::ArrowUp, Modifiers::empty());
    assert!(!is_mobile(&s.state), "Large is the desktop layout again");
    on_the_group_at(&mut s, InterfaceSize::Large, "up: Large");

    s.press(Named::ArrowUp, Modifiers::empty());
    on_the_group_at(&mut s, InterfaceSize::Normal, "up: Normal");
}

/// The next press after a switch is not lost: ↓ ↓ crosses to the phone layout, then ↑ ↑ crosses
/// back, and both ↑ presses count (before J24b the second one did nothing).
#[test]
fn the_next_arrow_key_works_after_the_switch() {
    let mut s = settings();
    tab_to_the_group(&mut s);
    s.press(Named::ArrowDown, Modifiers::empty());
    s.press(Named::ArrowDown, Modifiers::empty());
    assert_eq!(s.state.interface_size, InterfaceSize::ExtraLarge);
    // Tab order after the switch: still on the group, so Shift+Tab goes to the stop before it.
    let (at, stops) = s.focus();
    assert!(
        at.is_some() && stops > 0,
        "a stop is focused: {at:?}/{stops}"
    );
    s.press(Named::ArrowUp, Modifiers::empty());
    s.press(Named::ArrowUp, Modifiers::empty());
    assert_eq!(
        s.state.interface_size,
        InterfaceSize::Normal,
        "both presses counted"
    );
}

/// A step change inside one layout kept the focus before J24b and still does.
#[test]
fn a_step_change_inside_one_layout_keeps_the_focus() {
    let mut s = settings();
    tab_to_the_group(&mut s);
    s.press(Named::ArrowDown, Modifiers::empty());
    on_the_group_at(&mut s, InterfaceSize::Large, "Normal -> Large");
    s.press(Named::ArrowUp, Modifiers::empty());
    on_the_group_at(&mut s, InterfaceSize::Normal, "Large -> Normal");
}

/// Tab goes on from the group in the order of the layout the window is in now: the page's next
/// stop (the shortcuts reset button, or the page's end) and Shift+Tab comes back to the group.
#[test]
fn tab_goes_on_from_the_group_in_the_new_layout() {
    let mut s = settings();
    tab_to_the_group(&mut s);
    s.press(Named::ArrowDown, Modifiers::empty());
    s.press(Named::ArrowDown, Modifiers::empty());
    let (at, _) = s.focus();
    let at = at.expect("the group is focused");
    s.tab();
    assert_ne!(s.focus().0, Some(at), "Tab leaves the group");
    s.press_shift_tab();
    assert_eq!(s.focus().0, Some(at), "Shift+Tab comes back to it");
    assert!(on_the_group(&mut s));
}

/// What the pointer does, stated: a click on an option publishes the same message as an arrow key,
/// so the group is focused afterwards (a ring after a click). Reported, and chosen as the smaller
/// evil: the message cannot tell the two apart (the same finding as for the disclosure controls).
#[test]
fn a_click_on_an_option_leaves_the_group_focused_as_well() {
    let mut s = settings();
    assert_eq!(s.focus().0, None, "nothing is focused at the start");
    let at = s.with(|p| {
        p.walk()
            .iter()
            .find(|w| w.text == "Large")
            .expect("the Large option")
            .bounds
            .center()
    });
    let events = click_events(at);
    let (_statuses, messages) = s.with(|p| p.send(&events, iced::mouse::Cursor::Available(at)));
    assert_eq!(messages.len(), 1, "one message: {messages:?}");
    for m in messages {
        s.apply(m);
    }
    assert_eq!(
        s.state.interface_size,
        InterfaceSize::Large,
        "the click chose it"
    );
    println!(
        "after a click: focus = {:?}, on the group = {}",
        s.focus(),
        on_the_group(&mut s)
    );
}
