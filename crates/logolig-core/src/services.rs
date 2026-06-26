//! Service layer.
//!
//! File I/O, SVG rasterisation, resizing, and other effectful operations.
//! The UI layer (logolig-app) performs all I/O through this layer.
//!
//! Implemented in Step 2:
//! - `ingest`         — accept PNG/SVG/WebP/JPEG and read logical size (async)
//! - `decode_png`     — decode PNG SourceAsset to Rgba8
//! - `rasterize_svg`  — rasterise SVG to Rgba8 at an arbitrary target size
//! - `resize`         — Rgba8 → Rgba8 at a different size (fast_image_resize, Lanczos3 default)
//!
//! Added in Step 3:
//! - `preview`        — builds a 16×16 + 120×120 preview cache from a SourceAsset
//!
//! Added in Step 4:
//! - `encode_png`     — Rgba8 → PNG byte buffer
//! - `ico_writer`     — packs multiple Rgba8 frames into a single ICO
//! - `html_snippet`   — generates the `<head>` HTML snippet
//! - `exporter`       — orchestrator: SourceAsset + ExportPlan → all artefacts
//!
//! Added in v1.1.0:
//! - `decode_webp`    — decode WebP SourceAsset to Rgba8 (static WebP via image-webp)
//!
//! Added in v1.2.0:
//! - `vectorize`      — Rgba8 → SVG string (vtracer wrapper, default settings)
//!
//! Added in v1.7.0:
//! - `transparency_audit` — classifies alpha status of an Rgba8 (detects fully-opaque /
//!   fully-transparent, common favicon mistakes)
//!
//! Added in v1.8.0:
//! - `manifest_writer` — assembles `manifest.webmanifest` JSON from
//!   `WebManifestSettings` + PNG sizes
//!
//! Added in v1.9.0:
//! - `monochrome` — Rgba8 → Rgba8 greyscale conversion (BT.709 luma).
//!   Used to generate the monochrome favicon set (mono/ subdirectory).
//!
//! Added in v1.11.0:
//! - `decode_jpeg` — JPEG → Rgba8. Accepted as input with an educational warning;
//!   JPEG cannot carry alpha so it is not ideal for favicons.

pub mod decode_jpeg;
pub mod decode_png;
pub mod decode_webp;
pub mod encode_png;
pub mod exporter;
pub mod flatten;
pub mod html_snippet;
pub mod ico_writer;
pub mod ingest;
pub mod manifest_writer;
pub mod monochrome;
pub mod preview;
pub mod rasterize_svg;
pub mod resize;
pub mod transparency_audit;
pub mod vectorize;
