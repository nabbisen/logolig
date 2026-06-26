//! Vectorisation service end-to-end tests (v1.2.0).
//!
//! - PNG → decode → vectorise returns an SVG string
//! - WebP → decode → vectorise also works
//! - Output SVG has at minimum a valid XML declaration and `<svg>` root
//! - Zero-size raster returns `Err`

mod fixtures;

use logolig_core::VtracerPreset;
use logolig_core::services::decode_png::decode as decode_png;
use logolig_core::services::decode_webp::decode as decode_webp;
use logolig_core::services::ingest::ingest_bytes;
use logolig_core::services::vectorize::vectorize;

#[test]
fn png_can_be_vectorized_to_svg_string() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let rgba = decode_png(&asset).unwrap();
    let svg = vectorize(&rgba, VtracerPreset::Default).expect("PNG vectorization should succeed");

    // Minimum SVG structure: <?xml …?> declaration and <svg …> root element
    assert!(
        svg.starts_with("<?xml"),
        "should start with XML declaration"
    );
    assert!(svg.contains("<svg"), "should contain <svg> element");
    assert!(svg.contains("</svg>"), "should be properly closed");
    // vtracer inserts a generator comment
    assert!(
        svg.contains("VTracer"),
        "should mention VTracer in generator comment"
    );
    // Source size (4×4) should be reflected in SVG width/height
    assert!(svg.contains(r#"width="4""#) && svg.contains(r#"height="4""#));
}

#[test]
fn webp_can_be_vectorized_to_svg_string() {
    let asset = ingest_bytes("tile.webp", fixtures::webp_8x8_blue()).unwrap();
    let rgba = decode_webp(&asset).unwrap();
    let svg = vectorize(&rgba, VtracerPreset::Default).expect("WebP vectorization should succeed");

    assert!(svg.starts_with("<?xml"));
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
    assert!(svg.contains(r#"width="8""#) && svg.contains(r#"height="8""#));
}

#[test]
fn vectorize_output_can_be_parsed_back_by_usvg() {
    // Verify the generated SVG can be re-parsed by usvg (resvg's parser).
    // This asserts both syntactic validity and self-consistency of the rendering pipeline.
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let rgba = decode_png(&asset).unwrap();
    let svg = vectorize(&rgba, VtracerPreset::Default).unwrap();

    let opt = usvg::Options::default();
    let tree =
        usvg::Tree::from_data(svg.as_bytes(), &opt).expect("vtracer output should parse with usvg");
    let size = tree.size();
    assert!(size.width() > 0.0 && size.height() > 0.0);
}

// v1.4.1: verify Sharp and PhotoRich presets also produce output. Quality comparison
// is subjective, so we only assert that all three presets produce valid SVG.
#[test]
fn all_presets_produce_valid_svg() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let rgba = decode_png(&asset).unwrap();

    for preset in VtracerPreset::all() {
        let svg = vectorize(&rgba, preset)
            .unwrap_or_else(|e| panic!("vectorize with {preset:?} failed: {e}"));
        assert!(
            svg.starts_with("<?xml"),
            "preset {preset:?} should produce XML"
        );
        assert!(
            svg.contains("<svg"),
            "preset {preset:?} should contain <svg>"
        );
        // Re-parseable by usvg
        let opt = usvg::Options::default();
        usvg::Tree::from_data(svg.as_bytes(), &opt)
            .unwrap_or_else(|e| panic!("preset {preset:?} output rejected by usvg: {e}"));
    }
}
