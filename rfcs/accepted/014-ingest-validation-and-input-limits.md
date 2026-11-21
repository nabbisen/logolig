# RFC 014 — Ingest validation policy and input limits

**Status.** Accepted (architect, under the owner's delegation of 2026-11-22 "review them carefully with my fundamental philosophy"; release 1.29.0; decisions in the section "Decisions 2026-11-22")
**Priority.** P2 (policy clarification plus hardening)
**Tracks.** Spec/implementation mismatch (extension vs magic bytes) and missing resource limits.
**Touches.** `crates/core/src/services/ingest.rs`, `domain/source_asset.rs`, `error.rs`, i18n keys, `docs/src/functional-spec.md`, README.

## Summary

Docs and requirements say the file extension and magic bytes must agree and that spoofed files are rejected. The code treats magic bytes as authoritative and ignores the extension. This RFC settles the policy and adds size limits so that a hostile or accidental huge input cannot exhaust memory.

## Evidence

Verified by source reading.

- `ingest::detect_kind(bytes, hint)` ends with `let _ = hint; None`; the extension hint is discarded. A `.txt` file containing PNG bytes is accepted as PNG. A file named `.png` that contains JPEG data is accepted as JPEG.
- `functional-spec.md` §2: "Magic bytes and file extension must agree (prevents extension spoofing)"; handoff FR-1 and dev instructions §7 say the same.
- `looks_like_svg` accepts any file whose first 1 KiB, if valid UTF-8, contains `<svg`.
- `ingest` reads the whole file with `tokio::fs::read` with no size cap. Decoders (`image::load_from_memory`) allocate the full RGBA buffer. A small compressed file with huge declared dimensions can request gigabytes.
- `SourceAsset::intrinsic_size` (parsed from the header) is available **before** decoding.
- Not verified by execution: the actual memory behaviour on a crafted file.

## Policy decision (D1)

| Option | Behaviour | Assessment |
|---|---|---|
| A (recommended) | Content decides. Extension is only a hint. Docs are corrected. An informational toast is shown when the extension contradicts the detected format ("This file is a JPEG although its name ends in .png"). | Friendly to the common "wrongly named file" case; safe because decoding is chosen by content, never by extension |
| B | Reject when extension and content disagree | Matches the current docs, but blocks legitimate mislabelled logos and adds no protection (the decoder never trusted the extension) |

## Input limits (D2, values are proposals for the owner)

1. Maximum file size: **64 MiB** (reject before reading fully, using `metadata().len()`, and again after reading).
2. Maximum pixel count: **100 megapixels** and maximum single dimension **16 384 px**, checked from `intrinsic_size` before decoding; when the header cannot be parsed, decode through a decoder with limits (`image::io::Limits`) rather than unbounded.
3. SVG: maximum size 8 MiB and a limit on nesting/elements as supported by `usvg`; resource handling is specified in RFC 015.
4. New `AppError` variants with i18n keys: `InputTooLarge { path, limit }`, `ImageTooLarge { path, pixels }`. User text states the limit and the next step.

## Tests (design-derived)

- Mismatched extension: accepted with the correct kind and an advisory flag (option A) / rejected (option B).
- File of `limit + 1` bytes is rejected without being read in full.
- PNG header declaring 60 000 × 60 000 is rejected before allocation.
- Truncated and zero-byte files return `Decode`/`UnsupportedFile`, never panic.
- Existing ingest tests unchanged.

## Acceptance criteria

Documented policy equals implemented policy; the limits are enforced and covered by tests; error messages exist in `en` and `ja`.

## Security notes

Local single-user app, so this is robustness hardening, not a remote-attack fix. The same limits matter for the planned v2 browser build, where memory is tighter.

## Risk

Low. Limits may reject unusually large but legitimate logos; the owner chooses the numbers.

## Addendum (architect, 2026-10-24, from the RFC 028 review)

The error toast for an unsupported file shows the **full path** ("Unsupported file format: /home/<user>/.../notes.png"): it wraps over five lines and exposes the directory and user name. Decide in this RFC's work that user-facing errors for a file show the **file name only** (the full path stays in logs and in the `AppError` fields). The translation templates `error_unsupported_file` and `error_io` (`{path}`) keep the placeholder; the caller passes the file name.

## Amendment (architect, 2026-11-22): public error enum

The new `InputTooLarge` and `ImageTooLarge` variants are added to the public `AppError` of the published `logolig` crate, which is not `#[non_exhaustive]`: as with `MessageKey` in 1.27.0 and 1.28.0, this is a source-compatibility note for downstream exhaustive matches, stated in the release notes; `#[non_exhaustive]` is on the 2.0 list (`ROADMAP.md`). Scoped for 1.29.0 (job K1) pending the owner's decisions on D1 and D2.

## Decisions 2026-11-22 (architect; the owner's philosophy: clean, safe and secure, robust; users must not be confused)

- **D1 Option A: content decides.** The extension is a hint, never a gate; decoding is chosen by content. **One** informational notice when a **known** extension contradicts the detected format ("This file is a JPEG although its name ends in .png"); an unknown or missing extension with recognised content is accepted silently. The notice and the existing JPEG "no transparency" advisory are **one** message, not two toasts, so a mislabelled JPEG is explained once.
- **D2 Limits, with a measurement gate.** File size **64 MiB**; SVG **8 MiB**; maximum side **16 384 px**; **pixel count 64 megapixels** (not 100: a 4-byte-per-pixel decode of 100 MP is 400 MB before the application's copies for preview and conversion). The job **measures** the peak resident memory of converting a synthetic image at the limit (`/usr/bin/time -v` or equivalent, release binary) and reports it; **if the peak exceeds 1.5 GiB the pixel limit is lowered until it does not**, and the architect confirms the number. The limits are checked from the header (`intrinsic_size`) **before** decoding, and the file size from `metadata().len()` before reading and again after.
- **D3 Regular files only.** A path that is not a regular file (a directory, a FIFO, a device such as `/dev/zero`) is rejected before reading with the "unsupported file" message: reading a FIFO would block the job for ever and a device would read without end. (Symlinks to regular files are followed.)
- **D4 File name only in user-facing errors** (addendum of 2026-10-24): the caller passes `file_name()`; the full path stays in logs and in the `AppError` fields. The translation templates keep `{path}`.
- **D5 Texts** state the limit and the next step in plain words, in English and Japanese ("This image is larger than 64 megapixels. Reduce its size and try again."), no jargon, no path. The new keys follow the key-parity test.
- **Tests, by level:** limits and policy as pure `ingest_bytes` tests (core, including crafted headers declaring huge dimensions, a truncated header, a mislabelled extension, an unknown extension); the non-regular-file case with a temp directory and a FIFO (`mkfifo`) in a test; the update-level test that exactly one notice results; the peak-memory measurement as recorded evidence (not a CI test).

## Result of K1 (architect, 2026-11-22): the pixel limit is 50 megapixels

Measured with the release binary (native Wayland, isolated session): idle 0.19 GiB; 8 000 × 8 000 (64 MP) peaked at **1.68 GiB** (over the 1.5 GiB ceiling of D2); 7 071 × 7 071 (49.99 MP) peaked at **1.36 GiB**, the Result screen after about 3 s. By the D2 rule the limit is **50 000 000 pixels** (`MAX_PIXELS`). This **supersedes the 64 MP of D2**; the other limits stand (64 MiB file, 8 MiB SVG, 16 384 px a side). A 48 MP phone photo (8 000 × 6 000) is accepted. Public API added by K1 (a source-compatibility note for the release notes): `AppError::{InputTooLarge, ImageTooLarge}`, `MessageKey::{ErrorInputTooLarge, ErrorImageTooLarge, ToastFormatMismatchTitle, ToastFormatMismatchBody}`, `FormatNote`, `Ingested`, `ingest_noted`, `ingest_bytes_noted`, `MAX_INPUT_BYTES`, `MAX_SVG_BYTES`, `MAX_SIDE`, `MAX_PIXELS`; nothing removed or renamed. The note is a separate return (`Ingested`), not a new field of the public `SourceAsset` (a new field would break downstream struct literals).

