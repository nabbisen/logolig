//! End-to-end tests for the export orchestrator: the Microsoft app logo output set.

#![allow(clippy::field_reassign_with_default)]

mod fixtures;

use logolig::ExportPlan;
use logolig::services::exporter::run_in_memory;
use logolig::services::ingest::ingest_bytes;

// ---------------------------------------------------------------------------
// v1.26.0: Microsoft app logo output set
// ---------------------------------------------------------------------------

#[test]
fn microsoft_app_logos_are_generated_with_expected_names_and_dimensions() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let mut plan = ExportPlan::default();
    plan.include_ico = false;
    plan.include_svg = false;
    plan.include_apple_touch = false;
    plan.include_html_snippet = false;
    plan.png_sizes.clear();
    plan.include_microsoft_app_logos = true;

    let artifacts = run_in_memory(&asset, &plan).expect("should generate Microsoft app logos");
    let expected = [
        ("StoreLogo.png", 50, 50),
        ("Square44x44Logo.png", 44, 44),
        ("Square150x150Logo.png", 150, 150),
        ("Wide310x150Logo.png", 310, 150),
    ];

    assert_eq!(artifacts.len(), expected.len());
    for (name, width, height) in expected {
        let art = artifacts
            .iter()
            .find(|a| a.relative_path.to_string_lossy() == name)
            .unwrap_or_else(|| panic!("missing {name}"));
        let img = image::load_from_memory_with_format(&art.bytes, image::ImageFormat::Png)
            .expect("Microsoft app logo should be a PNG")
            .to_rgba8();
        assert_eq!(
            img.dimensions(),
            (width, height),
            "wrong dimensions for {name}"
        );
    }
}

#[test]
fn microsoft_wide_logo_preserves_aspect_ratio_with_transparent_padding() {
    let asset = ingest_bytes("halftransp.png", fixtures::png_4x4_half_alpha_red()).unwrap();
    let mut plan = ExportPlan::default();
    plan.include_ico = false;
    plan.include_svg = false;
    plan.include_apple_touch = false;
    plan.include_html_snippet = false;
    plan.png_sizes.clear();
    plan.include_microsoft_app_logos = true;
    plan.keep_transparency = true;

    let artifacts = run_in_memory(&asset, &plan).expect("should generate logos");
    let wide = artifacts
        .iter()
        .find(|a| a.relative_path.to_string_lossy() == "Wide310x150Logo.png")
        .expect("wide logo should exist");
    let img = image::load_from_memory_with_format(&wide.bytes, image::ImageFormat::Png)
        .unwrap()
        .to_rgba8();
    assert_eq!(img.dimensions(), (310, 150));

    // A square source should be centred on a 310×150 canvas with transparent side padding.
    assert_eq!(
        img.get_pixel(0, 0)[3],
        0,
        "side padding should be transparent"
    );
    assert!(
        img.get_pixel(155, 75)[3] > 0,
        "centre should contain the fitted source image"
    );
}
