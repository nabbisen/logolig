# Logolig

[![License](https://img.shields.io/github/license/nabbisen/logolig)](LICENSE)
[![Documentation](https://docs.rs/logolig/badge.svg?version=latest)](https://docs.rs/logolig)
[![Documentation](https://docs.rs/logolig-app/badge.svg?version=latest)](https://docs.rs/logolig-app)
[![Documentation](https://docs.rs/logolig-i18n/badge.svg?version=latest)](https://docs.rs/logolig-i18n)    
[![crates.io](https://img.shields.io/crates/v/logolig?label=core)](https://crates.io/crates/logolig)
[![Dependency Status](https://deps.rs/crate/logolig/latest/status.svg)](https://deps.rs/crate/logolig)
[![crates.io](https://img.shields.io/crates/v/logolig-app?label=app)](https://crates.io/crates/logolig-app)
[![Dependency Status](https://deps.rs/crate/logolig-app/latest/status.svg)](https://deps.rs/crate/logolig-app)
[![crates.io](https://img.shields.io/crates/v/logolig-i18n?label=i18n)](https://crates.io/crates/logolig-i18n)
[![Dependency Status](https://deps.rs/crate/logolig-i18n/latest/status.svg)](https://deps.rs/crate/logolig-i18n)

A local-first, accessible logo asset generator GUI.

PNG / SVG / WebP / JPEG goes in. A polished favicon bundle and, when enabled,
small Microsoft app logo assets come out — all on your machine, with no upload
anywhere.

Built on [iced](https://iced.rs/) and [snora](https://github.com/nabbisen/snora).

## Why

Existing online favicon generators have problems:

- Image quality degrades on resize (jaggies, fuzziness, lost detail)
- Images are uploaded to a third-party server
- Output is bloated with sizes you don't need
- Generated HTML often skimps on accessibility
- The UI demands too many decisions up front

Logolig addresses all of these as **design requirements**, not
afterthoughts. See `docs/src/architecture.md` for the rationale.

## Installation

Requires Rust 1.85 or newer.

### From source

```sh
git clone https://github.com/nabbisen/logolig
cd logolig
cargo install --path crates/app
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

The app uses a three-item side navigation: **Home**, **Customize**, and
**Settings**. On narrow windows the same navigation moves to the bottom.

1. On **Home**, drop a PNG, SVG, WebP, or JPEG onto the window, or click
   anywhere inside the bordered drop-zone card to open the file picker.
   - JPEG inputs work, but the format cannot store transparency, so
     Logolig shows an educational toast suggesting a PNG with the
     background cut out for favicons that adapt to light and dark tabs.
2. Logolig converts immediately after ingest and then shows the **Result**
   screen. Each generated asset appears as a card with a thumbnail or
   document placeholder, dimensions where applicable, file size, and an
   individual download button.
3. Use **Preview** on the Result screen when you want to inspect the icon
   in browser-tab, phone-home, or checkerboard contexts before saving.
4. Save one asset with its card download button, or save the whole result
   set with **Download all (ZIP)**. Conversion itself is in-memory; files
   are written only when you choose a download target.
5. Use **← Back** to return to the drop zone. The last result remains in
   memory for the session and can be reopened from the **Last conversion**
   card without re-converting.

Use **Customize** for output settings: PNG sizes, SVG conversion mode,
keep-transparency behavior, and the collapsible **Advanced settings**
section for Apple touch icon, HTML snippet, web manifest, monochrome output,
resize algorithm, raster vectorization, and Microsoft app logos.

Use **Settings** for language and theme. Supported languages are English,
Japanese, and system default; themes are light, dark, and system default.

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
- Optional Microsoft app logos: `StoreLogo.png`, `Square44x44Logo.png`,
  `Square150x150Logo.png`, and `Wide310x150Logo.png`

The set is intentionally minimal. The SVG output is referenced first in the
generated HTML so modern browsers prefer it on high-DPI displays; older
browsers fall back to the ICO and PNGs.

If your input is a photograph or otherwise unsuitable for vectorization,
turn off **Vectorize raster sources to SVG** in Customize → Advanced settings to skip
`favicon.svg` for that run.

Optionally, enabling **Output `manifest.webmanifest`** (since v1.8.0) adds a
PWA `manifest.webmanifest` to the output, alongside a matching
`<link rel="manifest">` line in the HTML snippet. The manifest's `icons`
array mirrors the PNG sizes you've configured.

See `docs/src/export-spec.md` for the rationale on what is **not** emitted (no
`browserconfig.xml`, no `msapplication-*`).

## Layout

- `crates/core` — pure domain types and image processing.
  No iced / snora / GUI dependency. The dependency graph enforces
  this: importing iced from inside core is a compile-time error.
- `crates/i18n` — bundled dictionaries and locale resolution.
- `crates/app` — the iced + snora GUI binary, compiled to `logolig`.

Documentation lives under `docs/src/`:

- `architecture.md` — module layout, state model, message flow
- `ui-a11y.md` — Accessible-by-Default-and-by-Design (ABDD) commitments
- `export-spec.md` — output artifacts and HTML snippet shape

## Settings persistence

Your output settings, theme, and locale override are saved automatically as
you change them. Storage location:

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

When **Output `manifest.webmanifest`** is enabled in Customize → Advanced
settings,
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

When **Output `mono/` grayscale set** is enabled in Customize → Advanced
settings,
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

SVG monochrome is not yet supported. The naive "replace `fill`
attributes" approach breaks on gradients, external CSS, and inline
styles, so a future release may go raster → grayscale →
re-vectorize via vtracer instead.

## Language

Logolig follows your system language by default. Open **Settings** to choose:

- **System default** — use the OS locale (`LANG` on Linux,
  `NSLocale` on macOS, user UI language on Windows)
- **English** — explicit override
- **日本語** (`あ`) — Japanese (since v1.6.0)

The selection persists across sessions.
Locale detection accepts the common forms — `en`, `en-US`, `en_US`, `ja`,
`ja-JP`, `ja_JP`, even `ja_JP.UTF-8` from POSIX `LANG` — so most users
need no override.

If your OS locale isn't supported yet, Logolig falls back to English
without warning. New languages are added by dropping a TOML file under
`crates/i18n/locales/` and adding a `Locale` enum variant —
the enum's exhaustiveness check ensures every UI string and error
message has a translation, or the build fails.

## Tests

```sh
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo audit
```

The core tests cover ingest, decode, SVG rasterization, vectorization,
resize, preview cache, ICO writing, HTML snippet generation, in-memory and
disk export, settings round-trip, transparency classification, web manifest
JSON generation, Microsoft app logo output, and monochrome conversion. The
i18n tests cover dictionary loading, placeholder substitution, error
translation, and BCP-47 locale resolution. The app crate has focused
`update()` smoke tests for navigation, settings changes, history, and
drag/drop state.

## Versioning

This repository contains **v1**: the iced-based native desktop
build. v1.0.0 was the feature-complete release against the original
specification; subsequent v1.x releases (WebP input, SVG output,
advanced settings UX, persistence) extended it without breaking
compatibility.

A separate **v2** branch is planned that retargets the same
`logolig` to a leptos-based WebAssembly build for
privacy-preserving in-browser use. The split is possible because
`logolig` carries no GUI-framework dependency. The v1.4.0
`SettingsStore` trait is the seam: v2 will provide a `BrowserStore`
implementation backed by `localStorage` while reusing the same
`PersistedSettings` schema.

## License

Licensed under the Apache License, Version 2.0. See [`LICENSE`](LICENSE).

Copyright 2026 nabbisen
