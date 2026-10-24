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
        .write_to(
            &mut std::io::Cursor::new(&mut out),
            image::ImageFormat::WebP,
        )
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
        .write_to(
            &mut std::io::Cursor::new(&mut out),
            image::ImageFormat::Jpeg,
        )
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

/// Solid-colour PNG of any size (RFC 010 non-square fixtures).
pub fn png_rgba(width: u32, height: u32, rgba: [u8; 4]) -> Vec<u8> {
    let mut buf = image::RgbaImage::new(width, height);
    for px in buf.pixels_mut() {
        *px = image::Rgba(rgba);
    }
    let mut out = Vec::new();
    let dynamic = image::DynamicImage::ImageRgba8(buf);
    dynamic
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .expect("PNG encoding for fixture should not fail");
    out
}

// ---------------------------------------------------------------------------
// SVG fixtures for RFC 015 phase 2a (tests/svg.rs)
// ---------------------------------------------------------------------------

/// 64×64 SVG whose only content is an `<image>` pointing at `href`.
pub fn svg_with_image(href: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="64" height="64" viewBox="0 0 64 64"><image x="0" y="0" width="64" height="64" xlink:href="{href}"/></svg>"#
    )
}

/// 128×128 SVG that draws only text (no shapes).
pub const SVG_TEXT_ONLY: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="128" height="128" viewBox="0 0 128 128"><text x="8" y="100" font-size="100" font-family="sans-serif" fill="#000000">Aa</text></svg>"##;

/// 64×64 SVG with a text element and two `<image>` references (`href1`, `href2`).
pub fn svg_text_and_two_images(href1: &str, href2: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="64" height="64" viewBox="0 0 64 64"><image x="0" y="0" width="32" height="32" xlink:href="{href1}"/><image x="32" y="32" width="32" height="32" xlink:href="{href2}"/><text x="4" y="60" font-size="20" font-family="sans-serif">Hi</text></svg>"#
    )
}

/// 64×64 SVG with a solid green square (used as the "nested SVG" target).
pub const SVG_GREEN_64: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 64 64"><rect width="64" height="64" fill="#00c800"/></svg>"##;

/// Text inside nested groups (drawn).
pub const SVG_TEXT_IN_GROUPS: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><g><g><text x="2" y="30" font-size="20">Hi</text></g></g></svg>"##;

/// Text defined in `<defs>` and drawn through `<use>`.
pub const SVG_TEXT_VIA_USE: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="64" height="64"><defs><text id="t" x="2" y="30" font-size="20">Hi</text></defs><use xlink:href="#t"/></svg>"##;

/// Text only inside `<defs>`: never drawn.
pub const SVG_TEXT_ONLY_IN_DEFS: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><defs><text id="t" x="2" y="30" font-size="20">Hi</text></defs><rect width="8" height="8"/></svg>"##;

/// An empty text element: nothing to draw.
pub const SVG_EMPTY_TEXT: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><text x="2" y="30" font-size="20"></text></svg>"##;
