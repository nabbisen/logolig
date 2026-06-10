//! v1.16.0 — Result 画面 (旧 ExportReady を置き換え)。
//!
//! ## 構成 (PNG モック ③ 準拠)
//!
//! ```text
//!     ┌──────────────────────────────────────────────────┐
//!     │ ✓ 変換が完了しました!                            │  見出し
//!     │ 生成されたアセット一覧                            │  サブヘッド
//!     │ ┌─────────┐ ┌─────────┐ ┌─────────┐              │
//!     │ │favicon. │ │icon-16. │ │icon-32. │              │  ↕ アセット
//!     │ │  ico    │ │  png    │ │  png    │              │  ↕ カード
//!     │ │ [thumb] │ │ [thumb] │ │ [thumb] │              │  ↕ グリッド
//!     │ │ ICO 46K │ │16x16 .6K│ │32x32 1K │              │  ↕ (3 列)
//!     │ │  [↓]    │ │  [↓]    │ │  [↓]    │              │  ↕
//!     │ └─────────┘ └─────────┘ └─────────┘              │
//!     │ ...                                              │
//!     │ ┌──────────────────────────────────────────────┐ │
//!     │ │  ↓ すべてダウンロード (ZIP)                  │ │  ZIP 一括 DL
//!     │ └──────────────────────────────────────────────┘ │
//!     │ ▸ プレビューを見る                                │  折りたたみ (任意)
//!     │ [← Back]                                         │  戻る
//!     └──────────────────────────────────────────────────┘
//! ```

use iced::widget::{button, column, container, image, row, text, Space};
use iced::{Background, Border, Color, Element, Length, Padding, Theme};

use logolig_core::{MessageKey, Rgba8};

use crate::app::{resolve_theme, AppState, Message};
use crate::result::{ResultAssetItem, ResultAssetKind, ResultAssets};
use crate::ui::colors;

/// Result 画面のエントリポイント。
pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    let theme = resolve_theme(state);

    // result_assets が None の場合 (= 想定外、 phase A の旧フローで来た場合
    // など) は preview_panel にフォールバックして「壊れない」 ようにする。
    // 通常運用では IngestCompleted → Converting → ConvertCompleted で必ず
    // Some になっているので、 ここに来ない。
    let Some(assets) = state.result_assets.as_ref() else {
        return crate::ui::preview_panel::view(state);
    };

    // 1. 見出し: ✓ 変換が完了しました!
    let headline = container(
        row![
            // 成功マーカ。 ABDD §12 (色覚に依存しない) に従って ✓ 文字で
            // 「成功」 を表現。 サイズで主役感を出す。
            text("✓").size(28).color(colors::file_name(&theme)),
            text(t.t(MessageKey::ResultSuccessHeadline))
                .size(22)
                .color(colors::page_title(&theme)),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
    )
    .center_x(Length::Fill)
    .padding(Padding::default().top(8).bottom(4));

    // 2. サブヘッド: 生成されたアセット一覧
    let subheading = text(t.t(MessageKey::ResultAssetsSubheading))
        .size(13)
        .color(colors::muted_text(&theme));

    // 3. アセットカードグリッド (3 列)
    let grid = build_grid(assets, &theme);

    // 4. ZIP 一括 DL ボタン
    let download_all = container(
        button(
            row![
                text("↓").size(16),
                text(t.t(MessageKey::ResultDownloadAllButton)).size(15),
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center),
        )
        .padding([12, 28])
        .on_press(Message::DownloadAllRequested),
    )
    .center_x(Length::Fill)
    .padding(Padding::default().top(16).bottom(8));

    // 5. プレビュー折りたたみセクション (Q1 b) — 「任意に開く」
    // ここでは toggle ボタンとして見せる。 開けば旧 preview_panel の
    // View as / Surface ピッカー + プレビュー枠が下に展開される。
    let preview_toggle_label = format!(
        "{} {}",
        if state.result_preview_open { "▼" } else { "▶" },
        t.t(MessageKey::ResultPreviewToggle)
    );
    let preview_toggle = button(text(preview_toggle_label).size(13))
        .padding([6, 12])
        .on_press(Message::ResultPreviewToggled)
        .style(toggle_button_style);

    // 6. プレビューパネル (展開時のみ)
    let preview_panel: Element<'a, Message> = if state.result_preview_open {
        // 旧 preview_panel をそのまま埋め込む。 見出し / アクション行は本画面の
        // ものを使うため、 折りたたみ要素として preview_panel の view() を
        // そのまま呼ぶと「画面タイトル」 が二重表示になる。 ただし phase B では
        // 旧コードを温存しつつ最小コストで載せるため、 今は preview_panel::view
        // をそのまま埋め込む形にしておく (後の v1.x で「プレビューだけ」 を
        // 切り出すリファクタを検討)。
        crate::ui::preview_panel::view(state)
    } else {
        Space::new().height(Length::Shrink).into()
    };

    // 7. アクション行: 左に Back のみ (Re-select はファイル投入で自動的に
    //    再変換されるため、 v1.16 では明示的なボタンを廃止)
    let action_row = row![
        button(text(t.t(MessageKey::EditCancelButton)).size(13))
            .padding([8, 16])
            .on_press(Message::EditCancelled)
            .style(secondary_button_style),
        Space::new().width(Length::Fill),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // 全体構成。 アセット数が多いと縦に伸びるので、 全体を scrollable で
    // ラップ。
    let content = column![
        headline,
        container(subheading).center_x(Length::Fill),
        grid,
        download_all,
        container(preview_toggle).center_x(Length::Fill),
        preview_panel,
        action_row,
    ]
    .spacing(12)
    .padding(Padding::default().left(8).right(8).top(4).bottom(8));

    iced::widget::scrollable(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// アセットカードグリッド (3 列) を組む。
fn build_grid<'a>(assets: &'a ResultAssets, theme: &Theme) -> Element<'a, Message> {
    let mut col = column![].spacing(12);
    let mut cur_row = row![].spacing(12);
    let mut count_in_row = 0;
    for (idx, item) in assets.items.iter().enumerate() {
        cur_row = cur_row.push(asset_card(idx, item, theme));
        count_in_row += 1;
        if count_in_row == 3 {
            col = col.push(cur_row);
            cur_row = row![].spacing(12);
            count_in_row = 0;
        }
    }
    // 半端な末尾行があれば追加 (Space で右側を埋めて幅を整える)
    if count_in_row > 0 {
        for _ in count_in_row..3 {
            cur_row = cur_row.push(Space::new().width(Length::FillPortion(1)));
        }
        col = col.push(cur_row);
    }
    col.into()
}

/// 1 枚のアセットカード。
fn asset_card<'a>(
    idx: usize,
    item: &'a ResultAssetItem,
    theme: &Theme,
) -> Element<'a, Message> {
    // ファイル名 (短く)
    let file_name_text = text(item.file_name.clone())
        .size(13)
        .color(colors::file_name(theme));

    // サムネ領域: 画像系なら decode 済みラスタ、 テキスト系ならアイコン
    let thumb: Element<'a, Message> = match &item.thumbnail {
        Some(rgba) => raster_thumbnail(rgba),
        None => placeholder_thumbnail(item.kind, theme),
    };

    // メタ情報行: バッジ (ICO / PNG / SVG / HTML / JSON) + 寸法 (画像のみ) + サイズ
    let badge = text(item.kind.badge_label())
        .size(11)
        .color(colors::section_label(theme));
    let size_label = text(item.size_display())
        .size(11)
        .color(colors::muted_text(theme));
    let dim_text: Element<'a, Message> = if let Some(d) = item.dimensions_display() {
        text(d).size(11).color(colors::muted_text(theme)).into()
    } else {
        Space::new().width(Length::Shrink).into()
    };
    let meta_row = row![badge, dim_text, Space::new().width(Length::Fill), size_label]
        .spacing(8)
        .align_y(iced::Alignment::Center);

    // DL ボタン (↓ アイコンのみ、 a11y label は tooltip 経由で揃える)
    let dl_button = button(text("↓ ").size(14))
        .padding([4, 12])
        .on_press(Message::DownloadOneRequested(idx))
        .style(download_button_style);

    let inner = column![file_name_text, thumb, meta_row, container(dl_button).center_x(Length::Fill)]
        .spacing(8);

    container(inner)
        .padding(12)
        .width(Length::FillPortion(1))
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(Background::Color(palette.background.weak.color)),
                border: Border {
                    color: palette.background.strong.color,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

/// 画像サムネ。 サイズは `Length::Fill` で枠内に収める。
fn raster_thumbnail<'a>(rgba: &'a Rgba8) -> Element<'a, Message> {
    // iced::widget::image::Handle::from_rgba は (width, height, Vec<u8>) を取る
    let handle = image::Handle::from_rgba(rgba.width, rgba.height, rgba.pixels.to_vec());
    container(
        image(handle)
            .width(Length::Fill)
            .height(Length::Fixed(80.0))
            .content_fit(iced::ContentFit::Contain),
    )
    .center_x(Length::Fill)
    .center_y(Length::Fixed(80.0))
    .into()
}

/// テキスト系アセット (HTML snippet / Web manifest / SVG) のサムネプレースホルダ。
/// 文書アイコンを大きく表示。
fn placeholder_thumbnail<'a>(
    kind: ResultAssetKind,
    theme: &Theme,
) -> Element<'a, Message> {
    let glyph = match kind {
        ResultAssetKind::HtmlSnippet => "{}",
        ResultAssetKind::WebManifest => "{ }",
        ResultAssetKind::Svg => "<>",
        _ => "•",
    };
    container(text(glyph).size(28).color(colors::muted_text(theme)))
        .center_x(Length::Fill)
        .center_y(Length::Fixed(80.0))
        .into()
}

/// 折りたたみトグルボタンのスタイル (透明 + hover で薄塗り)。
fn toggle_button_style(
    theme: &Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let palette = theme.extended_palette();
    let bg = match status {
        iced::widget::button::Status::Hovered => palette.background.weak.color,
        _ => Color::TRANSPARENT,
    };
    iced::widget::button::Style {
        background: Some(Background::Color(bg)),
        text_color: palette.background.weak.text,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

/// 個別 DL ボタンのスタイル (補助的、 強調しすぎない)。
fn download_button_style(
    theme: &Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let palette = theme.extended_palette();
    let bg = match status {
        iced::widget::button::Status::Hovered => palette.background.weak.color,
        _ => Color::TRANSPARENT,
    };
    iced::widget::button::Style {
        background: Some(Background::Color(bg)),
        text_color: palette.background.weak.text,
        border: Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

/// 「← Back」 用の補助ボタンスタイル (preview_panel の secondary と揃える)。
fn secondary_button_style(
    theme: &Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let palette = theme.extended_palette();
    let bg = match status {
        iced::widget::button::Status::Hovered => palette.background.weak.color,
        _ => Color::TRANSPARENT,
    };
    iced::widget::button::Style {
        background: Some(Background::Color(bg)),
        text_color: palette.background.weak.text,
        border: Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}
