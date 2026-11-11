//! End-to-end tests for the export orchestrator: keeping or flattening transparency.

#![allow(clippy::field_reassign_with_default)]

mod fixtures;

use logolig::ExportPlan;
use logolig::services::exporter::run_in_memory;
use logolig::services::ingest::ingest_bytes;

// ---------------------------------------------------------------------------
// v1.21.0: keep_transparency integration tests
// ---------------------------------------------------------------------------
//
// Verifies that `ExportPlan::keep_transparency` is reflected correctly in
// PNG and ICO outputs. Unit logic for flatten is tested in services/flatten.rs;
// these tests check the full export pipeline end to end.

#[test]
fn keep_transparency_true_preserves_alpha_in_png_output() {
    // Semi-transparent red PNG → keep_transparency=true (default) →
    // output PNG pixels retain alpha < 255.
    let asset = ingest_bytes("halftransp.png", fixtures::png_4x4_half_alpha_red()).unwrap();
    let mut plan = ExportPlan::default();
    plan.png_sizes = vec![32];
    plan.include_ico = false;
    plan.include_apple_touch = false;
    plan.include_html_snippet = false;
    plan.include_svg = false;
    assert!(plan.keep_transparency, "default should be true");

    let artifacts = run_in_memory(&asset, &plan).expect("should succeed");

    // Decode the PNG and inspect the alpha channel.
    let png_art = artifacts
        .iter()
        .find(|a| a.relative_path.to_string_lossy() == "favicon-32.png")
        .expect("favicon-32.png should exist");
    let img = image::load_from_memory_with_format(&png_art.bytes, image::ImageFormat::Png).unwrap();
    let rgba = img.to_rgba8();
    // Input alpha=0x80 (128). Lanczos3 may shift alpha slightly on resize;
    // "not 255" is sufficient.
    let alpha_sample = rgba.get_pixel(0, 0)[3];
    assert!(
        alpha_sample < 255,
        "expected alpha < 255 (transparency preserved), got {}",
        alpha_sample
    );
}

#[test]
fn keep_transparency_false_flattens_to_fully_opaque_png() {
    // Semi-transparent red PNG → keep_transparency=false →
    // every output PNG pixel has alpha=255.
    let asset = ingest_bytes("halftransp.png", fixtures::png_4x4_half_alpha_red()).unwrap();
    let mut plan = ExportPlan::default();
    plan.png_sizes = vec![32];
    plan.include_ico = false;
    plan.include_apple_touch = false;
    plan.include_html_snippet = false;
    plan.include_svg = false;
    plan.keep_transparency = false; // enable flattening

    let artifacts = run_in_memory(&asset, &plan).expect("should succeed");

    let png_art = artifacts
        .iter()
        .find(|a| a.relative_path.to_string_lossy() == "favicon-32.png")
        .expect("favicon-32.png should exist");
    let img = image::load_from_memory_with_format(&png_art.bytes, image::ImageFormat::Png).unwrap();
    let rgba = img.to_rgba8();
    // Every pixel should now have alpha=255.
    for px in rgba.pixels() {
        assert_eq!(
            px[3], 255,
            "expected fully opaque after flatten, got alpha={}",
            px[3]
        );
    }
    // Half-alpha red (A=128) flattened against white:
    // R ≈ 0xCC + (255-0xCC)*128/255 ≈ 0xE6. Allow a range for float rounding.
    let center = rgba.get_pixel(2, 2);
    assert!(
        center[0] >= 0xD0 && center[0] <= 0xF0,
        "R should be ~0xE6 (red blended with white halfway), got {:#X}",
        center[0]
    );
}

#[test]
fn keep_transparency_false_does_not_affect_svg_output() {
    // SVG source → keep_transparency=false → SVG output copies asset.raw
    // as-is, so output matches input exactly
    // (flattening does not affect SVG — Q2-a).
    let asset = ingest_bytes("tile.svg", fixtures::SVG_16.as_bytes().to_vec()).unwrap();
    let mut plan = ExportPlan::default();
    plan.png_sizes = vec![]; // suppress PNG output
    plan.include_ico = false;
    plan.include_apple_touch = false;
    plan.include_html_snippet = false;
    plan.include_svg = true;
    plan.keep_transparency = false;

    let artifacts = run_in_memory(&asset, &plan).expect("should succeed");

    let svg_art = artifacts
        .iter()
        .find(|a| a.relative_path.to_string_lossy() == "favicon.svg")
        .expect("favicon.svg should exist");
    // SVG source: output is asset.raw (input bytes) unchanged.
    // Not affected by flattening.
    assert_eq!(svg_art.bytes, fixtures::SVG_16.as_bytes());
}

#[test]
fn keep_transparency_false_makes_ico_frames_fully_opaque() {
    // ICO output: all frames should also have alpha=255.
    let asset = ingest_bytes("halftransp.png", fixtures::png_4x4_half_alpha_red()).unwrap();
    let mut plan = ExportPlan::default();
    plan.png_sizes = vec![]; // no PNG needed for this test
    plan.include_apple_touch = false;
    plan.include_html_snippet = false;
    plan.include_svg = false;
    plan.include_ico = true;
    plan.keep_transparency = false;

    let artifacts = run_in_memory(&asset, &plan).expect("should succeed");
    let ico_art = artifacts
        .iter()
        .find(|a| a.relative_path.to_string_lossy() == "favicon.ico")
        .expect("favicon.ico should exist");
    // Decode each ICO frame and check alpha. Use the ico crate
    // (already a workspace dep, used by ico_writer) — handles both BMP
    // and PNG frames uniformly.
    // The image crate's `ico` feature is not enabled in the workspace,
    // so image::load_from_memory_with_format(_, Ico) would return Unsupported.
    use std::io::Cursor;
    let dir = ico::IconDir::read(Cursor::new(&ico_art.bytes)).expect("ico parse should succeed");
    assert!(
        !dir.entries().is_empty(),
        "ICO should have at least 1 frame"
    );
    for entry in dir.entries() {
        let img = entry.decode().expect("ICO frame decode should succeed");
        let rgba = img.rgba_data();
        // rgba is a flat byte slice (RGBA); alpha is at index % 4 == 3.
        for (i, byte) in rgba.iter().enumerate() {
            if i % 4 == 3 {
                assert_eq!(
                    *byte, 255,
                    "expected fully opaque ICO frame after flatten, got alpha={} at index {}",
                    byte, i
                );
            }
        }
    }
}
