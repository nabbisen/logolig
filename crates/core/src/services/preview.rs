//! Preview raster generation.
//!
//! The preview panel (§5.2) shows the source at the resolution of the
//! target context. This service generates two sizes from the source:
//!
//! - **16 × 16** — actual-pixel browser-tab favicon
//! - **120 × 120** — phone home-screen icon (60 pt at 2× DPI)
//!
//! Both follow the §6.2 quality policy:
//! - SVG: rendered individually per target size
//! - PNG / WebP / JPEG: decoded once, then resized per target size

use std::path::PathBuf;

use crate::domain::{ResizeAlgorithm, Rgba8, SourceAsset, SourceKind};
use crate::error::AppError;
use crate::services::{decode_jpeg, decode_png, decode_webp, rasterize_svg, resize};

/// Pre-resized rasters for the preview panel.
///
/// Storing `source_path` and `algorithm` lets the UI check
/// whether the current cache is still valid for the current state.
#[derive(Debug, Clone)]
pub struct PreviewCache {
    pub source_path: PathBuf,
    pub algorithm: ResizeAlgorithm,
    /// Browser-tab size (16×16, shown at actual pixels).
    pub tab_16: Rgba8,
    /// Phone home-screen size (120×120, assumes 2× DPI).
    pub icon_120: Rgba8,
}

/// Generate both sizes from the source. CPU-bound.
///
/// Intended to be called from `iced::Task::perform` (§2.4).
/// Can be wrapped in an `async` block; internally synchronous.
pub fn build_preview(
    asset: &SourceAsset,
    algorithm: ResizeAlgorithm,
) -> Result<PreviewCache, AppError> {
    let tab_16 = render_at(asset, 16, algorithm)?;
    let icon_120 = render_at(asset, 120, algorithm)?;
    Ok(PreviewCache {
        source_path: asset.path.clone(),
        algorithm,
        tab_16,
        icon_120,
    })
}

/// Single-size rendering:
/// - SVG          → rasterise directly at the target size
/// - PNG / WebP   → decode then resize
fn render_at(
    asset: &SourceAsset,
    size: u32,
    algorithm: ResizeAlgorithm,
) -> Result<Rgba8, AppError> {
    match asset.kind {
        SourceKind::Svg => rasterize_svg::rasterize(asset, size),
        SourceKind::Png => {
            let decoded = decode_png::decode(asset)?;
            resize::resize(&decoded, size, size, algorithm)
        }
        SourceKind::Webp => {
            let decoded = decode_webp::decode(asset)?;
            resize::resize(&decoded, size, size, algorithm)
        }
        SourceKind::Jpeg => {
            let decoded = decode_jpeg::decode(asset)?;
            resize::resize(&decoded, size, size, algorithm)
        }
    }
}
