//! The browser-tab and phone mock-ups of the preview panel, and their colours.

use iced::widget::{Space, column, container, image, row, text};
use iced::{Background, Border, Color, Element, Length, Theme};

use logolig::{PreviewCache, PreviewContext, Rgba8, ThemeMode};

use crate::app::Message;

use super::checker::checker_view;

// ---------------------------------------------------------------------------
// Per-context rendering
// ---------------------------------------------------------------------------

pub(super) fn render_context<'a>(
    cache: &'a PreviewCache,
    context: PreviewContext,
    bg: ThemeMode,
) -> Element<'a, Message> {
    match context {
        PreviewContext::BrowserTab16 => browser_tab_view(&cache.tab_16, bg),
        PreviewContext::SmartphoneIcon => smartphone_view(&cache.icon_120, bg),
        // Defensive: the outer `view` function routes TransparencyChecker
        // to `checker_view`, so this branch is never reached.
        // Return checker view defensively rather than panicking.
        PreviewContext::TransparencyChecker => checker_view(&cache.icon_120),
    }
}

/// Simulate a browser tab.
///
/// Critical: display the 16×16 at **actual pixels** (§6.2 — visual quality check).
/// `image::FilterMethod::Nearest` + `Length::Fixed(16.0)` prevent iced
/// from auto-scaling.
fn browser_tab_view<'a>(rgba: &'a Rgba8, bg: ThemeMode) -> Element<'a, Message> {
    let bg_color = chrome_bg_for(bg);
    let tab_color = tab_face_for(bg);
    let text_color = text_color_for(bg);

    // Render 16×16 at actual size. image::Handle::from_rgba requires Bytes (Vec<u8>):
    // clone the byte slice once (the cache is not consumed).
    let icon_bytes: Vec<u8> = rgba.as_bytes().to_vec();
    let handle = image::Handle::from_rgba(rgba.width, rgba.height, icon_bytes);
    let icon = image(handle)
        .filter_method(image::FilterMethod::Nearest)
        .width(Length::Fixed(16.0))
        .height(Length::Fixed(16.0));

    // Tab content: [favicon 16 px] [Page Title] [×]
    let tab_inner = row![
        icon,
        text("logolig.example.com").size(13).color(text_color),
        Space::new().width(Length::Fill),
        text("×").size(13).color(text_color),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .padding([6, 10]);

    // Tab "shape" (rounded top — iced 0.14 Radius is uniform,
    // so use the same radius on all corners; visually reads as a tab).
    let tab = container(tab_inner)
        .style(move |_t: &Theme| container::Style {
            background: Some(Background::Color(tab_color)),
            border: Border {
                radius: 8.0.into(),
                ..Border::default()
            },
            ..container::Style::default()
        })
        .width(Length::Fixed(280.0));

    // Browser chrome strip (space below the tab)
    let chrome = container(
        column![
            // Tab row (left-aligned)
            row![tab, Space::new().width(Length::Fill)].spacing(0),
            // Address-bar-like line
            container(
                text("https://logolig.example.com/")
                    .size(11)
                    .color(text_color),
            )
            .padding([4, 12])
            .width(Length::Fill),
        ]
        .spacing(0),
    )
    .style(move |_t: &Theme| container::Style {
        background: Some(Background::Color(bg_color)),
        border: Border {
            radius: 6.0.into(),
            ..Border::default()
        },
        ..container::Style::default()
    })
    .width(Length::Fixed(560.0))
    .padding([8, 8]);

    column![text("Browser tab — 16×16 actual size").size(12), chrome,]
        .spacing(8)
        .align_x(iced::alignment::Horizontal::Center)
        .into()
}

/// Simulate a phone home screen.
///
/// Displays 120×120 (60 pt at 2× DPI). Rounded-corner mask via
/// container Border::radius.
fn smartphone_view<'a>(rgba: &'a Rgba8, bg: ThemeMode) -> Element<'a, Message> {
    let wallpaper = wallpaper_for(bg);
    let label_color = text_color_for(bg);

    let icon_bytes: Vec<u8> = rgba.as_bytes().to_vec();
    let handle = image::Handle::from_rgba(rgba.width, rgba.height, icon_bytes);

    // Rounded corner: container border_radius acts as a clip mask.
    // iced does not clip images directly; wrapping in a container with radius
    // gives the rounded-corner appearance.
    let icon_card = container(
        image(handle)
            .filter_method(image::FilterMethod::Linear)
            .width(Length::Fixed(60.0))
            .height(Length::Fixed(60.0)),
    )
    .style(|_t: &Theme| container::Style {
        border: Border {
            radius: 14.0.into(),
            ..Border::default()
        },
        ..container::Style::default()
    })
    .padding(0);

    let app_label = text("Logolig").size(12).color(label_color);

    let icon_with_label = column![icon_card, app_label]
        .spacing(6)
        .align_x(iced::alignment::Horizontal::Center);

    // Simulated home-screen "wallpaper" box
    let home = container(icon_with_label)
        .style(move |_t: &Theme| container::Style {
            background: Some(Background::Color(wallpaper)),
            border: Border {
                radius: 24.0.into(),
                ..Border::default()
            },
            ..container::Style::default()
        })
        .width(Length::Fixed(220.0))
        .height(Length::Fixed(380.0))
        .center_x(Length::Fill)
        .center_y(Length::Fill);

    column![text("Smartphone home — 60pt @2x").size(12), home,]
        .spacing(8)
        .align_x(iced::alignment::Horizontal::Center)
        .into()
}

// ---------------------------------------------------------------------------
// Background colour selection
// ---------------------------------------------------------------------------
//
// `PreviewProfile::background` is the **preview context background**,
// independent of the app Theme. `System` is currently treated as Light
// (OS preference reading is planned for a later step).

fn chrome_bg_for(theme: ThemeMode) -> Color {
    match theme {
        ThemeMode::Dark => Color::from_rgb8(0x2b, 0x2b, 0x2e),
        ThemeMode::Light | ThemeMode::System => Color::from_rgb8(0xee, 0xee, 0xf2),
    }
}

fn tab_face_for(theme: ThemeMode) -> Color {
    match theme {
        ThemeMode::Dark => Color::from_rgb8(0x3c, 0x3c, 0x42),
        ThemeMode::Light | ThemeMode::System => Color::from_rgb8(0xff, 0xff, 0xff),
    }
}

fn text_color_for(theme: ThemeMode) -> Color {
    match theme {
        ThemeMode::Dark => Color::from_rgb8(0xea, 0xea, 0xea),
        ThemeMode::Light | ThemeMode::System => Color::from_rgb8(0x33, 0x33, 0x36),
    }
}

fn wallpaper_for(theme: ThemeMode) -> Color {
    match theme {
        ThemeMode::Dark => Color::from_rgb8(0x10, 0x14, 0x1c),
        ThemeMode::Light | ThemeMode::System => Color::from_rgb8(0x90, 0xa8, 0xc8),
    }
}
