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
use snora::{AppLayout, LayoutDirection, Sheet, SheetSize, render};

use logolig_core::{MessageKey, ThemeMode};
use logolig_i18n::Locale;

use crate::app::{resolve_theme, AppState, Message, Screen};
use crate::ui::{accessibility::marker, advanced_drawer, colors, drop_zone, preview_panel};

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
    // 統一された。 `Sheet::new` のデフォルト edge は `SheetEdge::Bottom` なので
    // 旧来と同じ「下から滑り出るドロワー」 として動く。 サイズ指定は
    // `with_height(SheetHeight::Half)` から `with_size(SheetSize::Half)` に
    // 改名 (Sheet を Top/Start/End に置けるよう、 axis に依存しない `Size` 名に
    // 統一された)。
    if state.advanced_open {
        let sheet = Sheet::new(advanced_drawer::view(state)).with_size(SheetSize::Half);
        layout = layout.sheet(sheet);
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
    let theme = resolve_theme(state);

    // ----- ヘッダ左のタイトル領域 -----
    //
    // v1.12.0: 画面の状態によって出すものを切り替える:
    // - startup 画面 (Empty): アプリ名 + tagline (アプリの自己紹介がここの主目的)
    // - 編集画面 (Preview / ExportReady): 現在処理中のファイル名 (画像が主役)
    // - その他 (Importing / Exporting): アプリ名のみ (短時間しか出ないので簡素に)
    //
    // 編集画面時にアプリ名を出さないのは「画面の主役は今ロード中の画像で、
    // アプリの自己紹介はもう済んだ」 という遷移を視覚化するため。 ファイル名
    // を本文相当の濃さで出して、 編集画面の「対象画像」 がここだと一目で
    // 分かるようにする。
    //
    // v1.14.0: 色は `crate::ui::colors` の theme-aware ヘルパ経由。 light/dark
    // 切替時に自動追従する。
    let title_block: Element<'_, Message> = match state.screen {
        Screen::Preview | Screen::ExportReady => {
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
        Screen::Empty | Screen::Importing | Screen::Exporting => {
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
