//! Canvas fitting helpers for non-square logo outputs.
//!
//! Favicon outputs historically render to square targets. Microsoft app logos
//! add a wide canvas (`Wide310x150Logo.png`), so they need a contain-fit helper
//! that preserves the source aspect ratio and centres it on a transparent canvas.

use std::sync::Arc;

use crate::domain::{ResizeAlgorithm, Rgba8};
use crate::error::AppError;
use crate::services::resize;

/// Fit `src` inside `target_w × target_h` without cropping or stretching.
///
/// The returned canvas is transparent outside the fitted source image.
/// Callers may flatten it later if their export plan requires opaque output.
pub fn contain_on_transparent_canvas(
    src: &Rgba8,
    target_w: u32,
    target_h: u32,
    algorithm: ResizeAlgorithm,
) -> Result<Rgba8, AppError> {
    if target_w == 0 || target_h == 0 {
        return Err(AppError::resize("target dimensions must be > 0"));
    }
    if src.width == 0 || src.height == 0 {
        return Err(AppError::resize("source dimensions must be > 0"));
    }

    let scale = (target_w as f64 / src.width as f64).min(target_h as f64 / src.height as f64);
    let fitted_w = ((src.width as f64 * scale).round() as u32).clamp(1, target_w);
    let fitted_h = ((src.height as f64 * scale).round() as u32).clamp(1, target_h);
    let fitted = resize::resize(src, fitted_w, fitted_h, algorithm)?;

    let mut pixels = vec![0u8; target_w as usize * target_h as usize * 4];
    let x0 = ((target_w - fitted_w) / 2) as usize;
    let y0 = ((target_h - fitted_h) / 2) as usize;
    let target_w_usize = target_w as usize;
    let fitted_w_usize = fitted_w as usize;

    for row in 0..fitted_h as usize {
        let dst_start = ((y0 + row) * target_w_usize + x0) * 4;
        let dst_end = dst_start + fitted_w_usize * 4;
        let src_start = row * fitted_w_usize * 4;
        let src_end = src_start + fitted_w_usize * 4;
        pixels[dst_start..dst_end].copy_from_slice(&fitted.pixels[src_start..src_end]);
    }

    Rgba8::try_from_raw(target_w, target_h, Arc::<[u8]>::from(pixels))
        .ok_or_else(|| AppError::resize("internal: rgba length mismatch"))
}
