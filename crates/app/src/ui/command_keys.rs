//! A key chord is a command, never text (RFC 027, key ownership).
//!
//! iced's `text_input` inserts the first character of a key event's `text` whatever modifier is
//! held (only Ctrl+C, X, V and A are special), and then captures the event. On the real keyboard
//! stack Ctrl+1 carries the text `"1"`, so with a field focused it typed a `1` and the shortcut
//! (Ctrl+1: Home) never reached the application, whose subscription sees only events nothing
//! captured. Ctrl+2, Ctrl+3, Ctrl+O and Ctrl+S carry control characters, which the field already
//! refuses, so only Ctrl+1 was visible; the rule below does not depend on that accident.
//!
//! [`without_command_text`] wraps the **root of the view** (`shell::view`), so every text input,
//! present and future, is covered and nothing has to be remembered. With the command modifier held (Ctrl; Cmd on macOS) and
//! **without Alt**, the key event reaches the field **without its text**: the field has nothing to
//! insert and does not capture it, so the event goes on to the application like any other
//! unused key. The field's own commands (Ctrl+C, X, V, A, and the word and line movements) read the
//! key, not the text, and work as before. Ctrl together with Alt is left alone: on Windows that is
//! how AltGr is reported, and AltGr types characters.

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Operation, Tree, tree};
use iced::advanced::{Clipboard, Shell, Widget, mouse, overlay, renderer};
use iced::keyboard;
use iced::{Element, Event, Length, Rectangle, Size, Theme, Vector};

/// `content`, the whole view, whose widgets never see the text of a command chord.
pub fn without_command_text<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    Element::new(WithoutCommandText {
        content: content.into(),
    })
}

struct WithoutCommandText<'a, Message> {
    content: Element<'a, Message>,
}

/// The event as the widgets should see it: a command chord carries no text. `None` = unchanged.
fn stripped(event: &Event) -> Option<Event> {
    let Event::Keyboard(keyboard::Event::KeyPressed {
        key,
        modified_key,
        physical_key,
        location,
        modifiers,
        text: Some(_),
        repeat,
    }) = event
    else {
        return None;
    };
    if !modifiers.command() || modifiers.alt() {
        return None;
    }
    Some(Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: modified_key.clone(),
        physical_key: *physical_key,
        location: *location,
        modifiers: *modifiers,
        text: None,
        repeat: *repeat,
    }))
}

impl<Message> Widget<Message, Theme, iced::Renderer> for WithoutCommandText<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::stateless()
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let replaced = stripped(event);
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            replaced.as_ref().unwrap_or(event),
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}
