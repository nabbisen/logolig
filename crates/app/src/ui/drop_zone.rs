//! Drop-zone screen (shown on launch and after editing is cancelled).
//!
//! Design principles:
//! - Show no settings — the user hasn't chosen a file yet
//! - Single large drop target fills the content area
//! - A file chooser button as the alternative to dragging. The button itself is operated with
//!   the mouse; the same file chooser opens with Ctrl+O (Cmd+O on macOS), see `crate::shortcuts`
//! - v1.26.1: the whole drop zone is clickable and native drag hover is
//!   highlighted when the platform reports it
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

use iced::widget::{column, container, row, space, text};
use iced::{Background, Border, Element, Length, Padding, Theme};

use snora::Icon;
use snora::lucide;
use snora::widget::icon_element_sized;

use logolig::MessageKey;

use crate::app::{AppState, Message, resolve_theme};
use crate::ui::colors;
use crate::ui::type_scale;

// v1.14.0: replaced hardcoded HEADLINE_COLOR constant with colors::drop_zone_headline
// — now tracks dark/light theme changes.

pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    let theme = resolve_theme(state);

    // v1.26.1: the entire drop-zone card is now the file-picker button.
    //
    // Previously only the small "Choose file…" button was clickable and the
    // surrounding card was a passive container. That made the startup page feel
    // broken when native OS drag/drop was unavailable (notably on Wayland, where
    // iced/winit currently does not deliver FileDropped). Making the full card a
    // large button keeps the visible "drag or click" promise accessible while
    // still accepting native FileDropped events on platforms that emit them.
    let choose_chip = container(text(t.t(MessageKey::ChooseFileButton)).size(type_scale::BODY))
        .padding([12, 22])
        .style(drop_zone_choose_chip_style);

    let inner = column![
        text(t.t(MessageKey::DropZoneHeadline))
            .size(type_scale::TITLE)
            .color(colors::drop_zone_headline(&theme)),
        text(t.t(MessageKey::DropZoneSecondary))
            .size(type_scale::SECONDARY)
            .color(card_text(&theme)),
        choose_chip,
        text(t.t(MessageKey::DropZoneAcceptedFormats))
            .size(type_scale::SECONDARY)
            .color(card_text(&theme)),
    ]
    .spacing(14)
    .align_x(iced::alignment::Horizontal::Center);

    // iced 0.14 has no dashed border; use a thin solid border and soft fill.
    // When the runtime reports a native file hover, use a stronger accent border
    // and tint. The same state is intentionally not required for operation: a
    // platform may support click-to-pick but not native file drop.
    let bordered = container(inner)
        .padding(48)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(move |theme: &Theme| drop_zone_card_style(theme, state.file_drag_hovering));

    let drop_area = snora::control(bordered)
        .padding(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .on_press(Message::PickFileRequested)
        .style(drop_zone_button_style);

    // v1.24.0: history card — shown only when a previous result is available.
    let history_card: Element<'a, Message> = if state.result_assets.is_some() {
        history_section(state, &theme)
    } else {
        space().into()
    };

    // Outer container provides margin between the drop area and the window edge.
    container(
        column![drop_area, history_card]
            .spacing(16)
            .padding(Padding::default().bottom(8)),
    )
    .padding(40)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Secondary text inside the drop-zone card: 7:1 on the card and on its hover tint.
fn card_text(theme: &Theme) -> iced::Color {
    colors::muted_on(
        theme,
        &[colors::surface_card(theme), colors::hover_tint(theme)],
    )
}

fn drop_zone_card_style(theme: &Theme, hovered: bool) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    // RFC 034 J21: the accent is the theme's primary (7:1 with the fill's own text); the hover tint
    // is a faint accent over the page that keeps body and secondary text at 7:1.
    let accent = palette.primary.base.color;
    let background = if hovered {
        colors::hover_tint(theme)
    } else {
        palette.background.weak.color
    };
    iced::widget::container::Style {
        background: Some(Background::Color(background)),
        border: Border {
            color: if hovered {
                accent
            } else {
                colors::control_border(theme)
            },
            width: if hovered { 2.5 } else { 2.0 },
            radius: 12.0.into(),
        },
        ..Default::default()
    }
}

fn drop_zone_choose_chip_style(theme: &Theme) -> iced::widget::container::Style {
    // The same accent as the primary button, at rest (RFC 034 J21).
    let fill = colors::primary_fill(theme, colors::PrimaryState::Rest);
    iced::widget::container::Style {
        background: Some(Background::Color(fill)),
        text_color: Some(colors::on_primary(theme, fill)),
        border: Border {
            color: fill,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

fn drop_zone_button_style(
    _theme: &Theme,
    _status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    // Keep the button itself visually invisible; the nested card owns all
    // visual affordance. This avoids double borders while preserving a single
    // large accessible click target.
    iced::widget::button::Style::default()
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

    let count_label = t.t_args(
        MessageKey::HistoryFileCount,
        &[("count", &asset_count.to_string())],
    );

    let section_label = text(t.t(MessageKey::HistoryLastConversionLabel))
        .size(type_scale::SECONDARY)
        .color(colors::muted_text(theme));

    let file_text = text(file_label)
        .size(type_scale::SECONDARY)
        .color(colors::file_name(theme));

    let count_text = text(count_label)
        .size(type_scale::SECONDARY)
        .color(colors::muted_text(theme));

    // "View results →" button with the History lucide icon.
    let view_btn = snora::control(
        row![
            icon_element_sized::<Message>(&Icon::Lucide(lucide::History), type_scale::ICON_SMALL),
            text(t.t(MessageKey::HistoryViewResultsButton)).size(type_scale::BODY),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
    .padding([6, 14])
    .on_press(Message::ShowLastResultRequested)
    .style(history_button_style);

    let info_row = row![
        file_text,
        text("•")
            .size(type_scale::SECONDARY)
            .color(colors::muted_text(theme)),
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
    // J21: the text is derived for the fill it sits on in this state.
    let fill = if matches!(status, iced::widget::button::Status::Hovered) {
        palette.background.strong.color
    } else {
        palette.background.base.color
    };
    iced::widget::button::Style {
        background: bg,
        text_color: colors::text_on(theme, fill),
        border: Border {
            color: colors::control_border(theme),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}
