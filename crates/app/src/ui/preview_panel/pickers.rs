//! The "View as" and "Surface" button groups of the preview panel, and the loading placeholder.

use iced::widget::{button, column, container, row, text};
use iced::{Background, Border, Element, Length, Theme};

use logolig::{MessageKey, PreviewContext, ThemeMode};

use crate::app::{AppState, Message, resolve_theme};
use crate::ui::accessibility::marker;
use crate::ui::colors;

// ---------------------------------------------------------------------------
// Context / background switcher UI (buttons operated with the mouse; there is no Tab focus
// traversal yet, RFC 027)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// v1.10.0: "View as" / "Surface" button groups
//
// Design:
// - 80px fixed label (`View as:` / `Surface:`) left of each group
//   signals the two groups are parallel structures
// - Active button has filled background; also prefixed with `▣` for
//   colour-blind safety (ABDD §12)
// - Surface group is disabled during Checker display (background has no effect)
// ---------------------------------------------------------------------------

/// "View as" group: Tab / Phone / Checker — three buttons.
pub(super) fn view_as_picker<'a>(
    state: &'a AppState,
    current: PreviewContext,
) -> Element<'a, Message> {
    let t = &state.translator;
    let mut buttons = row![].spacing(6).align_y(iced::Alignment::Center);
    for ctx in PreviewContext::all() {
        let active = ctx == current;
        let label = state.translator.t(context_message_key(ctx));
        buttons = buttons.push(picker_button(
            &label,
            active,
            Message::PreviewContextSelected(ctx),
        ));
    }

    row![
        text(t.t(MessageKey::PickerLabelViewAs))
            .size(13)
            .width(Length::Fixed(80.0))
            .color(colors::muted_text(&resolve_theme(state))),
        buttons,
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

/// "Surface" group: System / Light / Dark — three buttons.
/// Disabled during Checker context (background has no effect on rendering).
/// Disabled appearance: omit `on_press`.
pub(super) fn surface_picker<'a>(
    state: &'a AppState,
    context: PreviewContext,
    current: ThemeMode,
) -> Element<'a, Message> {
    let t = &state.translator;
    let respects_surface = context.respects_surface();
    let mut buttons = row![].spacing(6).align_y(iced::Alignment::Center);
    for theme in [ThemeMode::System, ThemeMode::Light, ThemeMode::Dark] {
        let active = theme == current;
        let label = state.translator.t(background_message_key(theme));
        let on_press = if respects_surface {
            Some(Message::PreviewBackgroundSelected(theme))
        } else {
            None
        };
        buttons = buttons.push(picker_button_optional(&label, active, on_press));
    }

    row![
        text(t.t(MessageKey::PickerLabelSurface))
            .size(13)
            .width(Length::Fixed(80.0))
            .color(colors::muted_text(&resolve_theme(state))),
        buttons,
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

/// A pressable picker button.
fn picker_button<'a>(label: &str, active: bool, on_press: Message) -> Element<'a, Message> {
    picker_button_optional(label, active, Some(on_press))
}

/// When `on_press = None`, returns a visually disabled picker button.
/// Active button uses `marker::READY` prefix + filled background
/// for two independent visual signals (colour-blind safe).
fn picker_button_optional<'a>(
    label: &str,
    active: bool,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let lbl = if active {
        format!("{} {}", marker::READY, label)
    } else {
        // Reserve space for the marker prefix even when inactive
        // to prevent row width from shifting between states.
        format!("  {}", label)
    };
    let mut btn = button(text(lbl).size(13)).padding([6, 12]);
    if let Some(msg) = on_press {
        btn = btn.on_press(msg);
    }
    // Active: fill with the theme primary colour.
    // iced 0.14: pass button::Style as a closure.
    btn = btn.style(move |theme: &Theme, status| {
        let palette = theme.extended_palette();
        let base = if active {
            palette.primary.base.color
        } else {
            palette.background.weak.color
        };
        let text_color = if active {
            palette.primary.base.text
        } else {
            palette.background.weak.text
        };
        // Hover: darken slightly (active = stronger, inactive = lighter).
        let bg = match status {
            iced::widget::button::Status::Hovered => {
                if active {
                    palette.primary.strong.color
                } else {
                    palette.background.strong.color
                }
            }
            _ => base,
        };
        iced::widget::button::Style {
            background: Some(Background::Color(bg)),
            text_color,
            border: Border {
                color: palette.background.strong.color,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }
    });
    btn.into()
}

pub(super) fn loading_placeholder<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    container(
        column![
            text(format!(
                "{} {}",
                marker::BUSY,
                t.t(MessageKey::ImportingMessage)
            ))
            .size(16),
        ]
        .spacing(6)
        .align_x(iced::alignment::Horizontal::Center),
    )
    .padding(40)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

fn context_message_key(ctx: PreviewContext) -> MessageKey {
    match ctx {
        PreviewContext::BrowserTab16 => MessageKey::PreviewBrowserTab,
        PreviewContext::SmartphoneIcon => MessageKey::PreviewSmartphoneHome,
        PreviewContext::TransparencyChecker => MessageKey::PreviewTransparencyChecker,
    }
}

fn background_message_key(theme: ThemeMode) -> MessageKey {
    match theme {
        ThemeMode::System => MessageKey::PreviewBackgroundSystem,
        ThemeMode::Light => MessageKey::PreviewBackgroundLight,
        ThemeMode::Dark => MessageKey::PreviewBackgroundDark,
    }
}
