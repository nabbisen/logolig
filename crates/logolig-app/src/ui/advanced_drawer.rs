//! 詳細設定ドロワー (§5.3)。
//!
//! 既定値は隠す。 `state.advanced_open == true` のときだけ shell が表示する。
//! ここで提供するもの:
//! - リサイズアルゴリズムの `pick_list` (5 アルゴリズム)
//! - SVG 出力のオン/オフ (v1.2.0)
//! - ラスタソースのベクトル化のオン/オフ (v1.2.0)
//! - 出力 PNG サイズ集合の現状表示
//! - ICO 内包サイズの現状表示
//!
//! どれもすぐに反映されるよう、 状態変更 Message を即送る。

use iced::widget::{button, checkbox, column, pick_list, row, text};
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
        // v1.2.0: SVG 出力 + ベクトル化トグル
        checkbox("Output favicon.svg", state.export_plan.include_svg)
            .on_toggle(Message::IncludeSvgToggled)
            .text_size(13),
        checkbox(
            "Vectorize raster sources to SVG (vtracer)",
            state.export_plan.vectorize_on_raster,
        )
        .on_toggle(Message::VectorizeOnRasterToggled)
        .text_size(13),
        text(
            "Tip: turn off vectorization for photos or noisy images. \
             Logos, line art and pixel art trace well."
        )
        .size(11),
        // PNG / ICO サイズの現状表示
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
