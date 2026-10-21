# RFC 010 — Non-square raster sources in square outputs

**Status.** Accepted (owner approval conditional on reasonableness; architect assessment below, 2026-10-21)
**Priority.** P1 (output quality is a stated product value)
**Tracks.** Defect/design gap: non-square PNG/WebP/JPEG sources are stretched into square favicon outputs.
**Touches.** `crates/core/src/services/exporter.rs` (`render_at_size`), `services/preview.rs` (`render_at`), `services/canvas.rs`, tests, `docs/src/export-spec.md`.

## Summary

For SVG sources, square outputs letterbox the drawing with transparent padding. For raster sources, `render_at_size` calls `resize(src, size, size)` directly, which distorts any non-square image. The Microsoft logos already use contain-fit. This RFC makes square outputs consistent and distortion-free.

## Evidence

Verified by source reading; **not executed**.

- `exporter::render_at_size` raster branch: `resize::resize(src, size, size, plan.algorithm)` with no aspect handling.
- `rasterize_svg::rasterize` centres non-square SVGs ("non-square SVGs are centred with transparent padding").
- `canvas::contain_on_transparent_canvas` exists and is used only by `render_to_contain_canvas` (Microsoft logos).
- `services/preview.rs::render_at` has the same direct `resize(..., size, size)`.
- No test in `crates/core/tests` uses a non-square raster; `rasterize.rs` notes its fixture is 1:1.
- The product statement (dev instructions §1, §19) puts output quality second after privacy.

## Options

| Option | Behaviour | Notes |
|---|---|---|
| A (recommended) | Contain-fit on a transparent square canvas | Matches SVG and Microsoft-logo behaviour; no cropping or stretching; no new setting |
| B | Centre-crop ("cover") | Loses logo content; surprising for logos with wide wordmarks |
| C | Keep stretching | Distorts the user's logo; only defensible as "garbage in, garbage out" |
| D | A plus an advisory toast for non-square input | Educational, in the spirit of the JPEG toast; adds one i18n key pair |

## Proposal

Adopt **A**, and offer **D** as an owner choice. Introduce one shared helper, for example `render_square(src, size, algorithm)`, built on `contain_on_transparent_canvas`, used by `render_at_size` and by `preview::render_at` so the preview matches the output. `keep_transparency = false` continues to flatten **after** padding, so the padding becomes white, consistent with every other raster output.

Square sources must take the existing fast path unchanged (`resize` already short-circuits identical dimensions; the helper must not add a second resampling pass for square input).

## Compatibility

This changes the pixels of outputs for non-square raster sources, which today are distorted. Square sources are bit-for-bit unchanged and a regression test must prove it. The dev instructions say default output stays stable unless a migration is explicitly accepted; this RFC therefore needs an **explicit owner decision** (D1).

`favicon.svg` produced by vtracer from a raster keeps the source aspect ratio in its `viewBox`; browsers letterbox SVG icons themselves, so no change is proposed there.

## Decisions (2026-10-21)

The owner approved D1 "only if reasonable enough" and asked that the philosophy *clean, safe, robust design; users must not be confused* be applied. Architect assessment:

- **D1 — contain-fit (option A): accepted.** Reasonable because (1) the application already contains this rule for SVG sources and for the Microsoft logos, so today's raster behaviour is the inconsistent one; (2) it is the only option that neither distorts nor discards the user's artwork; (3) distortion is not a contract anyone can rely on, while square sources, which are the intended input, stay bit-identical; (4) no new setting is introduced.
- **D2 — advisory toast (option D): accepted.** Contain-fit makes a wide wordmark a thin strip at 16 px. Without an explanation a user would read that as a defect. The toast is educational and non-blocking, in the tone of the JPEG notice ("Your logo is wider than it is tall, so it is shown with empty space above and below. A square version of your logo gives a larger icon."). Needs `en` and `ja` keys; shown once per loaded image, never for square sources.

Conditions of acceptance:

1. First task of the handoff is a failing test that demonstrates the current distortion (confirms the defect by execution; the evidence above is from source reading).
2. Golden bytes for square sources are captured **before** the change and must be unchanged after it.
3. The `docs/src/export-spec.md` rule is part of the same change.

## Tests (design-derived)

- 2:1 and 1:2 synthetic rasters: output is square; the opaque bounding box has aspect ratio 2:1 / 1:2 (± rounding) and is centred; padding pixels have alpha 0.
- Same sources with `keep_transparency = false`: padding is white, alpha 255.
- Square source: output bytes identical before and after the change (golden bytes captured first).
- Preview and export agree for a non-square source at 16 px.
- ICO frames and `apple-touch-icon.png` covered by the same helper.

## Acceptance criteria

All tests above pass; `docs/src/export-spec.md` documents the contain-fit rule for every source kind.

## Risk

Low-medium: user-visible output change, mitigated by golden tests for square input and an explicit owner decision.
