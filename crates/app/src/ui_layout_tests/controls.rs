//! The check every migrated control passes (RFC 027, job J14b).
//!
//! A control is migrated when the keyboard can reach it and does what the pointer does. For one
//! control, found by the text it shows (the `nth` such text on the page), this proves:
//!
//! * it is a **Tab stop** (a focus target holds the text's centre) and Tab can reach it;
//! * **Enter**, **Space** and a **click** each publish **exactly one** message, **the same** one,
//!   equal to `expected` (the message the pointer published before the migration).
//!
//! Each way starts from a fresh interface (focus is state of the tree), through the real `view`.

use iced::Size;
use iced::keyboard::{Key, key::Named};
use iced::mouse::Cursor;

use super::support::{Probe, click_events, key_press, probe_of};
use crate::app::{AppState, Message};

fn debug(messages: &[Message]) -> Vec<String> {
    messages.iter().map(|m| format!("{m:?}")).collect()
}

fn centre_of(p: &mut Probe<'_, Message>, label: &str, nth: usize) -> iced::Point {
    p.walk()
        .iter()
        .filter(|s| s.text == label)
        .nth(nth)
        .unwrap_or_else(|| panic!("no text {label:?} number {nth} on the page"))
        .bounds
        .center()
}

/// Where a control is: one of the navigation items, or on the page.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Region {
    Navigation,
    Page,
}

/// The Tab-order index of the focus target of `region` that holds `at`, if there is one. The layout
/// of an unscrolled page overlaps the bottom bar's coordinates, so the region must be said.
pub(super) fn stop_at(
    p: &mut Probe<'_, Message>,
    at: iced::Point,
    region: Region,
) -> Option<usize> {
    let stops = p.focus_targets();
    stops.iter().enumerate().position(|(i, t)| {
        t.bounds.contains(at)
            && p.is_navigation_stop(i, stops.len()) == (region == Region::Navigation)
    })
}

/// The control showing the `nth` text `label` is a Tab stop, and Enter, Space and a click each
/// publish exactly `expected` (one message, its `Debug` text).
pub(super) fn press_three_ways(
    state: &AppState,
    size: Size,
    (region, label, nth): (Region, &str, usize),
    expected: &str,
) {
    let who = format!("{label:?} #{nth} at {}x{}", size.width, size.height);
    let expected = vec![expected.to_string()];

    let mut p = probe_of(state, size);
    let at = centre_of(&mut p, label, nth);
    let stop = stop_at(&mut p, at, region);
    assert!(stop.is_some(), "{who}: not a Tab stop");

    let mut p = probe_of(state, size);
    let (_s, messages) = p.send(&click_events(at), Cursor::Available(at));
    assert_eq!(debug(&messages), expected, "{who}: a click");

    for key in [Named::Enter, Named::Space] {
        let mut p = probe_of(state, size);
        p.focus_target(stop.expect("a stop"));
        let (_s, messages) = p.send(&[key_press(Key::Named(key))], Cursor::Unavailable);
        assert_eq!(debug(&messages), expected, "{who}: {key:?}");
    }
}
