# Export Specification

Defines the artifacts Logolig writes to disk and the HTML
snippet it generates. Implements §7 of the build spec.

## Default artifact set

The default `ExportPlan` produces six artifacts. The set is chosen
to be **minimal but practically sufficient** for a modern web project:

| File | Purpose |
| --- | --- |
| `favicon.ico` | Backwards compatibility (Windows / older browsers). Contains 16, 32, and 48 px frames so each size is rasterized **independently** rather than scaled from a single source — this preserves quality at small sizes (§6.2). |
| `apple-touch-icon.png` | 180×180 PNG; iOS / iPadOS home-screen icon. |
| `favicon-32.png` | High-DPI browser tab. |
| `favicon-192.png` | PWA / Android home-screen install. |
| `favicon-512.png` | PWA splash / `manifest.webmanifest` reference. |
| `<head>` HTML snippet | Pasteable markup; see below. |

A user can extend `png_sizes` or `ico_sizes` via the advanced
settings drawer (§5.3) but the default set is intentionally short.

## Quality strategy

Each output PNG is rendered:

- For SVG sources — by rasterizing the SVG **at the target size**
  via `resvg`. We never scale a single rendered bitmap up or down.
- For PNG sources — by decoding the source once and resizing to each
  target with `fast_image_resize`. The default algorithm is
  `Lanczos3` (`ResizeAlgorithm::Lanczos3`); other choices are
  available via the advanced drawer.

For the ICO container, **each frame is generated from the source at
that exact size**. ICO frames are not scaled from a "master"
bitmap — this is what produces the jaggy 16×16 favicons on most
online generators. We avoid that mode entirely.

## Source non-destructiveness

`SourceAsset::raw` (an `Arc<[u8]>`) is the canonical source. Every
output reads from it; nothing writes back to it. The user's input
file is never modified.

## HTML snippet

Generated for `<head>`. Constraints (§7.2):

- Semantic; uses appropriate `rel` values
- Modern; reflects current best practice — does **not** include
  obsolete `<meta name="msapplication-..." />` tags by default
- Brief; only what is necessary for the artifacts produced
- A11y-respecting; uses no broken or deprecated attributes

The actual default output (matching the `tests/html_snippet.rs`
expectations):

```html
<link rel="icon" href="/favicon.ico" sizes="any">
<link rel="icon" type="image/png" sizes="32x32" href="/favicon-32.png">
<link rel="icon" type="image/png" sizes="192x192" href="/favicon-192.png">
<link rel="icon" type="image/png" sizes="512x512" href="/favicon-512.png">
<link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png">
```

The path prefix is configurable per call to
`html_snippet::render(plan, base)` — for example pass
`"/static/icons/"`. Trailing slashes are normalized.

## Avoided bloat

The following are **not** emitted by default:

- Multiple `apple-touch-icon-*x*.png` variants. Modern iOS scales the
  single 180×180 well; the `-precomposed` variant is unnecessary on
  any iOS we target.
- `browserconfig.xml` and the Microsoft tile colors. They were Edge
  Legacy / IE11 only.
- A `manifest.webmanifest` file. v1 does not generate one because it
  is project-shaped, not favicon-shaped — the user often has their
  own manifest already. (A future v2 may add it as an advanced
  option once the WASM build introduces PWA-flavoured workflows.)

## Failure modes

Output failures surface as `AppError::Export(_)` from `logolig_core`
and become persistent error toasts in the UI (§ui-a11y "Errors as
toasts"). The export step is **transactional at the file level**:
either every requested artifact is written, or none is. The
implementation uses a hidden staging directory
(`.logolig-<pid>-<nanos>.tmp`) inside the chosen output directory,
writes every artifact there, and finalizes by atomic rename. A
`StagingGuard` (RAII) deletes the staging on any failure path,
including panics.
