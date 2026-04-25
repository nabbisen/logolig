//! プレビューパネル（§5.2 コンテキストプレビュー）。
//!
//! Step 3 で実装する内容:
//! - ブラウザタブ模写 (16x16 を実寸＋周辺文脈ごと表示)
//! - スマホホーム画面模写
//! - ライト / ダーク背景の即時切り替え
//! - キーボードナビゲーション
//!
//! Step 1 ではプレースホルダのみ。

use iced::widget::{button, column, text};
use iced::Element;

use crate::app::{AppState, Message};
use crate::ui::accessibility::label;

pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let asset_name = state
        .source_asset
        .as_ref()
        .map(|a| a.display_name())
        .unwrap_or_else(|| "(no source)".into());

    column![
        text(format!("Preview: {asset_name}")).size(20),
        text("Context preview is implemented in Step 3.").size(13),
        button(text("Export"))
            .padding([8, 14])
            .on_press(Message::ExportRequested),
        text(label::EXPORT_BTN).size(11),
    ]
    .spacing(12)
    .into()
}
