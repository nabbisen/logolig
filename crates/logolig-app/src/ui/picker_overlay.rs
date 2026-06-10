//! v1.18.0 — 言語 / テーマピッカーのオーバーレイ。
//!
//! PNG モック (新外部設計) の「言語メニュー」 / 「テーマメニュー」 に対応する
//! ポップアップ。 サイドバーのアイコンをクリックすると `active_picker` が
//! セットされ、 `AppLayout::context_menu(...)` slot にこの Element が乗る。
//! 外側クリックで `on_close_menus` が発火 → `Message::SidebarPickerClosed`
//! → `active_picker = None` で消える。
//!
//! ## 構成
//!
//! ```text
//! ┌──────────────────┐
//! │ 🌐 言語          │  ← ヘッダ (アイコン + 「言語」 / 「テーマ」)
//! │                  │
//! │ ✓ 日本語         │  ← 現在値にチェック (✓ プレフィックス)
//! │   English        │
//! │   システム設定に従う │  ← Locale ピッカーのみ「auto」 行
//! └──────────────────┘
//! ```

use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding, Theme};

use logolig_core::{MessageKey, ThemeMode};
use logolig_i18n::Locale;
use snora::lucide;
use snora::widget::icon_element_sized;
use snora::Icon;

use crate::app::{resolve_theme, AppState, Message, SidebarPicker};
use crate::ui::colors;

/// ポップアップの幅 (PNG モック準拠の 220px)。
const POPUP_WIDTH: f32 = 220.0;

/// `AppState::active_picker` に応じて適切なピッカーを描画する。 active_picker
/// が None の場合は呼ばれない (shell 側で gating)。
pub fn view<'a>(state: &'a AppState, picker: SidebarPicker) -> Element<'a, Message> {
    match picker {
        SidebarPicker::Locale => locale_view(state),
        SidebarPicker::Theme => theme_view(state),
    }
}

fn locale_view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    let theme = resolve_theme(state);

    let header = row![
        icon_element_sized::<Message>(&Icon::Lucide(lucide::Languages), 16.0),
        text(t.t(MessageKey::SidebarLabelLocale))
            .size(14)
            .color(colors::page_title(&theme)),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    // 選択肢: 日本語 / English / システム設定に従う (None)
    let current = state.locale_override;
    let options = column![
        picker_row(
            current == Some(Locale::Ja),
            t.t(MessageKey::LocaleNameJa),
            Message::LocalePicked(Some(Locale::Ja)),
            &theme,
        ),
        picker_row(
            current == Some(Locale::En),
            t.t(MessageKey::LocaleNameEn),
            Message::LocalePicked(Some(Locale::En)),
            &theme,
        ),
        picker_row(
            current.is_none(),
            t.t(MessageKey::LocaleSystem),
            Message::LocalePicked(None),
            &theme,
        ),
    ]
    .spacing(2);

    popup_container(column![header, options].spacing(12))
}

fn theme_view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    let theme = resolve_theme(state);

    let header = row![
        icon_element_sized::<Message>(&Icon::Lucide(lucide::Moon), 16.0),
        text(t.t(MessageKey::SidebarLabelTheme))
            .size(14)
            .color(colors::page_title(&theme)),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let current = state.theme;
    let options = column![
        picker_row(
            current == ThemeMode::Light,
            t.t(MessageKey::ThemeNameLight),
            Message::ThemePicked(ThemeMode::Light),
            &theme,
        ),
        picker_row(
            current == ThemeMode::Dark,
            t.t(MessageKey::ThemeNameDark),
            Message::ThemePicked(ThemeMode::Dark),
            &theme,
        ),
        picker_row(
            current == ThemeMode::System,
            t.t(MessageKey::ThemeSystem),
            Message::ThemePicked(ThemeMode::System),
            &theme,
        ),
    ]
    .spacing(2);

    popup_container(column![header, options].spacing(12))
}

/// 選択肢 1 行。 active なら ✓ プレフィックス + 強調色、 そうでなければ
/// muted。 行全体がボタン (クリックで `on_press` 発火 → ピッカーが閉じる)。
fn picker_row<'a>(
    active: bool,
    label: String,
    on_press: Message,
    theme: &Theme,
) -> Element<'a, Message> {
    let prefix = if active { "✓ " } else { "  " };
    let label_color = if active {
        colors::page_title(theme)
    } else {
        colors::muted_text(theme)
    };
    let inner = row![
        text(prefix).size(13).color(label_color),
        text(label).size(13).color(label_color),
    ]
    .spacing(4)
    .align_y(Alignment::Center);

    button(inner)
        .padding(Padding::default().top(6).bottom(6).left(8).right(8))
        .width(Length::Fill)
        .on_press(on_press)
        .style(move |theme: &Theme, status| picker_row_style(theme, status, active))
        .into()
}

/// ポップアップ全体のコンテナ (背景 + 枠線 + padding)。
fn popup_container<'a>(content: iced::widget::Column<'a, Message>) -> Element<'a, Message> {
    container(content)
        .padding(16)
        .width(Length::Fixed(POPUP_WIDTH))
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(Background::Color(palette.background.base.color)),
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

fn picker_row_style(
    theme: &Theme,
    status: iced::widget::button::Status,
    active: bool,
) -> iced::widget::button::Style {
    let palette = theme.extended_palette();
    let bg = if active {
        Some(Background::Color(palette.background.weak.color))
    } else {
        match status {
            iced::widget::button::Status::Hovered => {
                Some(Background::Color(palette.background.weak.color))
            }
            _ => None,
        }
    };
    iced::widget::button::Style {
        background: bg,
        text_color: palette.background.base.text,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}
