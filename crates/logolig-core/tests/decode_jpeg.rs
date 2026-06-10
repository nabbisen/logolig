//! JPEG デコード + リサイズの end-to-end (v1.11.0)。
//!
//! - 8×8 の JPEG が正しく RGBA8 (8×8×4 = 256 バイト) に展開される
//! - JPEG ソースを 16×16 / 32×32 / 48×48 にリサイズしても寸法が合う
//! - JPEG はアルファを持たないため、 デコード後の RGBA は alpha=255 で埋まる
//! - decode_jpeg は JPEG 以外の入力を拒絶する (UnsupportedFile)

mod fixtures;

use logolig_core::services::decode_jpeg::decode;
use logolig_core::services::ingest::ingest_bytes;
use logolig_core::services::resize::resize;
use logolig_core::AppError;
use logolig_core::ResizeAlgorithm;

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
    // JPEG は alpha チャネルを持てないので、 image crate の to_rgba8 が
    // 不足する alpha を 255 で埋める。 全ピクセルが完全不透明であることを確認。
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
        // リサイズ後もアルファが保たれていること
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
