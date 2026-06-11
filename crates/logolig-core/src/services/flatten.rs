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

    for chunk in src_bytes.chunks_exact(4) {
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
mod tests {
    use super::*;

    fn make_rgba(width: u32, height: u32, pixels: Vec<u8>) -> Rgba8 {
        Rgba8::try_from_raw(width, height, Arc::<[u8]>::from(pixels))
            .expect("valid Rgba8 fixture")
    }

    #[test]
    fn fully_opaque_pixels_are_unchanged() {
        // All pixels fully opaque: RGB must be unchanged.
        let src = make_rgba(2, 1, vec![100, 150, 200, 255, 50, 75, 25, 255]);
        let out = flatten_to_white(&src);
        assert_eq!(out.pixels.as_ref(), &[100, 150, 200, 255, 50, 75, 25, 255]);
    }

    #[test]
    fn fully_transparent_pixels_become_white() {
        // Fully transparent pixels become white regardless of their RGB.
        let src = make_rgba(1, 1, vec![100, 150, 200, 0]);
        let out = flatten_to_white(&src);
        assert_eq!(out.pixels.as_ref(), &[255, 255, 255, 255]);
    }

    #[test]
    fn half_alpha_blends_halfway_to_white() {
        // alpha=128 (~50%) with R=0 → R' ≈ 127 (mid-grey), alpha becomes 255.
        let src = make_rgba(1, 1, vec![0, 0, 0, 128]);
        let out = flatten_to_white(&src);
        let bytes = out.pixels.as_ref();
        // Linear blend: round(0 * 128/255 + 255 * (1 - 128/255)) = round(127.0) = 127
        // Allow ±1 for floating-point rounding.
        assert!(
            (126..=128).contains(&bytes[0]),
            "expected ~127, got {}",
            bytes[0]
        );
        assert_eq!(bytes[3], 255, "alpha must be saturated");
    }

    #[test]
    fn output_alpha_is_always_saturated() {
        // Regardless of input alpha, every output pixel must be alpha=255.
        let src = make_rgba(
            4,
            1,
            vec![
                0, 0, 0, 0, //
                10, 20, 30, 64, //
                40, 80, 120, 192, //
                200, 100, 50, 255,
            ],
        );
        let out = flatten_to_white(&src);
        let bytes = out.pixels.as_ref();
        assert_eq!(bytes[3], 255);
        assert_eq!(bytes[7], 255);
        assert_eq!(bytes[11], 255);
        assert_eq!(bytes[15], 255);
    }

    #[test]
    fn dimensions_are_preserved() {
        // Dimensions must be preserved (trivial but guarded against regression).
        let src = make_rgba(7, 3, vec![0; 7 * 3 * 4]);
        let out = flatten_to_white(&src);
        assert_eq!(out.width, 7);
        assert_eq!(out.height, 3);
        assert_eq!(out.pixels.len(), 7 * 3 * 4);
    }
}
