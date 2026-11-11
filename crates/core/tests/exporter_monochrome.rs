//! End-to-end tests for the export orchestrator: the monochrome output set (`mono/` subdirectory).

mod fixtures;

use fixtures::fresh_tmp_dir;

use logolig::ExportPlan;
use logolig::services::exporter::run;
use logolig::services::ingest::ingest_bytes;

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
