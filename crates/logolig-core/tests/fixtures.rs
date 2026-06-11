//! Minimal test fixtures.
//!
//! Uses `ingest_bytes` (synchronous) to avoid pulling in an async runtime.
//! Async ingestion is tested separately via `#[tokio::test]`.

#![allow(dead_code)]

/// 16×16 SVG with a blue grid centred.
/// Has explicit width/height attributes for reliable resvg parsing.
///
/// Uses `r##"…"##` raw-string delimiter because the SVG contains
/// `fill="#3366cc"` — the `"#` sequence would terminate `r#"…"#` prematurely.
pub const SVG_16: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16">
  <rect x="2" y="2" width="12" height="12" fill="#3366cc"/>
</svg>"##;

/// 4×4 red PNG.
///
/// Raw bytes produced by encoding with the `image` crate, embedded directly.
/// 4×4 is chosen so the IHDR width/height is 0x00000004 — easy to verify
/// `parse_png_size` behaviour by eye.
pub fn png_4x4_red() -> Vec<u8> {
    let mut buf = image::RgbaImage::new(4, 4);
    for px in buf.pixels_mut() {
        *px = image::Rgba([0xCC, 0x33, 0x33, 0xFF]);
    }
    let mut out = Vec::new();
    let dynamic = image::DynamicImage::ImageRgba8(buf);
    dynamic
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .expect("PNG encoding for fixture should not fail");
    out
}

/// 8×8 blue WebP (v1.1.0 fixture).
///
/// The image crate WebP encoder requires at least 8×8,
/// hence larger than the 4×4 PNG fixture. Used for magic-byte + intrinsic_size tests.
pub fn webp_8x8_blue() -> Vec<u8> {
    let mut buf = image::RgbaImage::new(8, 8);
    for px in buf.pixels_mut() {
        *px = image::Rgba([0x33, 0x66, 0xCC, 0xFF]);
    }
    let mut out = Vec::new();
    let dynamic = image::DynamicImage::ImageRgba8(buf);
    dynamic
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::WebP)
        .expect("WebP encoding for fixture should not fail");
    out
}

/// 8×8 red JPEG (v1.11.0 fixture).
///
/// 8×8 is the safe minimum for JPEG. Encoded via the `jpeg` feature of the image crate.
/// Used for magic-byte (`FF D8 FF`) + `parse_jpeg_size` SOF marker tests.

pub fn jpeg_8x8_red() -> Vec<u8> {
    let mut buf = image::RgbImage::new(8, 8);
    for px in buf.pixels_mut() {
        *px = image::Rgb([0xCC, 0x33, 0x33]);
    }
    let mut out = Vec::new();
    let dynamic = image::DynamicImage::ImageRgb8(buf);
    dynamic
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Jpeg)
        .expect("JPEG encoding for fixture should not fail");
    out
}

/// 4×4 semi-transparent red PNG (v1.21.0 fixture for the keep-transparency tests).
///
/// All pixels `(R=0xCC, G=0x33, B=0x33, A=0x80)` (~50% opacity).
/// When flattened with `keep_transparency=false`, all pixels should become
/// `A=255` at a midpoint between red and white.
pub fn png_4x4_half_alpha_red() -> Vec<u8> {
    let mut buf = image::RgbaImage::new(4, 4);
    for px in buf.pixels_mut() {
        *px = image::Rgba([0xCC, 0x33, 0x33, 0x80]);
    }
    let mut out = Vec::new();
    let dynamic = image::DynamicImage::ImageRgba8(buf);
    dynamic
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .expect("PNG encoding for fixture should not fail");
    out
}
