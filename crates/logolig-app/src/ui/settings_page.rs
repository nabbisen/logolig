//! Settings page (v1.22.0).
//!
//! When the user selects "Settings" in the side nav, the main body switches
//! to this page. It replaces the v1.18.0 locale and theme popup overlays with
//! a full-page layout — more readable, one click to reach.

use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding, Theme};

use logolig_core::{MessageKey, ThemeMode};
use logolig_i18n::Locale;

use crate::app::{AppState, Message, resolve_theme};
use crate::ui::colors;

/// Root view. Called from `shell::body()`.
pub fn view(state: &AppState) -> Element<'_, Message> {
    let theme = resolve_theme(state);
    let t = &state.translator;

    let language_section = section(
        t.t(MessageKey::SidebarLabelLocale),
        column![
            choice_row(
                state.locale_override == Some(Locale::Ja),
                t.t(MessageKey::LocaleNameJa),
                Message::LocalePicked(Some(Locale::Ja)),
                &theme,
            ),
            choice_row(
                state.locale_override == Some(Locale::En),
                t.t(MessageKey::LocaleNameEn),
                Message::LocalePicked(Some(Locale::En)),
                &theme,
            ),
            choice_row(
                state.locale_override.is_none(),
                t.t(MessageKey::LocaleSystem),
                Message::LocalePicked(None),
                &theme,
            ),
        ]
        .spacing(4)
        .into(),
        &theme,
    );

    let theme_section = section(
        t.t(MessageKey::SidebarLabelTheme),
        column![
            choice_row(
                state.theme == ThemeMode::Light,
                t.t(MessageKey::ThemeNameLight),
                Message::ThemePicked(ThemeMode::Light),
                &theme,
            ),
            choice_row(
                state.theme == ThemeMode::Dark,
                t.t(MessageKey::ThemeNameDark),
                Message::ThemePicked(ThemeMode::Dark),
                &theme,
            ),
            choice_row(
                state.theme == ThemeMode::System,
                t.t(MessageKey::ThemeSystem),
                Message::ThemePicked(ThemeMode::System),
                &theme,
            ),
        ]
        .spacing(4)
        .into(),
        &theme,
    );

    container(
        column![language_section, theme_section]
            .spacing(32)
            .padding([24, 32])
            .max_width(480),
    )
    .width(Length::Fill)
    .into()
}

// ---------------------------------------------------------------------------
// Section
// ---------------------------------------------------------------------------

fn section<'a>(
    title: String,
    content: Element<'a, Message>,
    theme: &Theme,
) -> Element<'a, Message> {
    let heading = text(title).size(15).color(colors::page_title(theme));

    column![heading, content].spacing(12).into()
}

// ---------------------------------------------------------------------------
// Choice row (shared by language and theme)
// ---------------------------------------------------------------------------

fn choice_row<'a>(
    active: bool,
    label: String,
    on_press: Message,
    theme: &Theme,
) -> Element<'a, Message> {
    let text_color = if active {
        colors::page_title(theme)
    } else {
        theme.extended_palette().background.base.text
    };
    let check = text(if active { "✓" } else { "  " })
        .size(14)
        .color(text_color);
    let label_el = text(label).size(14).color(text_color);

    let inner = row![check, label_el].spacing(10).align_y(Alignment::Center);

    button(inner)
        .width(Length::Fill)
        .padding(Padding::default().top(10).bottom(10).left(16).right(16))
        .on_press(on_press)
        .style(move |t: &Theme, s| choice_style(t, s, active))
        .into()
}

fn choice_style(
    theme: &Theme,
    status: iced::widget::button::Status,
    active: bool,
) -> iced::widget::button::Style {
    let p = theme.extended_palette();
    let bg = if active {
        Some(Background::Color(p.background.strong.color))
    } else {
        match status {
            iced::widget::button::Status::Hovered => {
                Some(Background::Color(p.background.weak.color))
            }
            _ => None,
        }
    };
    iced::widget::button::Style {
        background: bg,
        text_color: p.background.base.text,
        border: Border {
            color: if active {
                p.background.strong.color
            } else {
                Color::TRANSPARENT
            },
            width: if active { 1.0 } else { 0.0 },
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}
