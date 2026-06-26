//! Drop-zone screen (shown on launch and after editing is cancelled).
//!
//! Design principles:
//! - Show no settings — the user hasn't chosen a file yet
//! - Single large drop target fills the content area
//! - Keyboard-accessible alternative path (file chooser button)
//!
//! ## v1.10.2: slimmed down
//!
//! Earlier versions had lengthy explanatory text and a format list.
//! v1.10.2 replaced it with a minimal prompt and a visual drop area.

use iced::widget::{button, column, container, text};
use iced::{Background, Border, Element, Length, Theme};

use logolig_core::MessageKey;

use crate::app::{AppState, Message, resolve_theme};
use crate::ui::colors;

// v1.14.0: replaced hardcoded HEADLINE_COLOR constant with colors::drop_zone_headline
// — now tracks dark/light theme changes.

pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    let theme = resolve_theme(state);

    // Drop area content: eye-catching prompt + Choose file… button only.
    let inner = column![
        text(t.t(MessageKey::DropZoneHeadline))
            .size(22)
            .color(colors::drop_zone_headline(&theme)),
        button(text(t.t(MessageKey::ChooseFileButton)).size(15))
            .padding([10, 22])
            .on_press(Message::PickFileRequested),
    ]
    .spacing(20)
    .align_x(iced::alignment::Horizontal::Center);

    // Drop area border (solid line + light background fill). iced 0.14 has no dashed border;
    // use a thin solid border and soft background to signal "drop here"
    // with generous corner radius to avoid an overly boxy look.
    // Generous padding floats the content in the centre.
    //
    // This container::style closure is called at render time with `&Theme`,
    // so it is already theme-aware.
    let bordered = container(inner)
        .padding(48)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(Background::Color(palette.background.weak.color)),
                border: Border {
                    color: palette.background.strong.color,
                    width: 2.0,
                    radius: 12.0.into(),
                },
                ..Default::default()
            }
        });

    // Outer container provides margin between the drop area and the window edge.

    container(bordered)
        .padding(40)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
