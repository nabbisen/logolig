//! WebP decode + resize end-to-end tests (v1.1.0).
//!
//! - 8×8 WebP decodes to the correct RGBA8 size (32×32 = 256 bytes)
//! - Resizing a WebP source to 16×16 or 32×32 yields correct dimensions
//! - decode_webp rejects non-WebP input (UnsupportedFile)

mod fixtures;

use logolig::AppError;
use logolig::ResizeAlgorithm;
use logolig::services::decode_webp::decode;
use logolig::services::ingest::ingest_bytes;
use logolig::services::resize::resize;

#[test]
fn webp_decode_returns_correct_dimensions() {
    let asset = ingest_bytes("tile.webp", fixtures::webp_8x8_blue()).unwrap();
    let rgba = decode(&asset).expect("WebP should decode");
    assert_eq!(rgba.width, 8);
    assert_eq!(rgba.height, 8);
    assert_eq!(rgba.as_bytes().len(), 8 * 8 * 4);
}

#[test]
fn webp_then_resize_to_favicon_sizes() {
    let asset = ingest_bytes("tile.webp", fixtures::webp_8x8_blue()).unwrap();
    let small = decode(&asset).unwrap();

    for size in [16u32, 32, 48] {
        let big = resize(&small, size, size, ResizeAlgorithm::Lanczos3).unwrap();
        assert_eq!(big.width, size);
        assert_eq!(big.height, size);
        assert_eq!(big.as_bytes().len(), (size as usize) * (size as usize) * 4);
        // Alpha channel should be fully opaque
        for chunk in big.as_bytes().chunks_exact(4) {
            assert_eq!(chunk[3], 0xFF);
        }
    }
}

#[test]
fn decode_webp_rejects_png_source() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let err = decode(&asset).expect_err("PNG should not decode as WebP");
    assert!(matches!(err, AppError::UnsupportedFile { .. }));
}
