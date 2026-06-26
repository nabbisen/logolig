//! JPEG decode + resize end-to-end tests (v1.11.0).
//!
//! - An 8×8 JPEG decodes correctly to RGBA8 (8×8×4 = 256 bytes)
//! - Resizing to 16×16 / 32×32 / 48×48 produces correct dimensions
//! - JPEG has no alpha channel; decoded RGBA has alpha=255 throughout
//! - Non-JPEG input is rejected with `UnsupportedFile`

mod fixtures;

use logolig::AppError;
use logolig::ResizeAlgorithm;
use logolig::services::decode_jpeg::decode;
use logolig::services::ingest::ingest_bytes;
use logolig::services::resize::resize;

#[test]
fn jpeg_decode_returns_correct_dimensions() {
    let asset = ingest_bytes("tile.jpg", fixtures::jpeg_8x8_red()).unwrap();
    let rgba = decode(&asset).expect("JPEG should decode");
    assert_eq!(rgba.width, 8);
    assert_eq!(rgba.height, 8);
    assert_eq!(rgba.as_bytes().len(), 8 * 8 * 4);
}

#[test]
fn jpeg_decode_fills_alpha_with_full_opacity() {
    // JPEG has no alpha channel; image::to_rgba8 fills the alpha byte
    // with 255. Verify every pixel is fully opaque.
    let asset = ingest_bytes("tile.jpg", fixtures::jpeg_8x8_red()).unwrap();
    let rgba = decode(&asset).unwrap();
    for chunk in rgba.as_bytes().chunks_exact(4) {
        assert_eq!(chunk[3], 0xFF, "JPEG decoded pixel must be fully opaque");
    }
}

#[test]
fn jpeg_then_resize_to_favicon_sizes() {
    let asset = ingest_bytes("tile.jpg", fixtures::jpeg_8x8_red()).unwrap();
    let small = decode(&asset).unwrap();

    for size in [16u32, 32, 48] {
        let big = resize(&small, size, size, ResizeAlgorithm::Lanczos3).unwrap();
        assert_eq!(big.width, size);
        assert_eq!(big.height, size);
        assert_eq!(big.as_bytes().len(), (size as usize) * (size as usize) * 4);
        // Alpha is still 255 after resize
        for chunk in big.as_bytes().chunks_exact(4) {
            assert_eq!(chunk[3], 0xFF);
        }
    }
}

#[test]
fn decode_jpeg_rejects_png_source() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let err = decode(&asset).expect_err("PNG should not decode as JPEG");
    assert!(matches!(err, AppError::UnsupportedFile { .. }));
}

#[test]
fn decode_jpeg_rejects_webp_source() {
    let asset = ingest_bytes("tile.webp", fixtures::webp_8x8_blue()).unwrap();
    let err = decode(&asset).expect_err("WebP should not decode as JPEG");
    assert!(matches!(err, AppError::UnsupportedFile { .. }));
}
