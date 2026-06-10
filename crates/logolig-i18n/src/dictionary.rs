//! 辞書 (v1.5.0)。
//!
//! 各ロケールの TOML を `include_str!` でバンドルし、 起動時に serde で
//! `Dictionary` 構造体にパースする。
//!
//! ## なぜ「全フィールド必須」 の構造体にするか
//!
//! TOML をハッシュマップ (`HashMap<String, String>`) で読むことも可能だが、
//! 「ja.toml がキーを 1 つ忘れた」 のような事故が実行時まで分からなくなる。
//! 構造体に名前付きフィールドで持つことで:
//!
//! 1. **コンパイル時に「キー漏れ」 が分かる**: `ja.toml` が `app_title` を
//!    持っていないなら serde の `missing field` エラーで起動時にパース失敗
//! 2. **テストでパースを走らせる**: `tests/i18n.rs` で全 locale の `include_str!`
//!    が serde で読めることを確認すれば、 リリース前に検出できる
//! 3. **`Translator::t()` の match が型安全**: `MessageKey::AppTitle` →
//!    `dict.app_title` の対応が静的に解決される
//!
//! ## キー命名
//!
//! TOML 側のキーは `MessageKey::AppTitle` を snake_case にした `app_title`。
//! serde の rename もデフォルト命名規則も `snake_case` なので、 enum バリアント
//! を pascal、 TOML キーを snake にする変換は自動。
//!
//! ## なぜ単一構造体を `Translator` に直接持たせず、 `Dictionary` を経由するか
//!
//! `Translator` は将来「locale 切替時に辞書差し替え」 する責務を持つ。
//! `Dictionary` をフィールドにすることで、 `Translator::for_locale(new)` で
//! 内部の `Dictionary` を入れ替えるだけで済む。

use serde::Deserialize;

use logolig_core::MessageKey;

use crate::locale::Locale;

const EN: &str = include_str!("../locales/en.toml");
const JA: &str = include_str!("../locales/ja.toml");

/// 辞書の全フィールド。 各ロケール TOML がこの shape に一致しなければ
/// `serde::Deserialize` でエラーになる。
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Dictionary {
    // App-wide
    pub app_title: String,

    // Drop zone
    pub drop_zone_instruction: String,
    pub drop_zone_secondary: String,
    pub drop_zone_accepted_formats: String,
    pub choose_file_button: String,
    pub change_file_button: String,
    pub importing_message: String,

    // Preview
    pub preview_browser_tab: String,
    pub preview_smartphone_home: String,
    pub preview_background_light: String,
    pub preview_background_dark: String,
    pub preview_background_system: String,
    pub preview_source_label: String,
    pub preview_no_source: String,

    // Advanced drawer
    pub advanced_title: String,
    pub advanced_blurb: String,
    pub section_resize: String,
    pub section_resize_blurb: String,
    pub algorithm_label: String,
    pub section_svg: String,
    pub section_svg_blurb: String,
    pub include_svg_label: String,
    pub vectorize_on_raster_label: String,
    pub preset_label: String,
    pub section_files: String,
    pub section_files_blurb: String,
    pub include_ico_label: String,
    pub include_apple_touch_label: String,
    pub include_html_snippet_label: String,
    pub section_png_sizes: String,
    pub section_png_sizes_blurb: String,
    pub section_ico_sizes: String,
    pub section_ico_sizes_blurb: String,
    pub size_add_button: String,
    pub size_chip_remove: String,
    pub size_input_placeholder: String,
    pub empty_set_label: String,
    pub reset_button: String,
    pub close_button: String,

    // Buttons / actions
    pub export_button: String,
    pub toggle_advanced_button: String,
    pub toggle_theme_button: String,

    // Resize algorithm names
    pub algorithm_lanczos3: String,
    pub algorithm_mitchell_netravali: String,
    pub algorithm_catmull_rom: String,
    pub algorithm_bilinear: String,
    pub algorithm_nearest: String,

    // vtracer preset names
    pub vtracer_preset_sharp: String,
    pub vtracer_preset_default: String,
    pub vtracer_preset_photo_rich: String,

    // Language selection
    pub section_language: String,
    pub section_language_blurb: String,
    pub language_english: String,
    pub language_japanese: String,
    pub language_system_default: String,

    // Toasts
    pub toast_export_title: String,
    pub toast_export_body: String,
    pub toast_reset_title: String,
    pub toast_reset_body: String,
    pub toast_settings_load_failed_title: String,
    pub toast_settings_load_failed_body: String,
    pub toast_settings_save_failed_title: String,
    pub toast_settings_save_failed_body: String,
    pub toast_size_already_in_set_title: String,
    pub toast_png_size_already_in_set_body: String,
    pub toast_ico_size_already_in_set_body: String,
    pub toast_invalid_size_title: String,
    pub toast_invalid_size_body: String,
    pub toast_size_out_of_range_title: String,
    pub toast_png_size_out_of_range_body: String,
    pub toast_ico_size_out_of_range_body: String,

    // Errors
    pub error_unsupported_file: String,
    pub error_io: String,
    pub error_decode: String,
    pub error_rasterize: String,
    pub error_resize: String,
    pub error_export: String,
    pub error_not_implemented: String,

    // Transparency checker (v1.7.0)
    pub toast_fully_opaque_title: String,
    pub toast_fully_opaque_body: String,
    pub toast_fully_transparent_title: String,
    pub toast_fully_transparent_body: String,
    pub preview_checker_label: String,

    // Web manifest (v1.8.0)
    pub section_web_manifest: String,
    pub section_web_manifest_blurb: String,
    pub include_web_manifest_label: String,
    pub web_manifest_name_label: String,
    pub web_manifest_name_placeholder: String,
    pub web_manifest_short_name_label: String,
    pub web_manifest_short_name_placeholder: String,
    pub web_manifest_theme_color_label: String,
    pub web_manifest_background_color_label: String,
    pub toast_invalid_color_title: String,
    pub toast_invalid_color_body: String,

    // Monochrome (v1.9.0)
    pub section_monochrome: String,
    pub section_monochrome_blurb: String,
    pub include_monochrome_label: String,

    // UI redesign (v1.10.0)
    pub picker_label_view_as: String,
    pub picker_label_surface: String,
    pub preview_transparency_checker: String,
    pub group_what_to_export: String,
    pub group_extras: String,
    pub group_rendering_quality: String,
    pub group_app_preferences: String,

    // Main panel refresh (v1.10.2)
    pub app_tagline: String,
    pub drop_zone_headline: String,
    pub tooltip_language: String,
    pub tooltip_theme: String,
    pub tooltip_advanced: String,
    pub tooltip_close: String,

    // JPEG support (v1.11.0)
    pub toast_jpeg_input_title: String,
    pub toast_jpeg_input_body: String,

    // Edit-screen flow + preview redesign (v1.12.0)
    pub page_title_edit: String,
    pub section_title_preview: String,
    pub edit_cancel_button: String,
    pub edit_repick_button: String,
}

impl Dictionary {
    /// バンドル済みの TOML から特定ロケール用の Dictionary を作る。
    ///
    /// パースは `include_str!` 経由でビルド時に取り込んだ静的な内容に対して
    /// 行うため、 失敗するのは TOML 構文ミスかフィールド漏れの場合のみ。
    /// テストでこれを起動時にチェックすれば本番で失敗することはない。
    pub fn load(locale: Locale) -> Self {
        let raw = match locale {
            Locale::En => EN,
            Locale::Ja => JA,
        };
        toml::from_str(raw).unwrap_or_else(|err| {
            // ここに到達するのはテストで気づけなかった TOML バグの場合のみ。
            // panic で気づきやすくする (リリースビルドでも静かに English に
            // 戻すより、 デバッグしやすい挙動を選択)。
            panic!(
                "logolig-i18n: bundled dictionary for {:?} failed to parse: {err}",
                locale
            )
        })
    }

    /// `MessageKey` から対応するテンプレート文字列を引き出す。
    /// 網羅性は match 式によりコンパイル時に強制される。
    pub fn lookup(&self, key: MessageKey) -> &str {
        match key {
            MessageKey::AppTitle => &self.app_title,

            MessageKey::DropZoneInstruction => &self.drop_zone_instruction,
            MessageKey::DropZoneSecondary => &self.drop_zone_secondary,
            MessageKey::DropZoneAcceptedFormats => &self.drop_zone_accepted_formats,
            MessageKey::ChooseFileButton => &self.choose_file_button,
            MessageKey::ChangeFileButton => &self.change_file_button,
            MessageKey::ImportingMessage => &self.importing_message,

            MessageKey::PreviewBrowserTab => &self.preview_browser_tab,
            MessageKey::PreviewSmartphoneHome => &self.preview_smartphone_home,
            MessageKey::PreviewBackgroundLight => &self.preview_background_light,
            MessageKey::PreviewBackgroundDark => &self.preview_background_dark,
            MessageKey::PreviewBackgroundSystem => &self.preview_background_system,
            MessageKey::PreviewSourceLabel => &self.preview_source_label,
            MessageKey::PreviewNoSource => &self.preview_no_source,

            MessageKey::AdvancedTitle => &self.advanced_title,
            MessageKey::AdvancedBlurb => &self.advanced_blurb,
            MessageKey::SectionResize => &self.section_resize,
            MessageKey::SectionResizeBlurb => &self.section_resize_blurb,
            MessageKey::AlgorithmLabel => &self.algorithm_label,
            MessageKey::SectionSvg => &self.section_svg,
            MessageKey::SectionSvgBlurb => &self.section_svg_blurb,
            MessageKey::IncludeSvgLabel => &self.include_svg_label,
            MessageKey::VectorizeOnRasterLabel => &self.vectorize_on_raster_label,
            MessageKey::PresetLabel => &self.preset_label,
            MessageKey::SectionFiles => &self.section_files,
            MessageKey::SectionFilesBlurb => &self.section_files_blurb,
            MessageKey::IncludeIcoLabel => &self.include_ico_label,
            MessageKey::IncludeAppleTouchLabel => &self.include_apple_touch_label,
            MessageKey::IncludeHtmlSnippetLabel => &self.include_html_snippet_label,
            MessageKey::SectionPngSizes => &self.section_png_sizes,
            MessageKey::SectionPngSizesBlurb => &self.section_png_sizes_blurb,
            MessageKey::SectionIcoSizes => &self.section_ico_sizes,
            MessageKey::SectionIcoSizesBlurb => &self.section_ico_sizes_blurb,
            MessageKey::SizeAddButton => &self.size_add_button,
            MessageKey::SizeChipRemove => &self.size_chip_remove,
            MessageKey::SizeInputPlaceholder => &self.size_input_placeholder,
            MessageKey::EmptySetLabel => &self.empty_set_label,
            MessageKey::ResetButton => &self.reset_button,
            MessageKey::CloseButton => &self.close_button,

            MessageKey::ExportButton => &self.export_button,
            MessageKey::ToggleAdvancedButton => &self.toggle_advanced_button,
            MessageKey::ToggleThemeButton => &self.toggle_theme_button,

            MessageKey::AlgorithmLanczos3 => &self.algorithm_lanczos3,
            MessageKey::AlgorithmMitchellNetravali => &self.algorithm_mitchell_netravali,
            MessageKey::AlgorithmCatmullRom => &self.algorithm_catmull_rom,
            MessageKey::AlgorithmBilinear => &self.algorithm_bilinear,
            MessageKey::AlgorithmNearest => &self.algorithm_nearest,

            MessageKey::VtracerPresetSharp => &self.vtracer_preset_sharp,
            MessageKey::VtracerPresetDefault => &self.vtracer_preset_default,
            MessageKey::VtracerPresetPhotoRich => &self.vtracer_preset_photo_rich,

            MessageKey::SectionLanguage => &self.section_language,
            MessageKey::SectionLanguageBlurb => &self.section_language_blurb,
            MessageKey::LanguageEnglish => &self.language_english,
            MessageKey::LanguageJapanese => &self.language_japanese,
            MessageKey::LanguageSystemDefault => &self.language_system_default,

            MessageKey::ToastExportTitle => &self.toast_export_title,
            MessageKey::ToastExportBody => &self.toast_export_body,
            MessageKey::ToastResetTitle => &self.toast_reset_title,
            MessageKey::ToastResetBody => &self.toast_reset_body,
            MessageKey::ToastSettingsLoadFailedTitle => {
                &self.toast_settings_load_failed_title
            }
            MessageKey::ToastSettingsLoadFailedBody => &self.toast_settings_load_failed_body,
            MessageKey::ToastSettingsSaveFailedTitle => {
                &self.toast_settings_save_failed_title
            }
            MessageKey::ToastSettingsSaveFailedBody => &self.toast_settings_save_failed_body,
            MessageKey::ToastSizeAlreadyInSetTitle => &self.toast_size_already_in_set_title,
            MessageKey::ToastPngSizeAlreadyInSetBody => {
                &self.toast_png_size_already_in_set_body
            }
            MessageKey::ToastIcoSizeAlreadyInSetBody => {
                &self.toast_ico_size_already_in_set_body
            }
            MessageKey::ToastInvalidSizeTitle => &self.toast_invalid_size_title,
            MessageKey::ToastInvalidSizeBody => &self.toast_invalid_size_body,
            MessageKey::ToastSizeOutOfRangeTitle => &self.toast_size_out_of_range_title,
            MessageKey::ToastPngSizeOutOfRangeBody => &self.toast_png_size_out_of_range_body,
            MessageKey::ToastIcoSizeOutOfRangeBody => &self.toast_ico_size_out_of_range_body,

            MessageKey::ErrorUnsupportedFile => &self.error_unsupported_file,
            MessageKey::ErrorIo => &self.error_io,
            MessageKey::ErrorDecode => &self.error_decode,
            MessageKey::ErrorRasterize => &self.error_rasterize,
            MessageKey::ErrorResize => &self.error_resize,
            MessageKey::ErrorExport => &self.error_export,
            MessageKey::ErrorNotImplemented => &self.error_not_implemented,

            // v1.7.0
            MessageKey::ToastFullyOpaqueTitle => &self.toast_fully_opaque_title,
            MessageKey::ToastFullyOpaqueBody => &self.toast_fully_opaque_body,
            MessageKey::ToastFullyTransparentTitle => &self.toast_fully_transparent_title,
            MessageKey::ToastFullyTransparentBody => &self.toast_fully_transparent_body,
            MessageKey::PreviewCheckerLabel => &self.preview_checker_label,

            // v1.8.0
            MessageKey::SectionWebManifest => &self.section_web_manifest,
            MessageKey::SectionWebManifestBlurb => &self.section_web_manifest_blurb,
            MessageKey::IncludeWebManifestLabel => &self.include_web_manifest_label,
            MessageKey::WebManifestNameLabel => &self.web_manifest_name_label,
            MessageKey::WebManifestNamePlaceholder => &self.web_manifest_name_placeholder,
            MessageKey::WebManifestShortNameLabel => &self.web_manifest_short_name_label,
            MessageKey::WebManifestShortNamePlaceholder => &self.web_manifest_short_name_placeholder,
            MessageKey::WebManifestThemeColorLabel => &self.web_manifest_theme_color_label,
            MessageKey::WebManifestBackgroundColorLabel => &self.web_manifest_background_color_label,
            MessageKey::ToastInvalidColorTitle => &self.toast_invalid_color_title,
            MessageKey::ToastInvalidColorBody => &self.toast_invalid_color_body,

            // v1.9.0
            MessageKey::SectionMonochrome => &self.section_monochrome,
            MessageKey::SectionMonochromeBlurb => &self.section_monochrome_blurb,
            MessageKey::IncludeMonochromeLabel => &self.include_monochrome_label,

            // v1.10.0
            MessageKey::PickerLabelViewAs => &self.picker_label_view_as,
            MessageKey::PickerLabelSurface => &self.picker_label_surface,
            MessageKey::PreviewTransparencyChecker => &self.preview_transparency_checker,
            MessageKey::GroupWhatToExport => &self.group_what_to_export,
            MessageKey::GroupExtras => &self.group_extras,
            MessageKey::GroupRenderingQuality => &self.group_rendering_quality,
            MessageKey::GroupAppPreferences => &self.group_app_preferences,

            // v1.10.2
            MessageKey::AppTagline => &self.app_tagline,
            MessageKey::DropZoneHeadline => &self.drop_zone_headline,
            MessageKey::TooltipLanguage => &self.tooltip_language,
            MessageKey::TooltipTheme => &self.tooltip_theme,
            MessageKey::TooltipAdvanced => &self.tooltip_advanced,
            MessageKey::TooltipClose => &self.tooltip_close,

            // v1.11.0
            MessageKey::ToastJpegInputTitle => &self.toast_jpeg_input_title,
            MessageKey::ToastJpegInputBody => &self.toast_jpeg_input_body,

            // v1.12.0
            MessageKey::PageTitleEdit => &self.page_title_edit,
            MessageKey::SectionTitlePreview => &self.section_title_preview,
            MessageKey::EditCancelButton => &self.edit_cancel_button,
            MessageKey::EditRepickButton => &self.edit_repick_button,
        }
    }
}
