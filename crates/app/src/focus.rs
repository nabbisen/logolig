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
/// no guess: the first finds the focused control and **which scroll areas it is inside** (by the
/// tree: an area's callback comes right before the traversal of its children), the second asks
/// each of those to scroll **by the least amount** that shows the control and a margin for its
/// ring. Coordinates alone cannot say whether a control is inside an area: the layout of an
/// unscrolled area overlaps whatever lies below it (a bottom bar, a footer).
pub(crate) fn keep_in_view<T: Send + 'static>() -> Task<T> {
    iced::advanced::widget::operate::<()>(FindFocused::default()).discard()
}

/// A control that opens and closes a section below it (RFC 027, job J14d): the "Advanced settings"
/// header and the Result screen's preview toggle.
///
/// Pressing one **adds or removes Tab stops**, and snora's rule for that case is "nothing is
/// focused" (`Inert`, "After a removal": it is written for controls that go away, and does not
/// describe one that adds stops after itself). A keyboard user who opened a section would land
/// at the top of the page. So the `update` arm that opens or closes the section asks for the focus
/// back on the control, by its fixed `Id`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Disclosure {
    AdvancedHeader,
    PreviewToggle,
}

impl Disclosure {
    /// The control's `widget::Id`. Fixed, because every `view` builds the control again and an id
    /// made at build time would never be the same twice.
    fn id(self) -> Id {
        Id::new(match self {
            Self::AdvancedHeader => "disclosure-advanced-header",
            Self::PreviewToggle => "disclosure-preview-toggle",
        })
    }
}

/// `control`, given the fixed id that `keep_focus` looks for.
pub(crate) fn disclosure<'a>(
    control: snora::Control<'a, Message>,
    which: Disclosure,
) -> snora::Control<'a, Message> {
    control.id(which.id())
}

/// The task a disclosure control's press returns: focus the control again.
///
/// It is deterministic and needs no timer. After a message, the runtime builds the new interface
/// and then runs the task's widget operation (`iced_winit`, `run_instance`: `update`, then
/// `build_user_interfaces`, then `run_action`), and `Inert::operate` settles its pending press (it
/// takes the focus off, because the number of stops changed) **before** the operation looks at
/// anything: the operation focuses the control after that, and nothing takes it away. Both Enter
/// and Space and both opening and closing go through the same message, so one task serves them.
///
/// A click goes through the same message, and so it also leaves the control focused (a ring
/// after a click, which snora's own controls avoid by default): the message cannot tell the two
/// apart, and snora's public `Control` does not report focus changes.
pub(crate) fn keep_focus(which: Disclosure) -> Task<Message> {
    iced::widget::operation::focus(which.id())
}

/// The Interface-size group (RFC 034, job J24b).
///
/// Choosing a step with an arrow key can switch the layout (at Extra large a desktop window is a
/// phone-layout window): the navigation moves from first to last in the Tab order, and snora's
/// rule for a tree that changes shape is "nothing is focused". The keyboard user who is choosing
/// the size would lose the ring in the middle of the gesture, and the next arrow key would do
/// nothing. So `update` asks for the focus on the group again, by its fixed `Id`, in the same
/// step: the pattern, and the reason it needs no timer, are those of [`keep_focus`].
///
/// A click on an option publishes the same message, so it also leaves the group focused (see
/// `focus_size_group.rs`).
fn size_group_id() -> Id {
    Id::new("interface-size-group")
}

/// `group`, given the fixed id that `keep_size_group_focus` looks for.
pub(crate) fn size_group<'a, T>(
    group: snora::RadioGroup<'a, T, Message>,
) -> snora::RadioGroup<'a, T, Message>
where
    T: Clone + PartialEq + std::fmt::Display + 'a,
{
    group.id(size_group_id())
}

/// The task a change of the Interface size returns besides the minimum-size request: focus the
/// group again.
pub(crate) fn keep_size_group_focus() -> Task<Message> {
    iced::widget::operation::focus(size_group_id())
}

/// The id that marks a container as "keep this together with its control in view" (release 1.29.0,
/// job R2b): a form field with the line under it that explains an invalid value. The line is a part of
/// the field for the user (it is what the field says), so when the focused control sits inside such
/// a container, `keep_in_view` shows the **whole container**: a user who types a bad value must see
/// the explanation without scrolling. Controls outside such containers behave as before.
pub(crate) fn with_its_line() -> Id {
    Id::new("keep-the-line-with-its-control")
}

/// Room kept around the control: the focus ring sits 2 px outside it and is 2 px thick.
const RING: f32 = 4.0;

/// Pass 1: where is the focused control, and which scroll areas (by their number in traversal
/// order) hold it?
#[derive(Default)]
struct FindFocused {
    /// The number of scroll areas seen so far.
    seen: usize,
    /// The scroll area whose children are about to be traversed.
    entering: Option<usize>,
    /// The areas we are inside, outermost first.
    inside: Vec<usize>,
    /// The container marked by [`with_its_line`] whose children are about to be traversed.
    entering_group: Option<Rectangle>,
    /// The marked containers we are inside, outermost first.
    groups: Vec<Rectangle>,
    found: Option<(Rectangle, Vec<usize>)>,
}

impl Operation<()> for FindFocused {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<()>)) {
        let entered = self.entering.take();
        if let Some(area) = entered {
            self.inside.push(area);
        }
        let group = self.entering_group.take();
        if let Some(bounds) = group {
            self.groups.push(bounds);
        }
        operate(self);
        if group.is_some() {
            self.groups.pop();
        }
        if entered.is_some() {
            self.inside.pop();
        }
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        if id == Some(&with_its_line()) {
            self.entering_group = Some(bounds);
        }
    }

    fn scrollable(
        &mut self,
        _id: Option<&Id>,
        _bounds: Rectangle,
        _content_bounds: Rectangle,
        _translation: Vector,
        _state: &mut dyn Scrollable,
    ) {
        self.entering = Some(self.seen);
        self.seen += 1;
    }

    fn focusable(&mut self, _id: Option<&Id>, bounds: Rectangle, state: &mut dyn Focusable) {
        if state.is_focused() {
            // A field inside a marked container is shown together with the rest of it.
            let shown = self.groups.last().copied().unwrap_or(bounds);
            self.found = Some((shown, self.inside.clone()));
        }
    }

    fn finish(&self) -> Outcome<()> {
        match &self.found {
            Some((control, areas)) if !areas.is_empty() => Outcome::Chain(Box::new(ScrollInto {
                control: *control,
                areas: areas.clone(),
                seen: 0,
            })),
            _ => Outcome::None,
        }
    }
}

/// Pass 2: each scroll area that holds the control moves just enough.
struct ScrollInto {
    control: Rectangle,
    areas: Vec<usize>,
    seen: usize,
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
        let number = self.seen;
        self.seen += 1;
        if !self.areas.contains(&number) {
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
