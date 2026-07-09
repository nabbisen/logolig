//! Preview panel (§5.2 — contextual preview).
//!
//! Design principles:
//! - **Show how it looks in context, not just "show the image"**
//! - 16 × 16 rasters are shown at **actual pixel size** with no iced
//!   rescaling (`FilterMethod::Nearest`, `Length::Fixed(16.0)`)
//! - Surrounding context (tab bar, home-screen background) is painted
//!   with containers and colours only — no separate SVG assets
//! - Background colour switches via `PreviewProfile::background`;
//!   the raster itself is not touched

use iced::widget::{Space, button, column, container, image, row, stack, text};
use iced::{Background, Border, Color, Element, Length, Padding, Theme};

use logolig::{MessageKey, PreviewCache, PreviewContext, Rgba8, ThemeMode};

use crate::app::{AppState, Message, resolve_theme};
use crate::ui::accessibility::marker;
use crate::ui::colors;

pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    let theme = resolve_theme(state);
    // v1.12.0: file name moved to the header (shell::header);
    // preview_panel no longer needs it.

    let context = state
        .preview
        .as_ref()
        .map(|p| p.context)
        .unwrap_or(PreviewContext::BrowserTab16);
    let bg = state
        .preview
        .as_ref()
        .map(|p| p.background)
        .unwrap_or(ThemeMode::System);

    // Main preview area (only drawable once the cache is ready).
    // v1.10.0: TransparencyChecker context: no framing at all —
    // render icon_120 at actual size over a checkerboard pattern.
    let preview_area: Element<'a, Message> = match state.preview_cache.as_ref() {
        Some(cache) => match context {
            PreviewContext::TransparencyChecker => checker_view(&cache.icon_120),
            _ => render_context(cache, context, bg),
        },
        None => loading_placeholder(state),
    };

    // ----- v1.12.0 edit-screen layout -----
    //
    // Design goals:
    // 1. Visualise "this is the preview" with a border + soft fill
    // 2. Show view-as / surface pickers above the preview frame
    //    to signal "these buttons control the frame"
    // 3. Export button at bottom-right, emphasised,
    //    clearly labelled as the file-creation action
    // 4. Preview size stability: container uses `FillPortion(4)` (4/7 of height),
    //    capped with max_width/max_height. All modes centre within the same frame.

    // 5. Layout: title centred; "Preview" label top-left of frame; pickers centred;
    //    Export right-aligned. Avoids a purely left-heavy layout.
    //
    // Hierarchy expressed via (a) font size, (b) visual frame boundaries,
    // and (c) placement (top/mid/bottom, left/centre/right) — not text alone.

    // 1. Screen title (centred, medium weight)
    let page_title = container(
        text(t.t(MessageKey::PageTitleEdit))
            .size(20)
            .color(colors::page_title(&theme)),
    )
    .center_x(Length::Fill)
    .padding(Padding::default().top(4).bottom(4));

    // 2. Preview card — border + soft fill to mark the preview area
    let preview_card = container(
        column![
            // "Preview" label (top-left inside frame, section-title weight)
            text(t.t(MessageKey::SectionTitlePreview))
                .size(13)
                .color(colors::section_label(&theme)),
            // Picker buttons (centred)
            container(view_as_picker(state, context)).center_x(Length::Fill),
            container(surface_picker(state, context, bg)).center_x(Length::Fill),
            // Preview frame (content centred, 4/7 of height, max 560)
            container(preview_area)
                .width(Length::Fill)
                .height(Length::FillPortion(4))
                .max_width(560.0)
                .max_height(560.0)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
        ]
        .spacing(10)
        .align_x(iced::alignment::Horizontal::Center),
    )
    .padding(16)
    .style(|theme: &iced::Theme| {
        let palette = theme.extended_palette();
        iced::widget::container::Style {
            background: Some(iced::Background::Color(palette.background.weak.color)),
            border: iced::Border {
                color: palette.background.strong.color,
                width: 1.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        }
    });

    // v1.19.0: old action_row (Back / Re-select / Export) removed.
    //
    // Context: v1.16.0 turned preview_panel into a collapsible section
    // inside result_view. Back / Re-select / Export were moved to result_view.

    // v1.19.0 removed all Export* Messages, so the Export button here was
    // removed too.

    column![page_title, preview_card]
        .spacing(14)
        .padding(Padding::default().left(8).right(8).top(4).bottom(8))
        .into()
}

// v1.14.0: PAGE_TITLE_COLOR / SECTION_LABEL_COLOR / MUTED_TEXT hardcoded
// constants moved to theme-aware helpers in `crate::ui::colors`.

// v1.19.0: `secondary_button_style` (for Back / Re-select buttons) removed.
// The action_row itself was removed; use
// `crate::ui::advanced_drawer::secondary_drawer_button_style` as an equivalent
// if needed.

// ---------------------------------------------------------------------------
// Context / background switcher UI (uses buttons as keyboard-accessible §12)
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
fn view_as_picker<'a>(state: &'a AppState, current: PreviewContext) -> Element<'a, Message> {
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
fn surface_picker<'a>(
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

fn loading_placeholder<'a>(state: &'a AppState) -> Element<'a, Message> {
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

// ---------------------------------------------------------------------------
// v1.7.0: Transparency checker view
// ---------------------------------------------------------------------------

/// Checkerboard size: 240×240 with 12-px tiles. Per ABDD §12,
/// visually distinguishes transparent areas without relying on colour alone.
const CHECKER_SIDE: u32 = 240;
const CHECKER_TILE: u32 = 12;

/// Generated once and cached for the lifetime of the process.
/// Repeated allocation per frame would be wasteful; use `OnceLock`.
fn checker_handle() -> &'static image::Handle {
    use std::sync::OnceLock;
    static HANDLE: OnceLock<image::Handle> = OnceLock::new();
    HANDLE.get_or_init(|| {
        // Two light-grey shades. The mid-grey makes transparent areas
        // visible whether the logo is light or dark.
        let light: [u8; 4] = [0xE6, 0xE6, 0xE6, 0xFF];
        let dark: [u8; 4] = [0xC0, 0xC0, 0xC0, 0xFF];
        let n = (CHECKER_SIDE as usize) * (CHECKER_SIDE as usize);
        let mut pixels = Vec::with_capacity(n * 4);
        for y in 0..CHECKER_SIDE {
            for x in 0..CHECKER_SIDE {
                let cx = x / CHECKER_TILE;
                let cy = y / CHECKER_TILE;
                let color = if (cx + cy) % 2 == 0 { light } else { dark };
                pixels.extend_from_slice(&color);
            }
        }
        image::Handle::from_rgba(CHECKER_SIDE, CHECKER_SIDE, pixels)
    })
}

/// Transparency-checker view. No framing (no tab or phone border) —
/// the icon is overlaid at actual size on the checkerboard.
///
/// Design decisions:
/// - **No framing**: dedicated mode for inspecting transparency.
///   A tab or phone frame would distract from what to look at.
/// - **120 px display**: icon_120 at actual size — a small icon makes
///   transparent areas less obvious.
/// - **240×240 checkerboard**: leaves a visible margin around the icon
fn checker_view<'a>(rgba: &'a Rgba8) -> Element<'a, Message> {
    let icon_bytes: Vec<u8> = rgba.as_bytes().to_vec();
    let icon_handle = image::Handle::from_rgba(rgba.width, rgba.height, icon_bytes);

    // Checkerboard background: 240×240 at actual pixel size
    let checker_layer = container(
        image(checker_handle().clone())
            .width(Length::Fixed(CHECKER_SIDE as f32))
            .height(Length::Fixed(CHECKER_SIDE as f32))
            .filter_method(image::FilterMethod::Nearest),
    )
    .center_x(Length::Fill)
    .center_y(Length::Fill);

    // Icon layer: cache.icon_120 at actual size, centred over the checkerboard
    let icon_layer = container(
        image(icon_handle)
            .width(Length::Fixed(rgba.width as f32))
            .height(Length::Fixed(rgba.height as f32))
            .filter_method(image::FilterMethod::Nearest),
    )
    .center_x(Length::Fill)
    .center_y(Length::Fill);

    // stack: later elements are drawn on top (iced 0.14).
    container(stack![checker_layer, icon_layer])
        .width(Length::Fixed(CHECKER_SIDE as f32))
        .height(Length::Fixed(CHECKER_SIDE as f32))
        .into()
}

// ---------------------------------------------------------------------------
// Per-context rendering
// ---------------------------------------------------------------------------

fn render_context<'a>(
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
