# Logolig

[![License](https://img.shields.io/github/license/nabbisen/logolig)](https://github.com/nabbisen/logolig/blob/main/LICENSE)
[![crates.io](https://img.shields.io/crates/v/logolig-app?label=app)](https://crates.io/crates/logolig-app)
[![crates.io](https://img.shields.io/crates/v/logolig-core?label=core)](https://crates.io/crates/logolig-core)

A local-first, accessible logolig generator GUI built on
[iced 0.14](https://iced.rs/) and [snora 0.4](https://github.com/nabbisen/snora).

> Your image never leaves your machine. PNG / SVG goes in,
> a polished `logolig.ico` + Apple touch icon + high-resolution PNGs
> + a clean HTML `<head>` snippet come out.

## Why

Existing online logolig generators have problems:

- Image quality degrades on resize (jaggies, fuzziness, lost detail)
- Images are uploaded to a third-party server
- Output is bloated with sizes you don't need
- Generated HTML often skimps on accessibility
- The UI demands too many decisions up front

Logolig addresses all of these as **design requirements**, not
afterthoughts.

## Status

In-progress staged build. See `docs/architecture.md` for the four-step plan.
Currently at: **Step 2 — drop reception and image processing pipeline**.

## Installation

Requires Rust 1.85 or newer.

### From source (recommended while in development)

```sh
git clone https://github.com/nabbisen/logolig
cd logolig
cargo install --path crates/logolig-app
logolig
```

`cargo install` places a `logolig` binary in `~/.cargo/bin/`. Make sure that
directory is on your `$PATH`.

### From a published crate (after release)

```sh
cargo install logolig-app
logolig
```

## Running without installing

```sh
cargo run -p logolig-app
```

## Layout

- `crates/logolig-core` — pure domain types and image processing.
  No iced / snora dependency. Reusable from a future CLI or WASM
  frontend.
- `crates/logolig-app` — the iced + snora GUI binary (compiled to `logolig`).

Documentation lives under `docs/`:

- `architecture.md` — module layout, state model, message flow
- `ui-a11y.md` — Accessible-by-Default-and-by-Design (ABDD) commitments
- `export-spec.md` — output artifacts and HTML snippet shape
