//! SVG rasteriser end-to-end tests.
//!
//! - A 16×16 SVG renders at 16 / 32 / 64 with correct output dimensions
//! - Output is straight-alpha RGBA8
//! - Transparent background pixels have alpha=0
//! - Calling rasterise on a PNG source returns `UnsupportedFile`

mod fixtures;

use logolig::AppError;
use logolig::services::ingest::ingest_bytes;
use logolig::services::rasterize_svg::rasterize;

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
    // SVG_16 is 16×16, aspect 1:1. Same target_size → rendered without padding.
    // One pixel larger (17) → at least one row/column of transparent border,
    // because the viewBox is centred.
    let asset = ingest_bytes("tile.svg", fixtures::SVG_16.as_bytes().to_vec()).unwrap();
    let bmp = rasterize(&asset, 17).unwrap();

    // Corner pixel (0,0) is guaranteed to be transparent.
    let bytes = bmp.as_bytes();
    let alpha_at = |x: u32, y: u32| -> u8 {
        let idx = (y as usize * bmp.width as usize + x as usize) * 4 + 3;
        bytes[idx]
    };
    assert_eq!(alpha_at(0, 0), 0, "top-left corner should be transparent");
    assert_eq!(
        alpha_at(16, 16),
        0,
        "bottom-right corner should be transparent"
    );
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
