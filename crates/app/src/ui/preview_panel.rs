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
//!
//! `view` is here; the picker buttons are in `preview_panel/pickers.rs`, the checkerboard in
//! `preview_panel/checker.rs`, the browser-tab and phone mock-ups in `preview_panel/mockups.rs`
//! (RFC 017).

mod checker;
mod mockups;
mod pickers;

use iced::widget::{column, container, text};
use iced::{Element, Length, Padding};

use logolig::{MessageKey, PreviewContext, ThemeMode};

use crate::app::{AppState, Message, resolve_theme};
use crate::ui::colors;
use checker::checker_view;
use mockups::render_context;
use pickers::{loading_placeholder, surface_picker, view_as_picker};

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
