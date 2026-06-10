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
use iced::{Background, Border, Color, Element, Length, Padding, Theme};

use logolig_core::{MessageKey, PreviewCache, PreviewContext, Rgba8, ThemeMode};

use crate::app::{resolve_theme, AppState, Message};
use crate::ui::accessibility::marker;
use crate::ui::colors;

pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    let theme = resolve_theme(state);
    // v1.12.0: ファイル名はヘッダ左側に移動 (shell::header)。 preview_panel
    // からは display_name 取得が不要になった。

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

    // メインプレビュー領域 (キャッシュが揃ってから初めて描ける)。
    // v1.10.0: PreviewContext::TransparencyChecker のときは framing を一切
    // 付けず、 市松模様の上に icon_120 を実寸で重ねる専用ビューを使う。
    let preview_area: Element<'a, Message> = match state.preview_cache.as_ref() {
        Some(cache) => match context {
            PreviewContext::TransparencyChecker => checker_view(&cache.icon_120),
            _ => render_context(cache, context, bg),
        },
        None => loading_placeholder(state),
    };

    // ----- v1.12.0 編集画面の構成 -----
    //
    // 課題に答える:
    // 1. 「ここはプレビューです」 をプレビュー領域の枠で視覚化 (border + soft fill)
    // 2. 「このボタン群でいろいろな見た目を確認できます」 を view-as / surface
    //    ピッカーをプレビュー枠の上に置くことで「枠の操作」 と直感させる
    // 3. 「Export ボタンを押すことでファイル作成できます」 を画面下部・右寄せ・
    //    強調スタイルで明示
    // 4. プレビューサイズ不安定問題: container を `FillPortion(4)` で取って
    //    画面の縦 4/7 を占めるよう統一。 max_width/max_height で過大表示を防止。
    //    全モード (タブ風 / スマホ風 / チェッカー) で同じサイズ枠の中に center
    //    配置する。
    // 5. 配置: 画面タイトルは中央寄せ、 Preview ラベルは枠の左上、 ピッカーは
    //    中央寄せ、 Export は右寄せ。 単純な左寄せ偏重を避ける。
    //
    // テキスト依存しすぎを避けるため、 階層は (a) フォントサイズ差 (b) 枠の
    // 視覚的境界 (c) 配置 (上中下、 左中右) で表す。

    // 1. 画面タイトル (中央寄せ、 中程度の濃さ)
    let page_title = container(
        text(t.t(MessageKey::PageTitleEdit))
            .size(20)
            .color(colors::page_title(&theme)),
    )
    .center_x(Length::Fill)
    .padding(Padding::default().top(4).bottom(4));

    // 2. Preview カード - 枠 + 弱い背景塗りで「プレビュー領域」 を視覚化
    let preview_card = container(
        column![
            // 「Preview」 ラベル (枠内左上、 セクションタイトル相当)
            text(t.t(MessageKey::SectionTitlePreview))
                .size(13)
                .color(colors::section_label(&theme)),
            // ピッカー群 (中央寄せ)
            container(view_as_picker(state, context)).center_x(Length::Fill),
            container(surface_picker(state, context, bg)).center_x(Length::Fill),
            // プレビュー枠 (中身は中央配置 + サイズは画面の 4/7、 max 560 で天井)
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

    // 3. アクション行: 左に補助 (戻る / 再選択)、 右に主操作 (Export)
    let action_row = row![
        // 補助操作群 (controlled muted style)
        button(text(t.t(MessageKey::EditCancelButton)).size(13))
            .padding([8, 16])
            .on_press(Message::EditCancelled)
            .style(secondary_button_style),
        button(text(t.t(MessageKey::EditRepickButton)).size(13))
            .padding([8, 16])
            .on_press(Message::PickFileRequested)
            .style(secondary_button_style),
        // 中央スペース (左補助と右主操作を分離)
        Space::new().width(Length::Fill),
        // 主操作: Export — 大きく、 強調スタイル (theme primary)
        button(text(t.t(MessageKey::ExportButton)).size(15))
            .padding([10, 28])
            .on_press(Message::ExportRequested),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    column![page_title, preview_card, action_row]
        .spacing(14)
        .padding(Padding::default().left(8).right(8).top(4).bottom(8))
        .into()
}

// v1.14.0: PAGE_TITLE_COLOR / SECTION_LABEL_COLOR / MUTED_TEXT の hardcoded
// 定数は `crate::ui::colors` の theme-aware ヘルパに移行した。 dark/light の
// 切替に追従。

/// 補助ボタンのスタイル (戻る / 再選択用)。
/// theme primary (Export ボタン) と差別化するため、 透明背景 + 弱い枠線、
/// hover 時にだけ薄く塗る。
fn secondary_button_style(
    theme: &iced::Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let palette = theme.extended_palette();
    let bg = match status {
        iced::widget::button::Status::Hovered => palette.background.weak.color,
        _ => Color::TRANSPARENT,
    };
    iced::widget::button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: palette.background.weak.text,
        border: iced::Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// コンテキスト・背景の切り替え UI (キーボード代替経路として button を使う、 §12)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// v1.10.0: 「View as」 / 「Surface」 ボタン群
//
// 設計:
// - 群の左にラベル (`View as:` / `Surface:`) を 80px 固定幅で揃え、 視覚的に
//   2 つの群が同じ並列構造であることを示す
// - active ボタンは背景塗りで強調 (ABDD §12 の color-blind 安全のため、 文字
//   prefix `▣` も併用 — 色覚に依存せずに状態が分かる)
// - Surface 群は Checker 表示中は disabled (背景塗りが意味を持たないため)
// ---------------------------------------------------------------------------

/// 「View as」 群: タブ風 / スマホ風 / Checker の 3 ボタン。
fn view_as_picker<'a>(state: &'a AppState, current: PreviewContext) -> Element<'a, Message> {
    let t = &state.translator;
    let mut buttons = row![]
        .spacing(6)
        .align_y(iced::Alignment::Center);
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

/// 「Surface」 群: System / Light / Dark の 3 ボタン。
/// Checker コンテキスト中は背景設定が描画に影響しないため、 全ボタンが
/// 視覚的に灰色化 (押下しても効果なし) する。 disable は `on_press` を渡さない
/// ことで実現する。
fn surface_picker<'a>(
    state: &'a AppState,
    context: PreviewContext,
    current: ThemeMode,
) -> Element<'a, Message> {
    let t = &state.translator;
    let respects_surface = context.respects_surface();
    let mut buttons = row![]
        .spacing(6)
        .align_y(iced::Alignment::Center);
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

/// 押下可能な picker ボタン。
fn picker_button<'a>(label: &str, active: bool, on_press: Message) -> Element<'a, Message> {
    picker_button_optional(label, active, Some(on_press))
}

/// `on_press = None` のとき disabled な picker ボタンを返す。
/// active ボタンは `marker::READY` プレフィックス + 塗り背景で 2 通りの方法で
/// 強調 (色覚に依存しない原則)。
fn picker_button_optional<'a>(
    label: &str,
    active: bool,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let lbl = if active {
        format!("{} {}", marker::READY, label)
    } else {
        // 非 active のとき先頭 1 文字分のスペースを確保することで、
        // active/非 active で文字幅が違って row が揺れるのを防ぐ。
        format!("  {}", label)
    };
    let mut btn = button(text(lbl).size(13)).padding([6, 12]);
    if let Some(msg) = on_press {
        btn = btn.on_press(msg);
    }
    // active 時の視覚強調: テーマの primary 色で背景を塗る。
    // iced 0.14 の button::Style は closure で渡す。
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
        // hover で少し暗くする (active のときは更に主張、 非 active は薄い色)。
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
        // 防御的: 上位の `view` 関数が `TransparencyChecker` を `checker_view` に
        // 振り分けるため、 ここに来ることはない。 万一きてもアプリを落とさず
        // checker view を返す。
        PreviewContext::TransparencyChecker => checker_view(&cache.icon_120),
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
