//! 詳細設定ドロワー (§5.3)。
//!
//! 既定値は隠す。 `state.advanced_open == true` のときだけ shell が表示する。
//!
//! ## v1.10.0: 情報設計刷新
//!
//! セクションを 4 つの大グループに整理し、 ユーザの思考順 (何を出力するか →
//! どう描画するか → アプリ全体) に並べる:
//!
//! 1. **What to export** — アーティファクト種別 + サイズ群を集約
//! 2. **Extras** — 追加機能 (Web manifest / Monochrome) — 控えめなトーン
//! 3. **Rendering quality** — Resize algorithm のような描画品質設定
//! 4. **App preferences** — Language のようなアプリ全体の好み
//!
//! 既定値の PNG/ICO サイズは折りたたみ気味に表示し、 「普段触らなくてよい」
//! を視覚で伝える。 Active な選択状態はボタン背景塗り分けで強調。

use iced::widget::{button, checkbox, column, container, pick_list, row, text, text_input};
use iced::{Alignment, Color, Element, Length, Padding};

use logolig_core::{
    ExportPlan, MessageKey, ResizeAlgorithm, VtracerPreset, ICO_SIZE_MAX, ICO_SIZE_MIN,
    PNG_SIZE_MAX, PNG_SIZE_MIN,
};

use crate::app::{resolve_theme, AdvancedGroup, AppState, Message};
use crate::ui::colors;

// v1.14.0: HEADING_COLOR / MUTED_COLOR / BADGE_MUTED_BG の hardcoded 定数は
// `crate::ui::colors` の theme-aware ヘルパに移行した。 dark テーマ時に
// 「グレー文字がグレー背景に同化して読めない」 問題が解消される。

pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    let theme = resolve_theme(state);

    // ----- v1.15.0: 3 段構成 (タイトル / スクロール / フッタ固定) -----
    //
    // 課題と対応:
    // - 旧構造は 1 つの column の末尾に Reset/Close を置いていたため、 内容が
    //   多いと フッタが画面下端から押し出されてユーザが押せなかった。
    // - 改善指示書 §設定画面.1〜2 「フッタ固定」 + 「内部スクロール可能」
    //   への対応として、 column を 3 段に分割:
    //     (1) タイトル領域 (Length::Shrink) — 上に固定
    //     (2) スクロール領域 (Length::FillPortion(1)) — 余り空間を全部食う
    //         → 内部の column は scrollable! でラップされ、 内容が長くても
    //           内側だけがスクロール
    //     (3) フッタ (Length::Shrink) — 下に固定 (sticky)
    //
    //   フッタを最後に置いて Length::Shrink、 中段が FillPortion なので、
    //   中段が「余り全部」 を食う結果、 フッタは自然に画面下端に張り付く。
    //
    // - 改善指示書 §設定画面.4 「フッタ操作の意味明確化」 への対応:
    //   * Reset (左、 destructive 寄り): 枠線色を palette.danger.weak.color
    //     寄りにして「破壊的だが押せる」 を視覚化
    //   * Close (右、 補助): 中立的な secondary style
    //
    //   row 内で Space::new() で左右を分けて、 配置だけでも役割の違いが
    //   伝わるようにする。

    // (1) タイトル領域 (固定)
    let title_block = column![
        text(t.t(MessageKey::AdvancedTitle)).size(22),
        text(t.t(MessageKey::AdvancedBlurb))
            .size(12)
            .color(colors::muted_text(&theme)),
    ]
    .spacing(4);

    // (2) スクロール領域: 既存のアコーディオン群をそのまま scrollable に包む
    let groups = column![
        // ━━━ Group 1: What to export ━━━━━━━━━━━━━━━━━━━━━
        accordion_group(
            &t.t(MessageKey::GroupWhatToExport),
            state.advanced_groups.is_expanded(AdvancedGroup::WhatToExport),
            Message::AdvancedGroupToggled(AdvancedGroup::WhatToExport),
            column![
                // ファイル種別
                column![
                    checkbox(state.export_plan.include_ico)
                        .label(t.t(MessageKey::IncludeIcoLabel))
                        .on_toggle(Message::IncludeIcoToggled)
                        .text_size(13),
                    checkbox(state.export_plan.include_apple_touch)
                        .label(t.t(MessageKey::IncludeAppleTouchLabel))
                        .on_toggle(Message::IncludeAppleTouchToggled)
                        .text_size(13),
                    // SVG はチェックボックス + 配下のオプション (vectorize, preset)
                    // をインデントして従属関係を視覚化
                    svg_subsection(state),
                    checkbox(state.export_plan.include_html_snippet)
                        .label(t.t(MessageKey::IncludeHtmlSnippetLabel))
                        .on_toggle(Message::IncludeHtmlSnippetToggled)
                        .text_size(13),
                ]
                .spacing(8),
                // PNG / ICO サイズ
                size_subsection(
                    &t.t(MessageKey::SectionPngSizes),
                    &state.export_plan.png_sizes,
                    ExportPlan::default_png_sizes(),
                    &state.png_size_input,
                    Message::PngSizeRemoveRequested,
                    Message::PngSizeInputChanged,
                    Message::PngSizeAddRequested,
                    PNG_SIZE_MIN,
                    PNG_SIZE_MAX,
                    state,
                ),
                size_subsection(
                    &t.t(MessageKey::SectionIcoSizes),
                    &state.export_plan.ico_sizes,
                    ExportPlan::default_ico_sizes(),
                    &state.ico_size_input,
                    Message::IcoSizeRemoveRequested,
                    Message::IcoSizeInputChanged,
                    Message::IcoSizeAddRequested,
                    ICO_SIZE_MIN,
                    ICO_SIZE_MAX,
                    state,
                ),
            ]
            .spacing(12)
            .into(),
            colors::group_heading(&theme),
        ),

        // ━━━ Group 2: Extras (普段触らない追加機能) ━━━━━━
        accordion_group(
            &t.t(MessageKey::GroupExtras),
            state.advanced_groups.is_expanded(AdvancedGroup::Extras),
            Message::AdvancedGroupToggled(AdvancedGroup::Extras),
            column![
                // Web manifest
                subsection(
                    &t.t(MessageKey::SectionWebManifest),
                    Some(&t.t(MessageKey::SectionWebManifestBlurb)),
                    web_manifest_body(state),
                    colors::muted_text(&theme),
                ),
                // Monochrome
                subsection(
                    &t.t(MessageKey::SectionMonochrome),
                    Some(&t.t(MessageKey::SectionMonochromeBlurb)),
                    checkbox(state.export_plan.monochrome)
                        .label(t.t(MessageKey::IncludeMonochromeLabel))
                        .on_toggle(Message::IncludeMonochromeToggled)
                        .text_size(13)
                        .into(),
                    colors::muted_text(&theme),
                ),
            ]
            .spacing(12)
            .into(),
            colors::group_heading(&theme),
        ),

        // ━━━ Group 3: Rendering quality ━━━━━━━━━━━━━━━━━━
        accordion_group(
            &t.t(MessageKey::GroupRenderingQuality),
            state.advanced_groups.is_expanded(AdvancedGroup::RenderingQuality),
            Message::AdvancedGroupToggled(AdvancedGroup::RenderingQuality),
            subsection(
                &t.t(MessageKey::SectionResize),
                Some(&t.t(MessageKey::SectionResizeBlurb)),
                algorithm_row(state),
                colors::muted_text(&theme),
            ),
            colors::group_heading(&theme),
        ),

        // ━━━ Group 4 (廃止): v1.10.2 で Language はヘッダのアイコンボタンに
        //                        移動したため、 App preferences グループは空に
        //                        なり廃止。 将来別の app-wide 設定が増えたら復活。
    ]
    .spacing(14);

    // scrollable! ラッパ。 デフォルトのスクロールバーを採用 (改善指示書
    // §設定画面.2 「初期表示で全体像を把握」 のため、 hover-only ではなく
    // 常時表示にして「下にもっと項目がある」 ことを視覚化する)。
    let scrollable_groups = container(
        iced::widget::scrollable(groups)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .height(Length::FillPortion(1))
    .width(Length::Fill);

    // (3) フッタ (固定)。 左に Reset (destructive 寄り)、 右に Close (補助)。
    // Space::new() で左右を分離 — レイアウトだけで役割の違いを伝える。
    let footer = row![
        // Reset: 「初期値に戻す」 destructive 寄り。 secondary base + danger
        // 寄りの枠線で「押せるが慎重に」 を表現。
        button(text(t.t(MessageKey::ResetButton)).size(13))
            .padding([8, 16])
            .on_press(Message::ExportPlanResetRequested)
            .style(reset_button_style),
        // 中央スペース: 左右の役割差を視覚化
        iced::widget::Space::new()
            .width(Length::Fill),
        // Close: 中立的な離脱
        button(text(t.t(MessageKey::CloseButton)).size(13))
            .padding([8, 16])
            .on_press(Message::AdvancedToggled)
            .style(close_button_style),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    column![title_block, scrollable_groups, footer]
        .spacing(14)
        .padding(20)
        .height(Length::Fill)
        .into()
}

/// アコーディオン式の大グループ (v1.10.3)。
///
/// 見出しを clickable なボタンとしてレンダーし、 `expanded == true` のとき
/// 中身を続けて描画する。 折りたたみ時はボタンだけ表示。
///
/// ## 表示
///
/// ```text
/// ▼ What to export    ← expanded のとき (中身が下に展開)
/// ▶ Extras            ← collapsed のとき (中身は描画しない)
/// ```
///
/// chevron (`▼` / `▶`) は state を 2 つの方法で示す:
/// 1. 視覚的記号 (シルエット差)
/// 2. 文字方向 (下向き = 下に展開、 右向き = まだ展開していない)
///
/// 色覚に依存しないため ABDD §12 と整合。
///
/// v1.14.0: 見出し文字色は `heading_color` パラメータで受け取る (呼び出し側で
/// `colors::group_heading(&theme)` を解決して渡す)。 ボタンの hover 背景塗りは
/// closure 内で `&Theme` を直接参照する。
fn accordion_group<'a>(
    label: &str,
    expanded: bool,
    on_toggle: Message,
    body: Element<'a, Message>,
    heading_color: Color,
) -> Element<'a, Message> {
    // chevron + 見出しを横並びにしたボタン。 ボタン全体がクリック領域になり、
    // ヒットエリアを広く取れる。
    let chevron = if expanded { "▼" } else { "▶" };
    let heading_row = row![
        text(chevron).size(11).color(heading_color),
        text(label.to_string()).size(13).color(heading_color),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let header_button = button(heading_row)
        .padding(Padding::default().top(8).bottom(8).left(0).right(0))
        .width(Length::Fill)
        .on_press(on_toggle)
        // ボタンの背景は透明、 hover 時だけうっすら塗る (見出しが「ボタン」 と
        // 主張しすぎないように、 でもクリック可能であることは伝える)。
        // text_color はテーマ palette の標準テキスト色 (background.base.text)
        // を採用 — 見出しテキスト自体の色は上の `heading_row` で既に明示的に
        // 設定済みのため、 button style 側はテーマに任せる。
        .style(|theme: &iced::Theme, status| {
            let palette = theme.extended_palette();
            let bg = match status {
                iced::widget::button::Status::Hovered => palette.background.weak.color,
                _ => Color::TRANSPARENT,
            };
            iced::widget::button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: palette.background.base.text,
                border: iced::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }
        });

    if expanded {
        // 展開時: ボタン + 本体を縦に並べて返す。 本体は左に少し余白を取って
        // 「見出しの中身」 であることを示す。
        column![
            header_button,
            container(body).padding(Padding::default().left(16).top(4).bottom(4)),
        ]
        .spacing(2)
        .into()
    } else {
        // 折りたたみ時: ヘッダーボタンのみ。
        header_button.into()
    }
}

/// SVG セクション: チェックボックス + 配下のオプションをインデント表示。
fn svg_subsection<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    let main_toggle = checkbox(state.export_plan.include_svg)
        .label(t.t(MessageKey::IncludeSvgLabel))
        .on_toggle(Message::IncludeSvgToggled)
        .text_size(13);

    // include_svg が off のときは配下のオプションも見せる必要なし
    if !state.export_plan.include_svg {
        return main_toggle.into();
    }

    // 配下: vectorize toggle と preset pick_list。 段下げで従属関係を示す。
    let nested = column![
        checkbox(state.export_plan.vectorize_on_raster)
            .label(t.t(MessageKey::VectorizeOnRasterLabel))
            .on_toggle(Message::VectorizeOnRasterToggled)
            .text_size(13),
        vtracer_preset_row(state),
    ]
    .spacing(6);

    // インデント表現: 左 padding + 上下少しの padding
    column![
        main_toggle,
        container(nested).padding(Padding::default().left(20).top(4)),
    ]
    .spacing(4)
    .into()
}

// ---------------------------------------------------------------------------
// 共通レイアウトヘルパ
// ---------------------------------------------------------------------------

/// サブセクション (グループの中の小単位)。 タイトル + 任意の blurb + 中身。
/// 旧 `section` ヘルパの後継だが、 v1.10 でグループ階層が増えた都合で名前を
/// 変えた。
///
/// v1.14.0: blurb の色を呼び出し側で解決した `muted_color` で受け取る。
fn subsection<'a>(
    title: &str,
    blurb: Option<&str>,
    body: Element<'a, Message>,
    muted_color: Color,
) -> Element<'a, Message> {
    let mut col = column![
        text(title.to_string()).size(13),
    ]
    .spacing(2);
    if let Some(b) = blurb {
        col = col.push(text(b.to_string()).size(11).color(muted_color));
    }
    col = col.push(container(body).padding(Padding::default().top(4)));
    container(col).padding(Padding::default().top(2).bottom(2)).into()
}

/// PNG / ICO サイズ用サブセクション。 既定値のままなら控えめ表示
/// (チップ群の代わりに「at defaults: 32 / 192 / 512」 のような 1 行サマリ +
/// 小さい "Edit" ボタン)。 ユーザが変更を加えていたら従来通りの編集 UI を
/// 全展開して見せる。
#[allow(clippy::too_many_arguments)]
fn size_subsection<'a>(
    title: &str,
    sizes: &'a [u32],
    defaults: &[u32],
    input_value: &'a str,
    on_remove: fn(u32) -> Message,
    on_input: fn(String) -> Message,
    on_submit: Message,
    min: u32,
    max: u32,
    state: &'a AppState,
) -> Element<'a, Message> {
    let at_defaults = sizes == defaults;
    if at_defaults && input_value.is_empty() {
        // 既定値表示: 折りたたみ気味の 1 行
        let summary: String = sizes
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(" / ");
        // v1.14.0: theme は呼び出し時に解決して、 「at defaults」 テキストの
        // 色とバッジ背景色のどちらにも反映する。 background_color を closure
        // 内でも `&Theme` ベースに置く。
        let theme = resolve_theme(state);
        let body = container(
            text(format!("at defaults: {summary}"))
                .size(12)
                .color(colors::muted_text(&theme)),
        )
        .padding(Padding::default().top(2).bottom(2).left(8).right(8))
        .style(|theme: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(colors::badge_muted_bg(theme))),
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });
        // タイトル + 既定値バッジ + 隣接の編集呼び出し用入力欄 (展開のきっかけ)
        // 入力欄を空のまま見せておくことで、 そこに数値を打てば自然と編集モードに
        // 移行する (折りたたみを開く専用ボタンを設けない)。
        let placeholder = state.translator.t_args(
            MessageKey::SizeInputPlaceholder,
            &[("min", &min.to_string()), ("max", &max.to_string())],
        );
        let nudge = text_input(&placeholder, input_value)
            .on_input(on_input)
            .on_submit(on_submit)
            .size(12)
            .width(Length::Fixed(140.0));
        column![
            text(title.to_string()).size(13),
            row![body, nudge,]
                .spacing(8)
                .align_y(Alignment::Center),
        ]
        .spacing(4)
        .into()
    } else {
        // 編集モード: 従来の chip + 入力欄を展開
        column![
            text(title.to_string()).size(13),
            size_set_editor(
                state,
                sizes,
                input_value,
                on_remove,
                on_input,
                on_submit,
                min,
                max,
            ),
        ]
        .spacing(4)
        .into()
    }
}

/// v1.8.0: Web manifest セクションの中身。
///
/// トグル off の時はラベルのみ。 on の時は 4 つのテキスト入力 (name /
/// short_name / theme_color / background_color) を縦に並べる。
///
/// 設計判断:
/// - 入力中の検証はしない (`#FF` まで打って警告を出す UX を避ける)
/// - 検証は export 直前 (`Message::ExportRequested`) でまとめて行う
/// - 永続化はキー入力ごと (`persist_settings`) — 既存の挙動 (size 入力など)
///   と一貫させる
fn web_manifest_body<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    let toggle = checkbox(state.export_plan.web_manifest.is_some())
        .label(t.t(MessageKey::IncludeWebManifestLabel))
        .on_toggle(Message::IncludeWebManifestToggled)
        .text_size(13);

    let Some(manifest) = state.export_plan.web_manifest.as_ref() else {
        // off 状態では toggle のみ表示。 入力フィールドを最初から見せると
        // 「これは何?」 と認知負荷が増えるため、 on の時だけ展開する。
        return column![toggle].spacing(8).into();
    };

    // on 状態: 4 つのフィールドを縦に並べる。
    // 各フィールドは「ラベル + text_input」 の row。 ラベルは固定幅で揃えると
    // 美しいが、 v1.8 では labeled_input ヘルパで row 内縦揃えを統一する。
    column![
        toggle,
        labeled_input(
            &t.t(MessageKey::WebManifestNameLabel),
            &manifest.name,
            &t.t(MessageKey::WebManifestNamePlaceholder),
            Message::WebManifestNameChanged,
        ),
        labeled_input(
            &t.t(MessageKey::WebManifestShortNameLabel),
            &manifest.short_name,
            &t.t(MessageKey::WebManifestShortNamePlaceholder),
            Message::WebManifestShortNameChanged,
        ),
        labeled_input(
            &t.t(MessageKey::WebManifestThemeColorLabel),
            &manifest.theme_color,
            "#RRGGBB",
            Message::WebManifestThemeColorChanged,
        ),
        labeled_input(
            &t.t(MessageKey::WebManifestBackgroundColorLabel),
            &manifest.background_color,
            "#RRGGBB",
            Message::WebManifestBackgroundColorChanged,
        ),
    ]
    .spacing(8)
    .into()
}

/// 「ラベル + text_input」 の 1 行 row。 v1.8 のフォーム入力で繰り返し使うので
/// ヘルパ化。 ラベル幅は固定 (140px) でテキスト入力幅と揃える。
fn labeled_input<'a>(
    label: &str,
    value: &str,
    placeholder: &str,
    on_change: fn(String) -> Message,
) -> Element<'a, Message> {
    row![
        text(label.to_string())
            .size(13)
            .width(Length::Fixed(140.0)),
        text_input(placeholder, value)
            .on_input(on_change)
            .size(13)
            .width(Length::Fixed(220.0)),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

// ---------------------------------------------------------------------------
// 共通レイアウトヘルパ
// ---------------------------------------------------------------------------

fn algorithm_row<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    // 翻訳用のラッパ。 ResizeAlgorithm に直接 Display を実装したくない (core が
    // i18n に依存しないため) ので、 UI 層でラップする。
    let options: Vec<LocalizedAlgorithm> = ResizeAlgorithm::all()
        .iter()
        .map(|a| LocalizedAlgorithm {
            value: *a,
            label: state.translator.t(algorithm_message_key(*a)),
        })
        .collect();
    let selected = options
        .iter()
        .find(|opt| opt.value == state.export_plan.algorithm)
        .cloned();
    let picker = pick_list(options, selected, |opt: LocalizedAlgorithm| {
        Message::AlgorithmChanged(opt.value)
    })
    .text_size(13);

    row![text(t.t(MessageKey::AlgorithmLabel)).size(13), picker]
        .spacing(10)
        .align_y(Alignment::Center)
        .into()
}

fn vtracer_preset_row<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    let options: Vec<LocalizedPreset> = VtracerPreset::all()
        .iter()
        .map(|p| LocalizedPreset {
            value: *p,
            label: state.translator.t(preset_message_key(*p)),
        })
        .collect();
    let selected = options
        .iter()
        .find(|opt| opt.value == state.export_plan.vtracer_preset)
        .cloned();
    let picker = pick_list(options, selected, |opt: LocalizedPreset| {
        Message::VtracerPresetChanged(opt.value)
    })
    .text_size(13);

    row![text(t.t(MessageKey::PresetLabel)).size(13), picker]
        .spacing(10)
        .align_y(Alignment::Center)
        .into()
}

fn size_set_editor<'a>(
    state: &'a AppState,
    sizes: &'a [u32],
    input_value: &'a str,
    on_remove: fn(u32) -> Message,
    on_input: fn(String) -> Message,
    on_submit: Message,
    min: u32,
    max: u32,
) -> Element<'a, Message> {
    let t = &state.translator;
    let mut chips_row = row![].spacing(6).align_y(Alignment::Center);
    for size in sizes {
        chips_row = chips_row.push(size_chip(state, *size, on_remove));
    }
    if sizes.is_empty() {
        chips_row = chips_row.push(text(t.t(MessageKey::EmptySetLabel)).size(12));
    }

    let placeholder = t.t_args(
        MessageKey::SizeInputPlaceholder,
        &[("min", &min.to_string()), ("max", &max.to_string())],
    );
    let input = text_input(&placeholder, input_value)
        .on_input(on_input)
        .on_submit(on_submit.clone())
        .size(13)
        .width(Length::Fixed(160.0));

    let add_button = button(text(t.t(MessageKey::SizeAddButton)).size(13)).on_press(on_submit);

    column![
        chips_row,
        row![input, add_button]
            .spacing(8)
            .align_y(Alignment::Center),
    ]
    .spacing(8)
    .into()
}

fn size_chip<'a>(
    state: &'a AppState,
    size: u32,
    on_remove: fn(u32) -> Message,
) -> Element<'a, Message> {
    let inner = row![
        text(format!("{size}")).size(12),
        button(text(state.translator.t(MessageKey::SizeChipRemove)).size(12))
            .on_press(on_remove(size)),
    ]
    .spacing(4)
    .align_y(Alignment::Center);

    container(inner)
        .padding(Padding::default().top(2).bottom(2).left(8).right(4))
        .into()
}

// ---------------------------------------------------------------------------
// pick_list 用の「翻訳済みラベル + 値」 ラッパ
//
// pick_list は T: Display + Clone + Eq を要求する。 ResizeAlgorithm 等に
// Display を直接実装すると core が翻訳責任を持つことになる(言語が固定される)。
// 代わりに UI 層でラッパを作り、 翻訳済みのラベルを Display で返す。
// 値は元の enum をそのまま保持して message に乗せる。
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocalizedAlgorithm {
    value: ResizeAlgorithm,
    label: String,
}

impl std::fmt::Display for LocalizedAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocalizedPreset {
    value: VtracerPreset,
    label: String,
}

impl std::fmt::Display for LocalizedPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

// ---------------------------------------------------------------------------
// enum → MessageKey マップ (UI 層に置く: core を i18n から独立させるため)
// ---------------------------------------------------------------------------

fn algorithm_message_key(alg: ResizeAlgorithm) -> MessageKey {
    match alg {
        ResizeAlgorithm::Lanczos3 => MessageKey::AlgorithmLanczos3,
        ResizeAlgorithm::MitchellNetravali => MessageKey::AlgorithmMitchellNetravali,
        ResizeAlgorithm::CatmullRom => MessageKey::AlgorithmCatmullRom,
        ResizeAlgorithm::Bilinear => MessageKey::AlgorithmBilinear,
        ResizeAlgorithm::Nearest => MessageKey::AlgorithmNearest,
    }
}

fn preset_message_key(preset: VtracerPreset) -> MessageKey {
    match preset {
        VtracerPreset::Sharp => MessageKey::VtracerPresetSharp,
        VtracerPreset::Default => MessageKey::VtracerPresetDefault,
        VtracerPreset::PhotoRich => MessageKey::VtracerPresetPhotoRich,
    }
}

// ---------------------------------------------------------------------------
// v1.15.0: フッタ用ボタンスタイル
// ---------------------------------------------------------------------------

/// Reset ボタン (左、 destructive 寄り)。
///
/// 「初期値に戻す」 = 入力データの破壊なので、 通常の secondary より一段強い
/// 注意喚起が必要。 ただし「ファイル削除」 のような irreversible 行為では
/// ないため、 完全な danger スタイル (palette.danger.base 塗り) は重すぎる。
///
/// 落とし所として:
/// - 透明背景 + danger.weak.color の枠線で「警告寄り」 を示す
/// - text_color は通常テキスト相当 (palette.background.base.text) で読める強さ
/// - hover 時は danger.weak.color を薄く塗って「押下対象」 を強調
fn reset_button_style(
    theme: &iced::Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let palette = theme.extended_palette();
    let bg = match status {
        iced::widget::button::Status::Hovered => Color {
            a: 0.15,
            ..palette.danger.weak.color
        },
        _ => Color::TRANSPARENT,
    };
    iced::widget::button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: palette.background.base.text,
        border: iced::Border {
            color: palette.danger.weak.color,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

/// Close ボタン (右、 補助)。
///
/// 中立的な離脱動作。 controlled muted style — preview_panel.rs の
/// `secondary_button_style` と視覚的に揃える (戻る/再選択ボタンと同じ
/// 強度で「補助操作」 と認識される)。
fn close_button_style(
    theme: &iced::Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let palette = theme.extended_palette();
    let bg = match status {
        iced::widget::button::Status::Hovered => palette.background.weak.color,
        _ => Color::TRANSPARENT,
    };
    iced::widget::button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: palette.background.weak.text,
        border: iced::Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}
