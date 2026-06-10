//! v1.20.0 — モバイル時の下部ナビ (`ui::bottom_nav`)。
//!
//! `ui::sidebar` の双子モジュール。 デスクトップで左サイドバーに置いて
//! いた 設定 / 言語 / テーマ の 3 アイコンを、 モバイル幅では画面下端の
//! 横並びナビに移す (PNG モック明示の挙動)。 発火する Message、 アクティブ
//! 判定、 視覚スタイルは sidebar.rs と完全に揃える — 「同じ機能の場所が
//! 変わるだけ」 のメンタルモデル。
//!
//! ## レイアウト
//!
//! ```text
//!  ┌──┬──┬──┐  ← 横並び 3 セル (= 3 等分)
//!  │⚙ │🅰│🌙│  ← アイコン
//!  │設定│言語│テーマ│  ← ラベル下段
//!  └──┴──┴──┘
//! ```
//!
//! ## 配置
//!
//! `AppLayout::footer()` slot に乗せる。 snora のスケルトン
//! (`header / side_bar | body / footer`) において、 footer は画面下端に
//! 固定表示される。 デスクトップ時は side_bar slot を使い、 モバイル時は
//! footer slot を使う、 という排他切替。
//!
//! ## 高さ
//!
//! 64px 固定。 アイコン (22px) + ラベル (11px) + 上下 padding (8px ×2) +
//! ラベル/アイコン間 spacing (4px) = 53px なので、 余裕を見て 64px。
//! Material Design の bottom navigation の標準的な値 (56px〜80px) と整合。

use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding, Theme};

use logolig_core::MessageKey;
use snora::lucide;
use snora::widget::icon_element_sized;
use snora::Icon;

use crate::app::{resolve_theme, AppState, Message, SidebarPicker};
use crate::ui::colors;

/// 下部ナビの固定高さ。 PNG モック準拠 + Material Design ガイドライン整合。
const BOTTOM_NAV_HEIGHT: f32 = 64.0;

/// 下部ナビ描画。 戻り値は `AppLayout::footer()` slot に渡す Element。
pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    let theme = resolve_theme(state);

    // 3 セル横並び。 各セルは Length::FillPortion(1) で 3 等分。
    // sidebar.rs の sidebar_button と同じ構造 (アイコン + ラベル下) だが、
    // セルの方向 (横並び vs 縦並び) と FillPortion による幅分配が異なる。
    let settings_cell = nav_cell(
        Icon::Lucide(lucide::Settings),
        t.t(MessageKey::SidebarLabelSettings),
        Some(Message::AdvancedToggled),
        state.advanced_open,
        &theme,
    );

    let locale_cell = nav_cell(
        Icon::Lucide(lucide::Languages),
        t.t(MessageKey::SidebarLabelLocale),
        Some(Message::SidebarPickerOpened(SidebarPicker::Locale)),
        state.active_picker == Some(SidebarPicker::Locale),
        &theme,
    );

    let theme_cell = nav_cell(
        Icon::Lucide(lucide::Moon),
        t.t(MessageKey::SidebarLabelTheme),
        Some(Message::SidebarPickerOpened(SidebarPicker::Theme)),
        state.active_picker == Some(SidebarPicker::Theme),
        &theme,
    );

    let nav_row = row![settings_cell, locale_cell, theme_cell].spacing(0);

    container(nav_row)
        .width(Length::Fill)
        .height(Length::Fixed(BOTTOM_NAV_HEIGHT))
        .style(|theme: &Theme| {
            // 下部ナビの背景は、 中央コンテンツ (background.base) と区別する
            // ため `background.weak` を使う (sidebar.rs と統一)。 上に細い
            // divider を引いて領域分割を視覚化。
            let palette = theme.extended_palette();
            container::Style {
                background: Some(Background::Color(palette.background.weak.color)),
                border: Border {
                    color: palette.background.strong.color,
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

/// 1 セル (アイコン + 下にラベル + クリックで Message 発火)。
///
/// `is_active` が true なら強調背景 + ラベル色を切替で「今このピッカー or
/// ドロワーが開いている」 をユーザに伝える (sidebar.rs と完全同一の振る舞い)。
fn nav_cell<'a>(
    icon: Icon,
    label: String,
    on_press: Option<Message>,
    is_active: bool,
    theme: &Theme,
) -> Element<'a, Message> {
    let icon_el: Element<'a, Message> = icon_element_sized(&icon, 22.0);

    let label_color = if is_active {
        colors::page_title(theme)
    } else {
        colors::muted_text(theme)
    };

    let inner = column![icon_el, text(label).size(11).color(label_color)]
        .spacing(4)
        .align_x(Alignment::Center);

    let mut btn = button(
        container(inner)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding(Padding::default().top(8).bottom(8)),
    )
    .width(Length::FillPortion(1))
    .height(Length::Fill)
    .style(move |theme: &Theme, status| nav_cell_style(theme, status, is_active));
    if let Some(msg) = on_press {
        btn = btn.on_press(msg);
    }
    btn.into()
}

/// セルのスタイル: 透明 / hover で薄塗り / active で強めの薄塗り。
/// sidebar.rs の `sidebar_button_style` と完全同一ロジック (定数の数値も
/// 一致させてある)。
fn nav_cell_style(
    theme: &Theme,
    status: iced::widget::button::Status,
    is_active: bool,
) -> iced::widget::button::Style {
    let palette = theme.extended_palette();
    let bg = if is_active {
        Some(Background::Color(Color {
            a: 0.35,
            ..palette.background.strong.color
        }))
    } else {
        match status {
            iced::widget::button::Status::Hovered => {
                Some(Background::Color(palette.background.strong.color))
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
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}
