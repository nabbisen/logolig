//! Keyboard focus movement (RFC 027, job J14a).
//!
//! iced 0.14 moves no focus on Tab by itself, and snora's router answers only the shortcut routes
//! an application opts into. The one engine for Tab and Shift+Tab is `snora::keyboard::traverse`
//! (the same two passes as iced's `focus_next` and `focus_previous`, started from the control
//! last clicked when nothing has focus); this module only hands it the key. `shortcuts.rs` stays
//! the one owner of shortcuts: a Tab is not a shortcut, and nothing handles it twice (a focused
//! text field passes Tab on, so the subscription's filter, which sees only unused keys, gets it).

use iced::Task;
use iced::keyboard::{Key, Modifiers, key::Named};

use crate::app::Message;

/// What a Tab (or Shift+Tab) that nothing used does: moves the focus to the next (previous) Tab
/// stop, in layout order. At the last stop the focus leaves the group and the next Tab starts
/// again (no wrap: snora's rule, so a keyboard user is never trapped in a group). Ctrl, Alt and
/// Cmd with Tab are not Tab: `traverse` answers `None` and nothing happens.
pub(crate) fn on_tab(modifiers: Modifiers) -> Task<Message> {
    snora::keyboard::traverse(Key::Named(Named::Tab), modifiers).unwrap_or_else(Task::none)
}
