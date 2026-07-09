//! End-to-end tests for the export orchestrator.
//!
//! Runs the exporter against a temporary directory and verifies that all
//! expected files are present with correct contents.

#![allow(clippy::field_reassign_with_default)]

mod fixtures;

use std::path::PathBuf;

use logolig::ExportPlan;
use logolig::services::exporter::run;
use logolig::services::ingest::ingest_bytes;

/// Create a unique temporary directory (temp_dir + nanoseconds).
fn fresh_tmp_dir(label: &str) -> PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    let dir = std::env::temp_dir().join(format!("logolig-test-{label}-{pid}-{nanos}"));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

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

// ---------------------------------------------------------------------------
// v1.9.0: monochrome output set (mono/ subdirectory)
// ---------------------------------------------------------------------------

#[test]
fn monochrome_emits_mono_subdir_with_png_and_ico() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let dir = fresh_tmp_dir("mono-png");
    let plan = ExportPlan {
        monochrome: true,
        ..ExportPlan::default()
    };

    let report = run(&asset, &plan, &dir).expect("export should succeed");

    // Normal outputs + all files in the mono/ subdirectory
    let expected_color = [
        "favicon.svg",
        "favicon.ico",
        "favicon-32.png",
        "favicon-192.png",
        "favicon-512.png",
        "apple-touch-icon.png",
        "favicon-snippet.html",
    ];
    let expected_mono = [
        "mono/favicon.ico",
        "mono/favicon-32.png",
        "mono/favicon-192.png",
        "mono/favicon-512.png",
    ];

    for name in expected_color {
        let p = dir.join(name);
        assert!(p.is_file(), "missing color artifact: {}", p.display());
    }
    for name in expected_mono {
        let p = dir.join(name);
        assert!(p.is_file(), "missing mono artifact: {}", p.display());
    }
    // mono/ directory exists
    assert!(dir.join("mono").is_dir());

    // Total: colour 7 + mono 4 = 11 (no SVG mono in v1.9.0)
    assert_eq!(
        report.artifacts.len(),
        expected_color.len() + expected_mono.len()
    );

    // mono/favicon-32.png must differ from favicon-32.png in bytes (greyscale)
    let color_bytes = std::fs::read(dir.join("favicon-32.png")).unwrap();
    let mono_bytes = std::fs::read(dir.join("mono/favicon-32.png")).unwrap();
    assert_ne!(
        color_bytes, mono_bytes,
        "mono PNG should differ from color PNG"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn monochrome_off_does_not_create_mono_dir() {
    // When monochrome=false (default), no mono/ directory is created
    // — guarantees existing users' output is not broken by the feature.
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let dir = fresh_tmp_dir("mono-off");
    let plan = ExportPlan::default(); // monochrome = false

    run(&asset, &plan, &dir).expect("export should succeed");
    assert!(
        !dir.join("mono").exists(),
        "mono/ should not exist when monochrome=false"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn monochrome_with_ico_off_skips_mono_ico() {
    // include_ico=false → no mono/favicon.ico either.
    // User's "no ICO" preference applies to the mono set too.
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let dir = fresh_tmp_dir("mono-no-ico");
    let plan = ExportPlan {
        monochrome: true,
        include_ico: false,
        ..ExportPlan::default()
    };

    run(&asset, &plan, &dir).expect("export should succeed");
    assert!(dir.join("mono").is_dir());
    assert!(!dir.join("mono/favicon.ico").exists());
    // PNG should still be present
    assert!(dir.join("mono/favicon-32.png").is_file());

    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// v1.19.0: run_in_memory unit tests
// ---------------------------------------------------------------------------
//
// `run_in_memory` is the in-memory variant of `run`, introduced in v1.16.0
// for the "drop file → auto-convert → Result screen" flow.
// Zero disk I/O; no temp directory needed.
//
// The existing 12 tests continue to cover `run` (disk version) — since `run`
// is now a thin wrapper around `run_in_memory`, passing tests for `run`
// also covers most of the in-memory logic. These additional tests verify:
// - Artifact count / order / relative paths
// - Byte-for-byte match with `run` output
// - Subdirectories (mono/) expressed in relative_path
// - Failure (empty ico_sizes) → nothing written (zero disk side-effects)

use logolig::services::exporter::run_in_memory;

#[test]
fn run_in_memory_returns_default_artifact_set_from_png_source() {
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let plan = ExportPlan::default();

    let artifacts = run_in_memory(&asset, &plan).expect("in-memory run should succeed");

    // v1.2.0 default: 7 files (same set as run).
    let expected_names = [
        "favicon.svg",
        "favicon.ico",
        "favicon-32.png",
        "favicon-192.png",
        "favicon-512.png",
        "apple-touch-icon.png",
        "favicon-snippet.html",
    ];
    let actual_names: Vec<String> = artifacts
        .iter()
        .map(|a| a.relative_path.to_string_lossy().to_string())
        .collect();
    for name in expected_names {
        assert!(
            actual_names.iter().any(|n| n == name),
            "missing artifact: {} (have: {:?})",
            name,
            actual_names
        );
    }
    assert_eq!(artifacts.len(), expected_names.len());
}

#[test]
fn run_in_memory_bytes_match_disk_run_byte_for_byte() {
    // Run both run_in_memory and run with the same asset + plan;
    // verify every artifact matches byte-for-byte. Confirms `run`
    // is a faithful wrapper (in-memory content survives the disk round-trip).
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let plan = ExportPlan::default();

    let in_memory = run_in_memory(&asset, &plan).expect("in-memory should succeed");

    let dir = fresh_tmp_dir("byte-match");
    let report = run(&asset, &plan, &dir).expect("disk should succeed");

    assert_eq!(in_memory.len(), report.artifacts.len());

    for art in &in_memory {
        let on_disk = dir.join(&art.relative_path);
        assert!(on_disk.is_file(), "expected on disk: {}", on_disk.display());
        let disk_bytes = std::fs::read(&on_disk).unwrap();
        assert_eq!(
            art.bytes,
            disk_bytes,
            "byte mismatch for {}",
            art.relative_path.display()
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_in_memory_encodes_mono_subdirectory_in_relative_path() {
    // When monochrome is enabled, verify artifacts have subdirectory-prefixed
    // relative paths like `mono/favicon-{size}.png`.
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let mut plan = ExportPlan::default();
    plan.monochrome = true;

    let artifacts = run_in_memory(&asset, &plan).expect("should succeed");

    let mono_paths: Vec<String> = artifacts
        .iter()
        .map(|a| a.relative_path.to_string_lossy().to_string())
        .filter(|p| p.starts_with("mono/") || p.starts_with("mono\\"))
        .collect();
    assert!(
        !mono_paths.is_empty(),
        "expected mono/ artifacts but found none. all: {:?}",
        artifacts
            .iter()
            .map(|a| a.relative_path.display().to_string())
            .collect::<Vec<_>>()
    );

    // At least one of mono/favicon.ico or mono/favicon-NN.png must be present
    // (ico is always present since include_ico=true).
    assert!(mono_paths.iter().any(|p| p.ends_with("favicon.ico")));
}

#[test]
fn run_in_memory_returns_err_on_empty_ico_sizes() {
    // An invalid plan (empty ico_sizes + include_ico=true) should return Err.
    // Being in-memory, zero disk side-effects are guaranteed
    // (no files are written).
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let mut plan = ExportPlan::default();
    plan.ico_sizes.clear();

    let result = run_in_memory(&asset, &plan);
    assert!(result.is_err(), "expected Err, got {:?}", result.is_ok());
}

#[test]
fn run_in_memory_skips_optional_artifacts_when_disabled() {
    // include_apple_touch=false / include_html_snippet=false / include_svg=false
    // include_ico=false → the ICO relative_path is absent.
    let asset = ingest_bytes("tile.png", fixtures::png_4x4_red()).unwrap();
    let mut plan = ExportPlan::default();
    plan.include_apple_touch = false;
    plan.include_html_snippet = false;
    plan.include_svg = false;
    plan.include_ico = false;

    let artifacts = run_in_memory(&asset, &plan).expect("should succeed");

    let names: Vec<String> = artifacts
        .iter()
        .map(|a| a.relative_path.to_string_lossy().to_string())
        .collect();

    assert!(
        !names.iter().any(|n| n == "apple-touch-icon.png"),
        "apple-touch should be omitted, got: {:?}",
        names
    );
    assert!(
        !names.iter().any(|n| n == "favicon-snippet.html"),
        "html snippet should be omitted, got: {:?}",
        names
    );
    assert!(
        !names.iter().any(|n| n == "favicon.svg"),
        "svg should be omitted, got: {:?}",
        names
    );
    assert!(
        !names.iter().any(|n| n == "favicon.ico"),
        "ico should be omitted, got: {:?}",
        names
    );

    // PNG remains (default 32/192/512).
    assert!(names.iter().any(|n| n == "favicon-32.png"));
    assert!(names.iter().any(|n| n == "favicon-192.png"));
    assert!(names.iter().any(|n| n == "favicon-512.png"));
}

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
