//! SVG rasteriser.
//!
//! To honour spec §6.2 "minimise blur and aliasing on characters and fine
//! lines", each target size is rendered **individually** from the SVG
//! source. Rasterising once at a large size and then downscaling still
//! produces resize-induced artefacts, so we re-apply the transform for
//! each size.

use std::sync::Arc;

use crate::domain::{Rgba8, SourceAsset, SourceKind};
use crate::error::AppError;

/// Render an SVG source into a square RGBA8 bitmap at the given size.
///
/// - Output is `target_size × target_size` (square, for favicon use)
/// - The SVG logical viewBox is scaled proportionally to `target_size`
///   (non-square SVGs are centred with transparent padding)
pub fn rasterize(asset: &SourceAsset, target_size: u32) -> Result<Rgba8, AppError> {
    if asset.kind != SourceKind::Svg {
        return Err(AppError::unsupported_file(format!(
            "rasterize_svg called on non-SVG source ({})",
            asset.kind.label()
        )));
    }
    if target_size == 0 {
        return Err(AppError::rasterize("target_size must be > 0"));
    }

    // 1. Parse into a usvg tree
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_data(&asset.raw, &opt)
        .map_err(|e| AppError::rasterize(format!("usvg parse: {e}")))?;

    let svg_size = tree.size();
    let svg_w = svg_size.width();
    let svg_h = svg_size.height();
    if svg_w <= 0.0 || svg_h <= 0.0 {
        return Err(AppError::rasterize("SVG has zero or negative size"));
    }

    // 2. Choose a scale that fits target_size while preserving aspect ratio
    let target = target_size as f32;
    let scale = (target / svg_w).min(target / svg_h);
    let drawn_w = svg_w * scale;
    let drawn_h = svg_h * scale;
    let tx = (target - drawn_w) * 0.5;
    let ty = (target - drawn_h) * 0.5;

    // 3. Allocate a Pixmap and render with resvg
    let mut pixmap = tiny_skia::Pixmap::new(target_size, target_size).ok_or_else(|| {
        AppError::rasterize(format!(
            "tiny_skia: cannot allocate pixmap {target_size}x{target_size}"
        ))
    })?;

    let transform = tiny_skia::Transform::from_scale(scale, scale).post_translate(tx, ty);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // 4. tiny-skia returns **premultiplied alpha**;
    //    convert to straight alpha before passing to PNG/ICO encoders.
    let pixels: Vec<u8> = pixmap
        .pixels()
        .iter()
        .flat_map(|p| {
            let d = p.demultiply();
            [d.red(), d.green(), d.blue(), d.alpha()]
        })
        .collect();

    Rgba8::try_from_raw(target_size, target_size, Arc::<[u8]>::from(pixels))
        .ok_or_else(|| AppError::rasterize("internal: rgba length mismatch"))
}
