//! プレビューパネル (§5.2 コンテキストプレビュー)。
//!
//! ここでの設計原則:
//! - **「画像を表示する」のではなく「使われる文脈の見え方を再現する」** こと
//! - 16×16 のラスタは **実寸ピクセルで表示** し、 iced 側で再スケールしない
//!   (`FilterMethod::Nearest` を使い、 サイズも `Length::Fixed(16.0)` で固定)
//! - 周辺文脈 (タブ枠、 ホーム画面背景) は **CSS でいうところの "framing"**
//!   として container + 色だけで描く。 SVG を別途用意しない
//! - 背景色は `PreviewProfile::background` で切り替え、 ラスタ自体には触れない
//!   (§5.2 「画像自体を破壊しない」)

use iced::widget::{button, column, container, image, row, text, Space};
use iced::{Background, Border, Color, Element, Length, Theme};

use logolig_core::{PreviewCache, PreviewContext, Rgba8, ThemeMode};

use crate::app::{AppState, Message};
use crate::ui::accessibility::{label, marker};

pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let asset_name = state
        .source_asset
        .as_ref()
        .map(|a| a.display_name())
        .unwrap_or_else(|| "(no source)".into());

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

    // メインプレビュー領域 (キャッシュが揃ってから初めて描ける)
    let preview_area: Element<'a, Message> = match state.preview_cache.as_ref() {
        Some(cache) => render_context(cache, context, bg),
        None => loading_placeholder(),
    };

    column![
        text(format!("Source: {asset_name}")).size(14),
        // コンテキスト選択 (キーボードでアクセス可能なボタン群、 §12)
        context_picker(context),
        background_picker(bg),
        // メインプレビュー枠
        container(preview_area)
            .padding(20)
            .center_x(Length::Fill),
        // 出力アクション
        row![
            button(text("Export")).padding([8, 14]).on_press(Message::ExportRequested),
            text(label::EXPORT_BTN).size(11),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(14)
    .into()
}

// ---------------------------------------------------------------------------
// コンテキスト・背景の切り替え UI (キーボード代替経路として button を使う、 §12)
// ---------------------------------------------------------------------------

fn context_picker<'a>(current: PreviewContext) -> Element<'a, Message> {
    let mk = |ctx: PreviewContext| -> Element<'a, Message> {
        let active = ctx == current;
        let lbl = if active {
            format!("{} {}", marker::READY, ctx.label())
        } else {
            ctx.label().to_string()
        };
        button(text(lbl))
            .padding([6, 12])
            .on_press(Message::PreviewContextSelected(ctx))
            .into()
    };

    row![
        text("Context:").size(13),
        mk(PreviewContext::BrowserTab16),
        mk(PreviewContext::SmartphoneIcon),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

fn background_picker<'a>(current: ThemeMode) -> Element<'a, Message> {
    let mk = |theme: ThemeMode| -> Element<'a, Message> {
        let active = theme == current;
        let lbl = if active {
            format!("{} {}", marker::READY, theme.label())
        } else {
            theme.label().to_string()
        };
        button(text(lbl))
            .padding([6, 12])
            .on_press(Message::PreviewBackgroundSelected(theme))
            .into()
    };

    row![
        text("Background:").size(13),
        mk(ThemeMode::System),
        mk(ThemeMode::Light),
        mk(ThemeMode::Dark),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

fn loading_placeholder<'a>() -> Element<'a, Message> {
    container(
        column![
            text(format!("{} Building preview…", marker::BUSY)).size(16),
            text("Resizing source for the chosen context (§5.2).").size(12),
        ]
        .spacing(6)
        .align_x(iced::alignment::Horizontal::Center),
    )
    .padding(40)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

// ---------------------------------------------------------------------------
// コンテキストごとの描画
// ---------------------------------------------------------------------------

fn render_context<'a>(
    cache: &'a PreviewCache,
    context: PreviewContext,
    bg: ThemeMode,
) -> Element<'a, Message> {
    match context {
        PreviewContext::BrowserTab16 => browser_tab_view(&cache.tab_16, bg),
        PreviewContext::SmartphoneIcon => smartphone_view(&cache.icon_120, bg),
    }
}

/// ブラウザタブの見え方を模写する。
///
/// 重要なのは 16×16 を **実寸** で表示すること (§6.2 縮小品質を視覚的に判断)。
/// `image::FilterMethod::Nearest` + `Length::Fixed(16.0)` で iced による
/// 自動スケーリングを禁じている。
fn browser_tab_view<'a>(rgba: &'a Rgba8, bg: ThemeMode) -> Element<'a, Message> {
    let bg_color = chrome_bg_for(bg);
    let tab_color = tab_face_for(bg);
    let text_color = text_color_for(bg);

    // 16×16 を実寸表示。 image::Handle::from_rgba は Bytes (= Vec<u8>) を要求するので
    // 1 度だけバイト列を複製する (cache 側はそのまま温存)。
    let icon_bytes: Vec<u8> = rgba.as_bytes().to_vec();
    let handle = image::Handle::from_rgba(rgba.width, rgba.height, icon_bytes);
    let icon = image(handle)
        .filter_method(image::FilterMethod::Nearest)
        .width(Length::Fixed(16.0))
        .height(Length::Fixed(16.0));

    // タブの中身: [favicon 16px] [Page Title text] [×]
    let tab_inner = row![
        icon,
        text("logolig.example.com")
            .size(13)
            .color(text_color),
        Space::new().width(Length::Fill),
        text("×").size(13).color(text_color),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .padding([6, 10]);

    // タブ "形" (上端を丸めたいが iced 0.14 の Radius は単一値型なので
    // 全角同じ半径で代用)。視覚的にはタブとして十分通じる。
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

    // ブラウザクロームの帯 (タブの下にぶら下がる空間)
    let chrome = container(
        column![
            // タブ列 (左寄せ)
            row![tab, Space::new().width(Length::Fill)].spacing(0),
            // アドレスバー風のライン
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

    column![
        text("Browser tab — 16×16 actual size").size(12),
        chrome,
    ]
    .spacing(8)
    .align_x(iced::alignment::Horizontal::Center)
    .into()
}

/// スマホホーム画面の見え方を模写する。
///
/// 60pt iOS アイコンの 2x DPI 想定で 120×120 を表示。 角丸マスクは
/// container の Border::radius でかけている。
fn smartphone_view<'a>(rgba: &'a Rgba8, bg: ThemeMode) -> Element<'a, Message> {
    let wallpaper = wallpaper_for(bg);
    let label_color = text_color_for(bg);

    let icon_bytes: Vec<u8> = rgba.as_bytes().to_vec();
    let handle = image::Handle::from_rgba(rgba.width, rgba.height, icon_bytes);

    // 角丸マスクを container の border_radius で再現。
    // iced は image を直接クリップしないため、 image を radius 付き container に
    // 入れることで角だけ削れて見える。
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

    // ホーム画面ぽい "壁紙" ボックス
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

    column![
        text("Smartphone home — 60pt @2x").size(12),
        home,
    ]
    .spacing(8)
    .align_x(iced::alignment::Horizontal::Center)
    .into()
}

// ---------------------------------------------------------------------------
// 色の選択
// ---------------------------------------------------------------------------
//
// `PreviewProfile::background` は **プレビュー文脈の背景色** を意味する。
// アプリ全体の Theme とは独立。 `System` は今のところ Light 相当として扱う
// (Step 4 で OS 設定読み込みを足す)。

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
