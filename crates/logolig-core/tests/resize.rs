//! 画像処理パイプラインの end-to-end:
//! `ingest → decode_png → resize` が崩れず通ることを確認する。

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
    // 4×4 全画素が #CC3333FF
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
    // 単色画像なので、リサイズ後も中心は近い色のまま (アルファは不変=255)
    let center = (8 * 16 + 8) * 4;
    let alpha = big.as_bytes()[center + 3];
    assert_eq!(alpha, 0xFF);
}

#[test]
fn resize_short_circuits_when_target_matches_source() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let small = decode(&asset).unwrap();
    let same = resize(&small, 4, 4, ResizeAlgorithm::Lanczos3).unwrap();
    // バイト列が同一であること（短絡パスを経由している）
    assert_eq!(same.as_bytes(), small.as_bytes());
}

#[test]
fn resize_with_nearest_preserves_solid_color() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let small = decode(&asset).unwrap();
    let big = resize(&small, 8, 8, ResizeAlgorithm::Nearest).unwrap();
    // Nearest なら単色は完全に色が保たれる
    for chunk in big.as_bytes().chunks_exact(4) {
        assert_eq!(chunk, [0xCC, 0x33, 0x33, 0xFF]);
    }
}
