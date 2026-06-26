//! Translation keys (v1.5.0).
//!
//! Every UI string and error message is referenced through this enum.
//! `logolig-i18n` holds per-locale dictionaries and maps `MessageKey` to strings.
//!
//! ## Why an enum
//!
//! String keys (e.g. `"error.io"`) would work, but using an enum gives:

//!
//! 1. **Exhaustiveness**: the dictionary `match` fails to compile the moment
//!    logolig-core adds a new key — translation drift is caught at compile time.
//! 2. **Rename safety**: renaming `ErrorIo` to `ErrorReadFailed` lets the IDE
//!    update every reference in one step.
//! 3. **Dead-key detection**: unused keys surface as `dead_code` warnings.
//!
//! ## Why in logolig-core
//!
//! `AppError::key()` returns a `MessageKey`, so core owns the enum.
//! Placing it in logolig-i18n would reverse the dependency direction.
//!
//! ## Structure
//!
//! The enum is intentionally flat. Nested namespaces like `"app.title"` are
//! avoided; prefix conventions (`AppTitle`) serve the same purpose and let
//! translators see all N keys in one place.

use serde::{Deserialize, Serialize};

/// Key for every UI string and error message.
///
/// To add a new string:
/// 1. Add a variant here.
/// 2. Add the key to each locale dictionary (e.g. en.toml).
/// 3. Fill the exhaustive match in `Translator` — it will fail to compile otherwise.
///
/// This three-step process makes translation omissions a compile-time error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageKey {
    // --- Application-wide ---
    /// App title ("Logolig").
    AppTitle,

    // --- Drop zone ---
    DropZoneInstruction,
    DropZoneSecondary,
    DropZoneAcceptedFormats,
    ChooseFileButton,
    ChangeFileButton,
    ImportingMessage,

    // --- Preview screen ---
    PreviewBrowserTab,
    PreviewSmartphoneHome,
    PreviewBackgroundLight,
    PreviewBackgroundDark,
    PreviewBackgroundSystem,
    PreviewSourceLabel,
    PreviewNoSource,

    // --- Advanced settings ---
    AdvancedTitle,
    AdvancedBlurb,
    SectionResize,
    SectionResizeBlurb,
    AlgorithmLabel,
    SectionSvg,
    SectionSvgBlurb,
    IncludeSvgLabel,
    VectorizeOnRasterLabel,
    PresetLabel,
    SectionFiles,
    SectionFilesBlurb,
    IncludeIcoLabel,
    IncludeAppleTouchLabel,
    IncludeHtmlSnippetLabel,
    SectionPngSizes,
    SectionPngSizesBlurb,
    SectionIcoSizes,
    SectionIcoSizesBlurb,
    SizeAddButton,
    SizeChipRemove,
    SizeInputPlaceholder,
    EmptySetLabel,
    ResetButton,
    CloseButton,

    // --- Buttons / actions ---
    ExportButton,
    ToggleAdvancedButton,
    ToggleThemeButton,

    // --- Resize algorithm names ---
    AlgorithmLanczos3,
    AlgorithmMitchellNetravali,
    AlgorithmCatmullRom,
    AlgorithmBilinear,
    AlgorithmNearest,

    // --- vtracer preset names ---
    VtracerPresetSharp,
    VtracerPresetDefault,
    VtracerPresetPhotoRich,

    // --- Locale selection (v1.5.0) ---
    SectionLanguage,
    SectionLanguageBlurb,
    LanguageEnglish,
    LanguageJapanese,
    LanguageSystemDefault,

    // --- Toast titles / bodies ---
    ToastExportTitle,
    ToastExportBody,
    ToastResetTitle,
    ToastResetBody,
    ToastSettingsLoadFailedTitle,
    ToastSettingsLoadFailedBody,
    ToastSettingsSaveFailedTitle,
    ToastSettingsSaveFailedBody,
    ToastSizeAlreadyInSetTitle,
    ToastPngSizeAlreadyInSetBody,
    ToastIcoSizeAlreadyInSetBody,
    ToastInvalidSizeTitle,
    ToastInvalidSizeBody,
    ToastSizeOutOfRangeTitle,
    ToastPngSizeOutOfRangeBody,
    ToastIcoSizeOutOfRangeBody,

    // --- Errors (AppError key mapping) ---
    ErrorUnsupportedFile,
    ErrorIo,
    ErrorDecode,
    ErrorRasterize,
    ErrorResize,
    ErrorExport,
    ErrorNotImplemented,

    // --- Transparency checker (v1.7.0) ---
    /// Warning toast title for a fully-opaque source.
    ToastFullyOpaqueTitle,
    /// Warning toast body for a fully-opaque source.
    ToastFullyOpaqueBody,
    /// Warning toast title for a fully-transparent source.
    ToastFullyTransparentTitle,
    /// Warning toast body for a fully-transparent source.
    ToastFullyTransparentBody,
    /// Transparency checker toggle label ("Show transparency checker").
    PreviewCheckerLabel,

    // --- Web manifest (v1.8.0) ---
    SectionWebManifest,
    SectionWebManifestBlurb,
    IncludeWebManifestLabel,
    WebManifestNameLabel,
    WebManifestNamePlaceholder,
    WebManifestShortNameLabel,
    WebManifestShortNamePlaceholder,
    WebManifestThemeColorLabel,
    WebManifestBackgroundColorLabel,
    /// Toast title when a hex colour input is not valid `#RRGGBB`.
    ToastInvalidColorTitle,
    /// Toast body when a hex colour input is invalid (includes the input placeholder).
    ToastInvalidColorBody,

    // --- Monochrome output (v1.9.0) ---
    /// Monochrome output section heading in settings.
    SectionMonochrome,
    /// Monochrome section description (mono/ subdirectory, PNG/ICO targets, etc.).
    SectionMonochromeBlurb,
    /// Checkbox label.
    IncludeMonochromeLabel,

    // --- v1.10.0: UI information architecture refresh ---
    /// "View as:" label on the main screen.
    PickerLabelViewAs,
    /// "Surface:" label on the main screen.
    PickerLabelSurface,
    /// "Transparency checker" preview context label.
    PreviewTransparencyChecker,
    /// "What to export" group heading in advanced settings.
    GroupWhatToExport,
    /// "Extras" group heading in advanced settings.
    GroupExtras,
    /// "Rendering quality" group heading in advanced settings.
    GroupRenderingQuality,
    /// "App preferences" group heading in advanced settings.
    GroupAppPreferences,

    // --- v1.10.2: Main screen refresh ---
    /// App tagline shown in muted text next to the app name.
    AppTagline,
    /// Slim eye-catcher text in the drop zone.
    DropZoneHeadline,
    /// Language button tooltip in the header.
    TooltipLanguage,
    /// Theme button tooltip in the header.
    TooltipTheme,
    /// Advanced settings button tooltip in the header.
    TooltipAdvanced,
    /// Close button tooltip in the header.
    TooltipClose,

    // --- v1.11.0: JPEG support ---
    /// Toast title for JPEG input: explains JPEG cannot store transparency
    /// (distinct from the generic "fully opaque" warning).
    ToastJpegInputTitle,
    /// Toast body for JPEG input. Suggests converting to PNG.
    ToastJpegInputBody,

    // --- v1.12.0: Edit screen navigation + preview area refresh ---
    /// Edit/preview screen page title ("Preview & Generate Favicon").
    PageTitleEdit,
    /// Preview panel section title.
    SectionTitlePreview,
    /// "Back" button on the edit screen (returns to the startup screen).
    EditCancelButton,
    /// "Re-select" button on the edit screen (re-opens the file picker).
    EditRepickButton,

    // --- v1.16.0: Screen structure revision (Empty / Converting / Result) ---
    /// Converting screen status message ("Please wait…").
    ImportingPleaseWait,
    /// Result screen headline "✓ Conversion complete!".
    ResultSuccessHeadline,
    /// Result screen sub-heading listing the generated assets.
    ResultAssetsSubheading,
    /// "Download all (ZIP)" button on the Result screen.
    ResultDownloadAllButton,
    /// Accessibility label / tooltip for the individual download button on each asset card.
    ResultDownloadOne,
    /// "View preview" collapsible section label on the Result screen.
    ResultPreviewToggle,

    // --- v1.17.0: Settings drawer converted to Right Sheet + flat layout ---
    /// Drawer title "Settings".
    SettingsTitle,
    /// "PNG output sizes" section heading.
    SectionPngOutputSizes,
    /// "Add custom size" button (appended to the PNG size list).
    AddCustomSize,
    /// "SVG conversion mode" section heading.
    SectionSvgConversion,
    /// SVG conversion slider left label ("Simple").
    SvgConversionSimple,
    /// SVG conversion slider right label ("Detailed").
    SvgConversionDetailed,
    /// "Misc" section heading.
    SectionMisc,
    /// "Keep transparency (alpha)" toggle label.
    KeepTransparency,
    /// "Advanced settings" collapsible section heading.
    AdvancedExtrasSection,

    // --- v1.18.0: Left sidebar + picker popups (nav redesigned in v1.22.0) ---
    /// Sidebar Settings icon label/tooltip (legacy; nav changed in v1.22.0).
    SidebarLabelSettings,
    /// Sidebar Language icon label/tooltip (legacy; nav changed in v1.22.0).
    SidebarLabelLocale,
    /// Sidebar Theme icon label/tooltip (legacy; nav changed in v1.22.0).
    SidebarLabelTheme,
    /// Language picker 'Japanese' row.
    LocaleNameJa,
    /// Language picker 'English' row.
    LocaleNameEn,
    /// Language picker 'Follow system' row (= `LocalePicked(None)`).
    LocaleSystem,
    /// Theme picker 'Light' row.
    ThemeNameLight,
    /// Theme picker 'Dark' row.
    ThemeNameDark,
    /// Theme picker 'Follow system' row.
    ThemeSystem,

    // --- v1.22.0: side-nav three items ---
    /// Side-nav 'Home' label.
    NavHome,
    /// Side-nav 'Customize' label.
    NavCustomize,
    /// Side-nav 'Settings' label.
    NavSettings,

    // --- v1.24.0: history card on the Empty screen ---
    /// Section label for the last-conversion card ("Last conversion").
    HistoryLastConversionLabel,
    /// Button label to return to the Result screen ("View results →").
    HistoryViewResultsButton,
}
