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
| **v1.10.2** | **Main panel refresh**: the top of the window is now a single header bar — left side shows the app name "Logolig" muted with a "— favicon ジェネレータ" tagline; right side is a row of four icon-only buttons (`文A`/`Aa`/`あ` for language cycle, `◐`/`☀`/`☾` for theme cycle, `⚙` for advanced, `✕` for close) each wrapped in an iced 0.14 `tooltip`. The footer was dropped because the advanced toggle moved into the header. Icon buttons are transparent until hovered; the close button is separated from the others by a wider 16px gap to reduce mis-clicks. The drop zone was slimmed to a single headline ("Drop PNG, SVG, or WebP" / "PNG / SVG / WebP をドロップ") plus the Choose file… button, wrapped in a soft-bordered card with rounded corners and a weak fill — iced 0.14 has no `border-style: dashed`, so the card uses a 2px solid border + tinted background instead. The language picker was removed from the advanced drawer (now reachable only via the header icon, which cycles System default → English → 日本語 in three clicks); the `language_row` UI helper, `LocalizedLocaleChoice` wrapper, and `App preferences` advanced group are all gone. App close is wired through `iced::window::latest().and_then(window::close)` because iced 0.14 doesn't expose the active window id directly. |
| **v1.10.3** | **Advanced drawer accordion**: each of the three groups ("What to export", "Extras", "Rendering quality") now has a clickable heading that toggles its body. Default state opens "What to export" only — Extras and Rendering quality start collapsed because most users never touch them. The chevron glyph reflects state (`▼` expanded, `▶` collapsed) so it conveys two ways: shape silhouette and direction. State lives in the new `AdvancedGroupExpansion` struct on `AppState`, with no persistence — every time the drawer is opened it resets to the default expansion, on the assumption that the user is starting a fresh task and the most relevant group should always be the first one they see. The `Message::AdvancedGroupToggled(AdvancedGroup)` handler is a one-line `state.advanced_groups.toggle(group)`. The previous `group_heading` helper was replaced by `accordion_group(label, expanded, on_toggle, body)` which renders the clickable header button and conditionally the body. |
| **v1.11.0** | **JPEG support**: new `SourceKind::Jpeg` (extension `.jpg`/`.jpeg`), magic-byte detection (`FF D8 FF` SOI), `services/decode_jpeg.rs` going through `image::load_from_memory` (which fills the missing alpha channel with 255). Width/height are parsed from the first SOF (Start Of Frame) marker — `parse_jpeg_size` walks the JPEG marker segments, skipping standalone markers (SOI/EOI/RSTn/TEM) and segments with length fields, until it finds a SOF (markers C0–C3, C5–C7, C9–CB, CD–CF), then reads height (BE, 2 bytes) and width (BE, 2 bytes) from the data area. Image crate `jpeg` feature added to workspace. Pipeline branches updated: `preview::render_at`, `exporter::run` (decoded_raster + SVG vectorize), `exporter::render_at_size`. Drop-zone i18n updated to "Drop PNG, SVG, WebP, or JPEG" / "PNG / SVG / WebP / JPEG をドロップ". Educational warning: since JPEG cannot store an alpha channel, the v1.7 transparency audit always returns `FullyOpaque` for JPEG inputs; rather than show the generic "no transparency detected" warning (which sounds like the user did something wrong), the `PreviewBuilt` handler branches on `SourceKind::Jpeg` and surfaces a JPEG-specific toast — `ToastJpegInputTitle` / `ToastJpegInputBody` — explaining that JPEG cannot have transparency and recommending PNG with a cut-out background. Tests: 5 decode_jpeg + 4 ingest cases (magic byte, both extensions, SOF parsing, truncated-file robustness). |
| **v1.12.0** | **Edit-screen flow + preview redesign**: the preview screen now has the back-navigation paths users were missing. A "← Back" button (`Message::EditCancelled`) clears the loaded source and returns to the empty drop-zone screen, and a "↻ Re-select" button reuses `Message::PickFileRequested` to open the file picker without leaving the screen — if the user cancels the picker, they stay on the current edit screen. Both auxiliary buttons sit on the left of the action row with a controlled muted style (transparent background, weak border, hover tint) to differentiate them from the primary Export button on the right (theme-primary, larger padding, heavier text). The header was made screen-aware: on the empty/importing/exporting screens it shows "Logolig — favicon ジェネレータ" as before, but on the preview/export-ready screens it shows the loaded file's display name in a slightly darker color (`FILE_NAME_COLOR`), making the loaded image the screen's subject. The preview screen got a centered page title ("プレビュー確認・Favicon ファイル作成" / "Preview & Generate Favicon") above a soft-bordered preview card. Inside the card, the View as / Surface picker rows are centered and sit *above* the preview frame, so they read as "controls for this preview". The preview frame itself uses `Length::FillPortion(4)` for its height with `max_width(560.0)` / `max_height(560.0)` caps and `Length::Fill` width — the size now scales with window height (≈4/7 of the available vertical space) instead of being pinned to whatever the inner content (tab mock, phone mock, checker) wanted, which fixes the "Export button jumps when switching modes" issue. ESC-to-back was deferred to a future release because it requires an `iced::keyboard::on_key_press` subscription that doesn't yet exist in the app. A separate request document, `docs/snora-upstream-request-bottom-sheet.md`, was prepared for the snora maintainer covering the BottomSheet scrolling and sticky-footer needs that will unblock v1.14.0. |
| **v1.13.0** | **snora 0.4 → 0.8 migration**: snora 0.8.0 ships an API restructure that resolves several pending logolig blockers in one upgrade. The framework was split into a 3-crate workspace (`snora-core` for vocabulary, `snora-widgets` for prefab visuals, `snora` as the engine umbrella); the bottom-only `BottomSheet` was generalized into `Sheet` + `SheetEdge` (Top/Bottom/Start/End) + `SheetSize` (axis-perpendicular size; `OneThird` / `Half` / `TwoThirds` / `Ratio(f32)` / `Pixels(f32)`); `AppLayout::bottom_sheet` was renamed to `AppLayout::sheet`; `lucide-icons` and `svg-icons` features are now usable; new `Tab` / `TabBar` / `Crumb` / `BreadcrumbAction` vocabulary was added (logolig is single-screen so doesn't consume them yet); the toast position default flipped from `BottomEnd` to `TopEnd`, which was exactly what logolig wanted in the never-shipped v1.16.0 plan — that whole roadmap line is now redundant and was removed. logolig's migration is small and surgical: `snora = { version = "0.8", default-features = false }` in workspace `Cargo.toml`; `BottomSheet` → `Sheet`, `SheetHeight` → `SheetSize`, `with_height(...)` → `with_size(...)`, `bottom_sheet(...)` → `sheet(...)` in `shell.rs`. `Sheet::new`'s default edge is `SheetEdge::Bottom`, so the visual behavior is identical to the previous `BottomSheet`. `lucide-icons` adoption (replacing the existing `文A` / `◐` / `☀` / `⚙` / `✕` Unicode glyphs in the header) is intentionally **out of scope for this version** — that's an independent UX decision, scheduled for a later v1.13.x or v1.14.x phase, so this release stays purely a dependency/API update. The same Sheet upgrade also clears the path for the v1.15.0 advanced-drawer scrolling work: with the new `Sheet` being a pure content carrier, wrapping the drawer body in `iced::widget::scrollable` becomes the obvious solution and no longer requires upstream coordination — the `docs/snora-upstream-request-bottom-sheet.md` request document is now historical context only. |
| **v1.14.0** | **Dark-mode color integrity + visual hierarchy**: light テーマ前提で hard-coded だった `Color::from_rgb(...)` 系の定数 (約 10 箇所) を `iced` の `extended_palette()` 経由の役割ベース色に置き換えた。 新モジュール `crates/logolig-app/src/ui/colors.rs` が役割関数 (`app_name(theme)` / `tagline(theme)` / `file_name(theme)` / `page_title(theme)` / `section_label(theme)` / `group_heading(theme)` / `muted_text(theme)` / `drop_zone_headline(theme)` / `badge_muted_bg(theme)`) を提供し、 各画面は `crate::app::resolve_theme(state)` で解決した `&Theme` を渡してテーマに追従する色を取得する (旧 `theme()` 関数を `pub(crate) fn resolve_theme()` に格上げ; 旧名は iced の `application().theme()` callback 互換のため薄いラッパとして残置)。 「テーマで変えるべき色」 と「意図的に hard-coded」 の境界を docstring で明示しており、 後者には (a) 透過チェッカーの市松グレー (`#E6E6E6` / `#C0C0C0` — 透明度の指標そのもの)、 (b) ブラウザタブ/スマホ/壁紙プレビューの色 (`chrome_bg_for` 系 — `Surface` ピッカーで切り替える独立軸) が含まれる。 視覚階層の役割名 (page title / section label / group heading / muted / drop zone headline) を統一したことで、 全画面でフォントサイズ ↔ 役割 ↔ テーマ追従の対応が明確になった。 `accordion_group` ヘルパは `heading_color: Color` パラメータを取るよう拡張、 `subsection` ヘルパは `muted_color: Color` パラメータを取るよう拡張、 `size_subsection` は内部で `resolve_theme(state)` を呼んで「at defaults」 バッジの色を解決する。 light/dark の切替は header の `◐/☀/☾` ボタンで動作し、 全テキスト・カード境界・バッジ背景・muted 説明文がテーマに追従する。 |
| **v1.15.0** | **Advanced drawer scroll + sticky footer**: 詳細設定ドロワーの旧構造は 1 つの column の末尾に Reset/Close を置いていたため、 アコーディオンを全展開すると フッタが Sheet 下端から押し出されて押せなかった。 改善指示書 §設定画面 1〜2 への対応として、 ドロワー内を 3 段に分割: (1) タイトル領域 (Length::Shrink、 上に固定)、 (2) `iced::widget::scrollable` でラップしたグループ群 (`Length::FillPortion(1)` で余り空間を全部食う)、 (3) フッタ (Length::Shrink、 下に固定 = sticky)。 内容が長くても中段だけが内部スクロールし、 タイトル と フッタは常時可視。 v1.13.0 で snora 0.8 の `Sheet` が pure content carrier として明文化されたため、 ラップは logolig 側で完結し snora 側の対応は不要だった。 同時に §設定画面.4 「フッタ操作の意味明確化」 に対応: Reset (左寄せ、 destructive 寄り — 透明背景 + `palette.danger.weak.color` の枠線、 hover 時に danger 色を alpha 0.15 で薄く塗り) と Close (右寄せ、 補助 — `secondary_button_style` 相当の中立スタイル) で見た目を分離し、 row 内の `Space::new()` で左右配置の差を作った。 これによりユーザは「Reset = 押せるが慎重に」 「Close = 普通の離脱」 を視覚で判別できる。 スクロールバーは iced 0.14 デフォルトの常時表示を採用 (改善指示書 §設定画面.2 「初期表示で全体像を把握」 のため、 hover-only ではなく「下にもっと項目がある」 ことを視覚化)。 Sheet 全体のサイズは引き続き `SheetSize::Half` (画面の 1/2)、 これは下のメイン編集画面とのバランスを保つため (Sheet を 2/3 に広げると preview が圧迫される)。 |
| **v1.16.0** | **画面構造の刷新 — 新外部設計の最初の段階**: ユーザから提示された新画面モックを起点に、 logolig の主構造を「投入 → 即変換 → 結果一覧 + 個別 DL / ZIP 一括 DL」 に作り直した。 旧 5 状態 (`Empty / Importing / Preview / Exporting / ExportReady`) を 3 状態に簡素化 (`Empty / Converting / Result`)。 新設計の中心は **Preview 状態の廃止** で、 v1.15 まではユーザがファイル投入後に「View as / Surface ピッカーで見え方を確認 → Export ボタンを押して書出」 だったが、 v1.16 では「投入したら即変換、 結果画面でアセットカード一覧 + 個別 DL or ZIP 一括 DL」 のフローに変わる。 Preview の確認機能は完全には廃止せず、 結果画面の「▶ プレビューを見る」 折りたたみセクションとして任意展開可能に残した (Q1-b 推奨)。 メモリ完結変換: 旧モデルは「Export = ディスクに直接書出」 だったが、 v1.16 は変換結果を `crate::result::ResultAssets` (アプリ状態) に in-memory で保持し、 ユーザが個別 DL / ZIP DL ボタンを押した時に初めて `rfd::AsyncFileDialog` で保存先を聞いて書出す。 これにより favicon 一式の生成と保存が分離され、 「変換だけして気に入らなければ捨てる」 が自然にできる。 ZIP 一括 DL は `zip = "2"` crate で in-memory に圧縮してから書出。 Convert は実装段階性のため、 当面は `exporter::run` を OS 一時ディレクトリに走らせて結果を読み戻す形で実装 (`task_queue::run_convert_in_memory`)、 サムネ用に PNG / ICO は `image` crate でデコードして `Rgba8` キャッシュ; `exporter::run_in_memory` の直接 API は v1.17 以降の課題として残置。 新 UI モジュール 2 つ追加: `ui::converting` (旧 Importing / Exporting を統合した処理中画面)、 `ui::result_view` (アセットカードグリッド 3 列、 `ResultAssetKind` でカード見た目を分岐 — PNG/ICO はラスタサムネ、 SVG は `<>` プレースホルダ、 manifest は `{ }`、 snippet は `{}`)。 各カードはファイル名、 サムネ、 バッジ (PNG/ICO/SVG/HTML/JSON)、 寸法 (画像系のみ)、 サイズ (KB/MB human-readable)、 ↓ 個別 DL ボタンを持つ。 PNG モックの「すべてダウンロード (ZIP)」 ボタンが下部に配置され、 ZIP 化して保存。 ResultAssets の合計バイト数は通常 1 MB 未満なのでメモリ保持コストは実質ゼロ。 BMP 入力は今回見送り (Q5-b、 別フェーズ)。 |
| v1.17.0 | 設定ドロワーの右サイド化 + flat 再編 (planned): PNG モックでは旧 Bottom Sheet ではなく **Right Sheet** に変更。 snora 0.8 の `Sheet::at(SheetEdge::End)` で実装可能 (snora 側への upstream 要望は不要)。 同時に、 旧アコーディオン 3 段構成を flat なセクション群に整理 (出力サイズ チェックボックス / ICO 生成トグル 2 つ / SVG 変換方式スライダー / その他)。 |
| v1.18.0 | ヘッダーアイコン群の左サイド縦並び化 (planned): PNG モック準拠で、 旧の右上横並びアイコン (言語 / テーマ / 設定 / 閉じる) を **左サイド縦並び** に移行。 デスクトップ・オフラインブラウザでこの種のツールを初めて触るユーザにも「アイコン = 機能」 の対応が分かりやすい配置を採用 (Q7-a)。 モバイルレイアウト時は下部ナビに移行する想定 (PNG モック明示)。 |
| v1.19.0 | 旧 Export* メッセージ系統の整理 (planned): v1.16.0 で `convert_in_memory_task` 経路が主導線になったが、 旧 `ExportRequested` / `ExportDirPicked` / `ExportCompleted` は phase A 互換のため残置中。 Result 画面に到達する経路を `ConvertCompleted` のみに収束させ、 旧パスを削除する。 `exporter::run_in_memory` 直接 API もこのタイミングで導入を検討 (現状の OS 一時ディレクトリ経由は実装段階性のための仮実装)。 |
| v1.20.0+ (option) | モバイルレイアウト対応 / BMP 入力 (planned): 横幅で flex する `ResponsiveSheet` 様の振る舞い、 下部ナビゲーション、 ハンバーガー メニュー など (PNG モックのモバイル表示準拠)。 BMP は `image` crate の `bmp` feature を有効化するだけで実装は軽い。 |

v1.0.0 is the feature-complete iced/native build. It is in
maintenance mode: security and critical-bug fixes only.

## Where v2 is going

v2 retargets the same `logolig-core` to a leptos-based WebAssembly
build, distributed as a privacy-preserving in-browser app
(Service Worker for offline, no upload). The split is feasible
specifically because v1's `logolig-core` carries no GUI-framework
dependency — the workspace boundary that was set up in Step 1
becomes the seam.

Two changes in `logolig-core` are anticipated for v2 but are
deliberately not made in v1 (they would be churn without immediate
payoff in v1):

1. **Make ingest WASM-friendly.** `services/ingest::ingest_bytes`
   (already present, used by tests) becomes the canonical API; the
   `tokio::fs::read`-based `ingest(path)` wrapper moves to a
   native-only feature gate.
2. **Split exporter into a pure half and a fs half.** The pure half
   produces `Vec<(PathBuf, Vec<u8>)>`; the fs half writes
   transactionally as today. Browsers will use the pure half plus a
   zip download or File System Access API.

Neither change affects v1 behaviour. Both are recorded here so the
v1 reader knows why the boundary is shaped the way it is.
