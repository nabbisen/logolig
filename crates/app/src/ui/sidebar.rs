//! Side navigation bar (v1.22.0).
//!
//! Three nav items: Home / Customize / Settings. Clicking one fires
//! `Message::NavPageSelected(page)`, which swaps the entire main body.
//!
//! Replaces the v1.18.0 icon trio (settings / language / theme) that opened
//! drawers and popup overlays.

use iced::font::Weight;
use iced::widget::{column, container, row, text};
use iced::{Alignment, Background, Border, Color, Element, Font, Length, Padding, Theme};

use snora::Icon;
use snora::lucide;
use snora::widget::icon_element_sized;

use logolig::MessageKey;

use crate::app::{AppState, Message, NavPage, resolve_theme};
use crate::ui::colors;
use crate::ui::type_scale;

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
            .padding(Padding::default().top(12).bottom(12).left(6).right(0)),
    )
    .width(Length::Fixed(SIDEBAR_WIDTH))
    .height(Length::Fill)
    .style(move |t: &Theme| sidebar_style(t))
    .into()
}

/// Mobile bottom navigation bar. Passed to `AppLayout::footer(...)`.
pub fn bottom_view(state: &AppState) -> Element<'_, Message> {
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

    // RING_GAP at both ends and below: the ring of the first and the last item (and the bottom of
    // every item, which fills the bar) is 4 px outside it and would be cut by the window edge.
    container(row![nav_home, nav_customize, nav_settings].spacing(0))
        .padding(
            Padding::default()
                .left(RING_GAP)
                .right(RING_GAP)
                .bottom(RING_GAP),
        )
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
    let icon_color = colors::nav_label(theme, active);

    let icon_el = icon_element_sized::<Message>(icon, type_scale::ICON_LARGE);
    let label_el = text(label)
        .size(type_scale::SECONDARY)
        .color(icon_color)
        .font(label_font(active));

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

    let item = snora::control(inner)
        .width(Length::Fill)
        .padding(Padding::default().top(10).bottom(10).left(4).right(4))
        .on_press(on_press)
        .style(move |t: &Theme, s| nav_item_style(t, s, active));

    // The bar on the edge facing the content (the page is to the right of the sidebar). It is
    // always laid out, and only coloured when the item is active, so nothing moves.
    // RING_GAP between them: the focus ring (2 px stroke, 2 px outside the item) must fit between
    // the item and the bar, or it covers the bar.
    row![item, bar(active, Length::Fixed(BAR), Length::Fill)]
        .spacing(RING_GAP)
        .height(Length::Shrink)
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
    let label_color = colors::nav_label(theme, active);

    let icon_el = icon_element_sized::<Message>(icon, type_scale::ICON_LARGE);
    let label_el = text(label)
        .size(type_scale::SECONDARY)
        .color(label_color)
        .font(label_font(active));

    let inner = container(
        column![icon_el, label_el]
            .align_x(Alignment::Center)
            .spacing(3),
    )
    .width(Length::Fill)
    .align_x(Alignment::Center);

    let item = snora::control(inner)
        .width(Length::Fill)
        .padding(Padding::default().top(6).bottom(8))
        .on_press(on_press)
        .style(move |t: &Theme, s| nav_item_style(t, s, active));

    // The bar on the edge facing the content (the page is above the bottom bar).
    column![bar(active, Length::Fill, Length::Fixed(BAR)), item]
        .spacing(RING_GAP)
        .into()
}

// ---------------------------------------------------------------------------
// The active-page cue (RFC 021 Q1, D7): a filled item, a bold label and a thin accent bar on
// the edge facing the content. Three cues, so the state is never colour alone.
// ---------------------------------------------------------------------------

/// The desktop sidebar's width (RFC 034 J22): 112 px, up from 90, so that the Japanese labels
/// ("カスタマイズ") are one line at 14 px. The bound set by the architect.
pub(crate) const SIDEBAR_WIDTH: f32 = 112.0;

/// Thickness of the accent bar, in pixels.
const BAR: f32 = 3.0;

/// Room between an item and the bar: the ring's offset (2 px) plus its stroke (2 px).
const RING_GAP: f32 = 4.0;

/// The label's font: bold when the page is the active one.
pub(crate) fn label_font(active: bool) -> Font {
    Font {
        weight: if active { Weight::Bold } else { Weight::Normal },
        ..Font::DEFAULT
    }
}

/// The accent bar of a nav item: the accent colour when active, nothing otherwise, the same
/// size either way.
fn bar<'a>(active: bool, width: Length, height: Length) -> Element<'a, Message> {
    container(iced::widget::Space::new())
        .width(width)
        .height(height)
        .style(move |theme: &Theme| container::Style {
            background: active.then(|| Background::Color(colors::nav_accent(theme))),
            ..Default::default()
        })
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
    // The text of an item on the `strong` fill (active or hovered) is derived for that fill (J21); on
    // the bare sidebar it is the body colour.
    let text_color = if bg.is_some() {
        colors::text_on(theme, p.background.strong.color)
    } else {
        p.background.base.text
    };
    iced::widget::button::Style {
        background: bg,
        text_color,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}
