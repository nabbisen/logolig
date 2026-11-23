# Logolig Functional Specification

> This document describes **what logolig can do and how it behaves** in the
> current tree (release 1.28.0; the previous release is 1.27.0), as observable
> behaviour rather than implementation detail.
> It is intended as input for UX review and external design decisions.
>
> The document separates **functional specification** (what it does) from
> **current UI layout** (how it presents it). The latter is the current
> design choice and is open to revision without changing the functional
> contract.

## 1. Purpose

A local-first favicon and logo asset generator. Takes a single image (logo source) as
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

1. **File dialog** — click the drop zone or the "Choose file" control
2. **Drag and drop** — anywhere on the window when the platform delivers native file-drop events

The file dialogs (choosing a file, and the two Save dialogs) are provided by the desktop's
portal. Where the system has no portal, or a portal with no file-dialog backend, and no `zenity`
program, no dialog can appear: the application then says so in one message instead of doing
nothing, and dragging a file onto the window still works. Known limit: a portal that accepts a
dialog and never answers leaves the application waiting; if a dialog never appears and never
answers, restart the application.

### Validation

**The content decides, the extension is a hint.** The format is recognised by its magic bytes (above), whatever the file is called, and decoding is chosen by that format, never by the name.

- A file whose name has a **known** extension (`.png`, `.svg`, `.webp`, `.jpg`, `.jpeg`) that contradicts its content (for example a JPEG named `logo.png`) is accepted as what it is, and **one** notice says so (see the notice table in §10). An unknown or missing extension with recognised content (`logo`, `logo.dat`) is accepted without any notice.
- Content that is not one of the four formats is rejected with "not a supported image" and the next step (choose a PNG, SVG, WebP or JPEG file), whatever the extension says: a text file named `logo.png` is rejected, a PNG named `notes.txt` is accepted.
- Only **regular files** are read. A folder, a named pipe or a device is rejected before anything is read (reading one could block or never end). A link to a regular file is followed.

### Input limits

Checked before the image is decoded, from the file's length and from the dimensions its header declares, so no pixel buffer is allocated for an image that is over a limit:

| Limit | Value |
|---|---|
| File size | 64 MiB (SVG: 8 MiB) |
| Width or height of a raster image | 16 384 px |
| Pixels of a raster image | 50 megapixels |

An image over a limit is not loaded; one message states the limit and the next step (use a smaller file, or reduce the size of the image) in English or Japanese. An image whose header cannot be read is decoded with the same limits as a ceiling, so a damaged file cannot make the application allocate without bound. Error messages about a file name the **file only**, never its folder.

## 3. Output (generated artifacts)

The following artifacts are produced depending on user settings.
Items marked ON are enabled by default.

| Artifact | Content | Default |
|---|---|---|
| `favicon.ico` | Multi-size ICO containing fixed 16/32/48 px frames | ON |
| `apple-touch-icon.png` | Single 180×180 px PNG | ON |
| `favicon-32.png` / `favicon-192.png` / `favicon-512.png` | Per-size PNGs (user-editable 16–1024 px) | ON |
| `favicon.svg` | (a) original SVG copied as-is, or (b) vtracer-vectorised from raster | ON |
| `favicon-snippet.html` | Ready-to-paste `<link>` tag block | ON |
| `manifest.webmanifest` | PWA Web App Manifest JSON | OFF |
| `mono/` subdirectory | Monochrome (BT.709 greyscale) versions of the above | OFF |
| Microsoft app logos | `StoreLogo.png`, `Square44x44Logo.png`, `Square150x150Logo.png`, `Wide310x150Logo.png` | OFF |

### Microsoft app logos

The Microsoft app logo setting is an advanced, opt-in output group. When
enabled, Logolig generates four PNG files at the output root: `StoreLogo.png`
(50×50), `Square44x44Logo.png` (44×44), `Square150x150Logo.png` (150×150),
and `Wide310x150Logo.png` (310×150). The wide logo uses contain-fit rendering
on a transparent canvas so the source logo is not cropped or stretched.

### Export paths

The application generates every artifact **in memory** and writes bytes only after the
user picks a save target (one file or the ZIP). The ZIP is built in memory and written in
one step, so a failed save leaves no half-written archive.

The library `logolig` also offers a disk export (`services::exporter::run`) that writes into
a directory with all-or-nothing semantics: it writes to a staging directory, then renames
atomically into the output directory, and removes the staging directory on failure. The
application does not use it; the integration tests of the library crate do.

### Size customisation

PNG size lists are editable in the UI (16–1024 px): six preset sizes are checkboxes, a size
typed into the custom-size field becomes a checked entry that can be unchecked again. PNG
customisations are persisted across sessions.

ICO frames are **16, 32 and 48 px and cannot be changed in the UI**. (The persisted plan
does carry an `ico_sizes` field (its add method accepts 16–256 px), which the application does
not expose and the exporter honours, so only an edited settings file can change it.)

### vtracer presets (SVG generation from raster)

Three presets, named in the interface as follows:

- **Default (balanced)**: the default.
- **Sharp (logos, icons)**: an opt-in preset intended for logos and icons.
- **Photo-rich (gradients)**: smooth curves for photo-like or gradient sources.

One group of these three choices, in the Customize page's "SVG conversion" section, is the only
control for this setting.

### Resize algorithm

Five algorithms, named in the interface as follows: **Lanczos3 (sharp)** (the default, highest
quality), **Mitchell-Netravali**, **Catmull-Rom**, **Bilinear** (faster) and **Nearest (pixel
art)** (no interpolation).

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


### Drop-zone behaviour (v1.26.1)

The Empty screen uses the full drop-zone card as one large click target. This
keeps the startup action accessible even on platforms where native OS file-drop
events are unavailable or unreliable. When the runtime reports file-hover events,
the card border and background are strengthened to show that a file can be
dropped.

Choosing or dropping a new file while one is still being converted replaces it: the
newest file wins, and anything the earlier file produces afterwards (its result or
its error) is discarded without a message. **A file that cannot be loaded (too large, not an image, unreadable) leaves the previous result in place:** the application shows the error and returns to the screen it was on, with the last result and its "View results →" card unchanged; the previous result is replaced only when the new file has loaded. (If a conversion was still running when the file was rejected, that conversion is the earlier file's and is discarded; the application shows the start screen.) Only one file chooser can be open at a
time; asking for a second one (the button or Ctrl+O) does nothing until the first is
closed. A file dropped while the chooser is open is accepted; the file chosen later
then replaces it.

### Transition triggers

| Action | Result |
|---|---|
| Drop / choose file | Empty → Converting → Result |
| Individual download | Save dialog → single file written |
| Download all (ZIP) | Save dialog → zip bundle written |
| ← Back (Result screen) | Result → Empty, with the last result kept for session history |
| Esc key | Closes the newest message; with none, Result → Empty (the same as ← Back). Other keyboard shortcuts: §14 |

ZIP bundle entries carry a local wall-clock modified timestamp so desktop
archive tools show useful "Last Modified" metadata alongside the created file.
The ZIP timestamp format has no timezone field, so this is intentionally local
time rather than UTC.

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

- File name and badge (PNG / ICO / SVG / HTML / JSON). Files in a subdirectory
  show their relative path as the title (for example `mono/favicon-32.png`), so
  a colour file and its monochrome twin are never visually identical.
- Raster thumbnail (image assets) or document icon placeholder
- Dimensions (image assets only) and human-readable file size
- Per-card "Save" button (with the download icon)

The screen has three parts: a fixed header with "← Back", a middle that scrolls (the
headline, the cards and the preview section), and a fixed footer with "Download all (ZIP)", so
that both buttons are in view at every window size. The preview section is a collapsible toggle
("Preview", with a chevron that shows open or closed) below the grid.

### Preview panel

The collapsible preview offers three modes:

| Mode | Content |
|---|---|
| **Browser tab** | 16×16 icon at actual pixel size inside a simulated tab bar |
| **Phone home** | 120×120 icon inside a simulated home screen |
| **Checker** | Icon over a checkerboard (for inspecting transparency) |

A Surface picker (System / Light / Dark) controls the preview background; System follows the operating system as the Theme setting does (see Settings, Theme).
The Surface picker is disabled in Checker mode (background has no effect
on the checkerboard).

Each picker is a vertical group of radio options under its label: the chosen option is a
filled dot in a ring, the group is one Tab stop in the focus order and ↑ ↓ ← → choose
(colour-blind safe per ABDD §12). The Surface group is dimmed as a whole in Checker mode.

## 8. Customize page

Replaces the former right-side settings drawer (v1.22.0). Full-window
width, always reachable via the Customize nav item.

### Sections

**PNG output sizes** — six preset checkboxes (16/32/48/96/192/512) plus
a custom-size input. Checked sizes are included in the export.

**SVG conversion mode** — one group of three named choices, "Sharp (logos,
icons)", "Default (balanced)" and "Photo-rich (gradients)", mapping to the
vtracer presets Sharp / Default / PhotoRich. The same group is the only control
for this setting (there is no separate preset list under the SVG option).

**Other** — "Keep transparency (alpha)" checkbox. When off, raster outputs
are composited against white (Porter-Duff over) before writing.

**Advanced** (collapsible) — infrequently-changed settings:
- Include SVG output / vectorise raster sources (two checkboxes; the vectorisation one only while SVG output is on)
- Apple touch icon (180×180 PNG)
- HTML snippet file
- Web manifest (off by default; name, short name, theme colour, background colour)
- Monochrome output set
- Resize algorithm (the five-option group)

**Footer** — "Reset to defaults" (↻) restores all settings to defaults. It sits below the scrolling part, so it is always in view.

## 9. Settings page

Language and theme selection, reachable via the Settings nav item.

The page scrolls as one when the window is short.

### Language

A group of three choices: **日本語**, **English** and **System** (follow the operating
system's language). The OS locale is detected at startup (BCP-47 and POSIX `ja_JP.UTF-8`
forms both supported). Choosing applies at once and is persisted.

### Theme

A group of three choices: **Light**, **Dark** and **System**. Choosing applies at once and is
persisted across sessions.

**System follows the operating system, where it reports a theme.** The application asks at start
and follows changes while it runs: when the system changes between light and dark, the page repaints
within a moment. On Linux the theme comes from the desktop portal (`org.freedesktop.portal.Settings`,
`org.freedesktop.appearance` `color-scheme`: dark, light or no preference). Where nothing reports a
theme (no portal, or a system with no preference) the page is **Light**. The System row says which
case applies, in words: "System (currently Dark)", "System (currently Light)" or "System (not
reported; using Light)" (the last one covers "no portal" and "no preference" alike: the program cannot
tell them apart). Light and Dark ignore the system. Only the choice (System, Light or Dark) is saved;
what the system reported is not. The Surface picker's **System** surface in the preview follows the
same value. **Windows and macOS were not exercised:** the same code path is expected to work there (the
windowing library reports the theme on both) but it has not been run.

### Keyboard shortcuts

A list of the global keyboard shortcuts (keys and what they do, in the interface language):
Ctrl+O "Choose a file", Ctrl+S "Download all (ZIP)", Esc "Close the message, or go back to the
start", Ctrl+1 / 2 / 3 "Go to Home, Customize or Settings", F1 "Show this list" (Cmd replaces
Ctrl on macOS). F1 opens this page.

## 10. Notifications (toasts)

Toasts appear at the top-right corner (TopEnd). How long a toast stays scales with the length
of its message, so that it can be read: at least 4 seconds and at most 20 seconds (a
longer message, or a long file path, stays longer; Japanese text is counted by character).
The "saved" toast keeps a minimum of 7 seconds. Error toasts stay until they are dismissed.
A toast does not expire while the mouse pointer rests on it; when the pointer leaves, its
countdown starts again from the full time, so the message can be read to the end.
Every toast has a close button (`×`).

| Event | Content |
|---|---|
| Download complete | "Saved N files to `<path>`" |
| Transparency warning (PNG/SVG/WebP) | Advisory when the source is fully opaque |
| **File name does not match the content** | "The file name does not match its content": the content is *X* but the name suggests *Y*; Logolig goes by the content. Shown once per loaded file, only when a **known** extension contradicts the content. For a JPEG the same toast also holds the JPEG explanation (below), so a mislabelled JPEG is explained once. |
| **JPEG educational warning** | "JPEG cannot store transparency. Consider converting to PNG." |
| **Non-square source advisory** | "Your logo is not square" / the logo is shown in full with empty space around it, so the icon may look small. Shown once per loaded image when the sides differ by more than 2 % (any source kind). |
| **SVG parts not shown** | "Some parts of your SVG are not shown": one toast per loaded SVG when it refers to image files on the computer (not read, for safety) and/or contains text (not drawn yet); the body holds the applicable sentences, images first. Nothing is shown for an SVG without either. |
| **No file dialog available** | "No file dialog available" with a body that says to drag a file onto the window or to install a desktop portal with a file dialog (a similar body for the Save dialogs). Shown when the application has positive evidence that no dialog can appear; **persistent** (stays until dismissed); **shown once**: while it is on screen, asking again does not add another. |
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
- Full export plan (checkboxes, PNG sizes, ICO frame sizes (default 16/32/48), vtracer preset, resize
  algorithm, web manifest fields, monochrome toggle, keep-transparency
  toggle)

Not persisted: the Advanced accordion's expanded/collapsed state
(session-only).

## 14. Accessibility (ABDD §12)

- Active state is never colour alone: the current nav item has a filled item, a
  bold label and a thin accent bar toward the content; the chosen language,
  theme and other single choices are a filled dot in a ring; a checked option is
  a ticked box. The marks are drawn from the bundled icon font, not typed.
- Keyboard: every control can be reached and operated without the mouse. Tab and
  Shift+Tab move the focus through the controls in the order of the screen (side
  navigation, then the page; no wrap), a neutral ring shows the focused control and
  the page scrolls to keep it in view; Enter and Space do what a click does; the
  arrow keys choose within a group of choices. Global shortcuts: Ctrl+O (choose a
  file, Home page), Ctrl+S (download all), Esc (close the newest message, or go
  back; in a text field it leaves the field), Ctrl+1/2/3 (Home, Customize,
  Settings) and F1 (Settings, where the list is shown); Cmd replaces Ctrl on macOS.
  A shortcut works only when the button it stands for is available, and never
  types into a text field. See `ui-a11y.md` ("Keyboard support").
- Screen readers are not supported: iced 0.14 exposes no accessibility tree.

## 15. Offline operation

No network communication. All processing runs on the local CPU.

| Task | Library |
|---|---|
| Image decode (PNG / WebP / JPEG) | `image` crate |
| SVG rasterisation | `resvg` (SVG text is not drawn yet: no fonts are loaded; images are read only from `data:` URLs, never from files) |
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
| v1.22.0 | Side-nav redesign | Three-page nav: Home / Customize / Settings |
| v1.23.0 | snora 0.18 | Dependency update, English translation completed, smoke tests |
| v1.24.0 | History | Lucide download icons; the last result is kept and reachable from the start screen ("View results →") |
| v1.25.0 | snora 0.25 | Dependency update |
| v1.26.0 | Microsoft app logos | Opt-in `StoreLogo`, `Square44x44Logo`, `Square150x150Logo`, `Wide310x150Logo` |
| v1.26.1 | Drop zone | Drag-and-drop repaired; the whole card is the click target; hover feedback |
| v1.26.2 | Stability | Release-gate and documentation stabilisation |
| v1.27.0 | First public release | Non-square logos shown in full; keyboard shortcuts; ZIP written all-or-nothing; safer SVG sources; toast time follows the text; faster large conversions |
| v1.28.0 | Keyboard and interface correctness | Every control reachable and operable by keyboard (Tab order, focus ring, always in view); radio groups, checkboxes and an accent bar as selection cues; "Back" and "Download all" always in view; a result always belongs to the file chosen; one notice when no file dialog exists |

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
