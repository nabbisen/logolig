//! `transparency_audit` behaviour tests (v1.7.0).
//!
//! Properties verified:
//! 1. All-alpha=255 image is classified `FullyOpaque`
//! 2. All-alpha=0 image is classified `FullyTransparent`
//! 3. Mixed image is classified `HasTransparency`
//! 4. A single differing pixel flips the classification (boundary condition)
//! 5. Empty image (width=0 or height=0) is treated as `FullyTransparent`
//! 6. `needs_warning` is false for mixed, true for the two warning cases

use std::sync::Arc;

use logolig::Rgba8;
use logolig::services::transparency_audit::{TransparencyReport, audit};

/// Create a `width × height` image with every pixel's alpha set to `alpha`.
/// RGB is zeroed (only alpha classification matters here).
fn solid_alpha_image(width: u32, height: u32, alpha: u8) -> Rgba8 {
    let n = (width as usize) * (height as usize);
    let mut buf = Vec::with_capacity(n * 4);
    for _ in 0..n {
        buf.extend_from_slice(&[0, 0, 0, alpha]);
    }
    Rgba8::try_from_raw(width, height, Arc::from(buf.into_boxed_slice()))
        .expect("solid_alpha_image: dimensions must match buffer length")
}

/// All-opaque image except the pixel at `index`, which has alpha=0.
fn alpha_with_one_transparent(width: u32, height: u32, transparent_pixel_idx: usize) -> Rgba8 {
    let n = (width as usize) * (height as usize);
    let mut buf = Vec::with_capacity(n * 4);
    for i in 0..n {
        let alpha = if i == transparent_pixel_idx { 0 } else { 255 };
        buf.extend_from_slice(&[0, 0, 0, alpha]);
    }
    Rgba8::try_from_raw(width, height, Arc::from(buf.into_boxed_slice()))
        .expect("alpha_with_one_transparent: dimensions must match buffer length")
}

#[test]
fn fully_opaque_is_detected() {
    let img = solid_alpha_image(4, 4, 255);
    assert_eq!(audit(&img), TransparencyReport::FullyOpaque);
}

#[test]
fn fully_transparent_is_detected() {
    let img = solid_alpha_image(4, 4, 0);
    assert_eq!(audit(&img), TransparencyReport::FullyTransparent);
}

#[test]
fn mixed_alpha_at_boundary_pixel_counts_as_has_transparency() {
    // 16 pixels: all opaque except the last one (alpha=0)
    // → "mostly opaque, one transparent pixel" should still be HasTransparency
    let img = alpha_with_one_transparent(4, 4, 15); // 16 pixels; index 15 is the last
    assert_eq!(audit(&img), TransparencyReport::HasTransparency);
}

#[test]
fn mixed_alpha_at_first_pixel_counts_as_has_transparency() {
    // Verify early exit (return when min==0 && max==255) is correct
    // First pixel transparent, rest opaque
    let img = alpha_with_one_transparent(4, 4, 0);
    assert_eq!(audit(&img), TransparencyReport::HasTransparency);
}

#[test]
fn semi_transparent_uniform_treated_as_has_transparency() {
    // Uniformly half-transparent image (all pixels alpha=128).
    // Neither FullyOpaque nor FullyTransparent → classified as HasTransparency.
    // (Harmless for favicons; excluded from warning)
    let img = solid_alpha_image(2, 2, 128);
    assert_eq!(audit(&img), TransparencyReport::HasTransparency);
}

#[test]
fn empty_image_is_fully_transparent() {
    // width=0 → zero pixels.
    let img = Rgba8::try_from_raw(0, 0, Arc::from(Vec::<u8>::new().into_boxed_slice()))
        .expect("zero-size with empty buffer must be valid");
    assert_eq!(audit(&img), TransparencyReport::FullyTransparent);
}

#[test]
fn needs_warning_only_for_extreme_cases() {
    assert!(TransparencyReport::FullyOpaque.needs_warning());
    assert!(TransparencyReport::FullyTransparent.needs_warning());
    assert!(!TransparencyReport::HasTransparency.needs_warning());
}

#[test]
fn single_pixel_opaque_image_is_fully_opaque() {
    // Boundary: 1×1 image does not crash audit
    let img = solid_alpha_image(1, 1, 255);
    assert_eq!(audit(&img), TransparencyReport::FullyOpaque);
}

#[test]
fn single_pixel_transparent_image_is_fully_transparent() {
    let img = solid_alpha_image(1, 1, 0);
    assert_eq!(audit(&img), TransparencyReport::FullyTransparent);
}
