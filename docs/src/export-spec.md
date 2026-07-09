# Export Specification

Defines the artifacts Logolig writes to disk and the HTML
snippet it generates. Implements §7 of the build spec.

## Default artifact set

The default `ExportPlan` produces seven artifacts (v1.2.0). The set is
chosen to be **minimal but practically sufficient** for a modern web
project:

| File | Purpose |
| --- | --- |
| `favicon.svg` | Vector format. Modern browsers prefer it on high-DPI displays. SVG sources are written verbatim (non-destructive); PNG/WebP sources are vectorized via [vtracer](https://github.com/visioncortex/vtracer) using its defaults. |
| `favicon.ico` | Backwards compatibility (Windows / older browsers). Contains 16, 32, and 48 px frames so each size is rasterized **independently** rather than scaled from a single source — this preserves quality at small sizes (§6.2). |
| `apple-touch-icon.png` | 180×180 PNG; iOS / iPadOS home-screen icon. |
| `favicon-32.png` | High-DPI browser tab. |
| `favicon-192.png` | PWA / Android home-screen install. |
| `favicon-512.png` | Large favicon / optional manifest icon reference. |
| `<head>` HTML snippet | Pasteable markup; see below. |

A user can extend `png_sizes` from the Customize page. The default set is
intentionally short. Two toggles
in v1.2.0 control the SVG output:

- `include_svg` (default: `true`) — write `favicon.svg` at all
- `vectorize_on_raster` (default: `true`) — when the source is PNG/WebP,
  attempt vectorization. Turn this off for photos or noisy images where
  tracing produces poor results

When the input is already SVG, `vectorize_on_raster` is irrelevant: the
source bytes are copied to `favicon.svg` unchanged.


## Optional Microsoft app logos (v1.26.0)

When `ExportPlan::include_microsoft_app_logos` is enabled, Logolig adds a
small Microsoft app logo set to the result bundle:

| File | Dimensions | Purpose |
| --- | ---: | --- |
| `StoreLogo.png` | 50×50 | Store/logo identity asset |
| `Square44x44Logo.png` | 44×44 | Small app identity asset |
| `Square150x150Logo.png` | 150×150 | Square tile logo |
| `Wide310x150Logo.png` | 310×150 | Wide tile logo |

This setting is off by default and appears under Advanced settings. The
feature intentionally avoids the full Windows scale-qualified asset matrix:
v1.26.0 only implements the four practical filenames requested for the
current product flow. The renderer uses contain-fit placement on a
transparent canvas so a source logo is not cropped or stretched, including
the wide 310×150 output. The existing "Keep transparency" setting still
applies; when disabled, these PNGs are flattened to white like other raster
outputs.

## Quality strategy

Each output PNG is rendered:

- For SVG sources — by rasterizing the SVG **at the target size**
  via `resvg`. We never scale a single rendered bitmap up or down.
- For PNG sources — by decoding the source once and resizing to each
  target with `fast_image_resize`. The default algorithm is
  `Lanczos3` (`ResizeAlgorithm::Lanczos3`); other choices are
  available from Customize → Advanced settings.

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
expectations) — note that **SVG is listed first** so modern browsers
that support it select it before falling back to ICO/PNG:

```html
<link rel="icon" type="image/svg+xml" href="/favicon.svg">
<link rel="icon" href="/favicon.ico" sizes="any">
<link rel="icon" type="image/png" sizes="32x32" href="/favicon-32.png">
<link rel="icon" type="image/png" sizes="192x192" href="/favicon-192.png">
<link rel="icon" type="image/png" sizes="512x512" href="/favicon-512.png">
<link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png">
```

If `vectorize_on_raster=false` for a raster source, or if `include_svg`
is off, the `<link rel="icon" type="image/svg+xml">` line is omitted —
the HTML snippet always reflects what was actually written.

The path prefix is configurable per call to
`html_snippet::render(plan, base)` — for example pass
`"/static/icons/"`. Trailing slashes are normalized.

## Avoided bloat

The following are **not** emitted by default:

- Multiple `apple-touch-icon-*x*.png` variants. Modern iOS scales the
  single 180×180 well; the `-precomposed` variant is unnecessary on
  any iOS we target.
- `browserconfig.xml` and Microsoft tile color metadata. They were Edge
  Legacy / IE11 only. v1.26.0 can optionally generate four Microsoft app
  logo PNGs, but it still does not emit legacy metadata by default.
- A `manifest.webmanifest` file by default. It is available as an
  advanced opt-in because manifests are project-shaped and users often
  already have one.

## Failure modes

Output failures surface as `AppError::Export(_)` from `logolig`
and become persistent error toasts in the UI (§ui-a11y "Errors as
toasts"). The export step is **transactional at the file level**:
either every requested artifact is written, or none is. The
implementation uses a hidden staging directory
(`.logolig-<pid>-<nanos>.tmp`) inside the chosen output directory,
writes every artifact there, and finalizes by atomic rename. A
`StagingGuard` (RAII) deletes the staging on any failure path,
including panics.
