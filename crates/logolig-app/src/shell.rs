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

use iced::widget::{button, column, container, row, text, tooltip, Space};
use iced::{Background, Border, Color, Element, Length, Theme};
use snora::{AppLayout, BottomSheet, LayoutDirection, SheetHeight, render};

use logolig_core::{MessageKey, ThemeMode};
use logolig_i18n::Locale;

use crate::app::{AppState, Message, Screen};
use crate::ui::{accessibility::marker, advanced_drawer, drop_zone, preview_panel};

/// アプリ名の文字色 (controlled muted、 主役を引き立てる)。
const APP_NAME_COLOR: Color = Color::from_rgb(0.35, 0.35, 0.35);
/// タグライン (アプリ名の隣の説明文) の色 — さらに薄く。
const TAGLINE_COLOR: Color = Color::from_rgb(0.55, 0.55, 0.55);
/// アイコンボタンの padding 周りで使う余白 (横並び時のクリック誤爆防止)。
const ICON_BUTTON_PADDING: [u16; 2] = [6, 10];

pub fn view(state: &AppState) -> Element<'_, Message> {
    let mut layout = AppLayout::new(body(state))
        .header(header(state))
        .direction(LayoutDirection::Ltr)
        .toasts(state.toasts.clone())
        .on_close_modals(Message::CloseModals);

    // 詳細設定が開いているときだけ BottomSheet を取り付ける (§5.3)。
    if state.advanced_open {
        let sheet = BottomSheet::new(advanced_drawer::view(state)).with_height(SheetHeight::Half);
        layout = layout.bottom_sheet(sheet);
    }

    render(layout)
}

// ----------------------------------------------------------------------
// 各スロット
// ----------------------------------------------------------------------

fn body(state: &AppState) -> Element<'_, Message> {
    match state.screen {
        Screen::Empty => drop_zone::view(state),
        Screen::Importing | Screen::Exporting => busy_view(state),
        Screen::Preview | Screen::ExportReady => preview_panel::view(state),
    }
}

/// 新しいヘッダ (v1.10.2)。
///
/// 左側にアプリ名 + tagline、 右側にアイコンボタン群 (言語 / テーマ / 詳細 /
/// 閉じる)。 ボタン間に小さな Space を挟んで誤クリックを防ぐ。
fn header(state: &AppState) -> Element<'_, Message> {
    let t = &state.translator;

    // ----- アプリ名 + tagline -----
    // 字間を少し広げて (letter-spacing 相当)、 控えめな色で表示する。
    // iced 0.14 には letter-spacing が無いため、 サイズと太さで「アプリ名らしさ」
    // を出す。 さらに横に小さい 「— favicon ジェネレータ」 を添える。
    let app_name = text(t.t(MessageKey::AppTitle))
        .size(20)
        .color(APP_NAME_COLOR);
    let tagline_text = format!("— {}", t.t(MessageKey::AppTagline));
    let tagline = text(tagline_text).size(13).color(TAGLINE_COLOR);
    let title_block = row![app_name, tagline]
        .spacing(8)
        .align_y(iced::Alignment::Center);

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

fn busy_view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    container(
        column![
            text(format!("{} {}", marker::BUSY, t.t(MessageKey::ImportingMessage))).size(20),
        ]
        .spacing(6)
        .align_x(iced::alignment::Horizontal::Center),
    )
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}
