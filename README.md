# Logolig

[![License](https://img.shields.io/github/license/nabbisen/logolig)](https://github.com/nabbisen/logolig/blob/main/LICENSE)
[![crates.io](https://img.shields.io/crates/v/logolig-app?label=app)](https://crates.io/crates/logolig-app)
[![crates.io](https://img.shields.io/crates/v/logolig-core?label=core)](https://crates.io/crates/logolig-core)

A local-first, accessible favicon generator GUI.

PNG / SVG / WebP goes in. A polished `favicon.svg`, `favicon.ico`, an
Apple touch icon, high-resolution PNGs, and a clean HTML `<head>` snippet
come out — all on your machine, with no upload anywhere.

Built on [iced 0.14](https://iced.rs/) and
[snora 0.4](https://github.com/nabbisen/snora).

## Why

Existing online favicon generators have problems:

- Image quality degrades on resize (jaggies, fuzziness, lost detail)
- Images are uploaded to a third-party server
- Output is bloated with sizes you don't need
- Generated HTML often skimps on accessibility
- The UI demands too many decisions up front

Logolig addresses all of these as **design requirements**, not
afterthoughts. See `docs/architecture.md` for the rationale.

## Installation

Requires Rust 1.85 or newer.

### From source

```sh
git clone https://github.com/nabbisen/logolig
cd logolig
cargo install --path crates/logolig-app
logolig
```

`cargo install` places a `logolig` binary in `~/.cargo/bin/`. Make
sure that directory is on your `$PATH`.

### From crates.io (after publication)

```sh
cargo install logolig-app
logolig
```

## Running without installing

```sh
cargo run -p logolig-app --release
```

## Usage

The window has a slim header at the top with four icon buttons on the
right — language (`文A`/`Aa`/`あ`), theme (`◐`/`☀`/`☾`), advanced (`⚙`),
and close (`✕`). Each button shows a tooltip when you hover over it.
Language and theme each cycle through their three states with one click.

1. Drop a PNG, SVG, WebP, or JPEG onto the window, or click **Choose
   file…** inside the bordered drop card on the empty screen.
   - JPEG inputs work, but the format cannot store transparency, so
     Logolig shows an educational toast suggesting you re-export your
     logo as a PNG with the background cut out for a proper favicon
     that adapts to dark and light browser tabs.
2. Once an image loads, inspect the preview using the **View as** picker:
   - **Browser tab** — 16×16 in a tab-frame mock
   - **Phone home** — the icon at home-screen size on a phone-style mock
   - **Checker** — drop the framing entirely and show the icon over a
     light/dark grey checker pattern so transparent regions are obvious
3. Switch the **Surface** (System / Light / Dark) to see how the icon
   reads on each background. The Surface picker greys out automatically
   when Checker is active because background tinting doesn't apply there.
4. (Optional) Click the gear icon (**⚙**) to open the advanced drawer.
   The drawer is organized into three accordion groups, each with a
   clickable heading (`▶` collapsed, `▼` expanded). Only **What to
   export** is open by default — Extras and Rendering quality start
   collapsed since most users don't touch them. Click any heading to
   show or hide its contents. The expansion state is per-session: it
   resets to the default each time you open the drawer.
   - **What to export** — file kinds (ICO, Apple touch, SVG, HTML
     snippet) and PNG / ICO size sets. Size sets at their defaults
     (32 / 192 / 512 for PNG, 16 / 32 / 48 for ICO) display as a quiet
     "at defaults: …" badge; type a number into the adjacent input and
     the full chip editor expands.
   - **Extras** — opt-in extras (Web manifest for PWA, Monochrome
     `mono/` set). Most users skip these.
   - **Rendering quality** — resize algorithm (Lanczos3 default).

   (Language used to live here too; it moved to the header icon button
   in v1.10.2 so it's reachable without opening the drawer.)
5. Click **Export**, choose an output directory, and the artifacts
   are written atomically (all-or-nothing — see `docs/export-spec.md`)

The generated `favicon-snippet.html` is paste-ready for your `<head>`.

Active picker buttons (View as / Surface, vtracer preset) are
indicated **both** with a filled background and a `▣` text prefix, so
the active state is conveyed without relying on color alone (ABDD §12).

## What gets written

By default, seven artifacts:

- `favicon.svg` (vector — SVG source is preserved as-is; raster sources are
  vectorized via [vtracer](https://github.com/visioncortex/vtracer))
- `favicon.ico` (with 16, 32, 48 frames, each rendered independently)
- `apple-touch-icon.png` (180×180)
- `favicon-32.png` / `favicon-192.png` / `favicon-512.png`
- `favicon-snippet.html` (the `<link>` block for your `<head>`)

The set is intentionally minimal. The SVG output is referenced first in the
generated HTML so modern browsers prefer it on high-DPI displays; older
browsers fall back to the ICO and PNGs.

If your input is a photograph or otherwise unsuitable for vectorization,
turn off **Vectorize raster sources to SVG** in the advanced drawer to skip
`favicon.svg` for that run.

Optionally, enabling **Output `manifest.webmanifest`** (since v1.8.0) adds a
PWA `manifest.webmanifest` to the output, alongside a matching
`<link rel="manifest">` line in the HTML snippet. The manifest's `icons`
array mirrors the PNG sizes you've configured.

See `docs/export-spec.md` for the rationale on what is **not** emitted (no
`browserconfig.xml`, no `msapplication-*`).

## Layout

- `crates/logolig-core` — pure domain types and image processing.
  No iced / snora / GUI dependency. The dependency graph enforces
  this: importing iced from inside core is a compile-time error.
- `crates/logolig-app` — the iced + snora GUI binary, compiled to
  `logolig`.

Documentation lives under `docs/`:

- `architecture.md` — module layout, state model, message flow
- `ui-a11y.md` — Accessible-by-Default-and-by-Design (ABDD) commitments
- `export-spec.md` — output artifacts and HTML snippet shape

## Settings persistence

Your advanced settings (resize algorithm, output toggles, PNG / ICO sizes,
theme) are saved automatically as you change them. Storage location:

- Linux: `$XDG_CONFIG_HOME/logolig/settings.json` (or `~/.config/logolig/settings.json`)
- macOS: `~/Library/Application Support/logolig/settings.json`
- Windows: `%APPDATA%/logolig/settings.json`

The file is plain JSON; you can hand-edit it or delete it to reset to
defaults. Logolig reads it on startup and falls back to defaults if it
doesn't exist or can't be parsed (older logolig versions can read newer
files because of `serde(default)` forward compatibility).

If saving fails (read-only config dir, etc.), a warning toast is shown
and the app keeps running — your changes stay in memory for the session.

## Transparency

Logolig audits each loaded image's alpha channel and warns about two
common favicon mistakes:

- **Fully opaque image** — every pixel has alpha=255. The favicon will
  show as a white square on dark browser tabs. Usually you want to
  remove the background before export.
- **Empty image** — every pixel has alpha=0. Likely the wrong file was
  loaded.

The warning shows once per loaded image as a Toast and lets you
continue regardless. Halo detection and pre-multiplied-alpha analysis
are intentionally not included — their thresholds risk false positives
on legitimate anti-aliasing.

Toggle **Show transparency checker** in the preview panel to swap the
browser-tab / smartphone framing for a checker-pattern view: light
and dark grey 12px tiles with the icon overlaid at native size, so
transparent regions are visually unambiguous.

## Web manifest (PWA)

When **Output `manifest.webmanifest`** is enabled in the advanced drawer,
Logolig writes a [W3C Web App Manifest](https://www.w3.org/TR/appmanifest/)
alongside the favicons. The four user-editable fields are:

- **Name** — the full app name shown on the home screen
- **Short name** — fallback for narrow contexts (recommended ≤12 chars)
- **Theme color** — the browser UI accent (`#RRGGBB`)
- **Background color** — the splash-screen background (`#RRGGBB`)

The manifest's `icons` array is generated from the PNG sizes you have
configured, so it stays in sync with what's actually written to disk.
`start_url` and `display` are fixed at `"/"` and `"standalone"` for v1.8 —
favicon users rarely need to change these, and exposing every W3C field
would defeat the "fewer settings" design (§5). v1.8.x can lift this
limitation if a real need surfaces.

Color values are validated at export time (not while typing), so `#FF…`
in mid-edit doesn't trigger a warning. If a malformed color is detected at
export, a Toast is shown and the export is blocked until it's fixed.

## Monochrome output

When **Output `mono/` grayscale set** is enabled in the advanced drawer,
Logolig adds a grayscale version of each PNG and the ICO under a `mono/`
subdirectory:

```
your-output-dir/
├── favicon.svg
├── favicon.ico
├── apple-touch-icon.png
├── favicon-32.png
├── favicon-192.png
├── favicon-512.png
├── favicon-snippet.html
└── mono/
    ├── favicon.ico
    ├── favicon-32.png
    ├── favicon-192.png
    └── favicon-512.png
```

The grayscale conversion uses the [BT.709 luma formula](https://en.wikipedia.org/wiki/Rec._709)
(`Y = 0.2126 R + 0.7152 G + 0.0722 B`), the modern sRGB-aligned standard.
Alpha is preserved per pixel — transparent regions stay transparent.

Typical use cases:

- **Single-color print** (business cards, faxes, embroidery) where the
  color version would muddy
- **Theme-aware mask icons** referenced from CSS via `mask-image` so the
  user agent can recolor the icon to match light/dark mode
- **Stencil and design-asset re-use** where you want a flat tone version
  of the logo

### Wiring monochrome into your `<head>` (optional)

Logolig does **not** auto-inject `<link>` lines for monochrome icons —
how you use them is too project-specific to template. The most common
pattern uses CSS `prefers-color-scheme` to swap which icon the browser
should pick. Here is the diff you can paste into the snippet Logolig
generated:

```diff
 <link rel="icon" type="image/svg+xml" href="/favicon.svg">
 <link rel="icon" href="/favicon.ico" sizes="any">
 <link rel="icon" type="image/png" sizes="32x32" href="/favicon-32.png">
 <link rel="icon" type="image/png" sizes="192x192" href="/favicon-192.png">
 <link rel="icon" type="image/png" sizes="512x512" href="/favicon-512.png">
 <link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png">
+
+<!-- Dark-mode icon overrides (monochrome set) -->
+<link rel="icon" type="image/png" sizes="32x32"
+      href="/mono/favicon-32.png" media="(prefers-color-scheme: dark)">
+<link rel="icon" type="image/png" sizes="192x192"
+      href="/mono/favicon-192.png" media="(prefers-color-scheme: dark)">
+<link rel="icon" type="image/png" sizes="512x512"
+      href="/mono/favicon-512.png" media="(prefers-color-scheme: dark)">
```

Alternative: use the mono PNG as a CSS mask-image so the icon picks up
the user's accent color automatically:

```css
.app-icon {
  background: currentColor;
  mask-image: url("/mono/favicon-192.png");
  mask-size: cover;
}
```

ICO is shipped as `mono/favicon.ico` for completeness — its primary use
case is legacy `favicon.ico` URLs at the site root, which can't easily
be theme-swapped, but it's there if you need it.

SVG monochrome is not yet supported in v1.9.0. The naive "replace `fill`
attributes" approach breaks on gradients, external CSS, and inline
styles, so a future v1.9.x release will go raster → grayscale →
re-vectorize via vtracer instead.

## Language

Logolig follows your system language by default. Click the language icon
in the header (top right) to cycle through the three states:

- **System default** (`文A`) — use the OS locale (`LANG` on Linux,
  `NSLocale` on macOS, user UI language on Windows)
- **English** (`Aa`) — explicit override
- **日本語** (`あ`) — Japanese (since v1.6.0)

The icon glyph reflects the current state, so you can tell at a glance
which language is active. The selection persists across sessions.
Locale detection accepts the common forms — `en`, `en-US`, `en_US`, `ja`,
`ja-JP`, `ja_JP`, even `ja_JP.UTF-8` from POSIX `LANG` — so most users
need no override.

If your OS locale isn't supported yet, Logolig falls back to English
without warning. New languages are added by dropping a TOML file under
`crates/logolig-i18n/locales/` and adding a `Locale` enum variant —
the enum's exhaustiveness check ensures every UI string and error
message has a translation, or the build fails.

## Tests

```sh
cargo test -p logolig-core
cargo test -p logolig-i18n
```

logolig-core has 101 integration tests covering ingest, decode (PNG /
WebP / JPEG), SVG rasterization, vectorization, resize, preview cache,
ICO writing, HTML snippet generation, transactional export, settings
round-trip, forward-compatible JSON deserialization, transparency-state
classification, Web manifest JSON generation, and BT.709 grayscale
conversion (alpha preservation, coefficient precision, exporter `mono/`
subdirectory wiring). logolig-i18n adds 16 tests covering dictionary
loading (English and Japanese), placeholder substitution, error
translation, BCP-47 locale resolution including POSIX forms, and a
regression check that Japanese UI keys actually differ from English.
Total: 117 tests.

## Versioning

This repository contains **v1**: the iced-based native desktop
build. v1.0.0 was the feature-complete release against the original
specification; subsequent v1.x releases (WebP input, SVG output,
advanced settings UX, persistence) extended it without breaking
compatibility.

A separate **v2** branch is planned that retargets the same
`logolig-core` to a leptos-based WebAssembly build for
privacy-preserving in-browser use. The split is possible because
`logolig-core` carries no GUI-framework dependency. The v1.4.0
`SettingsStore` trait is the seam: v2 will provide a `BrowserStore`
implementation backed by `localStorage` while reusing the same
`PersistedSettings` schema.

## License

Licensed under the Apache License, Version 2.0. See [`LICENSE`](LICENSE).
