//! Dictionary (v1.5.0).
//!
//! Bundles each locale's TOML via `include_str!` and deserialises it into
//! a `Dictionary` struct at startup.
//!
//! ## Why a struct with named fields (not a HashMap)?
//!
//! Reading TOML into a `HashMap<String, String>` would let a missing key
//! in `ja.toml` go unnoticed until runtime. Using a named-field struct:
//!
//! 1. **Missing keys fail at startup** (serde panics during deserialisation)
//! 2. **All keys are visible in one place** — adding a `MessageKey` variant
//!    means adding a struct field, which the compiler enforces
//! 3. **No string-key typos** — the compiler rejects misspelled field names

use serde::Deserialize;

use logolig_core::MessageKey;

use crate::locale::Locale;

const EN: &str = include_str!("../locales/en.toml");
const JA: &str = include_str!("../locales/ja.toml");

/// All dictionary fields. If a locale TOML does not match this shape,
/// `serde::Deserialize` will error at startup.
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

    // Screen structure revision (v1.16.0)
    pub importing_please_wait: String,
    pub result_success_headline: String,
    pub result_assets_subheading: String,
    pub result_download_all_button: String,
    pub result_download_one: String,
    pub result_preview_toggle: String,

    // Settings — right-sheet + flat section layout (v1.17.0)
    pub settings_title: String,
    pub section_png_output_sizes: String,
    pub add_custom_size: String,
    pub section_svg_conversion: String,
    pub svg_conversion_simple: String,
    pub svg_conversion_detailed: String,
    pub section_misc: String,
    pub keep_transparency: String,
    pub advanced_extras_section: String,

    // Left sidebar + picker popups (v1.18.0)
    pub sidebar_label_settings: String,
    pub sidebar_label_locale: String,
    pub sidebar_label_theme: String,
    pub locale_name_ja: String,
    pub locale_name_en: String,
    pub locale_system: String,
    pub theme_name_light: String,
    pub theme_name_dark: String,
    pub theme_system: String,

    // v1.22.0: side nav
    pub nav_home: String,
    pub nav_customize: String,
    pub nav_settings: String,
}

impl Dictionary {
    /// Build a `Dictionary` for the given locale from the bundled TOML.
    ///
    /// Parsing is applied to static content embedded at build time via `include_str!`.
    /// at startup; failure means either a TOML syntax error or a missing field.
    /// Checking this in a test at startup prevents production failures.
    pub fn load(locale: Locale) -> Self {
        let raw = match locale {
            Locale::En => EN,
            Locale::Ja => JA,
        };
        toml::from_str(raw).unwrap_or_else(|err| {
            // Only reachable if a TOML bug escaped the test suite.
            // Panic rather than silently falling back to English —
            // easier to debug.
            panic!(
                "logolig-i18n: bundled dictionary for {:?} failed to parse: {err}",
                locale
            )
        })
    }

    /// Look up the translation template string for a `MessageKey`.
    /// Exhaustiveness is enforced by the match expression at compile time.
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
            MessageKey::ToastSettingsLoadFailedTitle => &self.toast_settings_load_failed_title,
            MessageKey::ToastSettingsLoadFailedBody => &self.toast_settings_load_failed_body,
            MessageKey::ToastSettingsSaveFailedTitle => &self.toast_settings_save_failed_title,
            MessageKey::ToastSettingsSaveFailedBody => &self.toast_settings_save_failed_body,
            MessageKey::ToastSizeAlreadyInSetTitle => &self.toast_size_already_in_set_title,
            MessageKey::ToastPngSizeAlreadyInSetBody => &self.toast_png_size_already_in_set_body,
            MessageKey::ToastIcoSizeAlreadyInSetBody => &self.toast_ico_size_already_in_set_body,
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
            MessageKey::WebManifestShortNamePlaceholder => {
                &self.web_manifest_short_name_placeholder
            }
            MessageKey::WebManifestThemeColorLabel => &self.web_manifest_theme_color_label,
            MessageKey::WebManifestBackgroundColorLabel => {
                &self.web_manifest_background_color_label
            }
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

            // v1.16.0
            MessageKey::ImportingPleaseWait => &self.importing_please_wait,
            MessageKey::ResultSuccessHeadline => &self.result_success_headline,
            MessageKey::ResultAssetsSubheading => &self.result_assets_subheading,
            MessageKey::ResultDownloadAllButton => &self.result_download_all_button,
            MessageKey::ResultDownloadOne => &self.result_download_one,
            MessageKey::ResultPreviewToggle => &self.result_preview_toggle,

            // v1.17.0
            MessageKey::SettingsTitle => &self.settings_title,
            MessageKey::SectionPngOutputSizes => &self.section_png_output_sizes,
            MessageKey::AddCustomSize => &self.add_custom_size,
            MessageKey::SectionSvgConversion => &self.section_svg_conversion,
            MessageKey::SvgConversionSimple => &self.svg_conversion_simple,
            MessageKey::SvgConversionDetailed => &self.svg_conversion_detailed,
            MessageKey::SectionMisc => &self.section_misc,
            MessageKey::KeepTransparency => &self.keep_transparency,
            MessageKey::AdvancedExtrasSection => &self.advanced_extras_section,

            // v1.18.0
            MessageKey::SidebarLabelSettings => &self.sidebar_label_settings,
            MessageKey::SidebarLabelLocale => &self.sidebar_label_locale,
            MessageKey::SidebarLabelTheme => &self.sidebar_label_theme,
            MessageKey::LocaleNameJa => &self.locale_name_ja,
            MessageKey::LocaleNameEn => &self.locale_name_en,
            MessageKey::LocaleSystem => &self.locale_system,
            MessageKey::ThemeNameLight => &self.theme_name_light,
            MessageKey::ThemeNameDark => &self.theme_name_dark,
            MessageKey::ThemeSystem => &self.theme_system,
            // v1.22.0
            MessageKey::NavHome => &self.nav_home,
            MessageKey::NavCustomize => &self.nav_customize,
            MessageKey::NavSettings => &self.nav_settings,
        }
    }
}
