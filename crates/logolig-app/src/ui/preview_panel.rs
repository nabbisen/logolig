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

use iced::widget::{button, column, container, image, row, stack, text, Space};
use iced::{Background, Border, Color, Element, Length, Theme};

use logolig_core::{MessageKey, PreviewCache, PreviewContext, Rgba8, ThemeMode};

use crate::app::{AppState, Message};
use crate::ui::accessibility::{label, marker};

pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    let asset_name = state
        .source_asset
        .as_ref()
        .map(|a| a.display_name())
        .unwrap_or_else(|| t.t(MessageKey::PreviewNoSource));

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
        Some(cache) => {
            // v1.7.0: 透過チェッカー toggle が on のときは framing を一切付けず、
            // 市松模様の上に icon_120 を実寸で重ねる専用ビューを使う。
            if state.preview_checker {
                checker_view(&cache.icon_120)
            } else {
                render_context(cache, context, bg)
            }
        }
        None => loading_placeholder(state),
    };

    let source_label = format!("{}: {}", t.t(MessageKey::PreviewSourceLabel), asset_name);
    column![
        text(source_label).size(14),
        // コンテキスト選択 (キーボードでアクセス可能なボタン群、 §12)
        context_picker(state, context),
        background_picker(state, bg),
        // v1.7.0: 透過チェッカー toggle。 背景選択と独立した別軸の設定。
        transparency_checker_toggle(state),
        // メインプレビュー枠
        container(preview_area).padding(20).center_x(Length::Fill),
        // 出力アクション
        row![
            button(text(t.t(MessageKey::ExportButton)))
                .padding([8, 14])
                .on_press(Message::ExportRequested),
            text(label::EXPORT_BTN).size(11),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(14)
    .into()
}

/// v1.7.0: 透過チェッカー toggle (checkbox)。
///
/// 背景選択 (System/Light/Dark) と独立した toggle。 ON のとき、
/// プレビュー背景の上に市松模様 (グレー/白の格子) が重なって、 透明部分を
/// 視覚的に確認できる。 §12「色だけに依存しない」 ABDD 原則と整合: 透明度を
/// 「ない / ある」 の二項で見せる。
fn transparency_checker_toggle<'a>(state: &'a AppState) -> Element<'a, Message> {
    use iced::widget::checkbox;
    let t = &state.translator;
    checkbox(state.preview_checker)
        .label(t.t(MessageKey::PreviewCheckerLabel))
        .on_toggle(Message::PreviewCheckerToggled)
        .text_size(13)
        .into()
}

// ---------------------------------------------------------------------------
// コンテキスト・背景の切り替え UI (キーボード代替経路として button を使う、 §12)
// ---------------------------------------------------------------------------

fn context_picker<'a>(state: &'a AppState, current: PreviewContext) -> Element<'a, Message> {
    let t = &state.translator;
    let mk = |ctx: PreviewContext| -> Element<'a, Message> {
        let active = ctx == current;
        let label_text = state.translator.t(context_message_key(ctx));
        let lbl = if active {
            format!("{} {}", marker::READY, label_text)
        } else {
            label_text
        };
        button(text(lbl))
            .padding([6, 12])
            .on_press(Message::PreviewContextSelected(ctx))
            .into()
    };

    row![
        text(format!("{}:", t.t(MessageKey::PreviewBrowserTab))).size(13),
        mk(PreviewContext::BrowserTab16),
        mk(PreviewContext::SmartphoneIcon),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

fn background_picker<'a>(state: &'a AppState, current: ThemeMode) -> Element<'a, Message> {
    let mk = |theme: ThemeMode| -> Element<'a, Message> {
        let active = theme == current;
        let label_text = state.translator.t(background_message_key(theme));
        let lbl = if active {
            format!("{} {}", marker::READY, label_text)
        } else {
            label_text
        };
        button(text(lbl))
            .padding([6, 12])
            .on_press(Message::PreviewBackgroundSelected(theme))
            .into()
    };

    // 「Background:」 のような共通ラベルは v1.5.0 では出さず、 切替ボタン群だけ並べる
    // (個々の "System / Light / Dark" ラベル自体が機能を伝える)
    row![
        mk(ThemeMode::System),
        mk(ThemeMode::Light),
        mk(ThemeMode::Dark),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

fn loading_placeholder<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    container(
        column![
            text(format!("{} {}", marker::BUSY, t.t(MessageKey::ImportingMessage))).size(16),
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
// v1.7.0: 透過チェッカー (専用ビュー)
// ---------------------------------------------------------------------------

/// 市松模様のサイズ。 240×240 で 1 タイル 12px。 ABDD §12 の「色だけに依存しない」
/// に従い、 透明度の有無を視覚的に明示するための共通パターン。
const CHECKER_SIDE: u32 = 240;
const CHECKER_TILE: u32 = 12;

/// 1 度生成したらアプリ実行中ずっと同じ。 毎フレーム再生成すると無駄な
/// アロケーションが発生するため、 `OnceLock` でキャッシュする。
fn checker_handle() -> &'static image::Handle {
    use std::sync::OnceLock;
    static HANDLE: OnceLock<image::Handle> = OnceLock::new();
    HANDLE.get_or_init(|| {
        // ライト寄りグレー 2 色の市松。 透明部分を「見やすい中間色」 で示す
        // ことで、 ロゴが白でも黒でも輪郭が判別しやすい。
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

/// 透過チェッカー専用ビュー。 framing (タブ枠 / スマホ枠) を一切付けず、
/// 市松背景の上にアイコンを実寸より少し大きく拡大して重ねる。
///
/// 設計判断:
/// - **framing を排除**: 透過状態の確認に集中するための専用モード。 タブ枠や
///   スマホホーム枠が同居すると「何を見てよいか」 がブレる
/// - **アイコンを 120px で表示**: cache.icon_120 の実寸そのまま (アイコンが
///   小さいと透明部分の存在感が薄れる)
/// - **市松を 240×240**: アイコンの周囲に余白があり、 透過範囲が明確に分かる
fn checker_view<'a>(rgba: &'a Rgba8) -> Element<'a, Message> {
    let icon_bytes: Vec<u8> = rgba.as_bytes().to_vec();
    let icon_handle = image::Handle::from_rgba(rgba.width, rgba.height, icon_bytes);

    // 市松背景: 240×240 を実寸表示
    let checker_layer = container(
        image(checker_handle().clone())
            .width(Length::Fixed(CHECKER_SIDE as f32))
            .height(Length::Fixed(CHECKER_SIDE as f32))
            .filter_method(image::FilterMethod::Nearest),
    )
    .center_x(Length::Fill)
    .center_y(Length::Fill);

    // アイコン層: cache.icon_120 を実寸表示。 中央寄せで市松の上に重ねる
    let icon_layer = container(
        image(icon_handle)
            .width(Length::Fixed(rgba.width as f32))
            .height(Length::Fixed(rgba.height as f32))
            .filter_method(image::FilterMethod::Nearest),
    )
    .center_x(Length::Fill)
    .center_y(Length::Fill);

    // stack で 2 層に重ねる (iced 0.14 の stack は後の要素ほど手前に描画)。
    container(stack![checker_layer, icon_layer])
        .width(Length::Fixed(CHECKER_SIDE as f32))
        .height(Length::Fixed(CHECKER_SIDE as f32))
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
