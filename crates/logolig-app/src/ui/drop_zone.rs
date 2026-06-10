//! ドロップゾーン画面 (起動直後)。
//!
//! §5.1:
//! - 設定項目を見せない
//! - シングルカラムの大きなドロップ領域
//! - キーボード操作のみで完結する代替経路 (ファイル選択ボタン)
//!
//! ## v1.10.2: スリム化 + 視覚的なドロップ領域
//!
//! v1.10.0 までは「Drop a PNG, SVG, or WebP image here, or activate to choose
//! a file」 のような長文 + 受け入れフォーマット一覧 + a11y 補助テキスト等で
//! 説明過多だった。 v1.10.2 では:
//!
//! - アイキャッチを `Drop PNG, SVG, or WebP` (1 行) にスリム化
//! - 受け入れフォーマット一覧を削除 (アイキャッチに含めて 1 行で完結)
//! - a11y 補助テキストを削除 (iced 0.14 には a11y API がないので意味がなかった)
//! - ドロップ領域を実線の角丸ボーダー + 薄い背景塗りで囲んで視覚化
//!   (iced 0.14 は dashed/dotted ボーダーを直接サポートしないため、 実線で代替)
//! - Choose file… ボタンとアイキャッチだけを領域内に配置

use iced::widget::{button, column, container, text};
use iced::{Background, Border, Color, Element, Length, Theme};

use logolig_core::MessageKey;

use crate::app::{AppState, Message};

/// ドロップ領域内のアイキャッチ色 (主役)。
const HEADLINE_COLOR: Color = Color::from_rgb(0.3, 0.3, 0.3);

pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;

    // ドロップ領域内のコンテンツ: アイキャッチ + Choose file… ボタンのみ
    let inner = column![
        text(t.t(MessageKey::DropZoneHeadline))
            .size(22)
            .color(HEADLINE_COLOR),
        button(text(t.t(MessageKey::ChooseFileButton)).size(15))
            .padding([10, 22])
            .on_press(Message::PickFileRequested),
    ]
    .spacing(20)
    .align_x(iced::alignment::Horizontal::Center);

    // ドロップ領域の枠 (実線 + 薄い背景塗り)。 iced 0.14 では dashed border が
    // 提供されていないため、 実線の細いボーダーと柔らかい背景塗りで「ここに
    // 落とせる領域」 を視覚化する。 角丸を大きめに取って box-y にしすぎず、
    // padding を多めに取って中身を中央に浮かせる。
    let bordered = container(inner)
        .padding(48)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(Background::Color(palette.background.weak.color)),
                border: Border {
                    color: palette.background.strong.color,
                    width: 2.0,
                    radius: 12.0.into(),
                },
                ..Default::default()
            }
        });

    // ドロップ領域 ↔ ウィンドウ縁の余白を確保するために、 さらに外側に padding
    // 付き container を被せる。
    container(bordered)
        .padding(40)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
