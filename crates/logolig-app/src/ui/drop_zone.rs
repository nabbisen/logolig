//! ドロップゾーン画面（起動直後）。
//!
//! §5.1:
//! - 設定項目を見せない
//! - シングルカラムの大きなドロップ領域
//! - キーボード操作のみで完結する代替経路（ファイル選択ボタン）
//! - フォーカス可視化はテーマのデフォルトに任せ、ラベルを意味のある文に
//!
//! v1.5.0: 全文言を `state.translator.t(MessageKey::...)` 経由で翻訳。

use iced::widget::{button, column, container, text};
use iced::{Element, Length};

use logolig_core::MessageKey;

use crate::app::{AppState, Message};
use crate::ui::accessibility::label;

pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    let body = column![
        text(t.t(MessageKey::DropZoneInstruction)).size(28),
        text(t.t(MessageKey::DropZoneSecondary)).size(14),
        text(t.t(MessageKey::DropZoneAcceptedFormats)).size(12),
        // ドラッグ&ドロップが使えない場合の代替経路 (§12)
        button(text(t.t(MessageKey::ChooseFileButton)).size(16))
            .padding([10, 18])
            .on_press(Message::PickFileRequested),
        // a11y のためのラベル提示 (キー化されていない位置情報)
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
