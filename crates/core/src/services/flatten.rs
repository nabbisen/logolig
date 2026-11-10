//! Alpha flattening service (added in v1.21.0).
//!
//! When `ExportPlan::keep_transparency = false`, composites every pixel's alpha
//! against a white background and returns a fully-opaque `Rgba8`.
//!
//! ## Use cases
//!
//! - Compatibility with older browsers / OS versions that handle transparent ICO/PNG poorly.
//! - Downstream tools that assume opaque icons.
//! - Users who simply prefer no alpha channel.
//!
//! ## Background colour
//!
//! Fixed at white (`#FFFFFF`), matching the convention of most favicon tools.
//! A future version could expose `flatten_color: [u8; 3]` on `ExportPlan` if demand arises.
//!
//! ## Compositing formula (Porter-Duff "over" against white)
//!
//! For each pixel `(R, G, B, A)` where values are 0–255:
//!
//! ```text
//! a  = A / 255
//! R' = round(R * a + 255 * (1 - a))
//! G' = round(G * a + 255 * (1 - a))
//! B' = round(B * a + 255 * (1 - a))
//! A' = 255
//! ```
//!
//! `A=255` pixels are unchanged. `A=0` pixels become pure white `(255,255,255,255)`.
//!
//! ## Pixel format
//!
//! `Rgba8` stores straight (un-premultiplied) sRGB, matching the output of
//! `image::DynamicImage::to_rgba8()` used by the decode services.

use std::sync::Arc;

use crate::domain::Rgba8;

/// Flatten alpha against white. Returns a fully-opaque `Rgba8` (every pixel alpha=255).
///
/// A zero-size input is handled gracefully; the export pipeline never produces one.
pub fn flatten_to_white(src: &Rgba8) -> Rgba8 {
    let pixel_count = (src.width as usize) * (src.height as usize);
    let expected_len = pixel_count * 4;
    debug_assert_eq!(
        src.pixels.len(),
        expected_len,
        "Rgba8 invariant: pixels.len() == width * height * 4"
    );

    let mut out: Vec<u8> = Vec::with_capacity(expected_len);
    let src_bytes = src.pixels.as_ref();

    for chunk in src_bytes.as_chunks::<4>().0 {
        let r = chunk[0];
        let g = chunk[1];
        let b = chunk[2];
        let a = chunk[3];

        // Hot-path optimisation: skip f32 arithmetic for the two common cases.
        // Most pixels in a typical favicon source are either fully opaque or fully transparent.
        let (rp, gp, bp) = match a {
            255 => (r, g, b),
            0 => (255, 255, 255),
            _ => {
                let af = a as f32 / 255.0;
                let one_minus = 1.0 - af;
                let rp = (r as f32 * af + 255.0 * one_minus).round() as u8;
                let gp = (g as f32 * af + 255.0 * one_minus).round() as u8;
                let bp = (b as f32 * af + 255.0 * one_minus).round() as u8;
                (rp, gp, bp)
            }
        };
        out.extend_from_slice(&[rp, gp, bp, 255]);
    }

    // try_from_raw validates width*height*4 length invariant.
    // We write the same pixel count, so this always returns Some.
    // unwrap_or is a defensive safety net.
    Rgba8::try_from_raw(src.width, src.height, Arc::<[u8]>::from(out))
        .unwrap_or_else(|| src.clone())
}

#[cfg(test)]
mod tests;
