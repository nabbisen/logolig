# Logolig

[![License](https://img.shields.io/github/license/nabbisen/logolig)](https://github.com/nabbisen/logolig/blob/main/LICENSE)
[![crates.io](https://img.shields.io/crates/v/logolig-app?label=app)](https://crates.io/crates/logolig-app)
[![crates.io](https://img.shields.io/crates/v/logolig-core?label=core)](https://crates.io/crates/logolig-core)

A local-first, accessible favicon generator GUI.

PNG / SVG goes in. A polished `favicon.ico`, an Apple touch icon,
high-resolution PNGs, and a clean HTML `<head>` snippet come out —
all on your machine, with no upload anywhere.

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

1. Drop a PNG or SVG onto the window, or click **Choose file…**
2. Inspect the **Browser tab (16×16)** and **Smartphone home** previews;
   toggle background between System / Light / Dark to spot contrast issues
3. (Optional) Open **Show advanced** to pick a different resize algorithm
4. Click **Export**, choose an output directory, and the six artifacts
   are written atomically (all-or-nothing — see `docs/export-spec.md`)

The generated `favicon-snippet.html` is paste-ready for your `<head>`.

## What gets written

By default, six artifacts:

- `favicon.ico` (with 16, 32, 48 frames, each rendered independently)
- `apple-touch-icon.png` (180×180)
- `favicon-32.png` / `favicon-192.png` / `favicon-512.png`
- `favicon-snippet.html` (the `<link>` block for your `<head>`)

The set is intentionally minimal. Increasing it is opt-in via the
advanced drawer (Step 3 surfaces the algorithm; future revisions may
expose size lists). See `docs/export-spec.md` for the rationale on
what is **not** emitted (no `browserconfig.xml`, no `msapplication-*`).

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

## Tests

```sh
cargo test -p logolig-core
```

logolig-core has 32 integration tests covering ingest, decode, SVG
rasterization, resize, preview cache, ICO writing, HTML snippet
generation, and the transactional exporter.

## Versioning

This repository contains **v1**: the iced-based native desktop
build. v1 is feature-complete against the original specification and
is now in **maintenance mode** — only security and critical-bug fixes
will land here.

A separate **v2** branch is planned that retargets the same
`logolig-core` to a leptos-based WebAssembly build for
privacy-preserving in-browser use. The split is possible because
`logolig-core` carries no GUI-framework dependency.

## License

Licensed under the Apache License, Version 2.0. See [`LICENSE`](LICENSE).
