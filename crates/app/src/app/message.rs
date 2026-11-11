//! The application's message type.

use std::path::PathBuf;

use logolig::{AppError, PreviewCache, PreviewContext, ResizeAlgorithm, SourceAsset, ThemeMode};
use logolig_i18n::Locale;

use super::NavPage;

// ----------------------------------------------------------------------
// Messages
// ----------------------------------------------------------------------

/// All events fired by the UI and service layers (§4.3).
///
/// `snora::AppLayout<Element, Message>` requires `Message: Clone`, so
/// every variant must be cloneable.
#[derive(Debug, Clone)]
pub enum Message {
    // Input
    FileDropped(PathBuf),
    /// A native file drag is hovering over the window.
    FileDragHovered,
    /// The native file drag left the window or was completed/cancelled.
    FileDragLeft,
    PickFileRequested,
    FilePicked(Option<PathBuf>),

    // Loading
    /// The file read of job `u64` finished (RFC 011: the id travels with every result).
    IngestCompleted(u64, Result<SourceAsset, AppError>),

    // Preview
    /// The preview of job `u64` is ready.
    PreviewBuilt(u64, Result<PreviewCache, AppError>),
    PreviewContextSelected(PreviewContext),
    PreviewBackgroundSelected(ThemeMode),

    // v1.16.0: in-memory conversion complete.
    /// Conversion finished; the ResultAssets bundle is ready.
    /// Distinct from the former `Message::ExportCompleted` (which wrote to disk);
    /// this variant only places the asset bundle into app state.
    /// The user triggers the actual write later via individual or ZIP download.
    ConvertCompleted(u64, Result<crate::result::ResultAssets, logolig::AppError>),

    /// v1.16.0: individual download button pressed. Carries the asset index.
    DownloadOneRequested(usize),
    /// v1.16.0: ZIP download-all button pressed.
    DownloadAllRequested,
    /// v1.16.0: result of the save-file dialog for individual download. None = cancelled.
    DownloadOneTargetPicked(usize, Option<std::path::PathBuf>),
    /// v1.16.0: result of the save-file dialog for ZIP download. None = cancelled.
    DownloadAllTargetPicked(Option<std::path::PathBuf>),
    /// v1.16.0: individual download write completed.
    DownloadOneCompleted(Result<std::path::PathBuf, logolig::AppError>),
    /// v1.16.0: ZIP download-all write completed.
    DownloadAllCompleted(Result<std::path::PathBuf, logolig::AppError>),

    /// v1.16.0: toggles the collapsible preview panel on the Result screen.
    ResultPreviewToggled,

    // v1.17.0: settings drawer became a Right Sheet
    /// Window resize notification. Updates `state.window_size`, used for mobile breakpoint detection.
    WindowResized(iced::Size<f32>),
    /// Toggles the 'Advanced extras' collapsible in the Customize page.
    AdvancedExtrasToggled,
    /// Adds a preset PNG size (checked in the Customize page).
    /// Expected to be one of 16/32/48/96/192/512.
    PngPresetSizeAdded(u32),

    // v1.22.0: side-nav page switch
    /// Side-nav item clicked — switches to the given page.
    NavPageSelected(NavPage),
    /// RFC 019 phase 2: a global keyboard shortcut was pressed (see `crate::shortcuts`).
    ShortcutPressed(crate::shortcuts::Shortcut),
    /// Locale confirmed in the Settings page. None = follow OS locale.
    LocalePicked(Option<Locale>),
    /// Theme confirmed in the Settings page.
    ThemePicked(ThemeMode),

    // Theme / UI
    // v1.19.0: old `ThemeToggled` (cycle UI) removed; replaced by `ThemePicked`.
    AlgorithmChanged(ResizeAlgorithm),
    /// v1.2.0: toggle SVG output on/off.
    IncludeSvgToggled(bool),
    /// v1.2.0: toggle raster-to-vector conversion on/off.
    VectorizeOnRasterToggled(bool),

    // v1.3.0: Customize page editing UI
    /// Whether to output apple-touch-icon.png.
    IncludeAppleTouchToggled(bool),
    /// Whether to output favicon-snippet.html.
    IncludeHtmlSnippetToggled(bool),
    /// PNG size set input text changed.
    PngSizeInputChanged(String),
    /// PNG size added (Add button or Enter).
    PngSizeAddRequested,
    /// PNG size removed (chip × button).
    PngSizeRemoveRequested(u32),

    // v1.4.1: vtracer preset switch + ExportPlan reset
    /// vtracer preset changed (Sharp / Default / PhotoRich).
    VtracerPresetChanged(logolig::VtracerPreset),
    /// Resets ExportPlan to defaults. Does not affect theme or locale.
    ExportPlanResetRequested,

    // v1.19.0: old `LocaleCycled` and `AppCloseRequested` removed.

    // `LocaleCycled` → replaced by `LocalePicked` in v1.18.0.

    // `AppCloseRequested` → removed when the ✕ button was dropped in v1.18.0.

    // v1.17.0: `AdvancedGroupToggled` removed (accordion structure replaced by flat layout).

    // v1.12.0: back navigation
    /// Back / Cancel button. Returns to the drop zone (Empty screen),
    /// discarding the loaded source and preview cache.
    /// ESC key binding may be added in a future version.
    EditCancelled,

    // v1.10.0: PreviewCheckerToggled removed; checker state lives in PreviewContext.

    // v1.8.0: Web manifest
    /// Toggles web manifest output. When turned on, inserts
    /// `WebManifestSettings::default()` into `state.export_plan.web_manifest`.
    IncludeWebManifestToggled(bool),
    /// Web manifest `name` field edited.
    WebManifestNameChanged(String),
    /// Web manifest `short_name` field edited.
    WebManifestShortNameChanged(String),
    /// Web manifest `theme_color` edited. Applied immediately;
    /// validated on submit / focus-loss.
    WebManifestThemeColorChanged(String),
    /// Web manifest `background_color` edited.
    WebManifestBackgroundColorChanged(String),

    // v1.9.0: monochrome output
    /// Toggles the `mono/` grayscale output set.
    IncludeMonochromeToggled(bool),

    // v1.26.0: Microsoft app logo output
    /// Toggles the minimal Microsoft app logo PNG set.
    IncludeMicrosoftAppLogosToggled(bool),

    // v1.21.0: keep-transparency toggle
    /// Whether to preserve alpha or flatten against a white background.
    /// `true` = preserve (modern favicon standard); `false` = flatten.
    /// Affects all raster outputs (PNG / ICO / mono); SVG is unaffected
    /// (flattening is a raster concept).
    KeepTransparencyToggled(bool),

    // v1.19.0: old Export* messages removed. v1.16.0 moved to the
    // ConvertCompleted path (drop → auto-convert → Result → per-asset DL).
    // The old 'Export button → directory picker → bulk write' flow is gone.
    // Deleted messages:
    // - ExportRequested (button removed)
    // - ExportDirPicked(Option<PathBuf>)
    // - ExportCompleted(Result<ExportReport, AppError>)
    // Individual / ZIP-all completion notifications use DownloadOneCompleted /
    // DownloadAllCompleted (v1.16.0).

    // Toast lifecycle
    ToastTick,
    /// The reader's attention reached or left a toast (pointer on it; RFC 032). Passed to
    /// `Holds::attend` as it comes: the value is `#[non_exhaustive]` and is not matched here.
    ToastAttention(u64, snora::ToastAttention),
    DismissToast(u64),

    // snora: outside-click to close modals
    CloseModals,

    // v1.24.0: history — return to the last Result screen without re-converting.
    /// Switch back to `Screen::Result` using the in-memory assets already in
    /// `state.result_assets`. Only reachable from the history card on the
    /// Empty screen, so `result_assets` is always `Some` when this fires.
    ShowLastResultRequested,

    // RFC 015 phase 2a: result of the off-thread SVG analysis.
    /// The SVG at `PathBuf` was analysed. Ignored when it is no longer the current source
    /// (a newer file was loaded in the meantime). Never sent for non-SVG sources.
    SvgAnalyzed(u64, PathBuf, logolig::SvgReport),
}
