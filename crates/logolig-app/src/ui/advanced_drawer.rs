//! 詳細設定ドロワー (§5.3)。
//!
//! 既定値は隠す。 `state.advanced_open == true` のときだけ shell が表示する。
//!
//! ## v1.5.0 で追加
//!
//! - 全文言を `state.translator.t(MessageKey::...)` 経由で翻訳
//! - **Language セクション** を追加: pick_list で `System default / English / 日本語` を選択
//!   (v1.5.0 では English のみ実体あり、 v1.6 で 日本語 が機能する)

use iced::widget::{button, checkbox, column, container, pick_list, row, text, text_input};
use iced::{Alignment, Element, Length, Padding};

use logolig_core::{
    MessageKey, ResizeAlgorithm, VtracerPreset, ICO_SIZE_MAX, ICO_SIZE_MIN, PNG_SIZE_MAX,
    PNG_SIZE_MIN,
};
use logolig_i18n::Locale;

use crate::app::{AppState, Message};

pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;

    column![
        // ヘッダ
        text(t.t(MessageKey::AdvancedTitle)).size(22),
        text(t.t(MessageKey::AdvancedBlurb)).size(12),
        // 1. リサイズアルゴリズム
        section(
            &t.t(MessageKey::SectionResize),
            &t.t(MessageKey::SectionResizeBlurb),
            algorithm_row(state),
        ),
        // 2. SVG 出力
        section(
            &t.t(MessageKey::SectionSvg),
            &t.t(MessageKey::SectionSvgBlurb),
            column![
                checkbox(state.export_plan.include_svg)
                    .label(t.t(MessageKey::IncludeSvgLabel))
                    .on_toggle(Message::IncludeSvgToggled)
                    .text_size(13),
                checkbox(state.export_plan.vectorize_on_raster)
                    .label(t.t(MessageKey::VectorizeOnRasterLabel))
                    .on_toggle(Message::VectorizeOnRasterToggled)
                    .text_size(13),
                vtracer_preset_row(state),
            ]
            .spacing(6)
            .into(),
        ),
        // 3. 出力ファイル種別
        section(
            &t.t(MessageKey::SectionFiles),
            &t.t(MessageKey::SectionFilesBlurb),
            column![
                checkbox(state.export_plan.include_ico)
                    .label(t.t(MessageKey::IncludeIcoLabel))
                    .on_toggle(Message::IncludeIcoToggled)
                    .text_size(13),
                checkbox(state.export_plan.include_apple_touch)
                    .label(t.t(MessageKey::IncludeAppleTouchLabel))
                    .on_toggle(Message::IncludeAppleTouchToggled)
                    .text_size(13),
                checkbox(state.export_plan.include_html_snippet)
                    .label(t.t(MessageKey::IncludeHtmlSnippetLabel))
                    .on_toggle(Message::IncludeHtmlSnippetToggled)
                    .text_size(13),
            ]
            .spacing(6)
            .into(),
        ),
        // 4. PNG サイズ集合
        section(
            &t.t(MessageKey::SectionPngSizes),
            &t.t(MessageKey::SectionPngSizesBlurb),
            size_set_editor(
                state,
                &state.export_plan.png_sizes,
                &state.png_size_input,
                Message::PngSizeRemoveRequested,
                Message::PngSizeInputChanged,
                Message::PngSizeAddRequested,
                PNG_SIZE_MIN,
                PNG_SIZE_MAX,
            ),
        ),
        // 5. ICO サイズ集合
        section(
            &t.t(MessageKey::SectionIcoSizes),
            &t.t(MessageKey::SectionIcoSizesBlurb),
            size_set_editor(
                state,
                &state.export_plan.ico_sizes,
                &state.ico_size_input,
                Message::IcoSizeRemoveRequested,
                Message::IcoSizeInputChanged,
                Message::IcoSizeAddRequested,
                ICO_SIZE_MIN,
                ICO_SIZE_MAX,
            ),
        ),
        // 6. 言語選択 (v1.5.0)
        section(
            &t.t(MessageKey::SectionLanguage),
            &t.t(MessageKey::SectionLanguageBlurb),
            language_row(state),
        ),
        // 7. Web manifest (v1.8.0)
        section(
            &t.t(MessageKey::SectionWebManifest),
            &t.t(MessageKey::SectionWebManifestBlurb),
            web_manifest_body(state),
        ),
        // 8. Monochrome (v1.9.0)
        section(
            &t.t(MessageKey::SectionMonochrome),
            &t.t(MessageKey::SectionMonochromeBlurb),
            checkbox(state.export_plan.monochrome)
                .label(t.t(MessageKey::IncludeMonochromeLabel))
                .on_toggle(Message::IncludeMonochromeToggled)
                .text_size(13)
                .into(),
        ),
        // フッタ: Reset と Close を横並び
        row![
            button(text(t.t(MessageKey::ResetButton)))
                .on_press(Message::ExportPlanResetRequested),
            button(text(t.t(MessageKey::CloseButton))).on_press(Message::AdvancedToggled),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(18)
    .padding(20)
    .into()
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

fn section<'a>(title: &str, blurb: &str, body: Element<'a, Message>) -> Element<'a, Message> {
    container(
        column![
            text(title.to_string()).size(15),
            text(blurb.to_string()).size(11),
            container(body).padding(Padding::default().top(4)),
        ]
        .spacing(4),
    )
    .padding(Padding::default().top(2).bottom(2))
    .into()
}

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

fn language_row<'a>(state: &'a AppState) -> Element<'a, Message> {
    // System default に加え、 各 Locale を選択肢として並べる。
    // 内部表現は Option<Locale>: None = system default。
    let mut options: Vec<LocalizedLocaleChoice> = Vec::new();
    options.push(LocalizedLocaleChoice {
        value: None,
        label: state.translator.t(MessageKey::LanguageSystemDefault),
    });
    for loc in Locale::all() {
        options.push(LocalizedLocaleChoice {
            value: Some(loc),
            label: state.translator.t(locale_message_key(loc)),
        });
    }
    let selected = options
        .iter()
        .find(|opt| opt.value == state.locale_override)
        .cloned();
    let picker = pick_list(options, selected, |opt: LocalizedLocaleChoice| {
        Message::LocaleChanged(opt.value)
    })
    .text_size(13);

    row![picker].into()
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocalizedLocaleChoice {
    value: Option<Locale>,
    label: String,
}

impl std::fmt::Display for LocalizedLocaleChoice {
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

fn locale_message_key(loc: Locale) -> MessageKey {
    match loc {
        Locale::En => MessageKey::LanguageEnglish,
        Locale::Ja => MessageKey::LanguageJapanese,
    }
}
