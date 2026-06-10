//! WebP デコード + リサイズの end-to-end (v1.1.0)。
//!
//! - 8×8 の WebP が正しく RGBA8 (32×32 = 256 バイト) に展開される
//! - WebP ソースを 16×16 や 32×32 にリサイズしても寸法が合う
//! - decode_webp は WebP 以外の入力を拒絶する (UnsupportedFile)

mod fixtures;

use logolig_core::services::decode_webp::decode;
use logolig_core::services::ingest::ingest_bytes;
use logolig_core::services::resize::resize;
use logolig_core::AppError;
use logolig_core::ResizeAlgorithm;

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
        // アルファチャンネルが完全不透明として保たれていること
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
