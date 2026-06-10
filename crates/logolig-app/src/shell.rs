//! トップレベルレイアウトの組み立て。
//!
//! ここは **snora::AppLayout の構築だけ** に専念する。
//! 各スロットの中身 (header / body / footer / bottom_sheet content) は
//! `crate::ui::*` の純粋関数に委譲する。

use iced::widget::{button, column, container, row, text, Space};
use iced::{Element, Length};
use snora::{AppLayout, BottomSheet, LayoutDirection, SheetHeight, render};

use logolig_core::MessageKey;

use crate::app::{AppState, Message, Screen};
use crate::ui::{
    accessibility::label, accessibility::marker, advanced_drawer, drop_zone, preview_panel,
};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let mut layout = AppLayout::new(body(state))
        .header(header(state))
        .footer(footer(state))
        .direction(LayoutDirection::Ltr)
        // toasts(Vec<Toast<Message>>) は値渡し。Toast は Clone derive 済み。
        .toasts(state.toasts.clone())
        // Bottom sheet を閉じる方向のクリックを `CloseModals` にまとめて受ける。
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

fn header(state: &AppState) -> Element<'_, Message> {
    let t = &state.translator;
    // テーマラベル: 翻訳された "Theme" + 翻訳された現在のモード名
    let theme_mode_key = match state.theme {
        logolig_core::ThemeMode::System => MessageKey::PreviewBackgroundSystem,
        logolig_core::ThemeMode::Light => MessageKey::PreviewBackgroundLight,
        logolig_core::ThemeMode::Dark => MessageKey::PreviewBackgroundDark,
    };
    let theme_label = format!(
        "{}: {}",
        t.t(MessageKey::ToggleThemeButton),
        t.t(theme_mode_key)
    );

    // busy 表示は色だけでなく文字マーカーでも示す (§12「色だけに依存しない」)
    let busy_indicator: Element<'_, Message> = if state.busy {
        text(format!("{} {}", marker::BUSY, t.t(MessageKey::ImportingMessage)))
            .size(13)
            .into()
    } else {
        // 空のゼロサイズスペース。レイアウトを安定させる。
        Space::new().into()
    };

    container(
        row![
            text(t.t(MessageKey::AppTitle)).size(22),
            busy_indicator,
            // 横方向 flex spacer。`horizontal_space()` は iced 0.14 で削除された。
            Space::new().width(Length::Fill),
            button(text(theme_label))
                .padding([6, 12])
                .on_press(Message::ThemeToggled),
        ]
        .align_y(iced::Alignment::Center)
        .spacing(12),
    )
    .padding([12, 20])
    .width(Length::Fill)
    .into()
}

fn footer(state: &AppState) -> Element<'_, Message> {
    let t = &state.translator;
    let advanced_label = if state.advanced_open {
        // 「Hide advanced」 と「Show advanced」 で切り替えるが、 v1.5.0 では
        // 同じキー (ToggleAdvancedButton) を使う。 必要なら v1.6 で
        // ToggleAdvancedShow / Hide の 2 キーに分ける。
        t.t(MessageKey::ToggleAdvancedButton)
    } else {
        t.t(MessageKey::ToggleAdvancedButton)
    };

    container(
        row![
            button(text(advanced_label))
                .padding([6, 12])
                .on_press(Message::AdvancedToggled),
            Space::new().width(Length::Fill),
            text(label::TOGGLE_ADVANCED_BTN).size(11),
        ]
        .align_y(iced::Alignment::Center)
        .spacing(12),
    )
    .padding([8, 20])
    .width(Length::Fill)
    .into()
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
