# Logolig

[![License](https://img.shields.io/github/license/nabbisen/logolig)](https://github.com/nabbisen/logolig/blob/main/LICENSE)
[![crates.io](https://img.shields.io/crates/v/logolig-app?label=app)](https://crates.io/crates/logolig-app)
[![crates.io](https://img.shields.io/crates/v/logolig-core?label=core)](https://crates.io/crates/logolig-core)

A local-first, accessible favicon generator GUI.

PNG / SVG / WebP goes in. A polished `favicon.svg`, `favicon.ico`, an
Apple touch icon, high-resolution PNGs, and a clean HTML `<head>` snippet
come out — all on your machine, with no upload anywhere.

Built on [iced](https://iced.rs/) and [snora](https://github.com/nabbisen/snora).

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

1. Drop a PNG, SVG, or WebP onto the window, or click **Choose file…**
2. Inspect the **Browser tab (16×16)** and **Smartphone home** previews;
   toggle background between System / Light / Dark to spot contrast issues
3. (Optional) Open **Show advanced** to:
   - pick a different resize algorithm (Lanczos3 default)
   - turn off SVG output or vectorization for unsuitable sources
   - skip individual artifact types (ICO, Apple touch, HTML snippet)
   - add or remove PNG / ICO sizes (chip-style editor, validated to
     16–1024 px for PNG and 16–256 px for ICO)
4. Click **Export**, choose an output directory, and the artifacts
   are written atomically (all-or-nothing — see `docs/export-spec.md`)

The generated `favicon-snippet.html` is paste-ready for your `<head>`.

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

## Language

Logolig follows your system language by default. Override the language
in **Show advanced → Language**:

- **System default** — use the OS locale (`LANG` on Linux, `NSLocale` on macOS,
  user UI language on Windows)
- **English** — explicit override
- **日本語** — Japanese (since v1.6.0)

The selection is persisted alongside other advanced settings. Locale
detection accepts the common forms — `en`, `en-US`, `en_US`, `ja`,
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

logolig-core has 80 integration tests covering ingest, decode, SVG
rasterization, vectorization, resize, preview cache, ICO writing, HTML
snippet generation, transactional export, settings round-trip,
forward-compatible JSON deserialization, transparency-state
classification, and Web manifest JSON generation. logolig-i18n adds 16
tests covering dictionary loading (English and Japanese), placeholder
substitution, error translation, BCP-47 locale resolution including
POSIX forms, and a regression check that Japanese UI keys actually
differ from English. Total: 96 tests.

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
