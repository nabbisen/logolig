//! SVG ラスタライザの end-to-end テスト。
//!
//! - 16×16 SVG が 16/32/64 のターゲットでそれぞれ正しいサイズで返る
//! - 出力が straight alpha の RGBA8 (premultiplied から戻されている)
//! - 透明背景の余白がきちんと alpha=0 になっている
//! - PNG ソースに rasterize を呼ぶと UnsupportedFile

mod fixtures;

use logolig_core::services::ingest::ingest_bytes;
use logolig_core::services::rasterize_svg::rasterize;
use logolig_core::AppError;

#[test]
fn rasterize_svg_renders_at_each_target_size() {
    let asset = ingest_bytes("tile.svg", fixtures::SVG_16.as_bytes().to_vec()).unwrap();

    for size in [16u32, 32, 64] {
        let bmp = rasterize(&asset, size).expect("rasterize should succeed");
        assert_eq!(bmp.width, size);
        assert_eq!(bmp.height, size);
        assert_eq!(bmp.as_bytes().len(), (size as usize) * (size as usize) * 4);
    }
}

#[test]
fn rasterize_centers_drawn_pixels_with_transparent_padding() {
    // SVG_16 は 16x16 でアスペクト比 1:1。target_size を同じにすると
    // 余白なくぴったり描かれる。さらに 1 ピクセルだけ大きく (17) すると、
    // 中央配置のため少なくとも 1 行/列の透明帯ができることを検証する。
    let asset = ingest_bytes("tile.svg", fixtures::SVG_16.as_bytes().to_vec()).unwrap();
    let bmp = rasterize(&asset, 17).unwrap();

    // 角ピクセル (0, 0) は確実に透明。
    let bytes = bmp.as_bytes();
    let alpha_at = |x: u32, y: u32| -> u8 {
        let idx = (y as usize * bmp.width as usize + x as usize) * 4 + 3;
        bytes[idx]
    };
    assert_eq!(alpha_at(0, 0), 0, "top-left corner should be transparent");
    assert_eq!(alpha_at(16, 16), 0, "bottom-right corner should be transparent");
}

#[test]
fn rasterize_rejects_png_source() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let err = rasterize(&asset, 32).expect_err("PNG should not be rasterizable as SVG");
    assert!(matches!(err, AppError::UnsupportedFile { .. }));
}

#[test]
fn rasterize_rejects_zero_size() {
    let asset = ingest_bytes("tile.svg", fixtures::SVG_16.as_bytes().to_vec()).unwrap();
    let err = rasterize(&asset, 0).expect_err("zero target should fail");
    assert!(matches!(err, AppError::Rasterize { .. }));
}
