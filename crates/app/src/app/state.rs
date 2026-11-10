//! Application state: screens, navigation pages, `AppState`, and the helpers that read it
//! (RFC 017: moved out of `app.rs`, unchanged).

use std::path::PathBuf;

use iced::Theme;
use snora::Toast;

use logolig::{
    ExportPlan, MessageKey, PreviewCache, PreviewProfile, SettingsStore, SourceAsset, ThemeMode,
};
use logolig_i18n::{Locale, Translator, detect_system_locale};

use super::Message;
use super::toast::push_warning_toast;

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
    pub source_path: Option<PathBuf>,
    pub source_asset: Option<SourceAsset>,
    pub preview: Option<PreviewProfile>,
    /// Resized raster cache for context preview rendering (§5.2).
    /// Rebuilt when the source asset is loaded or the resize algorithm changes.
    pub preview_cache: Option<PreviewCache>,
    pub export_plan: ExportPlan,
    /// snora toast queue. All error and success notifications go through this.
    pub toasts: Vec<Toast<Message>>,
    /// Monotonic counter used to assign unique IDs to toasts.
    pub next_toast_id: u64,
    /// Which toasts the reader is attending to (pointer on the toast). snora's expiry sweep
    /// keeps those and restarts their countdown when the attention leaves, so this lives as
    /// long as the application state (RFC 032): a temporary per tick would forget every hold.
    pub holds: snora::toast::Holds,

    // v1.3.0: text buffer for the custom-size input in the Customize page.
    // Transient UI state: belongs in the app layer, not in logolig (core).
    pub png_size_input: String,

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
            source_path: None,
            source_asset: None,
            preview: None,
            preview_cache: None,
            export_plan: ExportPlan::default(),
            toasts: Vec::new(),
            next_toast_id: 0,
            holds: snora::toast::Holds::default(),
            png_size_input: String::new(),
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
