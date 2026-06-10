//! ベクトル化サービスの end-to-end テスト (v1.2.0)。
//!
//! - PNG → デコード → ベクトル化が SVG 文字列を返す
//! - WebP → デコード → ベクトル化も同様に動く
//! - 出力 SVG が valid XML/SVG として最低限の構造を持つ
//! - サイズ 0 のラスタは Err

mod fixtures;

use logolig_core::services::decode_png::decode as decode_png;
use logolig_core::services::decode_webp::decode as decode_webp;
use logolig_core::services::ingest::ingest_bytes;
use logolig_core::services::vectorize::vectorize;
use logolig_core::VtracerPreset;

#[test]
fn png_can_be_vectorized_to_svg_string() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let rgba = decode_png(&asset).unwrap();
    let svg = vectorize(&rgba, VtracerPreset::Default)
        .expect("PNG vectorization should succeed");

    // 最低限の SVG 構造: <?xml ... ?> 宣言と <svg ...> ルート要素
    assert!(svg.starts_with("<?xml"), "should start with XML declaration");
    assert!(svg.contains("<svg"), "should contain <svg> element");
    assert!(svg.contains("</svg>"), "should be properly closed");
    // vtracer は generator コメントを入れる
    assert!(svg.contains("VTracer"), "should mention VTracer in generator comment");
    // 4×4 のソースサイズが SVG の width/height に反映されているはず
    assert!(svg.contains(r#"width="4""#) && svg.contains(r#"height="4""#));
}

#[test]
fn webp_can_be_vectorized_to_svg_string() {
    let asset = ingest_bytes("tile.webp", fixtures::webp_8x8_blue()).unwrap();
    let rgba = decode_webp(&asset).unwrap();
    let svg = vectorize(&rgba, VtracerPreset::Default)
        .expect("WebP vectorization should succeed");

    assert!(svg.starts_with("<?xml"));
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
    assert!(svg.contains(r#"width="8""#) && svg.contains(r#"height="8""#));
}

#[test]
fn vectorize_output_can_be_parsed_back_by_usvg() {
    // 生成された SVG が usvg (resvg のパーサ) で読み戻せることを確認。
    // これは「文法的に valid」 + 「描画パイプラインの自己整合性」の両方を保証する。
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let rgba = decode_png(&asset).unwrap();
    let svg = vectorize(&rgba, VtracerPreset::Default).unwrap();

    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_data(svg.as_bytes(), &opt)
        .expect("vtracer output should parse with usvg");
    let size = tree.size();
    assert!(size.width() > 0.0 && size.height() > 0.0);
}

// v1.4.1: Sharp / PhotoRich プリセットでも動くことを確認。 出力品質の比較は
// 主観的なので、 ここでは「3 種類すべてで valid SVG が生成される」 を保証する。
#[test]
fn all_presets_produce_valid_svg() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let rgba = decode_png(&asset).unwrap();

    for preset in VtracerPreset::all() {
        let svg = vectorize(&rgba, preset)
            .unwrap_or_else(|e| panic!("vectorize with {preset:?} failed: {e}"));
        assert!(svg.starts_with("<?xml"), "preset {preset:?} should produce XML");
        assert!(svg.contains("<svg"), "preset {preset:?} should contain <svg>");
        // usvg で読み戻し可能
        let opt = usvg::Options::default();
        usvg::Tree::from_data(svg.as_bytes(), &opt)
            .unwrap_or_else(|e| panic!("preset {preset:?} output rejected by usvg: {e}"));
    }
}
