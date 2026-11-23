//! Shared widget styles of the application (RFC 034, job J21): the primary button and the text field.
//!
//! Every colour comes from `colors` (derived against the surface it is drawn on), so a button's text
//! reaches 7:1 in every state that is meant to be read, and the borders and marks of a control
//! reach 3:1.

use iced::widget::{button, text_input};
use iced::{Background, Border, Theme};

use super::colors::{self, PrimaryState};

/// The primary button ("Download all (ZIP)"): a filled accent, white text on Light, black on Dark.
/// Rest, hovered and pressed are all at the rule; a disabled button is a flat `strong` fill with
/// secondary text (exempt from the ratio, still distinguishable from an enabled one).
pub fn primary_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let (fill, text) = match status {
        button::Status::Disabled => (
            colors::surface_strong(theme),
            colors::muted_on(theme, &[colors::surface_strong(theme)]),
        ),
        other => {
            let state = match other {
                button::Status::Hovered => PrimaryState::Hovered,
                button::Status::Pressed => PrimaryState::Pressed,
                _ => PrimaryState::Rest,
            };
            let fill = colors::primary_fill(theme, state);
            (fill, colors::on_primary(theme, fill))
        }
    };
    button::Style {
        background: Some(Background::Color(fill)),
        text_color: text,
        border: Border {
            radius: 8.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}

/// The text field: iced's default, with the placeholder as secondary text (7:1) and the border at
/// 3:1 (the default border was about 1.5:1).
pub fn input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let mut style = text_input::default(theme, status);
    style.placeholder = colors::placeholder(theme);
    style.border.color = match status {
        text_input::Status::Focused { .. } => theme.extended_palette().primary.base.color,
        _ => colors::control_border(theme),
    };
    style.border.width = 1.0;
    style
}
