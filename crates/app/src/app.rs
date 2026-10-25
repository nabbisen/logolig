//! Application core (§11).
//!
//! - holds `AppState`
//! - receives `Message` and drives state transitions
//! - wires `view` / `update` / `theme` / `subscription` into iced
//! - errors are surfaced as snora `Toast` notifications, not screen transitions
//!   (aligns with §12 ABDD: state is not expressed through colour alone)
//! - heavy work is offloaded via `iced::Task` to avoid blocking the UI thread (§2.4)

use std::path::PathBuf;
use std::time::Instant;

use iced::{Element, Subscription, Task, Theme};
use snora::{Toast, ToastIntent, ToastLifetime};

use logolig::{
    AppError, ExportPlan, MessageKey, PreviewCache, PreviewContext, PreviewProfile,
    ResizeAlgorithm, SettingsStore, SourceAsset, ThemeMode,
    services::transparency_audit::{TransparencyReport, audit as audit_transparency},
};
use logolig_i18n::{Locale, Translator, detect_system_locale};

// ----------------------------------------------------------------------
// State model
// ----------------------------------------------------------------------

/// Screen state (§4.2).
///
/// The former `Failed` variant has been removed. Errors are expressed as toasts.
/// Since v1.16.0 the screen state is simplified to three variants:
///
/// - `Empty`: waiting for file input. Drop zone shown.
/// - `Converting`: conversion in progress after file input.
///   Prior to v1.15 this was split into `Importing` and `Exporting`;
///   v1.16 unified them into a single in-memory conversion step.
/// - `Result`: conversion complete. Asset cards and download buttons shown.
///
/// The former `Preview` screen (context preview with surface picker) was removed
/// in v1.16.0. The new flow is drop → convert → inspect result.
/// A collapsible preview panel remains on the Result screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Empty,
    Converting,
    Result,
}

/// Application-wide state (§4.1).
#[derive(Debug)]
pub struct AppState {
    pub screen: Screen,
    pub theme: ThemeMode,
    /// Vestigial since v1.22.0. The Customize page is now a nav page;
    /// this field is reset to `false` on `CloseModals` and `EditCancelled`
    /// but never set to `true` by any handler. Kept for future use.
    pub advanced_open: bool,
    pub source_path: Option<PathBuf>,
    pub source_asset: Option<SourceAsset>,
    pub preview: Option<PreviewProfile>,
    /// Resized raster cache for context preview rendering (§5.2).
    /// Rebuilt when the source asset is loaded or the resize algorithm changes.
    pub preview_cache: Option<PreviewCache>,
    pub export_plan: ExportPlan,
    pub busy: bool,
    /// snora toast queue. All error and success notifications go through this.
    pub toasts: Vec<Toast<Message>>,
    /// Monotonic counter used to assign unique IDs to toasts.
    pub next_toast_id: u64,

    // v1.3.0: text buffer for the custom-size input in the Customize page.
    // Transient UI state: belongs in the app layer, not in logolig (core).
    pub png_size_input: String,
    pub ico_size_input: String,

    // v1.4.0: settings persistence store. Loaded at startup via load_or_default();
    // updated immediately on every settings change.
    //
    // Kept as `Option<>` so that a storage-init failure degrades gracefully:
    // the app continues to work; only persistence is disabled. E.g. if the
    // config directory is read-only.
    // `locale` is stored in PersistedSettings for i18n (v1.5) even though
    // v1.4 only reads and writes it without applying it.
    pub store: Option<crate::native_store::NativeStore>,

    // v1.5.0: i18n
    /// Active translator. Swap this with `Translator::for_locale(new)` on locale
    /// change; the entire UI reflects the new language after one repaint.
    pub translator: Translator,
    /// User-selected locale override. `None` means use the OS locale.
    /// Changed via the Settings page; persisted in `PersistedSettings.locale`.
    pub locale_override: Option<Locale>,

    // v1.7.0: transparency audit
    /// Transparency status of the most recently loaded image.
    /// `None` means no image is loaded or the audit has not run yet.
    /// Used to prevent duplicate warning toasts for the same image.
    pub transparency: Option<logolig::TransparencyReport>,
    // v1.10.0: `preview_checker: bool` was replaced by a PreviewContext variant
    // to prevent meaningless combined states (e.g. tab + checker simultaneously).

    // v1.17.0: `advanced_groups` (AdvancedGroupExpansion) removed;
    // Settings became flat in v1.17.0; accordion state no longer needed.

    // ---------------------------------------------------------------
    // v1.16.0: state for the new screen structure (Empty / Converting / Result)
    // ---------------------------------------------------------------
    /// In-memory conversion result.
    ///
    /// `Some` only while on the Result screen. Cleared on new file input;
    /// populated when conversion completes. Download and ZIP buttons read
    /// byte slices from here and write them via `rfd`.
    ///
    /// Prior to v1.16 conversion wrote directly to disk so no in-memory state
    /// was needed. v1.16 made conversion memory-only; the user triggers saves
    /// explicitly.
    pub result_assets: Option<crate::result::ResultAssets>,

    /// Whether the collapsible preview panel on the Result screen is open.
    /// Session-only; not persisted.
    pub result_preview_open: bool,

    // ---------------------------------------------------------------
    // v1.26.1: drag-and-drop feedback state
    // ---------------------------------------------------------------
    /// Whether an OS file drag is currently hovering over the window.
    ///
    /// This is visual feedback only. The actual import still starts from
    /// `Message::FileDropped(path)`. Some Linux Wayland compositors do not
    /// deliver native file-drop events through iced/winit; the drop zone is
    /// also a large click target so users always have an accessible fallback.
    pub file_drag_hovering: bool,

    // ---------------------------------------------------------------
    // v1.17.0 → v1.22.0: window state
    // ---------------------------------------------------------------
    /// Current window size in logical pixels.
    ///
    /// Used for the mobile breakpoint check (`is_mobile`).
    /// The former use to compute the Right Sheet width is no longer relevant;
    /// the Customize page now spans the full body width.

    ///
    /// Defaults to 1280×720; updated by `Message::WindowResized`.
    pub window_size: iced::Size<f32>,

    /// Whether the 'Advanced extras' collapsible in the Customize page is open.
    /// Groups the less-common settings (apple-touch, HTML snippet, web manifest,
    /// monochrome, resize algorithm, vectorize_on_raster). Session-only.
    pub advanced_extras_open: bool,

    // ---------------------------------------------------------------
    // v1.18.0 → v1.22.0: side nav
    // ---------------------------------------------------------------

    // ---------------------------------------------------------------
    /// Currently selected side-nav page.
    ///
    /// - `Home`      — core app flow (drop zone / converting / result).
    /// - `Customize` — output settings (replaces the former right-side drawer).
    /// - `Settings`  — language and theme (replaces the former popup pickers).
    pub nav_page: NavPage,
}

/// v1.22.0: side-nav page discriminant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NavPage {
    /// Main app screen (drop zone → converting → result).
    #[default]
    Home,
    /// Output settings (PNG sizes, ICO, apple-touch, monochrome, etc.).
    Customize,
    /// Language and theme selection.
    Settings,
}

// v1.17.0: AdvancedGroupExpansion / AdvancedGroup removed; settings are flat now.

impl AppState {
    /// Boot function. Loads persisted settings via NativeStore and applies them to AppState.
    ///
    /// On failure shows an error toast and falls back to a default AppState.
    /// The app remains functional even if persistence is unavailable (§ABDD).
    ///
    /// ## i18n initialisation (v1.5.0)
    ///
    /// 1. Use `PersistedSettings.locale` (BCP-47 tag, e.g. "en") if present.
    /// 2. Otherwise detect the OS locale via `sys-locale`.
    /// 3. Fall back to English if neither is supported.
    ///
    /// The resolved Translator is stored in `state.translator`; views call
    /// `state.translator.t(MessageKey::...)` to get localised strings.
    pub fn boot() -> Self {
        let mut state = Self::default();
        let store = crate::native_store::NativeStore::new();
        let mut persisted_locale_tag: Option<String> = None;
        match store.load_or_default() {
            Ok(persisted) => {
                state.export_plan = persisted.export_plan;
                state.theme = persisted.theme;
                persisted_locale_tag = persisted.locale.clone();
                state.store = Some(store);
            }
            Err(err) => {
                // Storage init failure is non-fatal — the app continues.
                // Translator is still default (English) at this point.
                // Once the locale is resolved the next repaint updates all UI strings.
                // This toast stays in English until it expires.
                let title = state.translator.t(MessageKey::ToastSettingsLoadFailedTitle);
                let body = state.translator.t_args(
                    MessageKey::ToastSettingsLoadFailedBody,
                    &[("error", &err.to_string())],
                );
                push_warning_toast(&mut state, &title, &body);
                // store stays None; subsequent persist_settings() calls are no-ops.
            }
        }

        // Locale resolution:
        //   1. PersistedSettings.locale (user override) takes priority.
        //   2. Fall back to OS detection.
        //   3. Use English if nothing resolves.
        let resolved_locale = persisted_locale_tag
            .as_deref()
            .and_then(Locale::from_bcp47)
            .map(|loc| (Some(loc), loc))
            .unwrap_or_else(|| (None, detect_system_locale()));
        state.locale_override = resolved_locale.0;
        state.translator = Translator::for_locale(resolved_locale.1);

        state
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            screen: Screen::default(),
            theme: ThemeMode::default(),
            advanced_open: false,
            source_path: None,
            source_asset: None,
            preview: None,
            preview_cache: None,
            export_plan: ExportPlan::default(),
            busy: false,
            toasts: Vec::new(),
            next_toast_id: 0,
            png_size_input: String::new(),
            ico_size_input: String::new(),
            store: None,
            translator: Translator::default(),
            locale_override: None,
            transparency: None,
            // v1.16.0
            result_assets: None,
            result_preview_open: false,
            // v1.26.1
            file_drag_hovering: false,
            // v1.17.0
            window_size: iced::Size::new(1280.0, 720.0),
            advanced_extras_open: false,
            // v1.18.0 → v1.22.0
            nav_page: NavPage::Home,
        }
    }
}

// ----------------------------------------------------------------------
// Messages
// ----------------------------------------------------------------------

/// All events fired by the UI and service layers (§4.3).
///
/// `snora::AppLayout<Element, Message>` requires `Message: Clone`, so
/// every variant must be cloneable.
#[allow(dead_code)]
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
    IngestCompleted(Result<SourceAsset, AppError>),

    // Preview
    PreviewBuilt(Result<PreviewCache, AppError>),
    PreviewContextSelected(PreviewContext),
    PreviewBackgroundSelected(ThemeMode),

    // v1.16.0: in-memory conversion complete.
    /// Conversion finished; the ResultAssets bundle is ready.
    /// Distinct from the former `Message::ExportCompleted` (which wrote to disk);
    /// this variant only places the asset bundle into app state.
    /// The user triggers the actual write later via individual or ZIP download.
    ConvertCompleted(Result<crate::result::ResultAssets, logolig::AppError>),

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
    /// No-op placeholder for partially-implemented UI toggles.
    NoOp,

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
    /// Navigate to the Customize page (replaces the former settings-drawer toggle).
    AdvancedToggled,
    AlgorithmChanged(ResizeAlgorithm),
    /// v1.2.0: toggle SVG output on/off.
    IncludeSvgToggled(bool),
    /// v1.2.0: toggle raster-to-vector conversion on/off.
    VectorizeOnRasterToggled(bool),

    // v1.3.0: Customize page editing UI
    /// Whether to output favicon.ico.
    IncludeIcoToggled(bool),
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
    /// ICO size set input text changed.
    IcoSizeInputChanged(String),
    /// ICO size added.
    IcoSizeAddRequested,
    /// ICO size removed.
    IcoSizeRemoveRequested(u32),

    // v1.4.1: vtracer preset switch + ExportPlan reset
    /// vtracer preset changed (Sharp / Default / PhotoRich).
    VtracerPresetChanged(logolig::VtracerPreset),
    /// Resets ExportPlan to defaults. Does not affect theme or locale.
    ExportPlanResetRequested,

    // v1.5.0: i18n
    /// Locale selected. `None` resets to the OS default.
    LocaleChanged(Option<Locale>),

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
    SvgAnalyzed(PathBuf, logolig::SvgReport),
}

// ----------------------------------------------------------------------
// Entry point
// ----------------------------------------------------------------------

/// Entry function called from `main`.
///
/// iced 0.14's `application` takes a boot function as its first argument.
/// The window title is supplied via the `.title(...)` builder method.
/// v1.5.0: title is localised via `state.translator.t(MessageKey::AppTitle)`
/// so it updates automatically on locale change.
pub fn run() -> iced::Result {
    iced::application(AppState::boot, update, view)
        .font(snora::lucide::LUCIDE_FONT_BYTES)
        .window(crate::window::settings())
        .title(window_title)
        .theme(theme)
        .subscription(subscription)
        .run()
}

fn window_title(state: &AppState) -> String {
    state.translator.t(MessageKey::AppTitle)
}

/// Resolves the iced `Theme` from current app state.
///
/// v1.14.0: exposed as `pub(crate)` so UI modules can resolve theme-palette colours.
pub(crate) fn resolve_theme(state: &AppState) -> Theme {
    match state.theme {
        // System: will query OS theme in a future step; currently falls back to Light.
        ThemeMode::System | ThemeMode::Light => Theme::Light,
        ThemeMode::Dark => Theme::Dark,
    }
}

/// Thin wrapper matching the `application().theme()` signature.
fn theme(state: &AppState) -> Theme {
    resolve_theme(state)
}

/// Returns `true` when the window is narrow enough to be treated as mobile (v1.20.0).
///
/// Window width below `MOBILE_BREAKPOINT_PX` (768 px) is considered mobile.
/// 768 px matches Bootstrap's `md` breakpoint and the original iPad mini portrait
/// width — a threshold widely used in web accessibility work.
///
/// Used throughout the UI: sidebar vs. bottom-nav selection, result-view column count,
/// Customize page width, and header padding.
///
/// On first frame `AppState::window_size` defaults to 1280×720, so the check
/// initially returns `false` (desktop). A resize event corrects it shortly after.
pub(crate) fn is_mobile(state: &AppState) -> bool {
    state.window_size.width < MOBILE_BREAKPOINT_PX
}

/// Mobile breakpoint in logical pixels. See [`is_mobile`] for details.
pub(crate) const MOBILE_BREAKPOINT_PX: f32 = 768.0;

fn subscription(state: &AppState) -> Subscription<Message> {
    // Combine two subscriptions:
    //   (a) snora Toast tick — auto-dismisses transient toasts
    //   (b) iced runtime events — window resize, native file drag/drop and global shortcuts.
    //
    // v1.26.1: use `iced::event::listen_with` instead of `window::events()`.
    // It receives the same window events but keeps all event routing in one
    // runtime-event subscription and lets us observe FileHovered /
    // FilesHoveredLeft for visible drop-zone feedback.
    let toasts = snora::toast::subscription(&state.toasts, || Message::ToastTick);

    let runtime_events = iced::event::listen_with(|event, status, _id| match event {
        // RFC 019 phase 2: global shortcuts. A key a widget used (text typed into a field,
        // its own editing combinations) is not a shortcut.
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key,
            physical_key,
            modifiers,
            repeat,
            ..
        }) if status == iced::event::Status::Ignored => {
            crate::shortcuts::shortcut_for(&key, physical_key, modifiers, repeat)
                .map(Message::ShortcutPressed)
        }
        iced::Event::Window(iced::window::Event::FileHovered(_)) => Some(Message::FileDragHovered),
        iced::Event::Window(iced::window::Event::FilesHoveredLeft) => Some(Message::FileDragLeft),
        iced::Event::Window(iced::window::Event::FileDropped(path)) => {
            Some(Message::FileDropped(path))
        }
        iced::Event::Window(iced::window::Event::Resized(size)) => {
            Some(Message::WindowResized(size))
        }
        // v1.17.0: also respond to Opened so window_size is set at startup,
        // not just after the first explicit resize.
        iced::Event::Window(iced::window::Event::Opened { size, .. }) => {
            Some(Message::WindowResized(size))
        }
        _ => None,
    });

    Subscription::batch([toasts, runtime_events])
}

fn view(state: &AppState) -> Element<'_, Message> {
    crate::shell::view(state)
}

// ----------------------------------------------------------------------
// Update (state transitions)
// ----------------------------------------------------------------------

fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::FileDropped(path) | Message::FilePicked(Some(path)) => {
            state.file_drag_hovering = false;
            start_ingest(state, path)
        }
        Message::FileDragHovered => {
            state.file_drag_hovering = true;
            Task::none()
        }
        Message::FileDragLeft => {
            state.file_drag_hovering = false;
            Task::none()
        }
        Message::FilePicked(None) => {
            // File picker cancelled — no state change.
            Task::none()
        }
        Message::PickFileRequested => {
            // Launch the native file picker as a Task (§12 accessibility alternative).
            // Result arrives as Message::FilePicked(Option<PathBuf>).
            // v1.12.0: re-select also reuses this message (keeps current screen,
            // cancelling returns to whatever screen was active).
            crate::task_queue::pick_file_task()
        }

        // v1.12.0: back navigation.
        //
        // v1.16.0: returns to Empty from any screen (Result, Converting, etc.).

        //
        // Persisted settings (export_plan, theme, locale) are preserved.
        // Resets transient UI state (nav page, extras open, etc.).
        Message::EditCancelled => {
            state.file_drag_hovering = false;
            // v1.24.0: result_assets and source_asset are intentionally kept so
            // the history card can offer "View results →" on the Empty screen.
            // Clearing them is deferred until a new file is ingested.
            state.preview = None;
            state.preview_cache = None;
            state.transparency = None;
            state.result_preview_open = false;
            state.screen = Screen::Empty;
            state.advanced_open = false;
            Task::none()
        }
        Message::ShowLastResultRequested => {
            // Return to the Result screen without re-converting.
            // result_assets is always Some here because the history card
            // only appears when it is Some.
            state.screen = Screen::Result;
            Task::none()
        }
        Message::IngestCompleted(Ok(asset)) => {
            // RFC 010: decide before `asset` is moved into the state.
            let non_square = is_non_square(asset.intrinsic_size);
            // RFC 015 phase 2a: only SVG sources are analysed for parts that are not shown.
            let is_svg = asset.kind == logolig::SourceKind::Svg;
            // Arc the asset so it can be moved into the async preview task.
            // SourceAsset is Clone (Arc<[u8]>-backed) so this is cheap.
            let arc = std::sync::Arc::new(asset.clone());
            state.source_asset = Some(asset);
            state.preview = Some(PreviewProfile::default());
            state.preview_cache = None;
            // v1.24.0: clear the previous history now that a new file is being
            // processed. The history card (if shown) disappears at this point.
            state.result_assets = None;
            // v1.16.0: ingest complete → auto-convert. Prior to v1.15 the app
            // waited on the Preview screen; v1.16 converts immediately.
            // Two parallel tasks are launched:
            //   (a) preview build (for the collapsible preview panel)
            //   (b) conversion (in-memory, assembles ResultAssets)
            // (b) completion transitions to Screen::Result.
            state.screen = Screen::Converting;
            state.busy = false;
            // v1.7.0: reset transparency status for the new image.
            // Audit runs in PreviewBuilt; this prevents:
            // - stale warnings carrying over on re-load
            // - duplicate warning toasts for the same image
            state.transparency = None;
            // RFC 010: one advisory per loaded image when the logo will be padded.
            if non_square {
                push_non_square_warning(state);
            }
            // Launch (a) and (b) in parallel; for an SVG also (c) the analysis for the advisory.
            let mut tasks = vec![
                crate::task_queue::build_preview_task(arc.clone(), state.export_plan.algorithm),
                crate::task_queue::convert_task(arc.clone(), state.export_plan.clone()),
            ];
            if is_svg {
                tasks.push(crate::task_queue::analyze_svg_task(arc));
            }
            Task::batch(tasks)
        }
        Message::SvgAnalyzed(path, report) => {
            // Stale result (another file was loaded since): ignore.
            let is_current = state
                .source_asset
                .as_ref()
                .is_some_and(|asset| asset.path == path);
            if is_current && (report.has_text || report.ignored_external_images > 0) {
                push_svg_parts_warning(state, report);
            }
            Task::none()
        }
        Message::IngestCompleted(Err(err)) => fail(state, err),

        Message::PreviewBuilt(Ok(cache)) => {
            // Verify the cache matches the current source and algorithm.
            // Discard stale cache (wrong source or algorithm).
            let asset_path = state.source_asset.as_ref().map(|a| a.path.clone());
            let still_valid = asset_path.as_ref() == Some(&cache.source_path)
                && cache.algorithm == state.export_plan.algorithm;
            if still_valid {
                // v1.7.0: transparency audit
                // Run the transparency audit on the first cache built for this image.
                // Skip if already audited (state.transparency is Some) to avoid
                // duplicate warning toasts when the preview is rebuilt (e.g. after
                // changing the resize algorithm). The audit itself is idempotent,
                // but showing it once per image is the right UX.
                let needs_audit = state.transparency.is_none();
                if needs_audit {
                    let report = audit_transparency(&cache.icon_120);
                    state.transparency = Some(report);
                    if report.needs_warning() {
                        // v1.11.0: JPEG cannot carry alpha so the audit always returns
                        // FullyOpaque. This is a format constraint, not a user mistake;
                        // show the JPEG-specific educational warning instead of the
                        // generic fully-opaque warning.

                        let is_jpeg = state
                            .source_asset
                            .as_ref()
                            .map(|a| a.kind == logolig::SourceKind::Jpeg)
                            .unwrap_or(false);
                        if is_jpeg {
                            push_jpeg_input_warning(state);
                        } else {
                            push_transparency_warning(state, report);
                        }
                    }
                }
                state.preview_cache = Some(cache);
            }
            Task::none()
        }
        Message::PreviewBuilt(Err(err)) => {
            // Preview build failure is non-fatal (source data is intact).
            // Stay on the current screen; notify via toast only.
            push_error_toast(state, err);
            Task::none()
        }
        Message::PreviewContextSelected(ctx) => {
            if let Some(p) = state.preview.as_mut() {
                p.context = ctx;
            }
            Task::none()
        }
        Message::PreviewBackgroundSelected(theme) => {
            if let Some(p) = state.preview.as_mut() {
                p.background = theme;
            }
            Task::none()
        }

        // v1.16.0: in-memory conversion complete
        Message::ConvertCompleted(Ok(assets)) => {
            let large_trace = is_large_traced_svg(state, &assets);
            state.result_assets = Some(assets);
            state.screen = Screen::Result;
            state.busy = false;
            // RFC 030: once, when the conversion has just produced it.
            if large_trace {
                push_svg_large_advisory(state);
            }
            Task::none()
        }
        Message::ConvertCompleted(Err(err)) => fail(state, err),

        // v1.16.0: download buttons
        Message::DownloadOneRequested(idx) => {
            // Extract the filename and open a save dialog with the right extension.
            let Some(assets) = state.result_assets.as_ref() else {
                return Task::none();
            };
            let Some(item) = assets.items.get(idx) else {
                return Task::none();
            };
            crate::task_queue::pick_save_one_task(idx, item.file_name.clone())
        }
        Message::DownloadAllRequested => crate::task_queue::pick_save_all_task(),
        Message::DownloadOneTargetPicked(_, None) => Task::none(),
        Message::DownloadOneTargetPicked(idx, Some(path)) => {
            let Some(assets) = state.result_assets.as_ref() else {
                return Task::none();
            };
            let Some(item) = assets.items.get(idx) else {
                return Task::none();
            };
            let bytes = item.bytes.clone();
            crate::task_queue::write_one_task(path, bytes)
        }
        Message::DownloadAllTargetPicked(None) => Task::none(),
        Message::DownloadAllTargetPicked(Some(path)) => {
            let Some(assets) = state.result_assets.as_ref() else {
                return Task::none();
            };
            // ZIP assembly runs as a task to avoid blocking the UI thread.
            let items_clone = assets.items.clone();
            crate::task_queue::write_zip_task(path, items_clone)
        }
        Message::DownloadOneCompleted(Ok(path)) => {
            let title = state.translator.t(MessageKey::ToastExportTitle);
            let body = format!("{}", path.display());
            push_long_success_toast(state, &title, &body);
            Task::none()
        }
        Message::DownloadOneCompleted(Err(err)) => {
            push_error_toast(state, err);
            Task::none()
        }
        Message::DownloadAllCompleted(Ok(path)) => {
            let title = state.translator.t(MessageKey::ToastExportTitle);
            let body = format!("{}", path.display());
            push_long_success_toast(state, &title, &body);
            Task::none()
        }
        Message::DownloadAllCompleted(Err(err)) => {
            push_error_toast(state, err);
            Task::none()
        }

        // v1.16.0: toggle the collapsible preview panel on the Result screen
        Message::ResultPreviewToggled => {
            state.result_preview_open = !state.result_preview_open;
            Task::none()
        }

        // v1.17.0: window resize → update window_size
        Message::WindowResized(size) => {
            state.window_size = size;
            Task::none()
        }
        // v1.17.0: toggle the Advanced extras collapsible
        Message::AdvancedExtrasToggled => {
            state.advanced_extras_open = !state.advanced_extras_open;
            Task::none()
        }
        // v1.17.0: PNG preset size checkbox toggled
        Message::PngPresetSizeAdded(size) => {
            if state.export_plan.add_png_size(size) {
                persist_settings(state);
            }
            Task::none()
        }
        // v1.17.0: placeholder toggle — currently a no-op
        Message::NoOp => Task::none(),

        // v1.22.0: side-nav page switch
        Message::NavPageSelected(page) => {
            state.nav_page = page;
            Task::none()
        }
        Message::ShortcutPressed(shortcut) => handle_shortcut(state, shortcut),
        Message::LocalePicked(opt) => {
            state.locale_override = opt;
            let resolved = opt.unwrap_or_else(detect_system_locale);
            state.translator = Translator::for_locale(resolved);
            persist_settings(state);
            Task::none()
        }
        Message::ThemePicked(theme) => {
            state.theme = theme;
            persist_settings(state);
            Task::none()
        }

        Message::AdvancedToggled => {
            // Navigate to the Customize page.
            state.nav_page = NavPage::Customize;
            Task::none()
        }
        Message::AlgorithmChanged(alg) => {
            state.export_plan.algorithm = alg;
            persist_settings(state);
            // Resize algorithm changed — rebuild the preview cache.
            // Discard stale cache (UI shows a loading state while cache is None).

            state.preview_cache = None;
            if let Some(asset) = state.source_asset.as_ref() {
                let arc = std::sync::Arc::new(asset.clone());
                crate::task_queue::build_preview_task(arc, alg)
            } else {
                Task::none()
            }
        }
        Message::IncludeSvgToggled(on) => {
            // Output plan change only; does not affect the preview.
            state.export_plan.include_svg = on;
            persist_settings(state);
            Task::none()
        }
        Message::VectorizeOnRasterToggled(on) => {
            state.export_plan.vectorize_on_raster = on;
            persist_settings(state);
            Task::none()
        }

        // -----------------------------------------------------------------
        // v1.3.0: Customize page editing UI handlers
        // -----------------------------------------------------------------
        Message::IncludeIcoToggled(on) => {
            state.export_plan.include_ico = on;
            persist_settings(state);
            Task::none()
        }
        Message::IncludeAppleTouchToggled(on) => {
            state.export_plan.include_apple_touch = on;
            persist_settings(state);
            Task::none()
        }
        Message::IncludeHtmlSnippetToggled(on) => {
            state.export_plan.include_html_snippet = on;
            persist_settings(state);
            Task::none()
        }
        Message::PngSizeInputChanged(s) => {
            // Lightweight digit filter — digits pass through; non-digits are ignored.
            // Whitespace from pastes is stripped at parse time.
            state.png_size_input = s;
            Task::none()
        }
        Message::PngSizeAddRequested => {
            let raw = state.png_size_input.trim().to_string();
            match parse_size(&raw, logolig::PNG_SIZE_MIN, logolig::PNG_SIZE_MAX) {
                Ok(size) => {
                    if state.export_plan.add_png_size(size) {
                        state.png_size_input.clear();
                        persist_settings(state);
                    } else {
                        // parse_size already caught out-of-range; duplicates reach here.
                        let title = state.translator.t(MessageKey::ToastSizeAlreadyInSetTitle);
                        let body = state.translator.t_args(
                            MessageKey::ToastPngSizeAlreadyInSetBody,
                            &[("size", &size.to_string())],
                        );
                        push_warning_toast(state, &title, &body);
                    }
                }
                Err(SizeParseError::Empty) => {
                    // Ignore Add on empty input (prevents accidental Enter-key repeats).
                }
                Err(SizeParseError::NotANumber) => {
                    let title = state.translator.t(MessageKey::ToastInvalidSizeTitle);
                    let body = state
                        .translator
                        .t_args(MessageKey::ToastInvalidSizeBody, &[("input", &raw)]);
                    push_warning_toast(state, &title, &body);
                }
                Err(SizeParseError::OutOfRange { min, max }) => {
                    let title = state.translator.t(MessageKey::ToastSizeOutOfRangeTitle);
                    let body = state.translator.t_args(
                        MessageKey::ToastPngSizeOutOfRangeBody,
                        &[("min", &min.to_string()), ("max", &max.to_string())],
                    );
                    push_warning_toast(state, &title, &body);
                }
            }
            Task::none()
        }
        Message::PngSizeRemoveRequested(size) => {
            if state.export_plan.remove_png_size(size) {
                persist_settings(state);
            }
            Task::none()
        }
        Message::IcoSizeInputChanged(s) => {
            state.ico_size_input = s;
            Task::none()
        }
        Message::IcoSizeAddRequested => {
            let raw = state.ico_size_input.trim().to_string();
            match parse_size(&raw, logolig::ICO_SIZE_MIN, logolig::ICO_SIZE_MAX) {
                Ok(size) => {
                    if state.export_plan.add_ico_size(size) {
                        state.ico_size_input.clear();
                        persist_settings(state);
                    } else {
                        let title = state.translator.t(MessageKey::ToastSizeAlreadyInSetTitle);
                        let body = state.translator.t_args(
                            MessageKey::ToastIcoSizeAlreadyInSetBody,
                            &[("size", &size.to_string())],
                        );
                        push_warning_toast(state, &title, &body);
                    }
                }
                Err(SizeParseError::Empty) => {}
                Err(SizeParseError::NotANumber) => {
                    let title = state.translator.t(MessageKey::ToastInvalidSizeTitle);
                    let body = state
                        .translator
                        .t_args(MessageKey::ToastInvalidSizeBody, &[("input", &raw)]);
                    push_warning_toast(state, &title, &body);
                }
                Err(SizeParseError::OutOfRange { min, max }) => {
                    let title = state.translator.t(MessageKey::ToastSizeOutOfRangeTitle);
                    let body = state.translator.t_args(
                        MessageKey::ToastIcoSizeOutOfRangeBody,
                        &[("min", &min.to_string()), ("max", &max.to_string())],
                    );
                    push_warning_toast(state, &title, &body);
                }
            }
            Task::none()
        }
        Message::IcoSizeRemoveRequested(size) => {
            if state.export_plan.remove_ico_size(size) {
                persist_settings(state);
            }
            Task::none()
        }

        // -----------------------------------------------------------------
        // v1.4.1: vtracer preset + ExportPlan reset
        // -----------------------------------------------------------------
        Message::VtracerPresetChanged(preset) => {
            state.export_plan.vtracer_preset = preset;
            persist_settings(state);
            Task::none()
        }
        Message::ExportPlanResetRequested => {
            // Reset ExportPlan only. theme, locale, and nav state are unaffected
            // (§v1.4.1 reset scope decision).
            state.export_plan = ExportPlan::default();
            // Also clear text input buffers so stale values don't reappear.
            state.png_size_input.clear();
            state.ico_size_input.clear();
            // Algorithm may have changed (reset restores Lanczos3); rebuild preview.
            // Discard stale cache; rebuild with the restored algorithm.
            state.preview_cache = None;
            persist_settings(state);
            // Transient success toast confirms the reset happened.
            let title = state.translator.t(MessageKey::ToastResetTitle);
            let body = state.translator.t(MessageKey::ToastResetBody);
            push_success_toast(state, &title, &body);
            // Rebuild preview cache if a source is loaded.
            if let Some(asset) = state.source_asset.as_ref() {
                let arc = std::sync::Arc::new(asset.clone());
                crate::task_queue::build_preview_task(arc, state.export_plan.algorithm)
            } else {
                Task::none()
            }
        }

        // -----------------------------------------------------------------
        // v1.5.0: locale change
        // -----------------------------------------------------------------
        Message::LocaleChanged(opt) => {
            // None = revert to OS locale; Some(loc) = use that locale.
            state.locale_override = opt;
            let resolved = opt.unwrap_or_else(detect_system_locale);
            state.translator = Translator::for_locale(resolved);
            persist_settings(state);
            Task::none()
        }

        // v1.19.0: old LocaleCycled / AppCloseRequested handlers removed.

        // -----------------------------------------------------------------
        // v1.10.3 → v1.17.0: accordion handlers removed; settings are flat now.
        // -----------------------------------------------------------------

        // -----------------------------------------------------------------
        // v1.7.0 → v1.10.0: transparency checker merged into PreviewContextSelected.
        // Dedicated message removed.
        // -----------------------------------------------------------------

        // -----------------------------------------------------------------
        // v1.8.0: web manifest settings
        // -----------------------------------------------------------------
        Message::IncludeWebManifestToggled(on) => {
            // On: insert WebManifestSettings::default(). Off: set to None.
            // Any user-entered values are discarded on toggle-off (simple state model).
            // Preserving values across toggle-off can be added in a future version.
            state.export_plan.web_manifest = if on {
                Some(logolig::WebManifestSettings::default())
            } else {
                None
            };
            persist_settings(state);
            Task::none()
        }
        Message::WebManifestNameChanged(s) => {
            if let Some(m) = state.export_plan.web_manifest.as_mut() {
                m.name = s;
                persist_settings(state);
            }
            Task::none()
        }
        Message::WebManifestShortNameChanged(s) => {
            if let Some(m) = state.export_plan.web_manifest.as_mut() {
                m.short_name = s;
                persist_settings(state);
            }
            Task::none()
        }
        Message::WebManifestThemeColorChanged(s) => {
            // No live validation — interrupting the user mid-type is poor UX.
            // `is_valid_color` is validated at export time, not here,
            // so partial values like #FF or #FFFFF are accepted while typing.
            if let Some(m) = state.export_plan.web_manifest.as_mut() {
                m.theme_color = s;
                persist_settings(state);
            }
            Task::none()
        }
        Message::WebManifestBackgroundColorChanged(s) => {
            if let Some(m) = state.export_plan.web_manifest.as_mut() {
                m.background_color = s;
                persist_settings(state);
            }
            Task::none()
        }

        // -----------------------------------------------------------------
        // v1.9.0: monochrome output set
        // -----------------------------------------------------------------
        Message::IncludeMonochromeToggled(on) => {
            // Simple bool flag. Same persist_settings pattern as include_ico
            // and include_apple_touch.
            //
            // No effect on the preview cache — monochrome output is generated
            // only at export time and has no preview representation.

            state.export_plan.monochrome = on;
            persist_settings(state);
            Task::none()
        }

        // -----------------------------------------------------------------
        // v1.26.0: Microsoft app logo output
        // -----------------------------------------------------------------
        Message::IncludeMicrosoftAppLogosToggled(on) => {
            // Advanced, opt-in output group. Like monochrome, it only affects
            // generated result assets; preview cache is unchanged.
            state.export_plan.include_microsoft_app_logos = on;
            persist_settings(state);
            Task::none()
        }

        // -----------------------------------------------------------------
        // v1.21.0: keep-transparency toggle
        // -----------------------------------------------------------------
        Message::KeepTransparencyToggled(on) => {
            // Persisted (Q4-a). Backward compat with older settings JSON is handled
            // by the struct-level `#[serde(default)]` on ExportPlan; missing fields
            // default to true.
            //
            // The preview cache does not reflect keep_transparency in v1.21.0.
            // The setting's effect is visible in Result card thumbnails and
            // downloaded files only.

            state.export_plan.keep_transparency = on;
            persist_settings(state);
            Task::none()
        }

        // v1.19.0: ExportRequested / ExportDirPicked / ExportCompleted handlers removed.
        // v1.16.0 moved to the ConvertCompleted + per-asset download path.
        Message::ToastTick => {
            snora::toast::sweep_expired(&mut state.toasts, Instant::now());
            Task::none()
        }
        Message::DismissToast(id) => {
            state.toasts.retain(|t| t.id != id);
            Task::none()
        }
        Message::CloseModals => {
            // Reset advanced_open for safety (vestigial but harmless since v1.22.0).
            state.advanced_open = false;
            Task::none()
        }
    }
}

/// Shared helper: starts an ingest task for the given file path.
///
/// v1.16.0: the former Screen::Importing was merged into Screen::Converting.
/// All phases (ingest, preview build, asset bundle) now share the Converting state.
fn start_ingest(state: &mut AppState, path: PathBuf) -> Task<Message> {
    state.file_drag_hovering = false;
    state.source_path = Some(path.clone());
    state.screen = Screen::Converting;
    state.busy = true;
    // Discard stale preview cache and asset bundle; they will be rebuilt
    // when the new ingest completes.
    state.preview_cache = None;
    state.result_assets = None;
    state.result_preview_open = false;
    crate::task_queue::ingest_task(path)
}

/// Common error handler.
///
/// - clears the busy flag
/// - returns to Result if a source is loaded, otherwise Empty
///   (v1.16: the former Preview state was removed)
/// - pushes a persistent error toast (stays until dismissed)
fn fail(state: &mut AppState, err: AppError) -> Task<Message> {
    state.busy = false;
    state.screen = if state.result_assets.is_some() {
        Screen::Result
    } else {
        Screen::Empty
    };
    push_error_toast(state, err);
    Task::none()
}

/// Apply a keyboard shortcut (RFC 019 phase 2).
///
/// A shortcut acts exactly when the control it stands for is available to the mouse, and it
/// then goes through `update` as the message that control sends, so there is one code path for
/// the action and one set of state checks (this also keeps a second ingest from starting while
/// one is running).
fn handle_shortcut(state: &mut AppState, shortcut: crate::shortcuts::Shortcut) -> Task<Message> {
    use crate::shortcuts::Shortcut;

    let on_home = state.nav_page == NavPage::Home;
    let message = match shortcut {
        // The drop zone, or Back and then the drop zone, is reachable.
        Shortcut::ChooseFile
            if on_home && matches!(state.screen, Screen::Empty | Screen::Result) =>
        {
            Message::PickFileRequested
        }
        // The "Download all (ZIP)" button is on the Result screen when there are assets.
        Shortcut::DownloadAll
            if on_home && state.screen == Screen::Result && state.result_assets.is_some() =>
        {
            Message::DownloadAllRequested
        }
        Shortcut::EscapeOrBack => match state.toasts.last() {
            // Toasts are pushed in order, so the last one is the newest.
            Some(toast) => Message::DismissToast(toast.id),
            None if on_home && state.screen == Screen::Result => Message::EditCancelled,
            None => return Task::none(),
        },
        Shortcut::Page(page) => Message::NavPageSelected(page),
        Shortcut::Help => Message::NavPageSelected(NavPage::Settings),
        Shortcut::ChooseFile | Shortcut::DownloadAll => return Task::none(),
    };
    update(state, message)
}

/// A traced `favicon.svg` larger than this (2 MB) earns one advisory toast (RFC 030).
/// Flat logos trace to about 10 KB; a photograph traced at the 512 px cap gives about 3 MB.
const SVG_ADVISORY_BYTES: usize = 2 * 1024 * 1024;

/// Whether `assets` holds a `favicon.svg` that was **traced from a raster** and is larger than
/// [`SVG_ADVISORY_BYTES`]. An SVG source is copied verbatim, not traced, so the advisory's
/// "looks like a photograph" would be false for it.
fn is_large_traced_svg(state: &AppState, assets: &crate::result::ResultAssets) -> bool {
    let traced = state
        .source_asset
        .as_ref()
        .is_some_and(|asset| asset.kind != logolig::SourceKind::Svg);
    traced
        && assets
            .items
            .iter()
            .any(|item| item.file_name == "favicon.svg" && item.bytes.len() > SVG_ADVISORY_BYTES)
}

/// Pushes the large-`favicon.svg` advisory (RFC 030).
fn push_svg_large_advisory(state: &mut AppState) {
    let title = state.translator.t(MessageKey::ToastSvgLargeTitle);
    let body = state.translator.t(MessageKey::ToastSvgLargeBody);
    push_warning_toast(state, &title, &body);
}

/// Pushes a persistent error toast. Stays until the user dismisses it.
/// v1.5.0: translates the error via `AppError::key()` + `args()`.
fn push_error_toast(state: &mut AppState, err: AppError) {
    let id = next_id(state);
    let body = state.translator.translate_error(&err);
    // Title is a generic 'operation failed' string.
    // Currently hardcoded to English; a dedicated MessageKey could be added later.

    state.toasts.push(
        Toast::new(
            id,
            ToastIntent::Error,
            "Operation failed",
            body,
            Message::DismissToast(id),
        )
        .persistent(),
    );
}

/// Pushes a short-lived success toast; its lifetime is proportional to its text (RFC 029).
fn push_success_toast(state: &mut AppState, title: &str, body: &str) {
    let id = next_id(state);
    state.toasts.push(
        Toast::new(
            id,
            ToastIntent::Success,
            title.to_string(),
            body.to_string(),
            Message::DismissToast(id),
        )
        .with_lifetime(crate::toast_time::lifetime_for(title, body)),
    );
}

/// Pushes a longer (7-second) transient success toast (v1.4.2).
///
/// v1.4.1 originally used a persistent toast that required manual dismiss.
/// User feedback showed persistent was overkill but 4 s was too short.
/// v1.4.2 settled on a 7-second transient.
///
/// 7 s rationale: snora's default 4 s suits short messages; export notifications
/// can include long paths and need extra read time. 10+ s feels intrusive,
/// so 7 s is the pragmatic middle ground.
///
/// Toast position is bottom-right (snora default). Improving this is tracked
/// as a future snora upstream request.
fn push_long_success_toast(state: &mut AppState, title: &str, body: &str) {
    let id = next_id(state);
    state.toasts.push(
        Toast::new(
            id,
            ToastIntent::Success,
            title.to_string(),
            body.to_string(),
            Message::DismissToast(id),
        )
        // 7 s is a minimum: a long saved path may need more (RFC 029).
        .with_lifetime(ToastLifetime::seconds(
            crate::toast_time::seconds_for(title, body).max(7),
        )),
    );
}

fn next_id(state: &mut AppState) -> u64 {
    let id = state.next_toast_id;
    state.next_toast_id += 1;
    id
}

// ----------------------------------------------------------------------
// v1.3.0: size input parsing + warning toast
// ----------------------------------------------------------------------

#[derive(Debug)]
enum SizeParseError {
    Empty,
    NotANumber,
    OutOfRange { min: u32, max: u32 },
}

/// Parses a digit string to `u32` and validates the range.
/// `add_*_size` also validates, but doing it here lets us show a specific
/// error (out-of-range / not a number / duplicate) to the user.
fn parse_size(raw: &str, min: u32, max: u32) -> Result<u32, SizeParseError> {
    if raw.is_empty() {
        return Err(SizeParseError::Empty);
    }
    let n: u32 = raw.parse().map_err(|_| SizeParseError::NotANumber)?;
    if n < min || n > max {
        return Err(SizeParseError::OutOfRange { min, max });
    }
    Ok(n)
}

/// Pushes a transient warning toast for input validation failures.
/// Sits between push_error_toast (persistent) and push_success_toast in severity.
fn push_warning_toast(state: &mut AppState, title: &str, body: &str) {
    let id = next_id(state);
    state.toasts.push(
        Toast::new(
            id,
            ToastIntent::Warning,
            title.to_string(),
            body.to_string(),
            Message::DismissToast(id),
        )
        .with_lifetime(crate::toast_time::lifetime_for(title, body)),
    );
}

/// v1.7.0: transparency audit warning. Shown when the loaded image is fully opaque
/// or fully transparent. Uses the standard transient lifetime — this is an advisory
/// notice, not a blocking error.
fn push_transparency_warning(state: &mut AppState, report: TransparencyReport) {
    let (title_key, body_key) = match report {
        TransparencyReport::FullyOpaque => (
            MessageKey::ToastFullyOpaqueTitle,
            MessageKey::ToastFullyOpaqueBody,
        ),
        TransparencyReport::FullyTransparent => (
            MessageKey::ToastFullyTransparentTitle,
            MessageKey::ToastFullyTransparentBody,
        ),
        TransparencyReport::HasTransparency => {
            // needs_warning() == false here; defensive no-op.
            return;
        }
    };
    let title = state.translator.t(title_key);
    let body = state.translator.t(body_key);
    push_warning_toast(state, &title, &body);
}

/// RFC 010: `true` when the intrinsic sides differ by more than 2 %
/// (`abs(w - h) * 100 > 2 * max(w, h)`). The tolerance avoids false alarms from
/// the `ceil()` rounding in `parse_svg_size`. Unknown size is never non-square.
fn is_non_square(size: Option<(u32, u32)>) -> bool {
    match size {
        Some((w, h)) => u64::from(w.abs_diff(h)) * 100 > 2 * u64::from(w.max(h)),
        None => false,
    }
}

/// RFC 010: educational warning for a non-square source. The logo is shown in
/// full with transparent padding, so the icon may look small. Transient, non-blocking.
fn push_non_square_warning(state: &mut AppState) {
    let title = state.translator.t(MessageKey::ToastNonSquareInputTitle);
    let body = state.translator.t(MessageKey::ToastNonSquareInputBody);
    push_warning_toast(state, &title, &body);
}

/// RFC 015 phase 2a: one combined warning for an SVG whose parts are not shown. The body holds
/// the applicable sentences (images first, then text), one per line, so there is never more than
/// one toast per loaded SVG. Transient and non-blocking.
fn push_svg_parts_warning(state: &mut AppState, report: logolig::SvgReport) {
    let title = state.translator.t(MessageKey::ToastSvgPartsTitle);
    let mut sentences: Vec<String> = Vec::with_capacity(2);
    if report.ignored_external_images > 0 {
        sentences.push(state.translator.t(MessageKey::ToastSvgImagesBody));
    }
    if report.has_text {
        sentences.push(state.translator.t(MessageKey::ToastSvgTextBody));
    }
    push_warning_toast(state, &title, &sentences.join("\n"));
}

/// v1.11.0: educational warning for JPEG input.
///
/// JPEG cannot carry alpha, so the v1.7 transparency audit always returns
/// FullyOpaque. Unlike the generic opaque warning (forgot to remove background),
/// this is a format constraint. A dedicated educational message suggests
/// using PNG for better favicon quality.
fn push_jpeg_input_warning(state: &mut AppState) {
    let title = state.translator.t(MessageKey::ToastJpegInputTitle);
    let body = state.translator.t(MessageKey::ToastJpegInputBody);
    push_warning_toast(state, &title, &body);
}

// ----------------------------------------------------------------------
// v1.4.0: settings persistence
// ----------------------------------------------------------------------

/// Assembles a `PersistedSettings` snapshot from the current `AppState`.
fn snapshot_persisted(state: &AppState) -> logolig::PersistedSettings {
    logolig::PersistedSettings {
        export_plan: state.export_plan.clone(),
        theme: state.theme,
        // v1.5.0: save the user locale override as a BCP-47 tag.
        // None means fall back to OS detection on next launch.
        locale: state.locale_override.map(|loc| loc.as_bcp47().to_string()),
    }
}

/// Saves settings immediately (eager-save strategy, §1.4.0).
///
/// No-op if `state.store` is `None` (storage init failed at startup).
/// Shows a warning toast on save failure; the app continues running.
///
/// Note: this is called on every settings change. The current
/// `PersistedSettings` payload is a few KB so I/O cost is negligible,
/// but a debounce / lazy-save strategy may be needed if it grows.
fn persist_settings(state: &mut AppState) {
    let Some(store) = state.store.as_ref() else {
        return;
    };
    let snapshot = snapshot_persisted(state);
    if let Err(err) = store.save(&snapshot) {
        // Use a transient warning (not persistent) to avoid flooding the UI.
        // v1.5.0: uses the active Translator.
        let title = state.translator.t(MessageKey::ToastSettingsSaveFailedTitle);
        let body = state.translator.t_args(
            MessageKey::ToastSettingsSaveFailedBody,
            &[("error", &err.to_string())],
        );
        push_warning_toast(state, &title, &body);
    }
}

// ----------------------------------------------------------------------
// Tests
// ----------------------------------------------------------------------
//
// Smoke tests for the `update` state machine. They exercise the
// synchronous state transitions only; the returned `Task` is dropped,
// since each handler tested here either returns `Task::none()` or applies
// its state change synchronously before returning.
//
// No GUI test harness (e.g. iced_test) is used. `update` takes
// `&mut AppState` and `AppState: Default`, so plain construction plus
// assertion is sufficient. `persist_settings` is a no-op when
// `state.store` is `None` (the default), so these tests never touch disk.

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use crate::result::{ResultAssetItem, ResultAssetKind, ResultAssets};

    fn dummy_source_asset() -> SourceAsset {
        SourceAsset {
            path: PathBuf::from("logo.png"),
            kind: logolig::SourceKind::Png,
            raw: Arc::<[u8]>::from([0x89, b'P', b'N', b'G'].as_slice()),
            intrinsic_size: Some((1, 1)),
        }
    }

    fn dummy_result_assets() -> ResultAssets {
        ResultAssets {
            items: vec![ResultAssetItem {
                file_name: "favicon-snippet.html".to_string(),
                relative_path: "favicon-snippet.html".to_string(),
                bytes: b"<link rel=\"icon\" href=\"/favicon.ico\">".to_vec(),
                kind: ResultAssetKind::HtmlSnippet,
                dimensions: None,
                thumbnail: None,
            }],
        }
    }

    #[test]
    fn nav_page_selected_switches_page() {
        let mut state = AppState::default();
        assert_eq!(state.nav_page, NavPage::Home); // precondition

        let _ = update(&mut state, Message::NavPageSelected(NavPage::Settings));
        assert_eq!(state.nav_page, NavPage::Settings);

        let _ = update(&mut state, Message::NavPageSelected(NavPage::Home));
        assert_eq!(state.nav_page, NavPage::Home);
    }

    #[test]
    fn advanced_toggled_routes_to_customize() {
        // v1.22.0 behaviour pin: the former settings-drawer toggle now
        // navigates to the Customize page rather than flipping a bool.
        let mut state = AppState::default();
        let _ = update(&mut state, Message::AdvancedToggled);
        assert_eq!(state.nav_page, NavPage::Customize);
    }

    #[test]
    fn theme_picked_updates_theme() {
        let mut state = AppState::default();
        assert_eq!(state.theme, ThemeMode::System); // default
        let _ = update(&mut state, Message::ThemePicked(ThemeMode::Dark));
        assert_eq!(state.theme, ThemeMode::Dark);
    }

    #[test]
    fn keep_transparency_toggle_sets_plan() {
        let mut state = AppState::default();
        assert!(state.export_plan.keep_transparency); // default true
        let _ = update(&mut state, Message::KeepTransparencyToggled(false));
        assert!(!state.export_plan.keep_transparency);
        let _ = update(&mut state, Message::KeepTransparencyToggled(true));
        assert!(state.export_plan.keep_transparency);
    }

    #[test]
    fn png_preset_size_add_inserts_sorted() {
        let mut state = AppState::default();
        assert_eq!(state.export_plan.png_sizes, vec![32, 192, 512]); // default
        let _ = update(&mut state, Message::PngPresetSizeAdded(64));
        assert_eq!(state.export_plan.png_sizes, vec![32, 64, 192, 512]);
    }

    #[test]
    fn edit_cancelled_preserves_last_result_history() {
        let mut state = AppState {
            screen: Screen::Result,
            source_asset: Some(dummy_source_asset()),
            result_assets: Some(dummy_result_assets()),
            result_preview_open: true,
            file_drag_hovering: true,
            ..AppState::default()
        };

        let _ = update(&mut state, Message::EditCancelled);

        assert_eq!(state.screen, Screen::Empty);
        assert!(state.source_asset.is_some());
        assert!(state.result_assets.is_some());
        assert!(state.preview.is_none());
        assert!(state.preview_cache.is_none());
        assert!(state.transparency.is_none());
        assert!(!state.result_preview_open);
        assert!(!state.file_drag_hovering);
    }

    #[test]
    fn show_last_result_requested_reopens_result_without_reconversion() {
        let mut state = AppState {
            screen: Screen::Empty,
            source_asset: Some(dummy_source_asset()),
            result_assets: Some(dummy_result_assets()),
            ..AppState::default()
        };

        let _ = update(&mut state, Message::ShowLastResultRequested);

        assert_eq!(state.screen, Screen::Result);
        assert_eq!(
            state.result_assets.as_ref().map(ResultAssets::count),
            Some(1)
        );
        assert_eq!(
            state
                .source_asset
                .as_ref()
                .map(|asset| asset.display_name()),
            Some("logo.png".to_string())
        );
    }

    #[test]
    fn file_drag_hover_messages_toggle_drop_zone_feedback() {
        let mut state = AppState::default();

        let _ = update(&mut state, Message::FileDragHovered);
        assert!(state.file_drag_hovering);

        let _ = update(&mut state, Message::FileDragLeft);
        assert!(!state.file_drag_hovering);
    }

    #[test]
    fn file_drop_clears_hover_and_starts_converting() {
        let mut state = AppState {
            file_drag_hovering: true,
            ..AppState::default()
        };
        let path = PathBuf::from("new-logo.png");

        let _ = update(&mut state, Message::FileDropped(path.clone()));

        assert!(!state.file_drag_hovering);
        assert_eq!(state.source_path, Some(path));
        assert_eq!(state.screen, Screen::Converting);
        assert!(state.busy);
        assert!(state.result_assets.is_none());
        assert!(!state.result_preview_open);
    }
    fn ingest_ok(state: &mut AppState, size: Option<(u32, u32)>) {
        let asset = SourceAsset {
            intrinsic_size: size,
            ..dummy_source_asset()
        };
        let _ = update(state, Message::IngestCompleted(Ok(asset)));
    }

    #[test]
    fn non_square_ingest_adds_exactly_one_advisory_toast() {
        let mut state = AppState::default();
        ingest_ok(&mut state, Some((64, 32)));
        assert_eq!(state.toasts.len(), 1);
        let expected = state.translator.t(MessageKey::ToastNonSquareInputTitle);
        assert_eq!(state.toasts[0].title, expected);
    }

    #[test]
    fn square_ingest_adds_no_toast() {
        let mut state = AppState::default();
        ingest_ok(&mut state, Some((100, 100)));
        assert!(state.toasts.is_empty());
    }

    #[test]
    fn nearly_square_ingest_inside_the_tolerance_adds_no_toast() {
        let mut state = AppState::default();
        ingest_ok(&mut state, Some((101, 100)));
        assert!(state.toasts.is_empty());
    }

    #[test]
    fn unknown_size_ingest_adds_no_toast() {
        let mut state = AppState::default();
        ingest_ok(&mut state, None);
        assert!(state.toasts.is_empty());
    }

    #[test]
    fn showing_the_last_result_again_does_not_repeat_the_advisory() {
        let mut state = AppState::default();
        ingest_ok(&mut state, Some((64, 32)));
        assert_eq!(state.toasts.len(), 1);
        state.result_assets = Some(dummy_result_assets());
        let _ = update(&mut state, Message::EditCancelled);
        let _ = update(&mut state, Message::ShowLastResultRequested);
        assert_eq!(state.toasts.len(), 1, "no second toast");
    }

    #[test]
    fn non_square_tolerance_boundary_is_two_percent() {
        assert!(!is_non_square(Some((100, 102)))); // 1.96 %: inside
        assert!(is_non_square(Some((100, 103)))); // 2.9 %: outside
        assert!(!is_non_square(Some((0, 0))));
        assert!(is_non_square(Some((1000, 1))));
    }
    // ---- RFC 015 phase 2a: SVG advisory ------------------------------------

    const SVG_TEXT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><text x="2" y="30" font-size="20">Hi</text></svg>"#;
    const SVG_IMAGE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="64" height="64"><image width="8" height="8" xlink:href="/some/file.png"/></svg>"#;
    const SVG_BOTH: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="64" height="64"><image width="8" height="8" xlink:href="/some/file.png"/><text x="2" y="30" font-size="20">Hi</text></svg>"#;
    const SVG_PLAIN: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><rect width="8" height="8"/></svg>"#;

    fn svg_asset(path: &str) -> SourceAsset {
        SourceAsset {
            path: PathBuf::from(path),
            kind: logolig::SourceKind::Svg,
            raw: Arc::<[u8]>::from(b"<svg/>".as_slice()),
            intrinsic_size: Some((64, 64)),
        }
    }

    /// State with `logo.svg` as the current source, as after a successful ingest.
    fn state_with_svg_source() -> AppState {
        AppState {
            source_asset: Some(svg_asset("logo.svg")),
            ..AppState::default()
        }
    }

    fn analyzed(state: &mut AppState, path: &str, svg: &str) {
        let report = logolig::services::svg::analyze(svg.as_bytes());
        let _ = update(state, Message::SvgAnalyzed(PathBuf::from(path), report));
    }

    #[test]
    fn svg_with_text_and_images_gets_one_toast_with_both_sentences_images_first() {
        let mut state = state_with_svg_source();
        analyzed(&mut state, "logo.svg", SVG_BOTH);
        assert_eq!(
            state.toasts.len(),
            1,
            "exactly one toast, never one per issue"
        );
        let t = &state.translator;
        assert_eq!(state.toasts[0].title, t.t(MessageKey::ToastSvgPartsTitle));
        let expected = format!(
            "{}\n{}",
            t.t(MessageKey::ToastSvgImagesBody),
            t.t(MessageKey::ToastSvgTextBody)
        );
        assert_eq!(state.toasts[0].message, expected);
    }

    #[test]
    fn svg_with_only_images_gets_only_the_images_sentence() {
        let mut state = state_with_svg_source();
        analyzed(&mut state, "logo.svg", SVG_IMAGE);
        assert_eq!(state.toasts.len(), 1);
        assert_eq!(
            state.toasts[0].message,
            state.translator.t(MessageKey::ToastSvgImagesBody)
        );
    }

    #[test]
    fn svg_with_only_text_gets_only_the_text_sentence() {
        let mut state = state_with_svg_source();
        analyzed(&mut state, "logo.svg", SVG_TEXT);
        assert_eq!(state.toasts.len(), 1);
        assert_eq!(
            state.toasts[0].message,
            state.translator.t(MessageKey::ToastSvgTextBody)
        );
    }

    #[test]
    fn plain_svg_gets_no_toast() {
        let mut state = state_with_svg_source();
        analyzed(&mut state, "logo.svg", SVG_PLAIN);
        assert!(state.toasts.is_empty());
    }

    #[test]
    fn stale_analysis_result_is_ignored() {
        let mut state = state_with_svg_source();
        analyzed(&mut state, "an-older-logo.svg", SVG_BOTH);
        assert!(
            state.toasts.is_empty(),
            "result for a file that is no longer current"
        );
        let mut no_source = AppState::default();
        analyzed(&mut no_source, "logo.svg", SVG_BOTH);
        assert!(no_source.toasts.is_empty(), "no current source at all");
    }

    #[test]
    fn analysis_is_started_only_for_svg_sources() {
        // Preview and conversion are always started (2 units); an SVG adds the analysis (3).
        let mut png = AppState::default();
        let task = update(&mut png, Message::IngestCompleted(Ok(dummy_source_asset())));
        assert_eq!(task.units(), 2, "PNG source: no analysis");
        let mut svg = AppState::default();
        let task = update(
            &mut svg,
            Message::IngestCompleted(Ok(svg_asset("logo.svg"))),
        );
        assert_eq!(task.units(), 3, "SVG source: analysis started");
    }

    #[test]
    fn going_back_and_showing_the_last_result_does_not_repeat_the_svg_toast() {
        let mut state = state_with_svg_source();
        analyzed(&mut state, "logo.svg", SVG_BOTH);
        assert_eq!(state.toasts.len(), 1);
        state.result_assets = Some(dummy_result_assets());
        let _ = update(&mut state, Message::EditCancelled);
        let _ = update(&mut state, Message::ShowLastResultRequested);
        assert_eq!(state.toasts.len(), 1, "no second toast");
    }
    // ---- RFC 029: toast lifetime proportional to the content ---------------

    use crate::toast_time::seconds_for;
    use snora::ToastLifetime;

    /// The lifetime a toast must have: transient, for the seconds its own text needs.
    fn expect_content_lifetime(toast: &Toast<Message>) {
        assert_eq!(
            toast.lifetime,
            ToastLifetime::seconds(seconds_for(&toast.title, &toast.message)),
            "toast {:?} / {:?}",
            toast.title,
            toast.message
        );
    }

    #[test]
    fn non_square_advisory_lives_as_long_as_its_text_needs() {
        let mut state = AppState::default();
        ingest_ok(&mut state, Some((64, 32)));
        assert_eq!(state.toasts.len(), 1);
        expect_content_lifetime(&state.toasts[0]);
        assert_eq!(
            state.toasts[0].lifetime,
            ToastLifetime::seconds(12),
            "English text"
        );
    }

    #[test]
    fn svg_parts_advisory_lives_as_long_as_its_text_needs() {
        let mut state = state_with_svg_source();
        analyzed(&mut state, "logo.svg", SVG_BOTH);
        assert_eq!(state.toasts.len(), 1);
        expect_content_lifetime(&state.toasts[0]);
        assert_eq!(
            state.toasts[0].lifetime,
            ToastLifetime::seconds(20),
            "English text, at the cap"
        );
    }

    #[test]
    fn jpeg_advisory_lives_as_long_as_its_text_needs() {
        let mut state = AppState::default();
        push_jpeg_input_warning(&mut state);
        assert_eq!(state.toasts.len(), 1);
        expect_content_lifetime(&state.toasts[0]);
        assert_eq!(
            state.toasts[0].lifetime,
            ToastLifetime::seconds(17),
            "English text"
        );
    }

    #[test]
    fn invalid_size_warning_lives_as_long_as_its_text_needs() {
        let mut state = AppState::default();
        let _ = update(&mut state, Message::PngSizeInputChanged("5".to_string()));
        let _ = update(&mut state, Message::PngSizeAddRequested);
        assert_eq!(state.toasts.len(), 1, "out-of-range size warns once");
        expect_content_lifetime(&state.toasts[0]);
    }

    #[test]
    fn short_success_toast_follows_the_content_rule() {
        let mut state = AppState::default();
        let _ = update(&mut state, Message::ExportPlanResetRequested);
        assert_eq!(
            state.toasts.len(),
            1,
            "reset confirms with one success toast"
        );
        expect_content_lifetime(&state.toasts[0]);
    }

    #[test]
    fn long_success_toast_keeps_seven_seconds_as_a_minimum() {
        let mut state = AppState::default();
        push_long_success_toast(&mut state, "Saved", "/tmp/a.png");
        assert_eq!(
            state.toasts[0].lifetime,
            ToastLifetime::seconds(7),
            "short text: the minimum"
        );
        // A very long path needs more than 7 s and gets it.
        let long_path = format!("/tmp/{}", "d".repeat(200));
        push_long_success_toast(&mut state, "Saved", &long_path);
        let needed = seconds_for("Saved", &long_path);
        assert!(needed > 7, "the fixture must need more than the minimum");
        assert_eq!(state.toasts[1].lifetime, ToastLifetime::seconds(needed));
    }

    #[test]
    fn error_toast_stays_until_dismissed() {
        let mut state = AppState::default();
        push_error_toast(&mut state, AppError::unsupported_file("x.png"));
        assert_eq!(state.toasts[0].lifetime, ToastLifetime::Persistent);
    }

    // ---- RFC 019 phase 2: global shortcuts -------------------------------

    use crate::shortcuts::Shortcut;

    fn result_state() -> AppState {
        AppState {
            screen: Screen::Result,
            source_asset: Some(dummy_source_asset()),
            result_assets: Some(dummy_result_assets()),
            ..AppState::default()
        }
    }

    fn press(state: &mut AppState, shortcut: Shortcut) -> Task<Message> {
        update(state, Message::ShortcutPressed(shortcut))
    }

    /// The task the mouse path would start for `message`, as a count of started units.
    fn units_of(state: &mut AppState, message: Message) -> usize {
        update(state, message).units()
    }

    #[test]
    fn choose_file_starts_the_same_task_as_the_button_on_empty_and_result() {
        for screen in [Screen::Empty, Screen::Result] {
            let mut state = AppState {
                screen,
                ..result_state()
            };
            let mouse = units_of(&mut state, Message::PickFileRequested);
            assert!(mouse > 0, "the mouse path starts a task");
            let key = press(&mut state, Shortcut::ChooseFile).units();
            assert_eq!(key, mouse, "{screen:?}");
        }
    }

    #[test]
    fn choose_file_does_nothing_while_converting_or_off_the_home_page() {
        let mut converting = AppState {
            screen: Screen::Converting,
            ..AppState::default()
        };
        assert_eq!(press(&mut converting, Shortcut::ChooseFile).units(), 0);
        for page in [NavPage::Customize, NavPage::Settings] {
            let mut state = AppState {
                nav_page: page,
                ..AppState::default()
            };
            assert_eq!(
                press(&mut state, Shortcut::ChooseFile).units(),
                0,
                "{page:?}"
            );
        }
    }

    #[test]
    fn download_all_works_only_on_the_result_screen_with_assets() {
        let mut state = result_state();
        let mouse = units_of(&mut state, Message::DownloadAllRequested);
        assert!(mouse > 0);
        assert_eq!(press(&mut state, Shortcut::DownloadAll).units(), mouse);

        let mut empty = AppState::default();
        assert_eq!(press(&mut empty, Shortcut::DownloadAll).units(), 0, "Empty");

        let mut converting = AppState {
            screen: Screen::Converting,
            result_assets: Some(dummy_result_assets()),
            ..AppState::default()
        };
        assert_eq!(
            press(&mut converting, Shortcut::DownloadAll).units(),
            0,
            "Converting"
        );

        let mut no_assets = AppState {
            screen: Screen::Result,
            ..AppState::default()
        };
        assert_eq!(
            press(&mut no_assets, Shortcut::DownloadAll).units(),
            0,
            "no assets"
        );

        for page in [NavPage::Customize, NavPage::Settings] {
            let mut state = AppState {
                nav_page: page,
                ..result_state()
            };
            assert_eq!(
                press(&mut state, Shortcut::DownloadAll).units(),
                0,
                "{page:?}"
            );
        }
    }

    #[test]
    fn escape_dismisses_the_newest_toast_first_and_does_not_navigate() {
        let mut state = result_state();
        push_warning_toast(&mut state, "first", "one");
        push_warning_toast(&mut state, "second", "two");
        let (first_id, second_id) = (state.toasts[0].id, state.toasts[1].id);

        let _ = press(&mut state, Shortcut::EscapeOrBack);
        assert_eq!(state.toasts.len(), 1);
        assert_eq!(state.toasts[0].id, first_id, "the newest one went first");
        assert_ne!(first_id, second_id);
        assert_eq!(
            state.screen,
            Screen::Result,
            "no navigation while a toast was open"
        );

        let _ = press(&mut state, Shortcut::EscapeOrBack);
        assert!(state.toasts.is_empty());
        assert_eq!(state.screen, Screen::Result);

        let _ = press(&mut state, Shortcut::EscapeOrBack);
        assert_eq!(state.screen, Screen::Empty, "with no toast, Esc goes back");
    }

    #[test]
    fn escape_dismisses_a_toast_on_any_page_and_screen() {
        for page in [NavPage::Home, NavPage::Customize, NavPage::Settings] {
            for screen in [Screen::Empty, Screen::Converting, Screen::Result] {
                let mut state = AppState {
                    nav_page: page,
                    screen,
                    ..AppState::default()
                };
                push_error_toast(&mut state, AppError::unsupported_file("x.png"));
                let _ = press(&mut state, Shortcut::EscapeOrBack);
                assert!(state.toasts.is_empty(), "{page:?} {screen:?}");
                assert_eq!(state.screen, screen);
                assert_eq!(state.nav_page, page);
            }
        }
    }

    #[test]
    fn escape_without_a_toast_goes_back_only_from_the_home_result_screen() {
        for (page, screen) in [
            (NavPage::Home, Screen::Empty),
            (NavPage::Home, Screen::Converting),
            (NavPage::Customize, Screen::Result),
            (NavPage::Settings, Screen::Result),
        ] {
            let mut state = AppState {
                nav_page: page,
                screen,
                ..result_state()
            };
            let _ = press(&mut state, Shortcut::EscapeOrBack);
            assert_eq!(state.screen, screen, "{page:?} {screen:?}");
            assert_eq!(state.nav_page, page);
        }
        let mut state = result_state();
        let _ = press(&mut state, Shortcut::EscapeOrBack);
        assert_eq!(state.screen, Screen::Empty);
        assert!(
            state.result_assets.is_some(),
            "history is kept, as for the Back button"
        );
    }

    #[test]
    fn page_shortcuts_and_help_set_the_nav_page_from_anywhere() {
        let mut state = AppState::default();
        for (shortcut, page) in [
            (Shortcut::Page(NavPage::Customize), NavPage::Customize),
            (Shortcut::Page(NavPage::Settings), NavPage::Settings),
            (Shortcut::Page(NavPage::Home), NavPage::Home),
            (Shortcut::Help, NavPage::Settings),
        ] {
            let _ = press(&mut state, shortcut);
            assert_eq!(state.nav_page, page, "{shortcut:?}");
        }
        // While converting, switching pages is allowed (the sidebar allows it too).
        let mut converting = AppState {
            screen: Screen::Converting,
            ..AppState::default()
        };
        let _ = press(&mut converting, Shortcut::Page(NavPage::Customize));
        assert_eq!(converting.nav_page, NavPage::Customize);
        assert_eq!(converting.screen, Screen::Converting);
    }

    // ---- RFC 030: advisory for a large traced favicon.svg ------------------

    fn assets_with_svg_of(bytes: usize) -> ResultAssets {
        let mut assets = dummy_result_assets();
        assets.items.push(ResultAssetItem {
            file_name: "favicon.svg".to_string(),
            relative_path: "favicon.svg".to_string(),
            bytes: vec![b' '; bytes],
            kind: ResultAssetKind::Svg,
            dimensions: None,
            thumbnail: None,
        });
        assets
    }

    fn converting_state(kind: logolig::SourceKind) -> AppState {
        let mut asset = dummy_source_asset();
        asset.kind = kind;
        AppState {
            screen: Screen::Converting,
            source_asset: Some(asset),
            ..AppState::default()
        }
    }

    #[test]
    fn a_traced_svg_over_two_megabytes_gets_exactly_one_advisory() {
        let mut state = converting_state(logolig::SourceKind::Jpeg);
        let _ = update(
            &mut state,
            Message::ConvertCompleted(Ok(assets_with_svg_of(SVG_ADVISORY_BYTES + 1))),
        );
        assert_eq!(state.screen, Screen::Result);
        assert_eq!(state.toasts.len(), 1);
        let t = &state.translator;
        let (title, body) = (
            t.t(MessageKey::ToastSvgLargeTitle),
            t.t(MessageKey::ToastSvgLargeBody),
        );
        assert_eq!(state.toasts[0].title, title);
        assert_eq!(state.toasts[0].message, body);
        assert_eq!(
            state.toasts[0].lifetime,
            ToastLifetime::seconds(seconds_for(&title, &body))
        );
    }

    #[test]
    fn a_svg_of_exactly_two_megabytes_or_none_gets_no_advisory() {
        let mut state = converting_state(logolig::SourceKind::Png);
        let _ = update(
            &mut state,
            Message::ConvertCompleted(Ok(assets_with_svg_of(SVG_ADVISORY_BYTES))),
        );
        assert!(
            state.toasts.is_empty(),
            "exactly 2 MB is not above the limit"
        );

        let mut state = converting_state(logolig::SourceKind::Png);
        let _ = update(
            &mut state,
            Message::ConvertCompleted(Ok(dummy_result_assets())),
        );
        assert!(state.toasts.is_empty(), "no favicon.svg item, no advisory");
    }

    #[test]
    fn a_large_svg_source_copied_verbatim_gets_no_photograph_advisory() {
        // An SVG source is copied as it is, not traced: the toast's claim would be false.
        let mut state = converting_state(logolig::SourceKind::Svg);
        let _ = update(
            &mut state,
            Message::ConvertCompleted(Ok(assets_with_svg_of(SVG_ADVISORY_BYTES + 1))),
        );
        assert!(state.toasts.is_empty());
    }

    #[test]
    fn the_large_svg_advisory_is_not_repeated_by_going_back_and_returning() {
        let mut state = converting_state(logolig::SourceKind::Jpeg);
        let _ = update(
            &mut state,
            Message::ConvertCompleted(Ok(assets_with_svg_of(SVG_ADVISORY_BYTES + 1))),
        );
        let _ = update(&mut state, Message::EditCancelled);
        let _ = update(&mut state, Message::ShowLastResultRequested);
        assert_eq!(state.toasts.len(), 1);
    }
}
