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
    │   │       ├── rasterize_svg.rs # SVG → Rgba8 (resvg + tiny-skia, per-size render)
    │   │       ├── resize.rs        # Rgba8 → Rgba8 (fast_image_resize, Lanczos3 default)
    │   │       ├── vectorize.rs     # Rgba8 → SVG string (vtracer, since v1.2)
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
        │   └── ui.rs / ui/
        │       ├── drop_zone.rs
        │       ├── preview_panel.rs
        │       ├── advanced_drawer.rs
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
| v1.7.0 | Transparency checker (planned) |
| v1.8.0 | Web manifest output (planned) |
| v1.9.0 | Monochrome output set (planned): opt-in extra output that produces a black-and-white version of every artifact for use cases like single-color print or theme-aware light/dark icons. Likely realized as `ExportPlan::monochrome: Option<Monochrome>` with a saturation knob and emit alongside the colored set under a `mono/` subdirectory. |
| v1.10.0 | Toast positioning (planned, depends on snora 0.5+): snora 0.4 hard-codes toast placement to bottom-end (right in LTR, left in RTL). User testing showed this is hard to notice for our screen layout; top-end (right side, near the title bar) would be more visible. The fix belongs upstream as `AppLayout::toast_position(ToastPosition)` so other snora users benefit too. logolig will adopt it via `ToastPosition::TopEnd` once available. |

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
