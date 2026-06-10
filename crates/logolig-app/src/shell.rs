//! トップレベルレイアウトの組み立て。
//!
//! ## v1.10.2: メイン画面情報設計刷新
//!
//! - **ヘッダ**: 左にアプリ名 ("Logolig" + tagline)、 右にアイコンボタン群
//!   (言語 / テーマ / 詳細 / 閉じる)。 アイコン間にスペーサーを置いて誤クリックを
//!   減らす
//! - **フッタ廃止**: 詳細トグルがヘッダに移ったため、 フッタの存在意義が消えた
//! - **アプリ名のタイポグラフィ**: 控えめな色 + 字間広げ + tagline 添え。
//!   アプリ名が「主張しすぎず、 でもアプリ名と分かる」 状態に
//!
//! 各スロットの中身 (header / body / footer / bottom_sheet content) は
//! `crate::ui::*` の純粋関数に委譲する。

use iced::widget::{button, container, row, text, tooltip, Space};
use iced::{Background, Border, Color, Element, Length, Theme};
use snora::{AppLayout, LayoutDirection, Sheet, SheetEdge, SheetSize, render};

use logolig_core::{MessageKey, ThemeMode};
use logolig_i18n::Locale;

use crate::app::{resolve_theme, AppState, Message, Screen};
use crate::ui::{accessibility::marker, advanced_drawer, colors, drop_zone};

// v1.14.0: 旧 APP_NAME_COLOR / TAGLINE_COLOR / FILE_NAME_COLOR の hardcoded
// 定数は `crate::ui::colors` モジュールの theme-aware ヘルパに移行した。
// dark テーマでも適切なコントラストを保ち、 light/dark の切替時に自動的に
// 追従する。

/// アイコンボタンの padding 周りで使う余白 (横並び時のクリック誤爆防止)。
const ICON_BUTTON_PADDING: [u16; 2] = [6, 10];

pub fn view(state: &AppState) -> Element<'_, Message> {
    let mut layout = AppLayout::new(body(state))
        .header(header(state))
        .direction(LayoutDirection::Ltr)
        .toasts(state.toasts.clone())
        .on_close_modals(Message::CloseModals);

    // 詳細設定が開いているときだけ Sheet を取り付ける (§5.3)。
    //
    // v1.13.0 (snora 0.8 移行): 旧 `BottomSheet` (Bottom 固定) は汎用 `Sheet` に
    // 統一された。
    //
    // v1.17.0: ドロワーを **画面右側** に移行 (`SheetEdge::End`)。 PNG モック
    // (新外部設計) 準拠。 LTR/RTL のロケールを問わず「論理的な end side」 に
    // 出るので国際化にも好適。 サイズは `画面幅 / 3` を基準にし、 clamp で
    // `[280px, 480px]` に抑える: 小さい画面ではラベルが読める下限を確保し、
    // 大きい画面では中央コンテンツ (Result View) を圧迫しない上限を設ける。
    if state.advanced_open {
        let sheet_pixels = drawer_pixel_width(state.window_size.width);
        let sheet = Sheet::new(advanced_drawer::view(state))
            .at(SheetEdge::End)
            .with_size(SheetSize::Pixels(sheet_pixels));
        layout = layout.sheet(sheet);
    }

    render(layout)
}

/// v1.17.0: Right Sheet の幅を画面幅から算出する。
///
/// 基準は「画面幅の 1/3」。 ただしウィンドウサイズが極端な場合の破綻を避ける
/// ため、 上限・下限を設ける:
/// - **下限 280px**: チェックボックス + ラベルが読める最低限の幅。 これより
///   狭くするとドロワーが「狭くて使いものにならない」 状態になる。
/// - **上限 480px**: ResultView (中央コンテンツ) が圧迫されない上限。 これ
///   より広くすると「画面の半分以上がドロワー」 になり、 主目的のアセット
///   一覧が見えなくなる。
fn drawer_pixel_width(window_width: f32) -> f32 {
    (window_width / 3.0).clamp(280.0, 480.0)
}

// ----------------------------------------------------------------------
// 各スロット
// ----------------------------------------------------------------------

fn body(state: &AppState) -> Element<'_, Message> {
    // v1.16.0: 5 状態 → 3 状態に簡素化。
    // - Empty: ドロップゾーン
    // - Converting: 円形プログレス + ファイル処理中の表示 (新 ui::converting)
    // - Result: アセットカードグリッド + 一括 DL + 折りたたみプレビュー
    //   (新 ui::result_view)
    match state.screen {
        Screen::Empty => drop_zone::view(state),
        Screen::Converting => crate::ui::converting::view(state),
        Screen::Result => crate::ui::result_view::view(state),
    }
}

/// 新しいヘッダ (v1.10.2)。
///
/// 左側にアプリ名 + tagline、 右側にアイコンボタン群 (言語 / テーマ / 詳細 /
/// 閉じる)。 ボタン間に小さな Space を挟んで誤クリックを防ぐ。
///
/// v1.16.0: Screen 列挙が簡素化されたため、 ヘッダの分岐も整理:
/// - Result 画面 (= ファイル処理済み) → ファイル名表示
/// - Empty / Converting → アプリ名 + tagline
fn header(state: &AppState) -> Element<'_, Message> {
    let t = &state.translator;
    let theme = resolve_theme(state);

    // ----- ヘッダ左のタイトル領域 -----
    //
    // v1.12.0: 画面の状態によって出すものを切り替える:
    // - Empty / Converting: アプリ名 + tagline
    // - Result: 現在処理中のファイル名 (画像が主役)
    //
    // v1.14.0: 色は `crate::ui::colors` の theme-aware ヘルパ経由。 light/dark
    // 切替時に自動追従する。
    let title_block: Element<'_, Message> = match state.screen {
        Screen::Result => {
            let file_name = state
                .source_asset
                .as_ref()
                .map(|a| a.display_name())
                .unwrap_or_default();
            text(file_name)
                .size(16)
                .color(colors::file_name(&theme))
                .into()
        }
        Screen::Empty | Screen::Converting => {
            // サイズで「アプリ名らしさ」 を出す。 さらに横に小さい
            // 「— favicon ジェネレータ」 を添える。
            let app_name = text(t.t(MessageKey::AppTitle))
                .size(20)
                .color(colors::app_name(&theme));
            let tagline_text = format!("— {}", t.t(MessageKey::AppTagline));
            let tagline = text(tagline_text).size(13).color(colors::tagline(&theme));
            row![app_name, tagline]
                .spacing(8)
                .align_y(iced::Alignment::Center)
                .into()
        }
    };

    // ----- busy インジケータ -----
    let busy_indicator: Element<'_, Message> = if state.busy {
        text(format!(
            "{} {}",
            marker::BUSY,
            t.t(MessageKey::ImportingMessage)
        ))
        .size(13)
        .into()
    } else {
        Space::new().into()
    };

    // ----- ヘッダ右のアイコンボタン群 -----
    // 言語 (文A) → テーマ (☀/☾/◐) → 詳細 (⚙) → 閉じる (✕)
    // 隙間 (Space::new().width(Fixed(8.0))) を間に挟んで誤クリックを防ぐ。
    let lang_btn = icon_button_with_tooltip(
        language_icon_glyph(state.locale_override),
        &t.t(MessageKey::TooltipLanguage),
        Message::LocaleCycled,
    );
    let theme_btn = icon_button_with_tooltip(
        theme_icon_glyph(state.theme),
        &t.t(MessageKey::TooltipTheme),
        Message::ThemeToggled,
    );
    let advanced_btn = icon_button_with_tooltip(
        "⚙",
        &t.t(MessageKey::TooltipAdvanced),
        Message::AdvancedToggled,
    );
    let close_btn = icon_button_with_tooltip(
        "✕",
        &t.t(MessageKey::TooltipClose),
        Message::AppCloseRequested,
    );

    container(
        row![
            title_block,
            // タイトルとアイコン群の間の自由空間
            busy_indicator,
            Space::new().width(Length::Fill),
            // アイコン群 (各ボタン間 8px 隙間)
            lang_btn,
            Space::new().width(Length::Fixed(8.0)),
            theme_btn,
            Space::new().width(Length::Fixed(8.0)),
            advanced_btn,
            // 閉じるボタンは「離して」 配置 (誤操作リスク高いため隙間を広めに取る)
            Space::new().width(Length::Fixed(16.0)),
            close_btn,
        ]
        .align_y(iced::Alignment::Center)
        .spacing(12),
    )
    .padding([12, 20])
    .width(Length::Fill)
    .into()
}

/// アイコン文字 1 文字 + tooltip を備えたボタン。 全ヘッダボタンの共通形。
fn icon_button_with_tooltip<'a>(
    glyph: &'a str,
    tip: &str,
    on_press: Message,
) -> Element<'a, Message> {
    let btn = button(text(glyph).size(16))
        .padding(ICON_BUTTON_PADDING)
        .on_press(on_press)
        .style(|theme: &Theme, status| {
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
        });
    // tooltip ウィジェットでラップ。 hover 時に説明テキストを表示。
    tooltip(btn, text(tip.to_string()).size(11), tooltip::Position::Bottom)
        .gap(4)
        .padding(6)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(Background::Color(palette.background.strong.color)),
                text_color: Some(palette.background.strong.text),
                border: Border {
                    color: palette.background.strong.color,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

/// 言語アイコンの glyph を現在のロケール上書き状態から決定する。
/// `文A` 1 種類でも問題はないが、 「現在の状態が分かる」 と「次の状態を予期できる」
/// の両方を満たすため、 None=システム / En=英 / Ja=日 で表示を変える:
/// - System: `文A` (中立、 多言語の象徴)
/// - English: `Aa` (ラテン文字)
/// - 日本語: `あ` (ひらがな代表)
fn language_icon_glyph(locale_override: Option<Locale>) -> &'static str {
    match locale_override {
        None => "文A",
        Some(Locale::En) => "Aa",
        Some(Locale::Ja) => "あ",
    }
}

/// テーマアイコンの glyph を現在のテーマから決定する。 状態反映型。
/// - System: `◐` (半円、 「自動」 の慣用)
/// - Light: `☀` (太陽)
/// - Dark: `☾` (三日月)
fn theme_icon_glyph(theme: ThemeMode) -> &'static str {
    match theme {
        ThemeMode::System => "◐",
        ThemeMode::Light => "☀",
        ThemeMode::Dark => "☾",
    }
}

// v1.16.0: 旧 busy_view は ui::converting::view に移行・拡充されたため削除。
// header の busy インジケータは header() 内で marker::BUSY を引き続き使用。