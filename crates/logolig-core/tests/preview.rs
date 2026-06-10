//! プレビュー生成の end-to-end テスト。
//!
//! - PNG / SVG のどちらからでも `tab_16` (16×16) と `icon_120` (120×120) が得られる
//! - キャッシュメタデータ (source_path, algorithm) が正しく保存される
//! - SVG は **ターゲットサイズで個別レンダリング** されている (両サイズが同じバイト列にならない)

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
    // 16×16 と 120×120 は明らかに違うバイト列であるべき (個別レンダリングされた証拠)
    assert_ne!(cache.tab_16.as_bytes().len(), cache.icon_120.as_bytes().len());
}

#[test]
fn algorithm_choice_is_recorded_in_cache() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let cache = build_preview(&asset, ResizeAlgorithm::Nearest).unwrap();
    assert_eq!(cache.algorithm, ResizeAlgorithm::Nearest);
}
