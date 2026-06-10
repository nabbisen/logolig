//! 詳細設定ドロワー (§5.3)。
//!
//! 既定値は隠す。 `state.advanced_open == true` のときだけ shell が表示する。
//!
//! ## v1.3.0 で改善された点
//!
//! - **チェックボックスでオプトアウト**: ICO / Apple touch / HTML スニペットを
//!   個別にオン/オフできる
//! - **PNG / ICO サイズの編集**: 既存サイズはチップ風の `[size ×]` ボタンで
//!   削除、 末尾のテキスト入力で新規追加
//! - **セクション分け**: 「Resize」「SVG」「Files」「Sizes」 の 4 区画
//! - **検証ロジックは core 任せ**: `ExportPlan::add_*_size` / `remove_*_size`
//!   が範囲外 / 重複を弾く

use iced::widget::{button, checkbox, column, container, pick_list, row, text, text_input};
use iced::{Alignment, Element, Length, Padding};

use logolig_core::{ResizeAlgorithm, ICO_SIZE_MAX, ICO_SIZE_MIN, PNG_SIZE_MAX, PNG_SIZE_MIN};

use crate::app::{AppState, Message};
use crate::ui::accessibility::label;

pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    column![
        // ヘッダ
        text("Advanced settings").size(22),
        text(
            "Defaults are tuned for quality and minimal output. \
             Adjust only if you have a specific need."
        )
        .size(12),
        // 1. リサイズアルゴリズム
        section(
            "Resize algorithm",
            "Lanczos3 is the recommended default. Switch to Nearest only for pixel art.",
            algorithm_row(state),
        ),
        // 2. SVG 出力 (v1.2.0 機能、 v1.3.0 で UI 整理)
        section(
            "SVG output",
            "Modern browsers prefer SVG on high-DPI displays. \
             Turn off vectorization for photos or noisy images.",
            column![
                checkbox(state.export_plan.include_svg)
                    .label("Output favicon.svg")
                    .on_toggle(Message::IncludeSvgToggled)
                    .text_size(13),
                checkbox(state.export_plan.vectorize_on_raster)
                    .label("Vectorize raster sources to SVG (vtracer)")
                    .on_toggle(Message::VectorizeOnRasterToggled)
                    .text_size(13),
            ]
            .spacing(6)
            .into(),
        ),
        // 3. 出力ファイル種別 (v1.3.0 で編集可能化)
        section(
            "Files to write",
            "Each artifact can be skipped if your project doesn't need it.",
            column![
                checkbox(state.export_plan.include_ico)
                    .label("favicon.ico (legacy compatibility)")
                    .on_toggle(Message::IncludeIcoToggled)
                    .text_size(13),
                checkbox(state.export_plan.include_apple_touch)
                    .label("apple-touch-icon.png (iOS / iPadOS home screen)")
                    .on_toggle(Message::IncludeAppleTouchToggled)
                    .text_size(13),
                checkbox(state.export_plan.include_html_snippet)
                    .label("favicon-snippet.html (paste-ready <head> markup)")
                    .on_toggle(Message::IncludeHtmlSnippetToggled)
                    .text_size(13),
            ]
            .spacing(6)
            .into(),
        ),
        // 4. PNG サイズ集合 (v1.3.0 編集 UI)
        section(
            "PNG sizes",
            "Each size becomes a separate favicon-{size}.png. 32 / 192 / 512 covers \
             tabs, PWA install, and high-DPI splash. Range: 16–1024 px.",
            size_set_editor(
                &state.export_plan.png_sizes,
                &state.png_size_input,
                Message::PngSizeRemoveRequested,
                Message::PngSizeInputChanged,
                Message::PngSizeAddRequested,
                PNG_SIZE_MIN,
                PNG_SIZE_MAX,
            ),
        ),
        // 5. ICO サイズ集合 (v1.3.0 編集 UI)
        section(
            "ICO frame sizes",
            "Sizes embedded in favicon.ico. Each frame is rendered independently \
             from the source for sharp small-size results. Range: 16–256 px (ICO format limit).",
            size_set_editor(
                &state.export_plan.ico_sizes,
                &state.ico_size_input,
                Message::IcoSizeRemoveRequested,
                Message::IcoSizeInputChanged,
                Message::IcoSizeAddRequested,
                ICO_SIZE_MIN,
                ICO_SIZE_MAX,
            ),
        ),
        // 閉じるボタン (キーボード代替経路)
        button(text("Close")).on_press(Message::AdvancedToggled),
        text(label::TOGGLE_ADVANCED_BTN).size(11),
    ]
    .spacing(18)
    .padding(20)
    .into()
}

// ---------------------------------------------------------------------------
// 共通レイアウトヘルパ: 「セクション」 = ヘッダ + 説明 + 中身
// ---------------------------------------------------------------------------

fn section<'a>(title: &'a str, blurb: &'a str, body: Element<'a, Message>) -> Element<'a, Message> {
    container(
        column![
            text(title).size(15),
            text(blurb).size(11),
            container(body).padding(Padding::default().top(4)),
        ]
        .spacing(4),
    )
    .padding(Padding::default().top(2).bottom(2))
    .into()
}

fn algorithm_row<'a>(state: &'a AppState) -> Element<'a, Message> {
    let options: Vec<ResizeAlgorithm> = ResizeAlgorithm::all().to_vec();
    let picker = pick_list(
        options,
        Some(state.export_plan.algorithm),
        Message::AlgorithmChanged,
    )
    .text_size(13);

    row![text("Algorithm:").size(13), picker]
        .spacing(10)
        .align_y(Alignment::Center)
        .into()
}

// ---------------------------------------------------------------------------
// サイズ集合エディタ (v1.3.0)
// ---------------------------------------------------------------------------
//
// 「現在の集合 (削除可能チップ群) + 追加用テキスト入力」 の構造。
// 集合は &[u32] として渡され、 削除/追加メッセージは関数引数で受ける。
//
// メッセージコンストラクタを fn(u32) -> Message / fn(String) -> Message として
// 渡すことで、 PNG / ICO 用の 2 セットを同じウィジェットで描ける。

fn size_set_editor<'a>(
    sizes: &'a [u32],
    input_value: &'a str,
    on_remove: fn(u32) -> Message,
    on_input: fn(String) -> Message,
    on_submit: Message,
    min: u32,
    max: u32,
) -> Element<'a, Message> {
    // チップ列。 1 行に詰め込みすぎると小さいウィンドウで切れるが、 PNG/ICO サイズが
    // 12 個を超えることは実用上ほぼ無いので、 単純な row でよい。
    let mut chips_row = row![].spacing(6).align_y(Alignment::Center);
    for size in sizes {
        chips_row = chips_row.push(size_chip(*size, on_remove));
    }
    if sizes.is_empty() {
        chips_row = chips_row.push(text("(empty)").size(12));
    }

    // 入力フィールド + Add ボタン
    let placeholder = format!("e.g. 64 ({min}–{max})");
    let input = text_input(&placeholder, input_value)
        .on_input(on_input)
        .on_submit(on_submit.clone())
        .size(13)
        .width(Length::Fixed(140.0));

    let add_button = button(text("Add").size(13)).on_press(on_submit);

    column![
        chips_row,
        row![input, add_button]
            .spacing(8)
            .align_y(Alignment::Center),
    ]
    .spacing(8)
    .into()
}

/// チップ 1 個: `[ 32 ×]` のような見た目を、 既存ウィジェットだけで作る。
fn size_chip<'a>(size: u32, on_remove: fn(u32) -> Message) -> Element<'a, Message> {
    let inner = row![
        text(format!("{size}")).size(12),
        button(text("×").size(12)).on_press(on_remove(size)),
    ]
    .spacing(4)
    .align_y(Alignment::Center);

    container(inner)
        .padding(Padding::default().top(2).bottom(2).left(8).right(4))
        .into()
}
