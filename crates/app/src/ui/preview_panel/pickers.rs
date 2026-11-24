//! The "View as" and "Surface" groups of the preview panel, and the loading placeholder.

use iced::widget::{column, container, text};
use iced::{Element, Length};

use logolig::{MessageKey, PreviewContext, ThemeMode};

use crate::app::{AppState, Message, resolve_theme};
use crate::ui::choice::radio_group;
use crate::ui::colors;
use crate::ui::cues;
use crate::ui::type_scale;

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
            .size(type_scale::BODY)
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
            .size(type_scale::BODY)
            .color(colors::muted_text(&resolve_theme(state))),
        radio_group(
            [ThemeMode::System, ThemeMode::Light, ThemeMode::Dark],
            &current,
            // The plain "System": the mock-up beside the picker shows what it resolves to (RFC 013 review);
            // the honest long label is the Settings page's.
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
        column![cues::busy(
            t.t(MessageKey::ImportingMessage),
            type_scale::BODY
        )]
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
