//! End-to-end tests for the export orchestrator: the default set and its options, written to a temporary directory.
//!
//! Runs the exporter against a temporary directory and verifies that all
//! expected files are present with correct contents.

#![allow(clippy::field_reassign_with_default)]

mod fixtures;

use fixtures::fresh_tmp_dir;

use logolig::ExportPlan;
use logolig::services::exporter::run;
use logolig::services::ingest::ingest_bytes;

#[test]
fn exports_default_artifact_set_from_png_source() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let dir = fresh_tmp_dir("png-default");
    let plan = ExportPlan::default();

    let report = run(&asset, &plan, &dir).expect("export should succeed");

    // v1.2.0 default: 7 files — adds favicon.svg (vectorised from raster source).
    let expected_names = [
        "favicon.svg",
        "favicon.ico",
        "favicon-32.png",
        "favicon-192.png",
        "favicon-512.png",
        "apple-touch-icon.png",
        "favicon-snippet.html",
    ];
    for name in expected_names {
        let p = dir.join(name);
        assert!(p.is_file(), "missing artifact: {}", p.display());
    }
    assert_eq!(report.artifacts.len(), expected_names.len());

    // No staging artefacts left behind (transactional rollback / cleanup check)
    let leftovers: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with(".logolig-"))
        .collect();
    assert!(
        leftovers.is_empty(),
        "staging directory should be cleaned up"
    );

    // Cleanup
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn exports_default_artifact_set_from_svg_source() {
    let asset = ingest_bytes("tile.svg", fixtures::SVG_16.as_bytes().to_vec()).unwrap();
    let dir = fresh_tmp_dir("svg-default");
    let plan = ExportPlan::default();

    run(&asset, &plan, &dir).expect("svg export should succeed");
    // SVG source: input is copied as-is to `favicon.svg` (v1.2.0)
    assert!(dir.join("favicon.svg").is_file(), "SVG output expected");
    let svg_content = std::fs::read(dir.join("favicon.svg")).unwrap();
    assert_eq!(
        svg_content,
        fixtures::SVG_16.as_bytes(),
        "SVG source must be copied byte-for-byte (non-destructive)"
    );

    assert!(dir.join("favicon.ico").is_file());
    assert!(dir.join("apple-touch-icon.png").is_file());
    assert!(dir.join("favicon-snippet.html").is_file());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn vectorize_off_omits_svg_file_for_raster_source() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let dir = fresh_tmp_dir("vectorize-off");
    let mut plan = ExportPlan::default();
    plan.vectorize_on_raster = false;

    let report = run(&asset, &plan, &dir).expect("export should succeed");

    // Raster source + vectorise off → no SVG output
    assert!(!dir.join("favicon.svg").exists());
    // HTML snippet also omits the `<link type="image/svg+xml">` line
    let html = std::fs::read_to_string(dir.join("favicon-snippet.html")).unwrap();
    assert!(!html.contains(r#"type="image/svg+xml""#));
    assert!(!html.contains("favicon.svg"));

    // Artifact count is one less (no SVG)
    assert_eq!(report.artifacts.len(), 6); // 7 - 1 (favicon.svg)

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn include_svg_off_omits_svg_for_svg_source_too() {
    // SVG source with include_svg=false → no SVG output either
    let asset = ingest_bytes("tile.svg", fixtures::SVG_16.as_bytes().to_vec()).unwrap();
    let dir = fresh_tmp_dir("include-svg-off");
    let mut plan = ExportPlan::default();
    plan.include_svg = false;

    run(&asset, &plan, &dir).unwrap();
    assert!(!dir.join("favicon.svg").exists());
    let html = std::fs::read_to_string(dir.join("favicon-snippet.html")).unwrap();
    assert!(!html.contains("favicon.svg"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn exports_default_artifact_set_from_webp_source() {
    let asset = ingest_bytes("tile.webp", fixtures::webp_8x8_blue()).unwrap();
    let dir = fresh_tmp_dir("webp-default");
    let plan = ExportPlan::default();

    run(&asset, &plan, &dir).expect("WebP export should succeed");
    assert!(dir.join("favicon.ico").is_file());
    assert!(dir.join("apple-touch-icon.png").is_file());
    assert!(dir.join("favicon-32.png").is_file());
    assert!(dir.join("favicon-snippet.html").is_file());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn ico_can_be_read_back_with_correct_frames() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let dir = fresh_tmp_dir("ico-roundtrip");
    let plan = ExportPlan::default();

    run(&asset, &plan, &dir).unwrap();

    // Re-parse the ICO with the ico crate and verify all three frames (16/32/48) are present
    let f = std::fs::File::open(dir.join("favicon.ico")).unwrap();
    let icondir = ico::IconDir::read(std::io::BufReader::new(f)).unwrap();
    let mut sizes: Vec<u32> = icondir.entries().iter().map(|e| e.width()).collect();
    sizes.sort_unstable();
    assert_eq!(sizes, vec![16, 32, 48]);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn html_snippet_file_contains_link_tags() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let dir = fresh_tmp_dir("html-content");
    run(&asset, &ExportPlan::default(), &dir).unwrap();

    let html = std::fs::read_to_string(dir.join("favicon-snippet.html")).unwrap();
    // v1.2.0 default: SVG comes first
    assert!(html.contains(r#"<link rel="icon" type="image/svg+xml" href="/favicon.svg">"#));
    assert!(html.contains(r#"<link rel="icon" href="/favicon.ico""#));
    assert!(html.contains(r#"rel="apple-touch-icon""#));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn fails_cleanly_when_output_dir_does_not_exist() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let bad = std::env::temp_dir().join("logolig-this-path-must-not-exist-yet-xyz");
    let _ = std::fs::remove_dir_all(&bad);
    let err = run(&asset, &ExportPlan::default(), &bad).expect_err("should fail");
    // Error is AppError::Export { .. }
    let s = err.to_string();
    assert!(s.contains("output directory"));
}

#[test]
fn no_apple_touch_omits_that_file() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let dir = fresh_tmp_dir("opt-out");
    let mut plan = ExportPlan::default();
    plan.include_apple_touch = false;
    plan.include_html_snippet = false;

    let report = run(&asset, &plan, &dir).unwrap();
    assert!(!dir.join("apple-touch-icon.png").exists());
    assert!(!dir.join("favicon-snippet.html").exists());
    // SVG (vectorised), PNG, and ICO remain (SVG was added in v1.2.0 default)
    assert!(dir.join("favicon.svg").is_file());
    assert!(dir.join("favicon.ico").is_file());
    assert!(dir.join("favicon-32.png").is_file());
    assert_eq!(
        report.artifacts.len(),
        // svg (1) + ico (1) + png_sizes (3) = 5
        2 + ExportPlan::default().png_sizes.len()
    );

    let _ = std::fs::remove_dir_all(&dir);
}
