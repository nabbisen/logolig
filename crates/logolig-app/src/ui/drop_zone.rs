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
//!
//! ## v1.24.0: history card
//!
//! When `result_assets` is `Some` (user pressed Back from the Result screen),
//! a small card below the drop zone lets them return to the last result
//! without re-converting.

use iced::widget::{button, column, container, row, space, text};
use iced::{Background, Border, Element, Length, Padding, Theme};

use snora::Icon;
use snora::lucide;
use snora::widget::icon_element_sized;

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

    // v1.24.0: history card — shown only when a previous result is available.
    let history_card: Element<'a, Message> = if state.result_assets.is_some() {
        history_section(state, &theme)
    } else {
        space().into()
    };

    // Outer container provides margin between the drop area and the window edge.
    container(
        column![bordered, history_card]
            .spacing(16)
            .padding(Padding::default().bottom(8)),
    )
    .padding(40)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// History card: shows the last-conversion summary and a "View results →" button.
fn history_section<'a>(state: &'a AppState, theme: &Theme) -> Element<'a, Message> {
    let t = &state.translator;

    // Source file display name (from the kept source_asset, or a fallback).
    let file_label = state
        .source_asset
        .as_ref()
        .map(|a| a.display_name().clone())
        .unwrap_or_default();

    // Asset count from result_assets.
    let asset_count = state.result_assets.as_ref().map(|r| r.count()).unwrap_or(0);

    let count_label = format!("{} files", asset_count);

    let section_label = text(t.t(MessageKey::HistoryLastConversionLabel))
        .size(12)
        .color(colors::muted_text(theme));

    let file_text = text(file_label).size(13).color(colors::file_name(theme));

    let count_text = text(count_label).size(12).color(colors::muted_text(theme));

    // "View results →" button with the History lucide icon.
    let view_btn = button(
        row![
            icon_element_sized::<Message>(&Icon::Lucide(lucide::History), 14.0),
            text(t.t(MessageKey::HistoryViewResultsButton)).size(13),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
    .padding([6, 14])
    .on_press(Message::ShowLastResultRequested)
    .style(history_button_style);

    let info_row = row![
        file_text,
        text("•").size(12).color(colors::muted_text(theme)),
        count_text,
        space(),
        view_btn,
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    container(column![section_label, info_row].spacing(6))
        .padding([10, 16])
        .width(Length::Fill)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(Background::Color(palette.background.weak.color)),
                border: Border {
                    color: palette.background.strong.color,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

fn history_button_style(
    theme: &Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let palette = theme.extended_palette();
    let bg = match status {
        iced::widget::button::Status::Hovered => {
            Some(Background::Color(palette.background.strong.color))
        }
        _ => Some(Background::Color(palette.background.base.color)),
    };
    iced::widget::button::Style {
        background: bg,
        text_color: palette.background.base.text,
        border: Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}
