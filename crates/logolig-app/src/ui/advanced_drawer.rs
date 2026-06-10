//! 詳細設定ドロワー (§5.3)。
//!
//! 既定値は隠す。 `state.advanced_open == true` のときだけ shell が表示する。
//! ここで提供するもの:
//! - リサイズアルゴリズムの `pick_list` (5 アルゴリズム)
//! - 出力 PNG サイズ集合の現状表示 (Step 4 で編集 UI を追加)
//! - ICO 内包サイズの現状表示
//!
//! どれもすぐに反映されるよう、 状態変更 Message を即送る。

use iced::widget::{button, column, pick_list, row, text};
use iced::Element;

use logolig_core::ResizeAlgorithm;

use crate::app::{AppState, Message};
use crate::ui::accessibility::label;

pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    column![
        text("Advanced settings").size(20),
        text("Defaults are tuned for quality and minimal output. Adjust only if you know why.")
            .size(12),
        // リサイズアルゴリズム選択
        algorithm_row(state),
        // PNG / ICO サイズの現状表示 (Step 4 で編集 UI を追加)
        text(format!("PNG sizes: {:?}", state.export_plan.png_sizes)).size(12),
        text(format!("ICO sizes: {:?}", state.export_plan.ico_sizes)).size(12),
        text(format!("Apple touch icon: {}", state.export_plan.include_apple_touch)).size(12),
        text(format!("HTML snippet: {}", state.export_plan.include_html_snippet)).size(12),
        // 閉じるボタン (キーボード代替経路)
        button(text("Close")).on_press(Message::AdvancedToggled),
        text(label::TOGGLE_ADVANCED_BTN).size(11),
    ]
    .spacing(10)
    .padding(20)
    .into()
}

fn algorithm_row<'a>(state: &'a AppState) -> Element<'a, Message> {
    // PickList は `&[T]` を受けるので、 `ResizeAlgorithm::all()` の固定配列を使う。
    // 配列を直接渡すと一時値の寿命問題で落ちるので、 ここで Vec に集める方針:
    // PickList::new は `L: Borrow<[T]> + 'a` を要求するため Vec<T> がそのまま乗る。
    let options: Vec<ResizeAlgorithm> = ResizeAlgorithm::all().to_vec();
    let picker = pick_list(
        options,
        Some(state.export_plan.algorithm),
        Message::AlgorithmChanged,
    )
    .text_size(13);

    row![text("Resize algorithm:").size(13), picker]
        .spacing(10)
        .align_y(iced::Alignment::Center)
        .into()
}
