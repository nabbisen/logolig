//! Side navigation bar (v1.22.0).
//!
//! Three nav items: Home / Customize / Settings. Clicking one fires
//! `Message::NavPageSelected(page)`, which swaps the entire main body.
//!
//! Replaces the v1.18.0 icon trio (settings / language / theme) that opened
//! drawers and popup overlays.

use iced::widget::{button, column, container, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding, Theme};

use snora::Icon;
use snora::lucide;
use snora::widget::icon_element_sized;

use logolig::MessageKey;

use crate::app::{AppState, Message, NavPage, resolve_theme};
use crate::ui::colors;

/// Desktop left sidebar. Passed to `AppLayout::side_bar(...)`.
pub fn view(state: &AppState) -> Element<'_, Message> {
    let theme = resolve_theme(state);
    let t = &state.translator;

    let nav_home = nav_item(
        &Icon::Lucide(lucide::Home),
        t.t(MessageKey::NavHome),
        state.nav_page == NavPage::Home,
        Message::NavPageSelected(NavPage::Home),
        &theme,
    );
    let nav_customize = nav_item(
        &Icon::Lucide(lucide::SlidersHorizontal),
        t.t(MessageKey::NavCustomize),
        state.nav_page == NavPage::Customize,
        Message::NavPageSelected(NavPage::Customize),
        &theme,
    );
    let nav_settings = nav_item(
        &Icon::Lucide(lucide::Settings),
        t.t(MessageKey::NavSettings),
        state.nav_page == NavPage::Settings,
        Message::NavPageSelected(NavPage::Settings),
        &theme,
    );

    container(
        column![nav_home, nav_customize, nav_settings]
            .spacing(4)
            .padding(Padding::default().top(12).bottom(12).left(6).right(6)),
    )
    .width(Length::Fixed(90.0))
    .height(Length::Fill)
    .style(move |t: &Theme| sidebar_style(t))
    .into()
}

/// Mobile bottom navigation bar. Passed to `AppLayout::footer(...)`.
pub fn bottom_view(state: &AppState) -> Element<'_, Message> {
    use iced::widget::row;

    let theme = resolve_theme(state);
    let t = &state.translator;

    let nav_home = bottom_nav_item(
        &Icon::Lucide(lucide::Home),
        t.t(MessageKey::NavHome),
        state.nav_page == NavPage::Home,
        Message::NavPageSelected(NavPage::Home),
        &theme,
    );
    let nav_customize = bottom_nav_item(
        &Icon::Lucide(lucide::SlidersHorizontal),
        t.t(MessageKey::NavCustomize),
        state.nav_page == NavPage::Customize,
        Message::NavPageSelected(NavPage::Customize),
        &theme,
    );
    let nav_settings = bottom_nav_item(
        &Icon::Lucide(lucide::Settings),
        t.t(MessageKey::NavSettings),
        state.nav_page == NavPage::Settings,
        Message::NavPageSelected(NavPage::Settings),
        &theme,
    );

    container(row![nav_home, nav_customize, nav_settings].spacing(0))
        .width(Length::Fill)
        .height(Length::Fixed(64.0))
        .style(move |t: &Theme| sidebar_style(t))
        .into()
}

// ---------------------------------------------------------------------------
// Desktop nav item (vertical stack)
// ---------------------------------------------------------------------------

fn nav_item<'a>(
    icon: &'a Icon,
    label: String,
    active: bool,
    on_press: Message,
    theme: &Theme,
) -> Element<'a, Message> {
    let icon_color = if active {
        colors::page_title(theme)
    } else {
        colors::muted_text(theme)
    };

    let icon_el = icon_element_sized::<Message>(icon, 22.0);
    let label_el = text(label).size(11).color(icon_color);

    // Wrap in a container to force horizontal centering within the button.
    // `.align_x` on the column alone is not sufficient when the button has
    // `width(Length::Fill)`.
    let inner = container(
        column![icon_el, label_el]
            .align_x(Alignment::Center)
            .spacing(4),
    )
    .width(Length::Fill)
    .align_x(Alignment::Center);

    button(inner)
        .width(Length::Fill)
        .padding(Padding::default().top(10).bottom(10).left(4).right(4))
        .on_press(on_press)
        .style(move |t: &Theme, s| nav_item_style(t, s, active))
        .into()
}

// ---------------------------------------------------------------------------
// Mobile nav item (horizontal row)
// ---------------------------------------------------------------------------

fn bottom_nav_item<'a>(
    icon: &'a Icon,
    label: String,
    active: bool,
    on_press: Message,
    theme: &Theme,
) -> Element<'a, Message> {
    let label_color = if active {
        colors::page_title(theme)
    } else {
        colors::muted_text(theme)
    };

    let icon_el = icon_element_sized::<Message>(icon, 22.0);
    let label_el = text(label).size(10).color(label_color);

    let inner = container(
        column![icon_el, label_el]
            .align_x(Alignment::Center)
            .spacing(3),
    )
    .width(Length::Fill)
    .align_x(Alignment::Center);

    button(inner)
        .width(Length::Fill)
        .padding(Padding::default().top(8).bottom(8))
        .on_press(on_press)
        .style(move |t: &Theme, s| nav_item_style(t, s, active))
        .into()
}

// ---------------------------------------------------------------------------
// Styles
// ---------------------------------------------------------------------------

fn sidebar_style(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(p.background.weak.color)),
        border: Border {
            color: p.background.strong.color,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

fn nav_item_style(
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
                Some(Background::Color(p.background.strong.color))
            }
            _ => None,
        }
    };
    iced::widget::button::Style {
        background: bg,
        text_color: p.background.base.text,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}
