# RFC 014 — Ingest validation policy and input limits

**Status.** Proposed
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
