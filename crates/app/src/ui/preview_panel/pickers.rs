//! The "View as" and "Surface" groups of the preview panel, and the loading placeholder.

use iced::widget::{column, container, text};
use iced::{Element, Length};

use logolig::{MessageKey, PreviewContext, ThemeMode};

use crate::app::{AppState, Message, resolve_theme};
use crate::ui::accessibility::marker;
use crate::ui::choice::radio_group;
use crate::ui::colors;

// ---------------------------------------------------------------------------
// "View as" and "Surface" (RFC 021): two single-choice groups, each a `snora::radio_group`
// under its label: one Tab stop per group, ↑ ↓ ← → choose, the selected option marked by a
// filled dot (a shape, not colour alone). The Surface group is disabled during the Checker
// display (the background has no effect there): the whole group is dimmed and takes no key.
// ---------------------------------------------------------------------------

/// "View as" group: Tab / Phone / Checker.
pub(super) fn view_as_picker<'a>(
    state: &'a AppState,
    current: PreviewContext,
) -> Element<'a, Message> {
    let t = &state.translator;
    column![
        text(t.t(MessageKey::PickerLabelViewAs))
            .size(13)
            .color(colors::muted_text(&resolve_theme(state))),
        radio_group(
            PreviewContext::all(),
            &current,
            |context| t.t(context_message_key(*context)),
            Message::PreviewContextSelected,
        ),
    ]
    .spacing(4)
    .into()
}

/// "Surface" group: System / Light / Dark. Disabled during the Checker context.
pub(super) fn surface_picker<'a>(
    state: &'a AppState,
    context: PreviewContext,
    current: ThemeMode,
) -> Element<'a, Message> {
    let t = &state.translator;
    column![
        text(t.t(MessageKey::PickerLabelSurface))
            .size(13)
            .color(colors::muted_text(&resolve_theme(state))),
        radio_group(
            [ThemeMode::System, ThemeMode::Light, ThemeMode::Dark],
            &current,
            |mode| t.t(background_message_key(*mode)),
            Message::PreviewBackgroundSelected,
        )
        .disabled(!context.respects_surface()),
    ]
    .spacing(4)
    .into()
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
