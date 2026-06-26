# Architecture

## Goals

The architecture exists to enforce four properties from the spec:

1. **Local-first.** No image data leaves the user's machine.
2. **Image quality.** Resize methods are pluggable; the default favors
   quality over speed.
3. **ABDD.** Accessibility is a typed concern, not a CSS afterthought.
4. **Maintainable code.** Domain logic is testable without a GUI runtime.

## Workspace layout

```
logolig/
├── Cargo.toml                       # [workspace] manifest
├── README.md
├── docs/
└── crates/
    ├── logolig-core/                # pure logic, no GUI deps
    │   ├── src/
    │   │   ├── lib.rs
    │   │   ├── error.rs             # AppError (Clone + Send), keyed via key()/args() since v1.5
    │   │   ├── message_key.rs       # MessageKey enum, the i18n vocabulary (v1.5+)
    │   │   ├── settings.rs          # SettingsStore<T> trait (v1.4+)
    │   │   ├── domain.rs / domain/  # SourceAsset, ExportPlan, PersistedSettings, ...
    │   │   └── services.rs / services/
    │   │       ├── ingest.rs        # async file load (PNG/SVG/WebP, magic-byte detection)
    │   │       ├── decode_png.rs    # PNG → Rgba8 (image crate)
    │   │       ├── decode_webp.rs   # WebP → Rgba8 (since v1.1)
    │   │       ├── decode_jpeg.rs   # JPEG → Rgba8 (since v1.11; alpha filled with 0xFF)
    │   │       ├── rasterize_svg.rs # SVG → Rgba8 (resvg + tiny-skia, per-size render)
    │   │       ├── resize.rs        # Rgba8 → Rgba8 (fast_image_resize, Lanczos3 default)
    │   │       ├── vectorize.rs     # Rgba8 → SVG string (vtracer, since v1.2)
    │   │       ├── transparency_audit.rs # alpha-channel classifier (FullyOpaque / FullyTransparent / HasTransparency, since v1.7)
    │   │       ├── manifest_writer.rs # WebManifestSettings + png_sizes → manifest.webmanifest JSON (since v1.8)
    │   │       ├── monochrome.rs    # Rgba8 → grayscale Rgba8 (BT.709 luma, alpha preserved, since v1.9)
    │   │       ├── preview.rs       # build 16×16 + 120×120 preview cache
    │   │       ├── encode_png.rs    # Rgba8 → PNG bytes
    │   │       ├── ico_writer.rs    # bundle Rgba8 frames into .ico
    │   │       ├── html_snippet.rs  # render <head> snippet
    │   │       └── exporter.rs      # transactional orchestrator (staging + atomic rename)
    │   └── tests/
    ├── logolig-i18n/                # translations (v1.5+)
    │   ├── src/
    │   │   ├── lib.rs               # Translator + Locale public API
    │   │   ├── locale.rs            # Locale enum, sys-locale OS detection
    │   │   ├── dictionary.rs        # bundled TOML → typed struct, exhaustive lookup
    │   │   └── translator.rs        # Translator { t(key), t_args(key, args), translate_error }
    │   ├── locales/
    │   │   ├── en.toml              # English dictionary (v1.5)
    │   │   └── ja.toml              # Japanese dictionary (v1.6)
    │   └── tests/
    └── logolig-app/                 # iced + snora GUI binary
        ├── src/
        │   ├── main.rs              # 5-line entry point
        │   ├── app.rs               # AppState / Message / update / view / run
        │   ├── shell.rs             # snora::AppLayout assembly
        │   ├── native_store.rs      # SettingsStore impl for the native target (v1.4+)
        │   ├── task_queue.rs        # iced::Task helpers
        │   ├── result.rs           # in-memory ResultAssets bundle (since v1.16)
        │   └── ui.rs / ui/
        │       ├── drop_zone.rs
        │       ├── preview_panel.rs
        │       ├── advanced_drawer.rs
        │       ├── colors.rs           # theme-aware color helpers (since v1.14)
        │       ├── converting.rs       # Converting screen (since v1.16)
        │       ├── result_view.rs      # Result screen w/ asset cards (since v1.16)
        │       ├── sidebar.rs          # Left sidebar, desktop (since v1.18)
        │       ├── picker_overlay.rs   # Locale/Theme popup (since v1.18)
        │       ├── bottom_nav.rs       # Bottom navigation, mobile (since v1.20)
        │       └── accessibility.rs
        └── tests/
```

The split is **enforced by the dependency graph**: `logolig-core`
declares no dependency on iced, snora, or `logolig-i18n`, so importing
them from inside `logolig-core` is a compile-time error.
`logolig-i18n` depends on `logolig-core` one-way (it speaks the
core's `MessageKey` vocabulary). `logolig-app` consumes both.
Architectural drift is caught by `cargo check`, not by code review.

The original spec (§9) suggested a single-crate layout with a `domain/
mod.rs` style. Two adjustments were made and signed off on:

- **No `mod.rs`** — Rust 2018+ style is mandated by the spec itself
  (§2.1), so each module is `foo.rs` next to a `foo/` directory.
- **Workspace** — preferred over a single crate because it makes the
  domain/services/UI separation a structural fact rather than a
  convention. snora itself uses the same split (`snora-core` /
  `snora`), so the pattern is idiomatic.

## State model

A single `AppState` lives in `logolig-app`. Pure functional `view` and
`update` operate on it:

```text
                ┌─────────────┐
                │   AppState  │
                └──────┬──────┘
                       │
        view(state)    │    update(state, message)
       (pure)          │   (state mutation)
                       │
                ┌──────▼──────┐
                │  iced::run  │
                └─────────────┘
```

`AppState` fields:

| Field | Purpose |
| --- | --- |
| `screen: Screen` | Current screen (Empty / Importing / Preview / ExportReady / Exporting) |
| `theme: ThemeMode` | System / Light / Dark |
| `advanced_open: bool` | Whether the advanced settings BottomSheet is open |
| `source_path` / `source_asset` | The loaded image (immutable; §6.4 non-destructive) |
| `preview: Option<PreviewProfile>` | Current preview context + background |
| `export_plan: ExportPlan` | What and how to write to disk |
| `busy: bool` | Whether a long task is in flight |
| `toasts: Vec<Toast<Message>>` | Notifications (errors, successes) |
| `next_toast_id: u64` | Monotonic id source for toasts |

### Screen states

There are **five** screens. `Failed` was deliberately omitted: errors
become `Toast::persistent()` notifications instead of taking over the
screen. A persistent error toast is dismissible from any screen and
keeps the user's context (their loaded image, their progress) intact.

```text
   Empty ──drop / pick──> Importing ──ok──> Preview ──export──> Exporting
                                                                  │
                                                                  ok
                                                                  ▼
                                                            ExportReady
```

On any error, the state machine returns to `Preview` (if a source is
loaded) or `Empty` (if not), and a persistent error toast is enqueued.

## Message flow

```
[user gesture]                   [task completion]
      │                                 │
      ▼                                 ▼
   Message::FileDropped            Message::IngestCompleted(Result<...>)
      │                                 │
      ▼                                 ▼
   start_ingest(path)               Ok → Screen::Preview
      │                             Err → fail() → toast
      ▼
   iced::Task::perform(
     logolig_core::services::
       ingest::ingest(path),
     Message::IngestCompleted)
```

Heavy work is run via `iced::Task::perform`, never on the UI thread.
The Task helpers live in `logolig-app/src/task_queue.rs` because they
depend on both `iced::Task` and `crate::app::Message`.

## Why snora

The spec mandates ABDD. `snora-core::LayoutDirection` (LTR/RTL)
together with `Edge::{Start, End}` express layout in **logical**
positions, so a single direction toggle mirrors the entire UI.
Toast lifecycle (transient vs. persistent), modal backdrop dismissal,
and bottom-sheet sizing are all expressed as enums rather than
boolean flags or magic numbers.

That alignment is why we chose snora over a hand-rolled iced shell.

## How v1 was built

Logolig v1 was assembled in four ordered increments. Each one closed
on a green `cargo check --workspace` and a passing `cargo test
-p logolig-core` before the next began. The progression is recorded
here both as build history and as a map for future contributors who
want to retrace the design decisions.

| Step | Deliverable | Released as |
| --- | --- | --- |
| 1 | Skeleton, state model, snora layout | v0.1.0 |
| 2 | Drop reception + image processing pipeline | v0.2.0 |
| 3 | Context preview UI + theme toggle + a11y polish | v0.3.0 |
| 4 | ICO writing, export, HTML snippet generation | v0.4.0 |
| — | Stabilization, freeze | **v1.0.0** |

After v1.0.0 stabilized, the v1 line continued with feature releases
that strengthen the core for v2 reuse:

| Version | Theme |
| --- | --- |
| v1.1.0 | WebP input (decoder, magic-byte detection, size parser) |
| v1.2.0 | SVG output (raw passthrough for SVG sources, vtracer for raster) |
| v1.3.0 | Advanced settings UX overhaul (chip-style size editors, per-artifact toggles, validation in core) |
| v1.4.0 | Persisted settings (`SettingsStore` trait in core; `NativeStore` wraps `app-json-settings` v2 ConfigManager; immediate-save on every user change; `PersistedSettings { export_plan, theme, locale }` with `serde(default)` for forward compat) |
| v1.4.1 | Internal improvements: vtracer presets (Sharp / Default / PhotoRich) for vector quality control; "Reset to defaults" button on advanced drawer; persistent success toast on export so the completion message stays visible until dismissed |
| **v1.4.2** | **Internal refinements**: Sharp preset re-tuned to a single-parameter delta from defaults (`corner_threshold: 60 → 80`) after user testing showed the v1.4.1 multi-parameter version did not improve quality and possibly worsened it; Export success toast switched from persistent back to a 7-second transient (long enough to read the path, short enough not to linger); Sharp preset now follows an empirical-tuning approach where future adjustments will be made one parameter at a time so their effect is observable |
| **v1.5.0** | **i18n base (English)**: new `logolig-i18n` crate with a typed `Translator` API; all UI strings keyed via `logolig_core::MessageKey` enum (compile-time exhaustiveness checking via the dictionary's `match` so translation files must satisfy the enum, or build fails); `AppError` keyed via `key()` and `args()` so error toasts translate automatically; `sys-locale` for OS locale detection with explicit override available in advanced settings ("Language" pick_list); English-only ship — Japanese arrives in v1.6.0 by adding `ja.toml` and a `Locale::Ja` variant |
| **v1.6.0** | **Japanese translation**: `Locale::Ja` variant added to `logolig-i18n`, full `ja.toml` dictionary covering all ~80 `MessageKey` variants, `from_bcp47` extended to recognise `ja` / `ja-JP` / `ja_JP` / `ja_JP.UTF-8` (POSIX locale form). The work was small because v1.5.0 had set up the structure: a new locale needed only an enum variant, a TOML file, one `include_str!` arm, and one `match` arm in `locale_message_key`. UI code, advanced drawer pick_list, error toasts, and persistence picked up Japanese automatically because everything routes through `MessageKey`. 7 new tests verify Japanese parses, differs from English on UI keys, and substitutes placeholders correctly. |
| **v1.7.0** | **Transparency checker**: a new `logolig-core::services::transparency_audit` classifies the loaded image into `FullyOpaque` / `FullyTransparent` / `HasTransparency` by single-pass alpha scan with early termination. Audit fires once per ingested image during the first valid `PreviewBuilt`, and a Warning toast is shown for the two extreme cases — typical favicon failure modes (white-square-on-dark for fully opaque sources, empty-canvas for fully transparent ones) that users would otherwise only catch after exporting and seeing the result in their browser. The preview panel adds a "Show transparency checker" checkbox: when on, framing (browser-tab chrome, smartphone home) is replaced with a dedicated checker view — light/dark grey 12px tiles with the icon overlaid at native size — so transparent regions become visually unambiguous. The checker handle is computed once via `OnceLock`. Halo and pre-multiplied-alpha detection are deliberately out of scope: their thresholds risk false positives on legitimate anti-aliasing. The two binary-decision extreme cases (alpha all 255, alpha all 0) carry no such ambiguity. |
| **v1.8.0** | **Web manifest output**: new `domain::WebManifestSettings` (name / short_name / theme_color / background_color), new `services::manifest_writer` produces `manifest.webmanifest` JSON with `icons` derived from `ExportPlan.png_sizes` so manifest icons stay in sync with what's actually written. `start_url = "/"` and `display = "standalone"` are deliberately fixed (favicon users rarely change them, and exposing them widens the UI surface for little gain — v1.8.x can add them if needed). The HTML snippet emits `<link rel="manifest">` when active. `ExportPlan.web_manifest: Option<WebManifestSettings>` opt-in by default. `serde_json` graduated from dev-dep to runtime workspace dep. Color validation runs at export-time, not on every keystroke (avoids the `#FF` mid-typing warning UX). 14 manifest_writer tests + 3 html_snippet tests + the same exhaustive-MessageKey-translation pattern catches dictionary drift at compile time. |
| **v1.9.0** | **Monochrome output set**: `services::monochrome` produces a BT.709 grayscale `Rgba8` from any input (alpha preserved, sRGB-aligned coefficients). `ExportPlan.monochrome: bool` opt-in, default `false`. When enabled, exporter writes a `mono/` subdirectory containing grayscale versions of each PNG size and (if `include_ico` is also on) the ICO. apple-touch-icon, the HTML snippet, and `manifest.webmanifest` are deliberately not mono'd — apple-touch is color-first on iOS, the snippet/manifest aren't pixel data. SVG monochrome is not yet implemented (color replacement on arbitrary SVG sources — paint attrs, inline styles, gradients, external CSS — is a separate problem; v1.9.x will revisit by going raster→grayscale→re-vectorize). The `finalize` rename phase was rewritten to handle subdirectory paths via `strip_prefix(output_dir)`, and staging cleanup uses `remove_dir_all` to handle the (intentionally empty by then) `mono/` staging subdir. The `<link rel="manifest">`-style HTML auto-injection is intentionally absent for mono — `prefers-color-scheme` integration is too project-specific to template; the README shows the diff users can paste in. 9 monochrome unit tests + 3 exporter integration tests verify file structure and that mono PNG bytes differ from color PNG bytes. |
| **v1.10.0** | **Information design redesign — main panel + advanced drawer**: the main panel was reorganized so the preview is the visual main subject (centered, with surrounding controls trimmed). The two pickers are now labeled and parallel: "View as: Browser tab / Phone home / Checker" (the v1.7 standalone `preview_checker` toggle was promoted to a `PreviewContext::TransparencyChecker` variant, making the three options mutually exclusive at the type level), and "Surface: System / Light / Dark" which is automatically disabled when Checker is active because surface tinting has no effect on the checker view. Active picker buttons get both a background-fill **and** the existing `▣` text prefix, so state is conveyed both chromatically and typographically (ABDD §12). The advanced drawer was regrouped into four explicit sections — **What to export** (file kinds + sizes), **Extras** (Web manifest, Monochrome — visually quieter), **Rendering quality** (resize algorithm), **App preferences** (language) — to match the user's mental order: what → how it's drawn → app-wide. Within "What to export" the SVG checkbox now nests vectorize-on-raster and the vtracer preset under itself with a 20px indent, making the parent/child relationship visible. PNG and ICO size sets get a quiet "at defaults: 32 / 192 / 512" badge when they match `ExportPlan::default_png_sizes()` / `default_ico_sizes()`; typing into an adjacent input switches them to the full chip editor. `Message::PreviewCheckerToggled` was removed since the context picker now covers it. |
| **v1.10.1** | **Crash fix**: `cosmic-text` 0.15 panicked on `text(...).size(0)` (line height cannot be 0). v1.10.0 introduced this with a "hidden a11y label" next to the Export button, but iced 0.14 has no a11y wiring so it was non-functional anyway. Removed; a11y label constants stay defined in `accessibility::label` for the future. |
| **v1.10.2** | **Main panel refresh**: the top of the window is now a single header bar — left side shows the app name "Logolig" muted with a "— favicon generator" tagline; right side is a row of four icon-only buttons (language-cycle glyph for language cycle, `◐`/`☀`/`☾` for theme cycle, `⚙` for advanced, `✕` for close) each wrapped in an iced 0.14 `tooltip`. The footer was dropped because the advanced toggle moved into the header. Icon buttons are transparent until hovered; the close button is separated from the others by a wider 16px gap to reduce mis-clicks. The drop zone was slimmed to a single headline ("Drop PNG, SVG, or WebP") plus the Choose file… button, wrapped in a soft-bordered card with rounded corners and a weak fill — iced 0.14 has no `border-style: dashed`, so the card uses a 2px solid border + tinted background instead. The language picker was removed from the advanced drawer (now reachable only via the header icon, which cycles System default → English → Japanese in three clicks); the `language_row` UI helper, `LocalizedLocaleChoice` wrapper, and `App preferences` advanced group are all gone. App close is wired through `iced::window::latest().and_then(window::close)` because iced 0.14 doesn't expose the active window id directly. |
| **v1.10.3** | **Advanced drawer accordion**: each of the three groups ("What to export", "Extras", "Rendering quality") now has a clickable heading that toggles its body. Default state opens "What to export" only — Extras and Rendering quality start collapsed because most users never touch them. The chevron glyph reflects state (`▼` expanded, `▶` collapsed) so it conveys two ways: shape silhouette and direction. State lives in the new `AdvancedGroupExpansion` struct on `AppState`, with no persistence — every time the drawer is opened it resets to the default expansion, on the assumption that the user is starting a fresh task and the most relevant group should always be the first one they see. The `Message::AdvancedGroupToggled(AdvancedGroup)` handler is a one-line `state.advanced_groups.toggle(group)`. The previous `group_heading` helper was replaced by `accordion_group(label, expanded, on_toggle, body)` which renders the clickable header button and conditionally the body. |
| **v1.11.0** | **JPEG support**: new `SourceKind::Jpeg` (extension `.jpg`/`.jpeg`), magic-byte detection (`FF D8 FF` SOI), `services/decode_jpeg.rs` going through `image::load_from_memory` (which fills the missing alpha channel with 255). Width/height are parsed from the first SOF (Start Of Frame) marker — `parse_jpeg_size` walks the JPEG marker segments, skipping standalone markers (SOI/EOI/RSTn/TEM) and segments with length fields, until it finds a SOF (markers C0–C3, C5–C7, C9–CB, CD–CF), then reads height (BE, 2 bytes) and width (BE, 2 bytes) from the data area. Image crate `jpeg` feature added to workspace. Pipeline branches updated: `preview::render_at`, `exporter::run` (decoded_raster + SVG vectorize), `exporter::render_at_size`. Drop-zone i18n updated to "Drop PNG, SVG, WebP, or JPEG". Educational warning: since JPEG cannot store an alpha channel, the v1.7 transparency audit always returns `FullyOpaque` for JPEG inputs; rather than show the generic "no transparency detected" warning (which sounds like the user did something wrong), the `PreviewBuilt` handler branches on `SourceKind::Jpeg` and surfaces a JPEG-specific toast — `ToastJpegInputTitle` / `ToastJpegInputBody` — explaining that JPEG cannot have transparency and recommending PNG with a cut-out background. Tests: 5 decode_jpeg + 4 ingest cases (magic byte, both extensions, SOF parsing, truncated-file robustness). |
| **v1.12.0** | **Edit-screen flow + preview redesign**: the preview screen now has the back-navigation paths users were missing. A "← Back" button (`Message::EditCancelled`) clears the loaded source and returns to the empty drop-zone screen, and a "↻ Re-select" button reuses `Message::PickFileRequested` to open the file picker without leaving the screen — if the user cancels the picker, they stay on the current edit screen. Both auxiliary buttons sit on the left of the action row with a controlled muted style (transparent background, weak border, hover tint) to differentiate them from the primary Export button on the right (theme-primary, larger padding, heavier text). The header was made screen-aware: on the empty/importing/exporting screens it shows "Logolig — favicon generator" as before, but on the preview/export-ready screens it shows the loaded file's display name in a slightly darker color (`FILE_NAME_COLOR`), making the loaded image the screen's subject. The preview screen got a centered page title ("Preview & Generate Favicon") above a soft-bordered preview card. Inside the card, the View as / Surface picker rows are centered and sit *above* the preview frame, so they read as "controls for this preview". The preview frame itself uses `Length::FillPortion(4)` for its height with `max_width(560.0)` / `max_height(560.0)` caps and `Length::Fill` width — the size now scales with window height (≈4/7 of the available vertical space) instead of being pinned to whatever the inner content (tab mock, phone mock, checker) wanted, which fixes the "Export button jumps when switching modes" issue. ESC-to-back was deferred to a future release because it requires an `iced::keyboard::on_key_press` subscription that doesn't yet exist in the app. A separate request document, `docs/snora-upstream-request-bottom-sheet.md`, was prepared for the snora maintainer covering the BottomSheet scrolling and sticky-footer needs that will unblock v1.14.0. |
| **v1.13.0** | **snora 0.4 → 0.8 migration**: snora 0.8.0 ships an API restructure that resolves several pending logolig blockers in one upgrade. The framework was split into a 3-crate workspace (`snora-core` for vocabulary, `snora-widgets` for prefab visuals, `snora` as the engine umbrella); the bottom-only `BottomSheet` was generalized into `Sheet` + `SheetEdge` (Top/Bottom/Start/End) + `SheetSize` (axis-perpendicular size; `OneThird` / `Half` / `TwoThirds` / `Ratio(f32)` / `Pixels(f32)`); `AppLayout::bottom_sheet` was renamed to `AppLayout::sheet`; `lucide-icons` and `svg-icons` features are now usable; new `Tab` / `TabBar` / `Crumb` / `BreadcrumbAction` vocabulary was added (logolig is single-screen so doesn't consume them yet); the toast position default flipped from `BottomEnd` to `TopEnd`, which was exactly what logolig wanted in the never-shipped v1.16.0 plan — that whole roadmap line is now redundant and was removed. logolig's migration is small and surgical: `snora = { version = "0.8", default-features = false }` in workspace `Cargo.toml`; `BottomSheet` → `Sheet`, `SheetHeight` → `SheetSize`, `with_height(...)` → `with_size(...)`, `bottom_sheet(...)` → `sheet(...)` in `shell.rs`. `Sheet::new`'s default edge is `SheetEdge::Bottom`, so the visual behavior is identical to the previous `BottomSheet`. `lucide-icons` adoption (replacing the existing Unicode-glyph icon buttons in the header) is intentionally **out of scope for this version** — that's an independent UX decision, scheduled for a later v1.13.x or v1.14.x phase, so this release stays purely a dependency/API update. The same Sheet upgrade also clears the path for the v1.15.0 advanced-drawer scrolling work: with the new `Sheet` being a pure content carrier, wrapping the drawer body in `iced::widget::scrollable` becomes the obvious solution and no longer requires upstream coordination — the `docs/snora-upstream-request-bottom-sheet.md` request document is now historical context only. |
| **v1.14.0** | **Dark-mode colour integrity + visual hierarchy**: ~10 hardcoded `Color::from_rgb(...)` constants (all assuming the light theme) replaced with role-based colours via `iced::extended_palette()`. New module `crates/logolig-app/src/ui/colors.rs` provides role functions (`app_name(theme)` / `tagline(theme)` / `file_name(theme)` / `page_title(theme)` / `section_label(theme)` / `group_heading(theme)` / `muted_text(theme)` / `drop_zone_headline(theme)` / `badge_muted_bg(theme)`); each screen receives a resolved `&Theme` via `crate::app::resolve_theme(state)`. The boundary between "theme-reactive" and "intentionally hardcoded" is documented: hardcoded items are (a) checkerboard greys (`#E6E6E6` / `#C0C0C0` — indicators of transparency itself) and (b) browser-tab / phone / wallpaper preview colours (controlled by the Surface picker, an independent axis). Unified role names across screens make font-size ↔ role ↔ theme-reactivity relationships explicit. `accordion_group` extended with `heading_color: Color`; `subsection` extended with `muted_color: Color`; `size_subsection` calls `resolve_theme(state)` for the "at defaults" badge colour. Light/dark switching via the `◐/☀/☾` header button; all text, card borders, badge backgrounds, and muted descriptions follow the theme. |
| **v1.15.0** | **Advanced drawer scroll + sticky footer**: the old layout placed Reset/Close at the end of a single column, so fully expanding all accordions pushed the footer off the bottom of the sheet. Fix: drawer split into three rows — (1) title area (`Length::Shrink`, pinned top), (2) `iced::widget::scrollable`-wrapped group content (`Length::FillPortion(1)`, fills remaining space), (3) footer (`Length::Shrink`, pinned bottom = sticky). Only the middle row scrolls; title and footer are always visible. Because snora 0.8's `Sheet` is documented as a pure content carrier, the wrapping is done entirely on the logolig side. Reset (left, destructive-leaning — transparent bg + `palette.danger.weak.color` border, hover tints with danger at alpha 0.15) and Close (right, neutral — `secondary_button_style`) are visually differentiated by position + border colour + hover feedback; `Space::new()` in the row creates the left-right separation. Always-visible scrollbar (iced 0.14 default) signals "more content below". Sheet size remains `SheetSize::Half`. |
| **v1.16.0** | **Screen structure revision — first stage of the new external design**: screen states simplified from 5 (`Empty / Importing / Preview / Exporting / ExportReady`) to 3 (`Empty / Converting / Result`). The Preview state is removed; the v1.15 flow "confirm via View-as pickers → press Export to write" becomes "drop file → auto-convert → Result screen with asset cards + individual DL / ZIP DL". Preview inspection is preserved as an optional collapsible section ("▶ Preview") on the Result screen. In-memory conversion: conversion results are held in `crate::result::ResultAssets` (app state) and written to disk only when the user presses a download button, decoupling generation from saving. ZIP bundle uses the `zip = "2"` crate. Two new UI modules: `ui::converting` (replaces the old Importing/Exporting screens) and `ui::result_view` (3-column asset card grid; `ResultAssetKind` drives card appearance — raster thumbnail for PNG/ICO, `<>` placeholder for SVG, etc.). Each card shows file name, thumbnail, badge (PNG/ICO/SVG/HTML/JSON), dimensions (images only), human-readable size, and a per-card download button. "Download all (ZIP)" button at the bottom. Total artifact bytes are typically < 1 MB, so in-memory holding cost is negligible. BMP input deferred (separate phase). |
| **v1.17.0** | **Settings drawer → Right Sheet + flat layout**: migrated from Bottom Sheet to Right Sheet (`SheetEdge::End`); content reorganised into a flat section structure matching the design mock. Settings not in the mock (apple-touch-icon / HTML snippet / web manifest / monochrome / resize algorithm / vectorize_on_raster) are collected under a collapsible "Advanced" chevron (accessible to power users without cluttering the main view). ICO generation section removed entirely — favicon.ico is always produced at fixed sizes (16/24/32/48), requiring no user input. PNG output sizes: six preset checkboxes (16/32/48/96/192/512) plus a custom-size input. SVG conversion: 3-position discrete slider (Simple ↔ Detailed) mapping to vtracer presets Sharp / Default / PhotoRich. Right Sheet width: `window_width / 3.0` clamped to `[280 px, 480 px]` (`shell::drawer_pixel_width`); calculated on the app side because snora 0.8 `SheetSize` has no min/max combination mechanism. Window size subscribed via `iced::window::events()` `Resized` / `Opened` → `Message::WindowResized`. Footer: only "↻ Reset" (left-aligned); Close consolidated into the × button. Old accordion helpers, `AdvancedGroup*` types, `badge_muted_bg`, and `close_button_style` removed. 9 new `MessageKey` entries + en/ja translations. Keep-transparency toggle: UI only in v1.17 (always true placeholder); full implementation in v1.21.0. |
| **v1.18.0** | **Header icons moved to a left vertical sidebar**: old top-right horizontal icon row (language / theme / settings / close) migrated to a left sidebar per the design mock. Header simplified to app name (Empty/Converting) or file name (Result) only. Close button removed entirely — delegated to the OS native window chrome (future browser-port consideration). Icons: lucide-icons via snora 0.8 `widgets` + `lucide-icons` features — `lucide::Settings` / `lucide::Languages` / `lucide::Moon`. Old cycle-UI `language_icon_glyph()` / `theme_icon_glyph()` removed. Language/theme pickers implemented as click-to-open popups via snora's `context_menu` slot (`AppLayout::context_menu(...)` + `on_close_menus`; outside-click dismisses); only one open at a time. Cycle UI replaced by direct selection (`LocalePicked(Option<Locale>)` / `ThemePicked(ThemeMode)`) with ✓ prefix on the current value. Sidebar: 90 px wide, icon (22 px) + label (11 px) two-row layout assembled on the logolig side (snora's `app_side_bar` widget is 64 px + tooltip only, not matching the mock's label-below style). Active state: `palette.background.strong.color` at alpha 0.35 + label colour switches to `page_title` (ABDD §12, not colour alone). `AppLayout::side_bar(...)` slot integrates with snora's skeleton and supports automatic LTR/RTL layout. 9 new `MessageKey` entries + en/ja translations. Simplified Chinese locale deferred to a separate phase. |
| **v1.19.0** | **Dead Message cleanup + `exporter::run_in_memory` direct API**: 6 dead `Message` variants removed — `ExportRequested` / `ExportDirPicked` / `ExportCompleted` (old "Export → pick dir → bulk write" flow, superseded by per-file DL / ZIP DL in v1.16), `LocaleCycled` / `ThemeToggled` (old cycle UI, superseded by direct pickers in v1.18), `AppCloseRequested` (old close button, removed in v1.18). Related handlers, `pick_export_dir_task`, `export_task`, old `action_row` in `preview_panel`, and orphaned `secondary_button_style` all removed. Core change: **`exporter::run_in_memory(asset, plan) -> Result<Vec<InMemoryArtifact>, AppError>`** added — pure in-memory, zero disk I/O, future browser-port friendly, no temp-directory leak risk. Old `exporter::run` (disk version) refactored into a thin wrapper that calls `run_in_memory` then writes atomically via a staging directory; existing 12 tests pass unchanged. `task_queue::convert_in_memory_task` renamed to `convert_task` and simplified to call `run_in_memory` directly (temp-directory create/write/read/cleanup cycle eliminated). 5 new unit tests for the in-memory API covering artifact count / order / relative paths / byte-level match with disk run / mono subdirectory representation / error cases / optional artifact toggles. |
| **v1.20.0** | **Mobile layout**: window width `< 768 px` (Bootstrap `md` breakpoint / old iPad mini portrait boundary) is treated as mobile. Detection centralised in `app::is_mobile(state) -> bool`; each UI module branches on it. Changes: (1) Left sidebar (90 px) → bottom nav (64 px tall, 3 equal cells) on mobile. New module `ui::bottom_nav` (twin of `ui::sidebar`) fires the same Messages and uses the same active-state style. `shell.rs` switches between `AppLayout::side_bar()` (desktop) and `AppLayout::footer()` (mobile). (2) Settings Right Sheet width on mobile: clamped to `[280, window_width - 16]` (nearly full-width, 16 px margin). Desktop retains v1.17 `[280, 480]`. `drawer_pixel_width(window_width, mobile)` signature updated. (3) Asset card grid: 2 columns mobile, 3 desktop. `result_view::build_grid` gains `columns: usize`. (4) Header horizontal padding: 20→8 on mobile; vertical unchanged. Startup transient: `AppState::window_size` defaults to 1280×720 so the first frame may render as desktop before the first resize event corrects it — harmless. |
| **v1.21.0** | **Keep-transparency toggle (full implementation)**: the "Keep transparency (alpha)" checkbox added in v1.17.0 as a UI-only placeholder (always true) is now a real setting. New `ExportPlan::keep_transparency: bool` field (default `true` — modern favicon standard, backward-compatible). Old settings JSON without this field is filled via struct-level `#[serde(default)]`. When false: every pixel is composited against a white background using Porter-Duff "over", producing a fully-opaque `Rgba8`. New module `services::flatten` implements `flatten_to_white(rgba) -> Rgba8` with 5 unit tests. Applied at the final stage of `exporter::render_at_size`, covering all raster outputs (PNG / ICO frames / apple-touch / mono PNG / mono ICO) in one place. SVG outputs are unaffected (flattening is a raster concept). JPEG sources are already alpha=255, so the setting has no effect on them. `Message::KeepTransparencyToggled(bool)` added; old `NoOp` placeholder replaced. Persisted. 4 integration tests added (transparency preserved with `true`; all pixels alpha=255 with `false`; SVG byte-identical regardless; ICO frames all alpha=255 with `false`). Hot-path optimisation: skip f32 arithmetic for alpha=255 and alpha=0 cases. |
| **v1.22.0** | **Side-nav redesign + full English translation + snora 0.18 upgrade**: the right-side settings drawer and sidebar icon/picker overlay approach was replaced by a three-item side navigation bar (Home / Customize / Settings); each item swaps the entire main body with no drawers or popups. Home is the existing app flow (drop zone → converting → Result); Customize is the former settings drawer content rendered full-page; Settings is language and theme selection. New modules: `ui::sidebar`, `ui::customize_page`, `ui::settings_page`; deleted: `ui::picker_overlay`, `ui::bottom_nav`. Three new `MessageKey` entries (`NavHome` / `NavCustomize` / `NavSettings`) with en/ja translations. All source comments and documentation translated to English throughout all three crates (~1,550 lines across 60 `.rs` files, 3 `Cargo.toml` files, and 3 `docs/` Markdown files). Five `update()` state-machine smoke tests added to `app.rs` (`#[cfg(test)]` only, no `iced_test` dependency). snora dependency bumped from 0.8 → 0.18 (additive only; notable inherited improvements: toast ordering bug fixed, `snora::keyboard::dismiss_on_escape` now available for the deferred ESC feature). Doc accuracy audit: `SidebarLabel*` docs corrected, `advanced_open` vestigial status documented, `window_size` doc updated, `AdvancedToggled` doc added. |
| **v1.23.0** | **snora 0.18 upgrade + English translation + smoke tests**: snora dependency bumped from 0.8 → 0.18 (additive; toast ordering bug fixed as a side-effect, `snora::keyboard::dismiss_on_escape` now available for the deferred ESC feature). All source comments and documentation translated to English throughout all three crates (~1,550 lines across 60 `.rs` files, 3 `Cargo.toml` files, 3 `docs/` Markdown files). Five `update()` state-machine smoke tests added to `app.rs` with no new dependencies. Doc accuracy audit: `SidebarLabel*` docs corrected, `advanced_open` vestigial status documented, `window_size` doc updated. |
| **v1.23.1** | **Patch: workspace resolver and edition regression fix**: `[workspace] resolver` restored to `"3"` and `[workspace.package] edition` restored to `"2024"`. Both were silently downgraded to `"2"` / `"2021"` during the Cargo.toml Japanese→English translation pass in v1.22.0 and carried into v1.23.0. No code changes; build semantics are now identical to v1.21.1. |
| **v1.24.0** | **Lucide download icons + history management**: Download buttons on the Result screen (per-card and "Download all") now use `lucide::Download` rendered via `icon_element_sized` instead of the Unicode `↓` glyph. History management: `EditCancelled` (← Back) no longer clears `result_assets` or `source_asset`; they are kept until a new file is ingested. When a previous result is present on the Empty screen a "Last conversion" card appears below the drop zone showing the source file name, asset count, and a `lucide::History` "View results →" button that returns to `Screen::Result` without re-converting (`Message::ShowLastResultRequested`). Old results are automatically replaced when a new file is ingested. 2 new `MessageKey` entries (`HistoryLastConversionLabel` / `HistoryViewResultsButton`) with en/ja translations. |
| v1.25.0+ (option) | Optional features (planned): (a) **Mobile UX refinements** — dynamic picker popup width (`window_width - 32 px`), accurate icon-adjacent positioning, bottom-sheet-style picker, hamburger menu (when a 4th sidebar icon is added). (b) **User-specified flatten colour** — v1.21.0 is white-only; black or arbitrary colour (`flatten_color: [u8; 3]`) is a straightforward extension. (c) **BMP input** — enable the `image` crate `bmp` feature; low implementation cost. (d) **Simplified Chinese locale** — same scope as the v1.6.0 Japanese addition (~100 translation keys). |

### Implementation handoff: `rfcs/`

Detailed implementation specs for the `v1.25.0+ (option)` themes above
live in [`rfcs/`](../rfcs/). Each priority sub-bullet (a–d) has its own
RFC document covering external design (where applicable), internal
design, requirements, test plan, and security considerations. The RFC
folder is the canonical handoff artifact for an implementer picking up
one of these themes; this ROADMAP row is intentionally a one-paragraph
summary.

| Sub-bullet | RFC |
| --- | --- |
| (a) Mobile UX refinements | [`rfcs/0001-mobile-ux-refinements.md`](../rfcs/0001-mobile-ux-refinements.md) |
| (b) User-specified flatten color | [`rfcs/0002-user-specified-flatten-color.md`](../rfcs/0002-user-specified-flatten-color.md) |
| (c) BMP input support | [`rfcs/0003-bmp-input-support.md`](../rfcs/0003-bmp-input-support.md) |
| (d) Simplified Chinese locale | [`rfcs/0004-locale-zh-cn.md`](../rfcs/0004-locale-zh-cn.md) |
