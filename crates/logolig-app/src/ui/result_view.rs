//! Result screen (v1.16.0; replaces ExportReady).
//!
//! ## Layout
//!
//! ```text
//! ┌──────────────────────────────────────────────────────┐
//! │ ✓  Conversion complete!                              │
//! │ Generated assets                                     │
//! ├──────────────────────────────────────────────────────┤
//! │ │favicon.ico│ │icon-16.png│ │icon-32.png│  …         │
//! │ │ [thumb]   │ │ [thumb]   │ │ [thumb]   │            │
//! │ │ ICO 46 K  │ │ 16×16 .6K │ │ 32×32 1 K │            │
//! │ │    [↓]    │ │    [↓]    │ │    [↓]    │            │
//! ├──────────────────────────────────────────────────────┤
//! │ ▶ Preview                                            │
//! │                                                      │
//! │             [↓ Download all]                         │
//! │ [← Back]                                             │
//! └──────────────────────────────────────────────────────┘
//! ```

use iced::widget::{button, column, container, image, row, text, Space};
use iced::{Background, Border, Color, Element, Length, Padding, Theme};

use logolig_core::{MessageKey, Rgba8};

use crate::app::{resolve_theme, AppState, Message};
use crate::result::{ResultAssetItem, ResultAssetKind, ResultAssets};
use crate::ui::colors;

/// Entry point for the Result screen.
pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    let theme = resolve_theme(state);

    // If result_assets is None (unexpected state or old flow)
    // fall back to the preview_panel so nothing breaks.
    // In normal operation, IngestCompleted → Converting → ConvertCompleted
    // always produces Some, so this branch is never reached.
    let Some(assets) = state.result_assets.as_ref() else {
        return crate::ui::preview_panel::view(state);
    };

    // 1. Headline: ✓ Conversion complete!
    let headline = container(
        row![
            // Success marker. Per ABDD §12, convey success via the ✓ character
            // (not colour alone). Large size makes it the focal element.
            text("✓").size(28).color(colors::file_name(&theme)),
            text(t.t(MessageKey::ResultSuccessHeadline))
                .size(22)
                .color(colors::page_title(&theme)),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
    )
    .center_x(Length::Fill)
    .padding(Padding::default().top(8).bottom(4));

    // 2. Sub-heading: generated assets list
    let subheading = text(t.t(MessageKey::ResultAssetsSubheading))
        .size(13)
        .color(colors::muted_text(&theme));

    // 3. Asset card grid (3 columns)
    // v1.20.0: 2 columns on mobile, 3 on desktop.
    // 1 column is too tall for 7–12 items; 2 columns fits 4–6 per screen,
    // an acceptable scroll amount on mobile.
    let columns = if crate::app::is_mobile(state) { 2 } else { 3 };
    let grid = build_grid(assets, columns, &theme);

    // 4. ZIP "Download all" button
    let download_all = container(
        button(
            row![
                text("↓").size(16),
                text(t.t(MessageKey::ResultDownloadAllButton)).size(15),
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center),
        )
        .padding([12, 28])
        .on_press(Message::DownloadAllRequested),
    )
    .center_x(Length::Fill)
    .padding(Padding::default().top(16).bottom(8));

    // 5. Preview collapsible section — optional, user-expanded
    // Shows as a toggle button; when expanded, embeds the preview_panel
    // with View-as / Surface picker.
    let preview_toggle_label = format!(
        "{} {}",
        if state.result_preview_open { "▼" } else { "▶" },
        t.t(MessageKey::ResultPreviewToggle)
    );
    let preview_toggle = button(text(preview_toggle_label).size(13))
        .padding([6, 12])
        .on_press(Message::ResultPreviewToggled)
        .style(toggle_button_style);

    // 6. Preview panel (expanded state only)
    let preview_panel: Element<'a, Message> = if state.result_preview_open {
        // Embeds the existing preview_panel directly. Headings / action rows
        // come from this screen, so using preview_panel::view() as a collapsible
        // can double-render titles. Acceptable in phase B;
        // refactor to "preview only" in a later version.
        // 
        // 
        crate::ui::preview_panel::view(state)
    } else {
        Space::new().height(Length::Shrink).into()
    };

    // 7. Action row: Back button only (re-selecting drops a new file,
    //    so explicit Re-select was removed in v1.16).
    let action_row = row![
        button(text(t.t(MessageKey::EditCancelButton)).size(13))
            .padding([8, 16])
            .on_press(Message::EditCancelled)
            .style(secondary_button_style),
        Space::new().width(Length::Fill),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // Full layout. Wrapped in a scrollable because the asset list can be long.
    // 
    // v1.22.0: "Download all" moved below the preview (RFC feedback —
    // users review the preview before downloading).
    let content = column![
        headline,
        container(subheading).center_x(Length::Fill),
        grid,
        container(preview_toggle).center_x(Length::Fill),
        preview_panel,
        download_all,
        action_row,
    ]
    .spacing(12)
    .padding(Padding::default().left(8).right(8).top(4).bottom(8));

    iced::widget::scrollable(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Build an asset card grid with the given number of columns.
///
/// v1.20.0: column count is a parameter (3 desktop / 2 mobile).
fn build_grid<'a>(
    assets: &'a ResultAssets,
    columns: usize,
    theme: &Theme,
) -> Element<'a, Message> {
    let mut col = column![].spacing(12);
    let mut cur_row = row![].spacing(12);
    let mut count_in_row = 0usize;
    for (idx, item) in assets.items.iter().enumerate() {
        cur_row = cur_row.push(asset_card(idx, item, theme));
        count_in_row += 1;
        if count_in_row == columns {
            col = col.push(cur_row);
            cur_row = row![].spacing(12);
            count_in_row = 0;
        }
    }
    // Flush any partial final row (fill trailing cells with Space)
    if count_in_row > 0 {
        for _ in count_in_row..columns {
            cur_row = cur_row.push(Space::new().width(Length::FillPortion(1)));
        }
        col = col.push(cur_row);
    }
    col.into()
}

/// One asset card.
fn asset_card<'a>(
    idx: usize,
    item: &'a ResultAssetItem,
    theme: &Theme,
) -> Element<'a, Message> {
    // Source file name (abbreviated)
    let file_name_text = text(item.file_name.clone())
        .size(13)
        .color(colors::file_name(theme));

    // Thumbnail area: decoded raster for images, placeholder icon for text
    let thumb: Element<'a, Message> = match &item.thumbnail {
        Some(rgba) => raster_thumbnail(rgba),
        None => placeholder_thumbnail(item.kind, theme),
    };

    // Meta row: badge (ICO / PNG / SVG / HTML / JSON) + dimensions (images) + size
    let badge = text(item.kind.badge_label())
        .size(11)
        .color(colors::section_label(theme));
    let size_label = text(item.size_display())
        .size(11)
        .color(colors::muted_text(theme));
    let dim_text: Element<'a, Message> = if let Some(d) = item.dimensions_display() {
        text(d).size(11).color(colors::muted_text(theme)).into()
    } else {
        Space::new().width(Length::Shrink).into()
    };
    let meta_row = row![badge, dim_text, Space::new().width(Length::Fill), size_label]
        .spacing(8)
        .align_y(iced::Alignment::Center);

    // Download button (↓ icon only; a11y label via tooltip)
    let dl_button = button(text("↓ ").size(14))
        .padding([4, 12])
        .on_press(Message::DownloadOneRequested(idx))
        .style(download_button_style);

    let inner = column![file_name_text, thumb, meta_row, container(dl_button).center_x(Length::Fill)]
        .spacing(8);

    container(inner)
        .padding(12)
        .width(Length::FillPortion(1))
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

/// Image thumbnail. `Length::Fill` scales it to fill the available frame.
fn raster_thumbnail<'a>(rgba: &'a Rgba8) -> Element<'a, Message> {
    // image::Handle::from_rgba takes (width, height, Vec<u8>)
    let handle = image::Handle::from_rgba(rgba.width, rgba.height, rgba.pixels.to_vec());
    container(
        image(handle)
            .width(Length::Fill)
            .height(Length::Fixed(80.0))
            .content_fit(iced::ContentFit::Contain),
    )
    .center_x(Length::Fill)
    .center_y(Length::Fixed(80.0))
    .into()
}

/// Placeholder thumbnail for text-based assets (HTML snippet / Web manifest / SVG).
/// Shows a large document icon.
fn placeholder_thumbnail<'a>(
    kind: ResultAssetKind,
    theme: &Theme,
) -> Element<'a, Message> {
    let glyph = match kind {
        ResultAssetKind::HtmlSnippet => "{}",
        ResultAssetKind::WebManifest => "{ }",
        ResultAssetKind::Svg => "<>",
        _ => "•",
    };
    container(text(glyph).size(28).color(colors::muted_text(theme)))
        .center_x(Length::Fill)
        .center_y(Length::Fixed(80.0))
        .into()
}

/// Collapsible toggle button style (transparent + subtle hover fill).
fn toggle_button_style(
    theme: &Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let palette = theme.extended_palette();
    let bg = match status {
        iced::widget::button::Status::Hovered => palette.background.weak.color,
        _ => Color::TRANSPARENT,
    };
    iced::widget::button::Style {
        background: Some(Background::Color(bg)),
        text_color: palette.background.weak.text,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

/// Individual download button style (auxiliary, not over-emphasised).
fn download_button_style(
    theme: &Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let palette = theme.extended_palette();
    let bg = match status {
        iced::widget::button::Status::Hovered => palette.background.weak.color,
        _ => Color::TRANSPARENT,
    };
    iced::widget::button::Style {
        background: Some(Background::Color(bg)),
        text_color: palette.background.weak.text,
        border: Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

/// "← Back" auxiliary button style (matches preview_panel secondary).
fn secondary_button_style(
    theme: &Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let palette = theme.extended_palette();
    let bg = match status {
        iced::widget::button::Status::Hovered => palette.background.weak.color,
        _ => Color::TRANSPARENT,
    };
    iced::widget::button::Style {
        background: Some(Background::Color(bg)),
        text_color: palette.background.weak.text,
        border: Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}
