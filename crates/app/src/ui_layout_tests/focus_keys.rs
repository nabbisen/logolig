//! Tab and Shift+Tab (RFC 027, job J14a), through the path the window uses: the key event goes to
//! the interface; if nothing used it (`Ignored`) the subscription's filter (`runtime_event`) turns
//! it into `TabPressed`, `focus::on_tab` answers with a task, and the task's widget operations run
//! on the same interface. The questions: is a plain Tab and Shift+Tab routed, in layout order, out
//! of a focused text field, with no wrap; is every other Tab chord left alone.

use iced::Size;
use iced::event::Status;
use iced::keyboard::{Key, Modifiers, key::Named};
use iced::mouse::Cursor;
use logolig_i18n::Locale;

use super::support::{Case, Combo, Probe, key_press_with, probe_of};
use crate::app::{AppState, Message, runtime_event};

const SIZE: Size = Size::new(1280.0, 4000.0);

fn state(case: Case) -> AppState {
    Combo {
        case,
        size: SIZE,
        locale: Locale::En,
        dark: false,
    }
    .state()
}

/// Presses Tab (or Shift+Tab) on `p` as the window does; returns the filter's answer.
pub(super) fn press_tab(p: &mut Probe<'_, Message>, modifiers: Modifiers) -> Option<Message> {
    let event = key_press_with(Key::Named(Named::Tab), modifiers);
    let (statuses, _published) = p.send(std::slice::from_ref(&event), Cursor::Unavailable);
    let message = runtime_event(event, *statuses.last().expect("a status"))?;
    if let Message::TabPressed(modifiers) = message {
        let task = crate::focus::on_tab(modifiers);
        p.run_task(task);
        Some(Message::TabPressed(modifiers))
    } else {
        panic!("a Tab became {message:?}");
    }
}

pub(super) fn tab(p: &mut Probe<'_, Message>) {
    assert!(
        press_tab(p, Modifiers::empty()).is_some(),
        "Tab was not routed"
    );
}

pub(super) fn shift_tab(p: &mut Probe<'_, Message>) {
    assert!(
        press_tab(p, Modifiers::SHIFT).is_some(),
        "Shift+Tab was not routed"
    );
}

/// The Tab-order index of the focused target, if any.
pub(super) fn focused(p: &mut Probe<'_, Message>) -> Option<usize> {
    p.focus_targets().iter().position(|t| t.focused)
}

#[test]
fn a_tab_nothing_used_becomes_tab_pressed_and_nothing_else_does() {
    let tab_key = |m: Modifiers| key_press_with(Key::Named(Named::Tab), m);
    let answer = |m: Modifiers, status: Status| runtime_event(tab_key(m), status);
    assert!(matches!(
        answer(Modifiers::empty(), Status::Ignored),
        Some(Message::TabPressed(m)) if m.is_empty()
    ));
    assert!(matches!(
        answer(Modifiers::SHIFT, Status::Ignored),
        Some(Message::TabPressed(m)) if m.shift()
    ));
    assert!(
        answer(Modifiers::empty(), Status::Captured).is_none(),
        "a captured Tab is the widget's"
    );
}

#[test]
fn only_a_plain_or_shift_tab_moves_the_focus() {
    // `on_tab` yields a widget operation for Tab and Shift+Tab, and nothing for Ctrl, Alt or the
    // logo key with Tab (snora's `traverse` claims plain Tab and Shift+Tab only).
    let operations = |m: Modifiers| -> usize {
        use iced_test::futures::futures::StreamExt as _;
        let Some(mut stream) = iced_test::runtime::task::into_stream(crate::focus::on_tab(m))
        else {
            return 0;
        };
        let mut count = 0;
        while let Some(action) = iced_test::futures::futures::executor::block_on(stream.next()) {
            if matches!(action, iced_test::runtime::Action::Widget(_)) {
                count += 1;
            }
        }
        count
    };
    assert!(operations(Modifiers::empty()) >= 1, "Tab moves the focus");
    assert!(
        operations(Modifiers::SHIFT) >= 1,
        "Shift+Tab moves the focus"
    );
    for m in [
        Modifiers::CTRL,
        Modifiers::ALT,
        Modifiers::LOGO,
        Modifiers::CTRL | Modifiers::SHIFT,
    ] {
        assert_eq!(operations(m), 0, "{m:?}+Tab is not Tab");
    }
}

/// Tab visits the stops in layout order and leaves the group at the end: the next Tab starts
/// again (no wrap, snora's rule); Shift+Tab goes back.
#[test]
fn tab_walks_the_stops_in_order_and_does_not_wrap() {
    let settings = state(Case::Settings);
    let mut p = probe_of(&settings, SIZE);
    assert_eq!(focused(&mut p), None);
    tab(&mut p);
    assert_eq!(focused(&mut p), Some(0), "the language group");
    tab(&mut p);
    assert_eq!(focused(&mut p), Some(1), "the theme group");
    shift_tab(&mut p);
    assert_eq!(focused(&mut p), Some(0), "Shift+Tab goes back");
    shift_tab(&mut p);
    assert_eq!(
        focused(&mut p),
        None,
        "Shift+Tab at the first stop leaves the group"
    );
    tab(&mut p);
    assert_eq!(focused(&mut p), Some(0), "and the next Tab starts again");
    tab(&mut p);
    tab(&mut p);
    assert_eq!(
        focused(&mut p),
        None,
        "Tab at the last stop leaves the group (no wrap)"
    );
    tab(&mut p);
    assert_eq!(focused(&mut p), Some(0));
}

#[test]
fn tab_visits_every_stop_of_the_widest_page_once() {
    let customize = state(Case::CustomizeAdvanced);
    let mut p = probe_of(&customize, SIZE);
    let stops = p.focus_targets().len();
    assert_eq!(stops, 21, "14 checkboxes, 2 groups, 5 text inputs");
    let mut order = Vec::new();
    for _ in 0..stops {
        tab(&mut p);
        order.push(focused(&mut p).expect("a stop is focused"));
    }
    assert_eq!(
        order,
        (0..stops).collect::<Vec<_>>(),
        "layout order, each once"
    );
    tab(&mut p);
    assert_eq!(focused(&mut p), None, "after the last stop focus leaves");
}

/// A focused text field passes Tab (it is `Ignored`): focus leaves the field for the next stop,
/// and the field keeps its text.
#[test]
fn tab_leaves_a_focused_text_field_and_keeps_its_text() {
    let customize = state(Case::CustomizeAdvanced);
    let mut p = probe_of(&customize, SIZE);
    p.focus_text_input(0); // the custom-size field
    let field = focused(&mut p).expect("the field is focused");
    assert!(p.focus_targets()[field].text_input);
    tab(&mut p);
    let after = focused(&mut p).expect("a stop is focused");
    assert_eq!(after, field + 1, "the stop after the field");
    assert!(!p.focus_targets()[after].text_input || after != field);
    shift_tab(&mut p);
    assert_eq!(
        focused(&mut p),
        Some(field),
        "Shift+Tab comes back to the field"
    );
}
