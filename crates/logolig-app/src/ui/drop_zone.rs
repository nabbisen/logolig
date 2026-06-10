//! ドロップゾーン画面（起動直後）。
//!
//! §5.1:
//! - 設定項目を見せない
//! - シングルカラムの大きなドロップ領域
//! - キーボード操作のみで完結する代替経路（ファイル選択ボタン）
//! - フォーカス可視化はテーマのデフォルトに任せ、ラベルを意味のある文に

use iced::widget::{button, column, container, text};
use iced::{Element, Length};

use crate::app::{AppState, Message};
use crate::ui::accessibility::label;

pub fn view<'a>(_state: &'a AppState) -> Element<'a, Message> {
    let body = column![
        text("Drop a PNG, SVG, or WebP").size(28),
        text("Local-first. Your image never leaves this device.").size(14),
        // ドラッグ&ドロップが使えない場合の代替経路 (§12)
        button(text("Choose file…").size(16))
            .padding([10, 18])
            .on_press(Message::PickFileRequested),
        // a11y のためのラベル提示
        text(label::DROP_ZONE).size(12),
    ]
    .spacing(16)
    .align_x(iced::alignment::Horizontal::Center);

    container(body)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .padding(40)
        .into()
}
