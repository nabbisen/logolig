//! Greyscale (monochrome) conversion service (v1.9.0).
//!
//! Generates a greyscale version of each raster artifact as an additional
//! favicon output. Intended uses: single-colour printed materials,
//! theme-aware mask icons (CSS `mask-image`), stencils.
//!
//! ## Conversion algorithm
//!
//! ITU-R BT.709 luma formula (sRGB-aligned):
//!
//! ```text
//! Y = 0.2126 R + 0.7152 G + 0.0722 B
//! ```
//!
//! Simple average `(R+G+B)/3` over-weights blue. BT.709 is the modern
//! standard for HDTV / sRGB / web content, which matches logolig's target.
//! Alpha is preserved unchanged.

use std::sync::Arc;

use crate::domain::Rgba8;

/// BT.709 luma coefficients (sRGB-aligned). Float arithmetic rounded to u8.
const COEF_R: f32 = 0.2126;
const COEF_G: f32 = 0.7152;
const COEF_B: f32 = 0.0722;

/// Convert one pixel (R, G, B) to a BT.709 luma value (u8).
///
/// Float arithmetic rounded to u8. Slower than bit-manipulation,
/// but negligible for favicon sizes (max 1024×1024 = 1 M pixels, < 10 ms).
#[inline]
fn luma_bt709(r: u8, g: u8, b: u8) -> u8 {
    let y = (r as f32) * COEF_R + (g as f32) * COEF_G + (b as f32) * COEF_B;
    // Round and clamp to u8. BT.709 coefficients sum to 1 so the result
    // stays in 0..=255, but guard against floating-point rounding just in case.
    y.round().clamp(0.0, 255.0) as u8
}

/// Convert `Rgba8` to a greyscale `Rgba8`. Alpha is preserved unchanged.
///
/// Returns a new `Rgba8`; the original is not modified. The pixel buffer
/// is freshly allocated and wrapped in `Arc<[u8]>`.
pub fn to_grayscale(image: &Rgba8) -> Rgba8 {
    let mut buf = Vec::with_capacity(image.pixels.len());
    for chunk in image.pixels.chunks_exact(4) {
        let y = luma_bt709(chunk[0], chunk[1], chunk[2]);
        buf.push(y); // R = Y
        buf.push(y); // G = Y
        buf.push(y); // B = Y
        buf.push(chunk[3]); // preserve alpha
    }
    Rgba8::try_from_raw(image.width, image.height, Arc::from(buf.into_boxed_slice()))
        .expect("monochrome: input dimensions match buffer length")
}

/// Owned variant: takes ownership instead of borrowing. Same implementation.
/// Avoids an unnecessary borrow when the caller already owns the `Rgba8`.
pub fn into_grayscale(image: Rgba8) -> Rgba8 {
    to_grayscale(&image)
}
