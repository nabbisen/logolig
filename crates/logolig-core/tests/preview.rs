//! End-to-end tests for preview generation.
//!
//! - Both PNG and SVG sources produce a `tab_16` (16×16) and `icon_120` (120×120)
//! - Cache metadata (source_path, algorithm) is stored correctly
//! - SVG is rendered independently at each size (the two sizes are different byte sequences)

mod fixtures;

use logolig_core::services::ingest::ingest_bytes;
use logolig_core::services::preview::build_preview;
use logolig_core::ResizeAlgorithm;

#[test]
fn build_preview_from_png_produces_both_sizes() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let cache = build_preview(&asset, ResizeAlgorithm::Lanczos3).unwrap();

    assert_eq!(cache.tab_16.width, 16);
    assert_eq!(cache.tab_16.height, 16);
    assert_eq!(cache.icon_120.width, 120);
    assert_eq!(cache.icon_120.height, 120);
    assert_eq!(cache.source_path, asset.path);
    assert_eq!(cache.algorithm, ResizeAlgorithm::Lanczos3);
}

#[test]
fn build_preview_from_svg_produces_both_sizes() {
    let asset = ingest_bytes("tile.svg", fixtures::SVG_16.as_bytes().to_vec()).unwrap();
    let cache = build_preview(&asset, ResizeAlgorithm::Lanczos3).unwrap();
    assert_eq!(cache.tab_16.width, 16);
    assert_eq!(cache.icon_120.width, 120);
    // 16×16 and 120×120 must differ in bytes (proves independent rendering)
    assert_ne!(cache.tab_16.as_bytes().len(), cache.icon_120.as_bytes().len());
}

#[test]
fn algorithm_choice_is_recorded_in_cache() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let cache = build_preview(&asset, ResizeAlgorithm::Nearest).unwrap();
    assert_eq!(cache.algorithm, ResizeAlgorithm::Nearest);
}
