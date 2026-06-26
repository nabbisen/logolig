# Logolig Functional Specification

> This document describes **what logolig can do and how it behaves** at
> v1.15.0, as observable behaviour rather than implementation detail.
> It is intended as input for UX review and external design decisions.
>
> The document separates **functional specification** (what it does) from
> **current UI layout** (how it presents it). The latter is the current
> design choice and is open to revision without changing the functional
> contract.

## 1. Purpose

A local-first favicon generator. Takes a single image (logo source) as
input and produces a **multi-format, multi-size** favicon bundle for web
use. Follows ABDD principles: accessible, internationalised, fully local.

### Out of scope

- Image editing (colour correction, cropping, painting) — logolig handles
  "favicon generation" only; it is not an image editor.
- Cloud upload / SaaS — all file I/O is on the local disk.
- Batch processing (multiple source images) — one image per session.

## 2. Input

### Accepted formats

| Format | Extension | Magic bytes | Notes |
|---|---|---|---|
| PNG | `.png` | `89 50 4E 47` | Transparency supported |
| SVG | `.svg` | `<?xml` / `<svg` | Vector source (rasterised by resvg) |
| WebP | `.webp` | `RIFF...WEBP` | Transparency supported |
| JPEG | `.jpg` / `.jpeg` | `FF D8 FF` | **No transparency** (alpha forced to 255) |

### Ingestion paths

1. **File dialog** — "Choose file" button
2. **Drag and drop** — anywhere on the window

### Validation

- Magic bytes and file extension must agree (prevents extension spoofing).
- Unrecognised formats produce an `UnsupportedFile` error.

## 3. Output (generated artifacts)

The following artifacts are produced depending on user settings.
Items marked ON are enabled by default.

| Artifact | Content | Default |
|---|---|---|
| `favicon.ico` | Multi-size ICO containing 16/24/32/48 px frames | ON |
| `apple-touch-icon.png` | Single 180×180 px PNG | ON |
| `favicon-16.png` … `favicon-512.png` | Per-size PNGs (default: 16/32/48/180/192/256/512) | ON |
| `favicon.svg` | (a) original SVG copied as-is, or (b) vtracer-vectorised from raster | ON |
| `favicon-snippet.html` | Ready-to-paste `<link>` tag block | ON |
| `manifest.webmanifest` | PWA Web App Manifest JSON | ON |
| `mono/` subdirectory | Monochrome (BT.709 greyscale) versions of the above | OFF |

### Atomic export

All artifacts appear together on success; no partial output is left on
failure. The implementation writes to a staging directory, then renames
atomically into the output directory.

### Size customisation

PNG and ICO size lists are editable in the UI (16–1024 px). Customisations
are persisted across sessions.

### vtracer presets (SVG generation from raster)

- **Sharp** (default): crisp contours for logos and icons
- **Default**: balanced
- **PhotoRich**: smooth curves for photo-like or gradient sources

### Resize algorithm

- **Lanczos3** (default): highest quality
- **Bilinear**: faster
- **Nearest**: pixel art

## 4. UI structure

### Screen states

The app moves through three states (v1.16.0):

```
  ┌──────────────────┐
  │  Empty (startup) │
  └────────┬─────────┘
           │ file dropped / chosen
           ▼
  ┌──────────────────┐
  │   Converting     │  ← brief (decode + conversion)
  └────────┬─────────┘
           │ conversion complete
           ▼
  ┌──────────────────┐
  │     Result       │  ← asset cards + download buttons
  └──────────────────┘
```

### Transition triggers

| Action | Result |
|---|---|
| Drop / choose file | Empty → Converting → Result |
| Individual download | Save dialog → single file written |
| Download all (ZIP) | Save dialog → zip bundle written |
| ← Back (Result screen) | Result → Empty (source cleared) |
| ESC key | **Not implemented** (deferred) |

## 5. Navigation (v1.22.0 side nav)

A three-item side navigation bar is always visible. Each item swaps the
main body:

| Nav item | Body content |
|---|---|
| **Home** | Main app flow (drop zone / converting / result) |
| **Customize** | Full-page output settings (was the right-side drawer) |
| **Settings** | Language and theme selection |

On mobile (window width < 768 px), the side nav moves to the bottom
of the screen.

## 6. Drop zone (Empty state)

- Central drop area with a soft-bordered card and a weak fill background.
- Headline: "Drop PNG, SVG, WebP, or JPEG" and a "Choose file…" button.
- The entire window accepts drag-and-drop.

## 7. Result screen

After conversion, assets are shown in a grid (3 columns desktop /
2 columns mobile). Each card shows:

- File name and badge (PNG / ICO / SVG / HTML / JSON)
- Raster thumbnail (image assets) or document icon placeholder
- Dimensions (image assets only) and human-readable file size
- Per-card download button (↓)

Below the grid: a "▶ Preview" collapsible section and a
"↓ Download all (ZIP)" button.

### Preview panel

The collapsible preview offers three modes:

| Mode | Content |
|---|---|
| **Browser tab** | 16×16 icon at actual pixel size inside a simulated tab bar |
| **Phone home** | 120×120 icon inside a simulated home screen |
| **Checker** | Icon over a checkerboard (for inspecting transparency) |

A Surface picker (White / Grey / Black) controls the preview background.
The Surface picker is disabled in Checker mode (background has no effect
on the checkerboard).

Active state is indicated by both background fill and the `▣` prefix
(colour-blind safe per ABDD §12).

## 8. Customize page

Replaces the former right-side settings drawer (v1.22.0). Full-window
width, always reachable via the Customize nav item.

### Sections

**PNG output sizes** — six preset checkboxes (16/32/48/96/192/512) plus
a custom-size input. Checked sizes are included in the export.

**SVG conversion mode** — three-position slider (Simple ↔ Detailed)
mapping to vtracer presets Sharp / Default / PhotoRich.

**Misc** — "Keep transparency (alpha)" toggle. When off, raster outputs
are composited against white (Porter-Duff over) before writing.

**Advanced** (collapsible) — infrequently-changed settings:
- Include SVG output / vectorise raster sources
- Apple touch icon (180×180 PNG)
- HTML snippet file
- Web manifest (name, short name, theme colour, background colour)
- Monochrome output set
- Resize algorithm

**Footer** — ↻ Reset button restores all settings to defaults.

## 9. Settings page

Language and theme selection, reachable via the Settings nav item.

### Language

Supported locales: **English** / **Japanese**. The OS locale is detected
at startup (BCP-47 and POSIX `ja_JP.UTF-8` forms both supported). The
selection is persisted.

### Theme

System / Light / Dark cycle. Persisted across sessions.

## 10. Notifications (toasts)

Toasts appear at the top-right corner (TopEnd) for a short duration.

| Event | Content |
|---|---|
| Download complete | "Saved N files to `<path>`" |
| Transparency warning (PNG/SVG/WebP) | Advisory when the source is fully opaque |
| **JPEG educational warning** | "JPEG cannot store transparency. Consider converting to PNG." |
| Error | Decode failure, write failure, etc. |

### JPEG vs. generic transparency warning

JPEG sources always trigger the JPEG-specific toast (format limitation),
not the generic "fully opaque" warning (which implies the user may have
accidentally removed transparency). The tone encourages switching to PNG
rather than assigning blame.

## 11. Internationalisation

`logolig-i18n` manages all UI strings as a dictionary.

- Supported locales: English / Japanese.
- Locale selection is persisted.
- `MessageKey` enum + exhaustive match ensures no translation can be
  missing at compile time.

## 12. Theme colours

`crate::ui::colors` provides theme-reactive helper functions. All text,
card borders, badges, and muted descriptions follow the active theme via
`iced::extended_palette()`.

### Intentionally hardcoded colours (not theme-reactive)

| Colour | Reason |
|---|---|
| Checkerboard greys (`#E6E6E6` / `#C0C0C0`) | These indicate transparency itself; making them theme-reactive would imply transparency changed when the theme changed. |
| Browser tab / phone / wallpaper preview colours | Controlled by the Surface picker — an independent axis. Linking to app theme would break useful combinations like Surface=Light + App=Dark. |

## 13. Persisted settings

Stored as JSON in the OS standard config directory.

- Locale selection
- Theme selection
- Full export plan (checkboxes, PNG/ICO sizes, vtracer preset, resize
  algorithm, web manifest fields, monochrome toggle, keep-transparency
  toggle)

Not persisted: the Advanced accordion's expanded/collapsed state
(session-only).

## 14. Accessibility (ABDD §12)

- Active state (current nav item, current language/theme) is indicated
  by both a visual fill **and** the `▣` prefix (not colour alone).
- Keyboard alternative path: the "Choose file…" button covers users who
  cannot use drag and drop.
- Accessibility labels are defined as constants in `ui::accessibility`
  for future screen-reader support (iced 0.14 does not yet expose a full
  a11y API).

## 15. Offline operation

No network communication. All processing runs on the local CPU.

| Task | Library |
|---|---|
| Image decode (PNG / WebP / JPEG) | `image` crate |
| SVG rasterisation | `resvg` (uses OS fonts) |
| Vectorisation | `vtracer` |
| Resize | `fast_image_resize` (SIMD-accelerated) |
| ICO assembly | `ico` crate |
| Settings persistence | `app-json-settings` (OS config dir) |

## 16. Version history summary (v1.10+)

| Version | Theme | Summary |
|---|---|---|
| v1.10.0 | UI information architecture | Main screen reorganised |
| v1.10.1 | Crash fix | cosmic-text line-height=0 panic avoided |
| v1.10.2 | Main screen refresh | Four header icons, tooltips, slim drop zone |
| v1.10.3 | Accordion layout | Advanced drawer reorganised into 3 groups |
| v1.11.0 | JPEG support | New input format + educational transparency toast |
| v1.12.0 | Edit screen flow | Back / Re-select paths, preview card, size stable |
| v1.13.0 | snora 0.8 migration | Dependency update, Sheet API generalised |
| v1.14.0 | Dark mode colour integrity | All text/borders/badges theme-reactive |
| v1.15.0 | Advanced drawer scroll + sticky footer | 3-section layout, Reset/Close differentiated |
| v1.16.0 | Screen structure revision | Empty / Converting / Result; in-memory conversion |
| v1.17.0 | Settings drawer → Right Sheet + flat layout | PNG-mock section structure, ICO section removed |
| v1.18.0 | Left sidebar + picker popups | lucide icons, context-menu pickers |
| v1.19.0 | Dead code removal + `run_in_memory` API | Direct in-memory export API in logolig (core) |
| v1.20.0 | Mobile layout | Sidebar ↔ bottom nav, responsive grid |
| v1.21.0 | Keep-transparency toggle | Flatten service, full test coverage |
| **v1.22.0** | **Side-nav redesign** | **Three-page nav: Home / Customize / Settings** |

## 17. Design review axes

The following questions may be useful when reviewing the external design.

**Screen purpose clarity** — Is "drop a file" immediately obvious on the
startup screen? Does the Result screen make "Download" the clear primary
action?

**Settings discoverability** — Most users export without changing any
settings. Is the Customize page easy to ignore when not needed, and easy
to find when it is?

**Preview mode utility** — Do users know which of the three preview modes
to use, and when?

**Toast timing** — Are toasts shown at the right moment and visible long
enough?

**Single image per session** — If batch processing of multiple sources
is ever needed, can the current architecture accommodate it, or would it
require a structural change?
