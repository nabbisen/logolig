//! Image-processing pipeline end-to-end:
//! verifies that `ingest → decode_png → resize` passes through correctly.

mod fixtures;

use logolig_core::services::decode_png::decode;
use logolig_core::services::ingest::ingest_bytes;
use logolig_core::services::resize::resize;
use logolig_core::ResizeAlgorithm;

#[test]
fn png_decode_returns_correct_dimensions() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let rgba = decode(&asset).unwrap();
    assert_eq!(rgba.width, 4);
    assert_eq!(rgba.height, 4);
    assert_eq!(rgba.as_bytes().len(), 4 * 4 * 4);
    // All 4×4 pixels are #CC3333FF
    for chunk in rgba.as_bytes().chunks_exact(4) {
        assert_eq!(chunk, [0xCC, 0x33, 0x33, 0xFF]);
    }
}

#[test]
fn resize_upscales_4x4_to_16x16_with_lanczos3() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let small = decode(&asset).unwrap();
    let big = resize(&small, 16, 16, ResizeAlgorithm::Lanczos3).unwrap();
    assert_eq!(big.width, 16);
    assert_eq!(big.height, 16);
    assert_eq!(big.as_bytes().len(), 16 * 16 * 4);
    // Solid-colour image: centre pixel stays the same colour after resize (alpha=255)
    let center = (8 * 16 + 8) * 4;
    let alpha = big.as_bytes()[center + 3];
    assert_eq!(alpha, 0xFF);
}

#[test]
fn resize_short_circuits_when_target_matches_source() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let small = decode(&asset).unwrap();
    let same = resize(&small, 4, 4, ResizeAlgorithm::Lanczos3).unwrap();
    // Byte slice is identical (went through the short-circuit path)
    assert_eq!(same.as_bytes(), small.as_bytes());
}

#[test]
fn resize_with_nearest_preserves_solid_color() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let small = decode(&asset).unwrap();
    let big = resize(&small, 8, 8, ResizeAlgorithm::Nearest).unwrap();
    // Nearest: solid colour is preserved exactly
    for chunk in big.as_bytes().chunks_exact(4) {
        assert_eq!(chunk, [0xCC, 0x33, 0x33, 0xFF]);
    }
}
