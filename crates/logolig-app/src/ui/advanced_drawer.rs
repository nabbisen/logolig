//! 詳細設定ドロワー（§5.3）。
//!
//! snora の `BottomSheet` の中身として描画される。
//! `state.advanced_open == true` のときのみ shell.rs から呼ばれる。
//!
//! Step 3 で:
//! - リサイズアルゴリズム選択（pick_list）
//! - PNG サイズ集合の編集
//! - サイズ別オーバーライド
//! を実装する。

use iced::widget::{button, column, text};
use iced::Element;

use crate::app::{AppState, Message};
use crate::ui::accessibility::label;

pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    column![
        text("Advanced settings").size(20),
        text("Detailed controls land here in Step 3.").size(12),
        text(format!(
            "Algorithm (current): {}",
            state.export_plan.algorithm.label()
        ))
        .size(12),
        text(format!("PNG sizes: {:?}", state.export_plan.png_sizes)).size(12),
        text(format!("ICO sizes: {:?}", state.export_plan.ico_sizes)).size(12),
        button(text("Close")).on_press(Message::AdvancedToggled),
        text(label::TOGGLE_ADVANCED_BTN).size(11),
    ]
    .spacing(8)
    .padding(20)
    .into()
}
