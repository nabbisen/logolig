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

use logolig_core::{MessageKey, ResizeAlgorithm, VtracerPreset};

use crate::app::{resolve_theme, AppState, Message};
use crate::ui::colors;

// v1.14.0: HEADING_COLOR / MUTED_COLOR / BADGE_MUTED_BG の hardcoded 定数は
// `crate::ui::colors` の theme-aware ヘルパに移行した。 dark テーマ時に
// 「グレー文字がグレー背景に同化して読めない」 問題が解消される。

pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    let theme = resolve_theme(state);

    // ----- v1.17.0: Right Sheet 化 + flat 再編 -----
    //
    // PNG モック (新外部設計) に準拠した構成:
    //  1. タイトル領域 (「設定」 + 右上の ✕)
    //  2. 出力サイズ (PNG): チェックボックス縦並び + 「+ カスタムサイズ追加」
    //  3. SVG 変換方式: 離散スライダー 3 段 (Sharp / Default / PhotoRich)
    //  4. その他: 透過 (アルファ) を維持する トグル
    //  5. ▶ 上級設定 (折りたたみ)
    //  6. フッタ: ↻ Reset (左寄せ)
    //
    // 旧 v1.15 のアコーディオン 3 段構成 (`accordion_group` × 3) は廃止し、
    // flat な縦並びセクションに整理する。 上級設定だけは「普段は触らないが
    // 機能としては残す」 ものを集約する隠し場所として 1 つだけ折りたたみを
    // 残す (chevron `▶` / `▼`)。
    //
    // ICO 生成セクションは v1.17.0 で意図的に削除 (Q4 で確定)。 favicon.ico
    // は内部デフォルト固定 (16/24/32/48 マルチサイズ) で常時 ON、 ユーザは
    // 触らない。 旧 ICO sizes 編集 UI も完全削除。

    // ============================================================
    // (1) タイトル領域: 「設定」 + 右上の ✕
    // ============================================================
    let title_row = row![
        text(t.t(MessageKey::SettingsTitle))
            .size(20)
            .color(colors::page_title(&theme)),
        iced::widget::Space::new().width(Length::Fill),
        button(text("✕").size(16))
            .padding([4, 10])
            .on_press(Message::AdvancedToggled)
            .style(title_close_button_style),
    ]
    .align_y(Alignment::Center);

    // ============================================================
    // (2) 出力サイズ (PNG): 6 個の固定チェックボックス + カスタム追加
    // ============================================================
    //
    // PNG モック準拠の固定 6 サイズ (16/32/48/96/192/512)。 旧 logolig の
    // `default_png_sizes()` (16/32/48/180/192/256/512) とは差分があり、
    // 96 が追加・180 と 256 が消えた。 180 (apple-touch) は上級設定の
    // include_apple_touch トグルで制御するためここからは外す。 256 はカス
    // タムサイズ追加で対応可能。
    //
    // ユーザの export_plan.png_sizes に対して各固定サイズの ON/OFF を
    // 反映する: チェックを入れたら追加、 外したら削除。 内部実装は既存の
    // PngSizeAddRequested / PngSizeRemoveRequested を流用。
    let preset_sizes: [u32; 6] = [16, 32, 48, 96, 192, 512];
    let mut sizes_col = column![
        text(t.t(MessageKey::SectionPngOutputSizes))
            .size(14)
            .color(colors::section_label(&theme)),
    ]
    .spacing(8);
    for px in preset_sizes {
        let checked = state.export_plan.png_sizes.contains(&px);
        let label = format!("{} × {}", px, px);
        sizes_col = sizes_col.push(
            checkbox(checked)
                .label(label)
                .on_toggle(move |on| {
                    if on {
                        Message::PngPresetSizeAdded(px)
                    } else {
                        Message::PngSizeRemoveRequested(px)
                    }
                })
                .text_size(13),
        );
    }

    // 固定リスト以外で既に追加されているサイズを「カスタムサイズ」 として
    // 並べる (削除可能)。 例: ユーザが過去に 256 を追加していたらここに
    // チェック済み状態で表示される。
    let mut custom_present: Vec<u32> = state
        .export_plan
        .png_sizes
        .iter()
        .copied()
        .filter(|p| !preset_sizes.contains(p))
        .collect();
    custom_present.sort();
    for px in &custom_present {
        let label = format!("{} × {}", px, px);
        let removed = *px;
        sizes_col = sizes_col.push(
            checkbox(true)
                .label(label)
                .on_toggle(move |_| Message::PngSizeRemoveRequested(removed))
                .text_size(13),
        );
    }

    // 「+ カスタムサイズ追加」 入力欄。 既存の text_input + Add ボタンを
    // 簡素化して 1 行に並べる。
    let custom_input = row![
        text_input("e.g. 256", &state.png_size_input)
            .on_input(Message::PngSizeInputChanged)
            .on_submit(Message::PngSizeAddRequested)
            .size(12)
            .padding(4)
            .width(Length::Fixed(80.0)),
        button(text(t.t(MessageKey::AddCustomSize)).size(12))
            .padding([4, 10])
            .on_press(Message::PngSizeAddRequested)
            .style(secondary_drawer_button_style),
    ]
    .spacing(6)
    .align_y(Alignment::Center);
    sizes_col = sizes_col.push(custom_input);

    // ============================================================
    // (3) SVG 変換方式: 離散スライダー 3 段
    // ============================================================
    //
    // 旧 vtracer プリセット (Sharp / Default / PhotoRich) を「シンプル ↔
    // 詳細」 という直感軸でスライダーに対応させる。 iced 0.14 の slider は
    // 連続値だが、 step を整数で扱うことで実質的に離散選択にできる:
    //   0 → Sharp (シンプル)
    //   1 → Default (中庸)
    //   2 → PhotoRich (詳細)
    //
    // PNG モックには「ラスタからベクトル化」 トグルや「favicon.svg 出力
    // ON/OFF」 は出ていないため、 これらは上級設定に隠す (= ここはあくまで
    // 「方式の精度」 を選ぶ UI)。 ベクトル化は旧 vectorize_on_raster の値
    // をそのまま使う (デフォルト false)。
    let preset_idx: i32 = match state.export_plan.vtracer_preset {
        VtracerPreset::Sharp => 0,
        VtracerPreset::Default => 1,
        VtracerPreset::PhotoRich => 2,
    };
    let svg_section = column![
        text(t.t(MessageKey::SectionSvgConversion))
            .size(14)
            .color(colors::section_label(&theme)),
        // スライダー下のラベル行 (シンプル / 詳細)
        row![
            text(t.t(MessageKey::SvgConversionSimple))
                .size(11)
                .color(colors::muted_text(&theme)),
            iced::widget::Space::new().width(Length::Fill),
            text(t.t(MessageKey::SvgConversionDetailed))
                .size(11)
                .color(colors::muted_text(&theme)),
        ],
        iced::widget::slider(0..=2, preset_idx, |v| {
            let preset = match v {
                0 => VtracerPreset::Sharp,
                1 => VtracerPreset::Default,
                _ => VtracerPreset::PhotoRich,
            };
            Message::VtracerPresetChanged(preset)
        })
        .step(1i32),
    ]
    .spacing(6);

    // ============================================================
    // (4) その他: 透過 (アルファ) を維持する トグル
    // ============================================================
    //
    // v1.21.0 で本実装。 旧 v1.17.0 では UI のみ用意して常時 ON 固定の
    // placeholder だったが、 v1.21.0 で `ExportPlan::keep_transparency` を
    // 追加し、 OFF 時は raster 出力 (PNG / ICO / mono PNG / mono ICO) を
    // 白背景でフラット化する処理が `services::flatten` で動く。 SVG は
    // 影響を受けない (Q2-a)。 永続化対象 (Q4-a)。
    let misc_section = column![
        text(t.t(MessageKey::SectionMisc))
            .size(14)
            .color(colors::section_label(&theme)),
        checkbox(state.export_plan.keep_transparency)
            .label(t.t(MessageKey::KeepTransparency))
            .on_toggle(Message::KeepTransparencyToggled)
            .text_size(13),
    ]
    .spacing(8);

    // ============================================================
    // (5) ▶ 上級設定 (折りたたみ): 旧設定の集約場所
    // ============================================================
    //
    // PNG モックには無いが機能としては残しておきたい設定をここに集約:
    // - Apple touch icon (180×180 PNG)
    // - HTML snippet 出力
    // - Web manifest 出力 + 各種フィールド
    // - Monochrome (BT.709 グレースケール)
    // - リサイズアルゴリズム
    // - ラスタからベクトル化 (vectorize_on_raster)
    let extras_chevron = if state.advanced_extras_open { "▼" } else { "▶" };
    let extras_header = button(
        row![
            text(extras_chevron)
                .size(11)
                .color(colors::group_heading(&theme)),
            text(t.t(MessageKey::AdvancedExtrasSection))
                .size(13)
                .color(colors::group_heading(&theme)),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding(Padding::default().top(8).bottom(8))
    .width(Length::Fill)
    .on_press(Message::AdvancedExtrasToggled)
    .style(extras_header_style);

    let extras_body: Element<'a, Message> = if state.advanced_extras_open {
        column![
            // Apple touch icon
            checkbox(state.export_plan.include_apple_touch)
                .label(t.t(MessageKey::IncludeAppleTouchLabel))
                .on_toggle(Message::IncludeAppleTouchToggled)
                .text_size(13),
            // HTML snippet
            checkbox(state.export_plan.include_html_snippet)
                .label(t.t(MessageKey::IncludeHtmlSnippetLabel))
                .on_toggle(Message::IncludeHtmlSnippetToggled)
                .text_size(13),
            // SVG 出力 + ラスタからベクトル化 (1 まとまり)
            svg_subsection(state),
            // Web manifest (子フィールド込み)
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
            // リサイズアルゴリズム
            subsection(
                &t.t(MessageKey::SectionResize),
                Some(&t.t(MessageKey::SectionResizeBlurb)),
                algorithm_row(state),
                colors::muted_text(&theme),
            ),
        ]
        .spacing(12)
        .into()
    } else {
        iced::widget::Space::new().height(Length::Shrink).into()
    };

    let extras_section = column![extras_header, extras_body].spacing(2);

    // ============================================================
    // 中段全体: スクロール可能領域
    // ============================================================
    let scroll_content = column![
        sizes_col,
        svg_section,
        misc_section,
        extras_section,
    ]
    .spacing(20);

    let scrollable_body = container(
        iced::widget::scrollable(scroll_content)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .height(Length::FillPortion(1))
    .width(Length::Fill);

    // ============================================================
    // (6) フッタ: ↻ Reset のみ (Close は右上 ✕ に統合)
    // ============================================================
    let footer = row![
        button(
            row![
                text("↻").size(13),
                text(t.t(MessageKey::ResetButton)).size(13),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .padding([8, 16])
        .on_press(Message::ExportPlanResetRequested)
        .style(reset_button_style),
        iced::widget::Space::new().width(Length::Fill),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    column![title_row, scrollable_body, footer]
        .spacing(14)
        .padding(20)
        .height(Length::Fill)
        .into()
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


// ---------------------------------------------------------------------------
// pick_list 用の i18n ラッパ (ResizeAlgorithm / VtracerPreset)
// ---------------------------------------------------------------------------
//
// `pick_list` の各オプションが `Display` を要求するが、 ResizeAlgorithm /
// VtracerPreset 自体に `Display` を impl すると i18n 依存が core に漏れる。
// そのため UI 層で「値 + 翻訳済ラベル」 のラッパを作って Display する。

#[derive(Debug, Clone)]
struct LocalizedAlgorithm {
    value: ResizeAlgorithm,
    label: String,
}

impl PartialEq for LocalizedAlgorithm {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for LocalizedAlgorithm {}

impl std::fmt::Display for LocalizedAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

#[derive(Debug, Clone)]
struct LocalizedPreset {
    value: VtracerPreset,
    label: String,
}

impl PartialEq for LocalizedPreset {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for LocalizedPreset {}

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

/// v1.17.0: 旧 `close_button_style` の後継。 ドロワー内の補助ボタン
/// (「+ カスタムサイズ追加」 等) で使う共通スタイル。 透明背景 + 中立の
/// 枠線で「補助操作」 と認識される。
fn secondary_drawer_button_style(
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

/// v1.17.0: タイトル領域右上の ✕ ボタン (Close = ドロワーを閉じる)。
///
/// hover 時にだけ薄く塗る。 ボタンとしての装飾は最小限 (枠線なし) — モダンな
/// アプリの「タイトルバー右上の ✕」 の一般的な見た目に揃える。
fn title_close_button_style(
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
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

/// v1.17.0: 「▶ 上級設定」 折りたたみセクションヘッダのボタンスタイル。
///
/// 全幅クリック領域、 hover で背景うっすら、 枠線無し。 アコーディオンの
/// chevron 風 UI として動く。
fn extras_header_style(
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
        text_color: palette.background.base.text,
        border: iced::Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}
