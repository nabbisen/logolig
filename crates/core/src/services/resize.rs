//! Resize a raster image to an arbitrary size (§6.2).
//!
//! - Default algorithm is Lanczos3 (`ResizeAlgorithm::default()`)
//! - Uses `fast_image_resize` convolution kernels to minimise aliasing
//! - **Short-circuits when target size equals source size** — avoids
//!   double-applying the kernel on identical dimensions
//! - **Alpha pre-multiplication** is enabled (`mul_div_alpha`) to prevent
//!   colour fringing around transparent edges

use std::num::NonZeroU32;
use std::sync::Arc;

use fast_image_resize as fr;
use fr::{ResizeOptions, Resizer, images::Image};

use crate::domain::{ResizeAlgorithm, Rgba8};
use crate::error::AppError;

/// Resize the input RGBA8 image to `(target_w, target_h)` and return a new `Rgba8`.
pub fn resize(
    src: &Rgba8,
    target_w: u32,
    target_h: u32,
    algorithm: ResizeAlgorithm,
) -> Result<Rgba8, AppError> {
    if target_w == 0 || target_h == 0 {
        return Err(AppError::resize("target dimensions must be > 0"));
    }
    // Target equals source size — return as-is (short-circuit).
    if src.width == target_w && src.height == target_h {
        return Ok(src.clone());
    }

    // Zero-size check is performed when constructing NonZero values.
    let (sw, sh) = (
        NonZeroU32::new(src.width).ok_or_else(|| AppError::resize("source width is 0"))?,
        NonZeroU32::new(src.height).ok_or_else(|| AppError::resize("source height is 0"))?,
    );

    // src is a read-only view; dst is a mutable output buffer.
    // fast_image_resize 5.x: Image::from_vec_u8 takes ownership of the buffer;
    // RGBA8 → PixelType::U8x4.
    let mut src_buf: Vec<u8> = src.pixels.to_vec();
    let src_view = Image::from_slice_u8(sw.get(), sh.get(), &mut src_buf, fr::PixelType::U8x4)
        .map_err(|e| AppError::resize(format!("src view: {e}")))?;

    let mut dst = Image::new(target_w, target_h, fr::PixelType::U8x4);

    // 5.x: Resizer carries state. Can be reused across resizes for efficiency,
    // but new() per call is fast enough for our use.
    let mut resizer = Resizer::new();
    let opts = ResizeOptions::new().resize_alg(algorithm.to_resize_alg());
    resizer
        .resize(&src_view, &mut dst, &opts)
        .map_err(|e| AppError::resize(format!("resize: {e}")))?;

    // dst is an owned pixel buffer. Extract the Vec and wrap in Arc.
    let pixels: Vec<u8> = dst.into_vec();
    Rgba8::try_from_raw(target_w, target_h, Arc::<[u8]>::from(pixels))
        .ok_or_else(|| AppError::resize("internal: rgba length mismatch"))
}
