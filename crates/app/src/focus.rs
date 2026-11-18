//! Keyboard focus movement (RFC 027, job J14a).
//!
//! iced 0.14 moves no focus on Tab by itself, and snora's router answers only the shortcut routes
//! an application opts into. The one engine for Tab and Shift+Tab is `snora::keyboard::traverse`
//! (the same two passes as iced's `focus_next` and `focus_previous`, started from the control
//! last clicked when nothing has focus); this module only hands it the key. `shortcuts.rs` stays
//! the one owner of shortcuts: a Tab is not a shortcut, and nothing handles it twice (a focused
//! text field passes Tab on, so the subscription's filter, which sees only unused keys, gets it).

use iced::advanced::widget::Id;
use iced::advanced::widget::operation::{Focusable, Operation, Outcome, Scrollable};
use iced::keyboard::{Key, Modifiers, key::Named};
use iced::widget::operation::AbsoluteOffset;
use iced::{Point, Rectangle, Task, Vector};

use crate::app::Message;

/// What a Tab (or Shift+Tab) that nothing used does: moves the focus to the next (previous) Tab
/// stop, in layout order. At the last stop the focus leaves the group and the next Tab starts
/// again (no wrap: snora's rule, so a keyboard user is never trapped in a group). Ctrl, Alt and
/// Cmd with Tab are not Tab: `traverse` answers `None` and nothing happens.
pub(crate) fn on_tab(modifiers: Modifiers) -> Task<Message> {
    snora::keyboard::traverse(Key::Named(Named::Tab), modifiers)
        .map(|moved| moved.chain(keep_in_view()))
        .unwrap_or_else(Task::none)
}

/// Scrolls the control that holds the focus into view, if a scroll area holds it. iced's scrollable
/// does not do this by itself, so a Tab into a control below the fold would show the reader
/// nothing. Run after a focus move (`on_tab` chains it). Two passes of one operation, no timer and
/// no guess: the first finds the focused control's bounds, the second asks each scroll area that
/// holds it to scroll **by the least amount** that shows the control and a margin for its ring.
pub(crate) fn keep_in_view<T: Send + 'static>() -> Task<T> {
    iced::advanced::widget::operate::<()>(FindFocused { found: None }).discard()
}

/// Room kept around the control: the focus ring sits 2 px outside it and is 2 px thick.
const RING: f32 = 4.0;

/// Pass 1: where is the focused control?
struct FindFocused {
    found: Option<Rectangle>,
}

impl Operation<()> for FindFocused {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<()>)) {
        operate(self);
    }

    fn focusable(&mut self, _id: Option<&Id>, bounds: Rectangle, state: &mut dyn Focusable) {
        if state.is_focused() {
            self.found = Some(bounds);
        }
    }

    fn finish(&self) -> Outcome<()> {
        match self.found {
            Some(control) => Outcome::Chain(Box::new(ScrollInto { control })),
            None => Outcome::None,
        }
    }
}

/// Pass 2: every scroll area whose content holds the control moves just enough.
struct ScrollInto {
    control: Rectangle,
}

impl Operation<()> for ScrollInto {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<()>)) {
        operate(self);
    }

    fn scrollable(
        &mut self,
        _id: Option<&Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        translation: Vector,
        state: &mut dyn Scrollable,
    ) {
        if !content_bounds.contains(self.control.center()) {
            return;
        }
        // What the area shows now, in the layout's coordinates (the control's are not scrolled).
        let shown = Rectangle::new(
            Point::new(bounds.x + translation.x, bounds.y + translation.y),
            bounds.size(),
        );
        let by = |lo: f32, hi: f32, view_lo: f32, view_hi: f32| {
            // The margin never exceeds what fits around the control.
            let room = ((view_hi - view_lo) - (hi - lo)).max(0.0) / 2.0;
            let margin = RING.min(room);
            if lo - margin < view_lo {
                lo - margin - view_lo
            } else if hi + margin > view_hi {
                hi + margin - view_hi
            } else {
                0.0
            }
        };
        let c = self.control;
        let dx = by(c.x, c.x + c.width, shown.x, shown.x + shown.width);
        let dy = by(c.y, c.y + c.height, shown.y, shown.y + shown.height);
        if dx != 0.0 || dy != 0.0 {
            state.scroll_by(AbsoluteOffset { x: dx, y: dy }, bounds, content_bounds);
        }
    }
}
